use anyhow::{anyhow, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::fetch::{
    ContinueRequestParams, EventRequestPaused, FulfillRequestParams,
};
use futures::StreamExt;
use oauth2::CsrfToken;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::oneshot;
use url::Url;

#[derive(Error, Debug)]
enum RequestError {
    #[error("No requests with required data. Timeout.")]
    Timeout,
}

const CONTENT_OK: &str = "<html><head></head><body><h1>OK</h1></body></html>";
const CONTENT_NOT_OK: &str = "<html><head></head><body><h1>NOT OK</h1></body></html>";

pub struct AuthBrowser {
    authorization_url: Url,
    callback_url: Url,
}

impl AuthBrowser {
    pub fn new(authorization_url: Url, callback_url: Url) -> Result<AuthBrowser> {
        Ok(AuthBrowser {
            authorization_url,
            callback_url,
        })
    }

    pub async fn get_code(&self, timeout: u64, csrf_token: CsrfToken) -> Result<String> {
        let (tx_browser, rx_browser) = oneshot::channel();
        let (tx_sleep, rx_sleep) = oneshot::channel();

        log::debug!("Opening chromium instance");
        let (mut browser, mut _handler) = Browser::launch(
            BrowserConfig::builder()
                .with_head()
                .enable_request_intercept()
                .build()
                .map_err(|e| anyhow!(e))?,
        )
        .await?;

        let handle = tokio::spawn(async move {
            while let Some(h) = _handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });

        let page = Arc::new(browser.new_page("about:blank").await?);

        let mut request_paused = page.event_listener::<EventRequestPaused>().await.unwrap();
        let intercept_page = page.clone();
        let callback_url = self.callback_url.to_owned();
        let intercept_handle = tokio::spawn(async move {
            while let Some(event) = request_paused.next().await {
                let request_url = Url::parse(&event.request.url).unwrap();
                if request_url.origin() == callback_url.origin()
                    && request_url.path() == callback_url.path()
                {
                    log::debug!("Received request to `--callback-url` {}", callback_url);
                    let state = request_url.query_pairs().find(|qp| qp.0.eq("state"));
                    let code = request_url.query_pairs().find(|qp| qp.0.eq("code"));

                    let code = match (state, code) {
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
                    };

                    if let Err(e) = intercept_page
                        .execute(
                            FulfillRequestParams::builder()
                                .request_id(event.request_id.clone())
                                .body(BASE64_STANDARD.encode(if code.is_some() {
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

                    if let Some(code) = code {
                        let _ = tx_browser.send(code);
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

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(timeout)).await;

            let _ = tx_sleep.send("timeout");
        });

        log::debug!("Opening authorization page {}", self.authorization_url);
        page.goto(self.authorization_url.as_str()).await?;
        page.wait_for_navigation().await?;

        let code = tokio::select! {
            _ = rx_sleep => {
                Err::<String, anyhow::Error>(RequestError::Timeout.into())
            }
            Ok(response) = rx_browser => {
                Ok::<String, anyhow::Error>(response)
            }
        };

        browser.close().await?;
        let _ = handle.await;
        let _ = intercept_handle.await;

        code
    }
}
