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
use tokio::time::sleep;
use std::borrow::Cow;
use std::ops::Add;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::sync::oneshot;
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
        &mut self,
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
        &mut self,
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
        &mut self,
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
}

pub struct AuthBrowser {
    browser: Option<Browser>,
    _open_pages_count: u16,
    headless: bool,
}

impl AuthBrowser {
    pub fn new(headless: bool) -> AuthBrowser {
        AuthBrowser {
            browser: None,
            headless,
            _open_pages_count: 0,
        }
    }

    pub async fn open_page(&mut self) -> Result<AuthPage> {
        let browser_page = self.lazy_open_page().await?;
        let page = AuthPage { page: browser_page };
        self._open_pages_count += 1;
        Ok(page)
    }

    async fn lazy_open_page(&mut self) -> Result<Page> {
        if self.browser.is_none() {
            let (tx, _) = oneshot::channel::<()>();

            let (browser, mut handler) = Self::launch_browser(self.headless).await?;

            tokio::spawn(async move {
                while let Some(h) = handler.next().await {
                    if h.is_err() {
                        tx.send(()).unwrap();
                        log::error!("Handler created an error");
                        break;
                    }
                }
            });

            self.browser = Some(browser);
        };

        let browser = self.browser.as_ref().unwrap();
        if self._open_pages_count == 0 {
            self.wait_for_first_page(browser).await
        } else {
            let page_config = CreateTargetParamsBuilder::default()
                .url("about:blank")
                .build()
                .map_err(|e| anyhow!(e))?;
            browser.new_page(page_config).await.map_err(|e| anyhow!(e))
        }
    }

    async fn wait_for_first_page(&self, browser: &Browser) -> Result<Page> {
        let mut retries = 10;

        if self._open_pages_count != 0 {
            let page_config = CreateTargetParamsBuilder::default()
                .url("about:blank")
                .build()
                .map_err(|e| anyhow!(e))?;
            return browser.new_page(page_config).await.map_err(|e| anyhow!(e));
        }

        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            log::debug!("Trying to reach the first page...");
            let pages = browser.pages().await?;
            match (pages.first(), retries) {
                (Some(page), _) => {
                    log::debug!("First page found");
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
}

#[cfg(test)]
mod tests {
    use tokio::join;
    use tokio::sync::Mutex;

    use super::*;

    #[tokio::test]
    async fn test_opens_a_browser() {
        env_logger::init();
        let auth_browser = Arc::new(Mutex::new(AuthBrowser::new(false)));

        let browser1 = auth_browser.clone();
        let handle1 = tokio::spawn(async move {
            let mut lock = browser1.lock().await;
            let page = lock.open_page().await;

            drop(lock);
            let x =  page.unwrap()
                .get_code(
                    2_000,
                    Url::parse("https://wykop.pl").unwrap(),
                    Url::parse("http://google.com/oauth/callback").unwrap(),
                    CsrfToken::new_random(),
                )
                .await;

            log::info!("Test1");
                x.unwrap();
            log::info!("Test1after");

            println!("Code");
        });

        let browser2 = auth_browser.clone();
        let handle2 = tokio::spawn(async move {
            let mut lock = browser2.lock().await;
            let page = lock.open_page().await;

            drop(lock);
            let x =  page.unwrap()
                .get_code(
                    2_000,
                    Url::parse("https://wykop.pl/mikroblog").unwrap(),
                    Url::parse("http://google.com/oauth/callback").unwrap(),
                    CsrfToken::new_random(),
                )
                .await;

            log::info!("Test2");
                x.unwrap();
            log::info!("Test2after");

            println!("Code");
        });

        log::info!("Joining");
        let (_, _) = join!(handle1, handle2);
        log::info!("After");
    }
}
