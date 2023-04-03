use crate::args::Arguments;
use crate::auth_server::AuthServer;
use crate::oauth_client::OAuthClient;
use crate::open_authorization_url::open_authorization_url;
use crate::token_info::TokenInfo;
use anyhow::Result;
use async_trait::async_trait;

use super::token_retriever::TokenRetriever;

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
        open_authorization_url(url.as_str(), &self.args.callback_url)?;

        let code = AuthServer::new(&self.args.callback_url)?
            .get_code(self.args.timeout, csrf)
            .await?;

        let token = self.oauth_client.exchange_code(&code, None).await?;

        Ok(TokenInfo::from_token_response(token))
    }
}
