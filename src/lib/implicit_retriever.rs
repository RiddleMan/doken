use crate::lib::args::Arguments;
use crate::lib::server::get_token_data;
use crate::lib::token_retriever::TokenRetriever;
use crate::{lib, TokenInfo};
use async_trait::async_trait;
use std::error::Error;
use std::process::Command;
use url::Url;

pub struct ImplicitRetriever<'a> {
    args: &'a Arguments,
}

impl<'a> ImplicitRetriever<'a> {
    pub fn new(args: &Arguments) -> ImplicitRetriever {
        ImplicitRetriever { args }
    }

    async fn get_authorization_url(&self) -> Result<String, Box<dyn Error>> {
        let authorization_url = if let Some(discovery_url) = self.args.discovery_url.to_owned() {
            log::debug!(
                "Using `--discovery-url`={} to get token_url and authorization_url ",
                discovery_url
            );

            lib::openidc_discovery::get_endpoints_from_discovery_url(discovery_url)
                .await?
                .1
        } else {
            self.args.authorization_url.to_owned().unwrap()
        };

        log::debug!("Resolved authorization_url={}", authorization_url);

        Ok(authorization_url)
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for ImplicitRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn Error>> {
        let authorization_url = self.get_authorization_url().await?;

        let mut url = Url::parse(authorization_url.as_str())?;

        url.query_pairs_mut()
            .append_pair("response_type", "token")
            .append_pair("response_mode", "form_post")
            .append_pair("client_id", self.args.client_id.as_str())
            .append_pair(
                "redirect_uri",
                format!("http://localhost:{}/", self.args.port).as_str(),
            )
            .append_pair("scope", self.args.scope.as_str());

        if let Some(ref audience) = self.args.audience {
            url.query_pairs_mut().append_pair("audience", audience);
        }

        log::debug!("Opening a browser...");
        let status = Command::new("open").arg(url.as_str()).status()?;

        if !status.success() {
            panic!("Url couldn't be opened.")
        }

        get_token_data(self.args.port).await
    }
}
