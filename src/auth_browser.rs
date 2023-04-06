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
use url::Url;

const CONTENT: &str = "<html><head></head><body><h1>OK</h1></body></html>";

pub struct AuthBrowser {
    authorization_url: Url,
    _callback_url: Url,
}

impl AuthBrowser {
    pub fn new(authorization_url: Url, _callback_url: Url) -> Result<AuthBrowser> {
        Ok(AuthBrowser {
            authorization_url,
            _callback_url,
        })
    }

    pub async fn get_code(&self, _timeout: u64, _csrf: CsrfToken) -> Result<String> {
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

        // Setup request interception
        log::debug!("Opening authorization page {}", self.authorization_url);
        let page = Arc::new(browser.new_page("about:blank").await?);

        let mut request_paused = page.event_listener::<EventRequestPaused>().await.unwrap();
        let intercept_page = page.clone();
        let callback_url = self._callback_url.to_owned();
        let intercept_handle = tokio::spawn(async move {
            while let Some(event) = request_paused.next().await {
                log::debug!("Request url: {}", event.request.url);
                let request_url = Url::parse(&event.request.url).unwrap();
                if request_url.origin() == callback_url.origin()
                    && request_url.path() == callback_url.path()
                {
                    if let Err(e) = intercept_page
                        .execute(
                            FulfillRequestParams::builder()
                                .request_id(event.request_id.clone())
                                .body(BASE64_STANDARD.encode(CONTENT))
                                .response_code(200)
                                .build()
                                .unwrap(),
                        )
                        .await
                    {
                        println!("Failed to fullfill request: {e}");
                    }
                } else if let Err(e) = intercept_page
                    .execute(ContinueRequestParams::new(event.request_id.clone()))
                    .await
                {
                    println!("Failed to continue request: {e}");
                }
            }
        });

        page.goto(self.authorization_url.as_str()).await?;
        page.wait_for_navigation().await?;

        tokio::time::sleep(Duration::from_millis(30_000)).await;

        browser.close().await?;
        let _ = handle.await;
        let _ = intercept_handle.await;

        Ok(String::new())
    }
}
