use crate::args::Arguments;
use crate::auth_server::AuthServer;
use crate::lib::token_info::TokenInfo;
use crate::OAuthClient;
use anyhow::Result;
use async_trait::async_trait;
use std::process::Command;

use super::token_retriever::TokenRetriever;

pub struct ImplicitRetriever<'a> {
    args: &'a Arguments,
    oauth_client: &'a OAuthClient<'a>,
}

impl<'a> ImplicitRetriever<'a> {
    pub fn new<'b>(
        args: &'b Arguments,
        oauth_client: &'b OAuthClient<'b>,
    ) -> ImplicitRetriever<'b> {
        ImplicitRetriever { args, oauth_client }
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for ImplicitRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo> {
        let (url, csrf) = self.oauth_client.implicit_url();

        log::debug!("Opening a browser with {url} ...");
        let status = Command::new("open").arg(url.as_str()).status()?;

        if !status.success() {
            panic!("Url couldn't be opened.")
        }

        AuthServer::new(self.args.port)?
            .get_token_data(self.args.timeout, csrf)
            .await
    }
}
