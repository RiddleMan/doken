use crate::lib::args::Arguments;
use crate::lib::auth_server::AuthServer;
use crate::lib::token_retriever::TokenRetriever;
use crate::{OAuthClient, TokenInfo};
use async_trait::async_trait;
use std::error::Error;
use std::process::Command;

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
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn Error>> {
        let (url, _) = self.oauth_client.implicit_url();

        log::debug!("Opening a browser...");
        let status = Command::new("open").arg(url.as_str()).status()?;

        if !status.success() {
            panic!("Url couldn't be opened.")
        }

        AuthServer::new(self.args.port).get_token_data().await
    }
}