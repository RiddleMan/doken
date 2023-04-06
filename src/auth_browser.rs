use anyhow::{anyhow, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use oauth2::CsrfToken;
use std::time::Duration;
use url::Url;

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

        log::debug!("Opening authorization page {}", self.authorization_url);
        let _ = browser.new_page(self.authorization_url.as_str()).await?;

        tokio::time::sleep(Duration::from_millis(30_000)).await;

        browser.close().await?;
        handle.await?;

        Ok(String::new())
    }
}
