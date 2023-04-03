use crate::args::Arguments;
use crate::auth_server::AuthServer;
use crate::lib::oauth_client::OAuthClient;
use crate::lib::token_retriever::TokenRetriever;
use crate::TokenInfo;
use anyhow::Result;
use async_trait::async_trait;
use std::process::Command;

pub struct AuthorizationCodeRetriever<'a> {
    oauth_client: &'a OAuthClient<'a>,
    args: &'a Arguments,
}

impl<'a> AuthorizationCodeRetriever<'a> {
    pub fn new<'b>(
        args: &'b Arguments,
        oauth_client: &'b OAuthClient<'b>,
    ) -> AuthorizationCodeRetriever<'b> {
        AuthorizationCodeRetriever { oauth_client, args }
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for AuthorizationCodeRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo> {
        let (url, csrf) = self.oauth_client.authorize_url(None);
        log::debug!("Using `{}` url to initiate user session", url);

        log::debug!("Opening a browser with {url} ...");
        let status = Command::new("open").arg(url.as_str()).status()?;

        if !status.success() {
            panic!("Url couldn't be opened.")
        }

        let code = AuthServer::new(self.args.port)?
            .get_code(self.args.timeout, csrf)
            .await?;

        let token = self.oauth_client.exchange_code(&code, None).await?;

        Ok(TokenInfo::from_token_response(token))
    }
}
