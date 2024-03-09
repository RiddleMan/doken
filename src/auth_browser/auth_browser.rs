use crate::token_info::TokenInfo;
use anyhow::{anyhow, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::fetch::{
    ContinueRequestParams, EventRequestPaused, FulfillRequestParams,
};
use chromiumoxide::cdp::browser_protocol::target::CreateTargetParamsBuilder;
use chromiumoxide::handler::viewport::Viewport;
use chromiumoxide::{Handler, Page};
use futures::StreamExt;
use oauth2::CsrfToken;
use std::borrow::Cow;
use std::ops::Add;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::sync::{oneshot, OnceCell};
use tokio::time::sleep;
use url::Url;

#[derive(Error, Debug)]
enum RequestError {
    #[error("No requests with required data. Timeout.")]
    Timeout,

    // TODO: Implement channels to all pages to close them
    #[error("The user closed the browser")]
    _BrowserClosed,
}

const CONTENT_OK: &str = "<html><head></head><body><h1>OK</h1></body></html>";
const CONTENT_NOT_OK: &str = "<html><head></head><body><h1>NOT OK</h1></body></html>";

pub struct AuthPage {
    page: Page,
}

impl AuthPage {
    async fn process_request<TResponse, F>(
        &self,
        timeout: u64,
        authorization_url: Url,
        callback_url: Url,
        f: F,
    ) -> Result<TResponse>
    where
        TResponse: Send + Clone + Sync + 'static,
        F: Send + Fn(Arc<EventRequestPaused>) -> Option<TResponse> + 'static,
    {
        let (tx_browser, rx_browser) = oneshot::channel();
        let mut request_paused = self
            .page
            .event_listener::<EventRequestPaused>()
            .await
            .unwrap();
        let intercept_page = self.page.clone();
        let callback_url = callback_url.to_owned();
        tokio::spawn(async move {
            while let Some(event) = request_paused.next().await {
                let request_url = Url::parse(&event.request.url).unwrap();
                if request_url.origin() == callback_url.origin()
                    && request_url.path() == callback_url.path()
                {
                    log::debug!("Received request to `--callback-url` {}", callback_url);

                    let response = f(event.clone());

                    if let Err(e) = intercept_page
                        .execute(
                            FulfillRequestParams::builder()
                                .request_id(event.request_id.clone())
                                .body(BASE64_STANDARD.encode(if response.is_some() {
                                    CONTENT_OK
                                } else {
                                    CONTENT_NOT_OK
                                }))
                                .response_code(200)
                                .build()
                                .unwrap(),
                        )
                        .await
                    {
                        log::error!("Failed to fullfill request: {e}");
                    }

                    if let Some(response) = response {
                        let _ = tx_browser.send(response);
                        break;
                    }
                } else if let Err(e) = intercept_page
                    .execute(ContinueRequestParams::new(event.request_id.clone()))
                    .await
                {
                    log::error!("Failed to continue request: {e}");
                }
            }
        });

        log::debug!("Opening authorization page {}", authorization_url);
        self.page.goto(authorization_url.as_str()).await?;

        let response = tokio::select! {
            _ = sleep(Duration::from_millis(timeout)) => {
                log::debug!("Timeout");
                Err::<TResponse, anyhow::Error>(RequestError::Timeout.into())
            }
            Ok(response) = rx_browser => {
                Ok::<TResponse, anyhow::Error>(response)
            }
            // _ = &mut self.rx_handle => {
            //     log::debug!("User closed the browser");
            //     Err::<TResponse, anyhow::Error>(RequestError::BrowserClosed.into())
            // }
        };

        response
    }

    pub async fn get_code(
        &self,
        timeout: u64,
        authorization_url: Url,
        callback_url: Url,
        csrf_token: CsrfToken,
    ) -> Result<String> {
        self.process_request(timeout, authorization_url, callback_url, move |event| {
            let request_url = Url::parse(&event.request.url).unwrap();
            let state = request_url.query_pairs().find(|qp| qp.0.eq("state"));
            let code = request_url.query_pairs().find(|qp| qp.0.eq("code"));

            match (state, code) {
                (Some((_, state)), Some((_, code))) => {
                    if state == *csrf_token.secret() {
                        let code = code.to_string();
                        log::debug!("Given code: {}", code);

                        Some(code)
                    } else {
                        log::debug!("Incorrect CSRF token. Ignoring...");

                        None
                    }
                }
                _ => {
                    log::debug!(
                        "Call to server without a state and/or a code parameter. Ignoring..."
                    );

                    None
                }
            }
        })
        .await
    }

    pub async fn get_token_data(
        &self,
        timeout: u64,
        authorization_url: Url,
        callback_url: Url,
        csrf_token: CsrfToken,
    ) -> Result<TokenInfo> {
        self.process_request(
            timeout,
            authorization_url,
            callback_url,
            move |event| match event.request.method.as_str() {
                "POST" => {
                    let body = event.request.post_data.as_ref().unwrap();

                    log::info!("This is what we get in POST: {:?}", body);
                    let form_params =
                        form_urlencoded::parse(body.as_bytes())
                            .collect::<Vec<(Cow<str>, Cow<str>)>>();

                    let (_, access_token) = form_params
                        .iter()
                        .find(|(name, _value)| name == "access_token")
                        .expect("Cannot find access_token in the HTTP Post request.");

                    let (_, expires_in) = form_params
                        .iter()
                        .find(|(name, _value)| name == "expires_in")
                        .expect("Cannot find expires_in in the HTTP Post request.");

                    let (_, state) = form_params
                        .iter()
                        .find(|(name, _value)| name == "state")
                        .expect("Cannot find state in the HTTP Post request.");

                    if state == csrf_token.secret() {
                        Some(TokenInfo {
                            access_token: access_token.to_string(),
                            refresh_token: None,
                            expires: Some(
                                SystemTime::now().add(Duration::from_secs(
                                    expires_in
                                        .parse::<u64>()
                                        .expect("expires_in is an incorrect number"),
                                )),
                            ),
                            scope: None,
                        })
                    } else {
                        log::debug!("Incorrect CSRF token. Aborting...");

                        None
                    }
                }
                _ => {
                    log::debug!(
                        "Call to server without a state and/or a code parameter. Ignoring..."
                    );

                    None
                }
            },
        )
        .await
    }

    pub async fn close(self) -> Result<()> {
        self.page.close().await.map_err(|e| anyhow!(e))?;
        Ok(())
    }
}

pub struct AuthBrowser {
    browser: OnceCell<Browser>,
    headless: bool,
}

impl AuthBrowser {
    pub fn new(headless: bool) -> AuthBrowser {
        AuthBrowser {
            browser: OnceCell::new(),
            headless,
        }
    }

    pub async fn open_page(&self) -> Result<AuthPage> {
        let browser_page = self.lazy_open_page().await?;
        let page = AuthPage { page: browser_page };
        Ok(page)
    }

    pub async fn browser(&self) -> &Browser {
        self.browser
            .get_or_init(|| async {
                let (tx, _) = oneshot::channel::<()>();

                let (browser, mut handler) = Self::launch_browser(self.headless).await.unwrap();

                tokio::spawn(async move {
                    while let Some(h) = handler.next().await {
                        if h.is_err() {
                            tx.send(()).unwrap();
                            log::error!("Handler created an error");
                            break;
                        }
                    }
                });
                browser
            })
            .await
    }

    pub async fn pages(&self) -> Result<Vec<Page>> {
        self.browser().await.pages().await.map_err(|e| anyhow!(e))
    }

    async fn lazy_open_page(&self) -> Result<Page> {
        let browser = self.browser().await;
        let page = self.wait_for_first_page(browser).await?;

        let create_new_page = || async {
            let page_config = CreateTargetParamsBuilder::default()
                .url("about:blank")
                .build()
                .map_err(|e| anyhow!(e))?;
            browser.new_page(page_config).await.map_err(|e| anyhow!(e))
        };
        match page.url().await? {
            Some(url) => {
                if url == "chrome://new-tab-page/" {
                    page.goto("about:blank").await.unwrap();
                    Ok(page)
                } else {
                    create_new_page().await
                }
            }
            _ => create_new_page().await,
        }
    }

    async fn wait_for_first_page(&self, browser: &Browser) -> Result<Page> {
        let mut retries = 10;

        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            log::debug!("Trying to reach the first page...");
            let pages = browser.pages().await?;
            match (pages.first(), retries) {
                (Some(page), _) => {
                    log::debug!("First page found");
                    log::debug!("Pages: {:?}", page.url().await?.unwrap());
                    return Ok(page.to_owned());
                }
                (None, 0) => {
                    log::debug!("Too many retries. Creating new page.");
                    let page_config = CreateTargetParamsBuilder::default()
                        .url("about:blank")
                        .build()
                        .map_err(|e| anyhow!(e))?;
                    return browser.new_page(page_config).await.map_err(|e| anyhow!(e));
                }
                (None, _) => {
                    log::debug!("Just another try");
                    retries -= 1;
                }
            }
        }
    }

    async fn launch_browser(headless: bool) -> Result<(Browser, Handler)> {
        log::debug!("Opening chromium instance");
        const WIDTH: u32 = 800;
        const HEIGHT: u32 = 1000;
        let viewport = Viewport {
            width: WIDTH,
            height: HEIGHT,
            ..Viewport::default()
        };

        let mut config = BrowserConfig::builder();

        if !headless {
            config = config.with_head();
        }

        config = config
            .viewport(viewport)
            .window_size(WIDTH, HEIGHT)
            .enable_request_intercept()
            .respect_https_errors()
            .enable_cache();

        Browser::launch(config.build().map_err(|e| anyhow!(e))?)
            .await
            .map_err(|e| anyhow!(e))
    }

    pub async fn close_all_tabs(&self) -> Result<()> {
        let pages = self.pages().await?;

        for page in pages {
            page.close().await.map_err(|e| anyhow!(e))?;
        }
        Ok(())
    }
}
