use anyhow::{anyhow, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::target::CreateTargetParamsBuilder;
use chromiumoxide::handler::viewport::Viewport;
use chromiumoxide::{Handler, Page};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::{oneshot, OnceCell};

use super::auth_page::AuthPage;

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
        let page = AuthPage::new(browser_page);
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
