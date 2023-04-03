use crate::args::Arguments;
use crate::auth_server::AuthServer;
use crate::oauth_client::OAuthClient;
use crate::open_authorization_url::open_authorization_url;
use crate::token_info::TokenInfo;
use anyhow::Result;
use async_trait::async_trait;
use oauth2::PkceCodeChallenge;

use super::token_retriever::TokenRetriever;

pub struct AuthorizationCodeWithPKCERetriever<'a> {
    oauth_client: &'a OAuthClient<'a>,
    args: &'a Arguments,
}

impl<'a> AuthorizationCodeWithPKCERetriever<'a> {
    pub fn new<'b>(
        args: &'b Arguments,
        oauth_client: &'b OAuthClient<'b>,
    ) -> AuthorizationCodeWithPKCERetriever<'b> {
        AuthorizationCodeWithPKCERetriever { oauth_client, args }
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for AuthorizationCodeWithPKCERetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (url, csrf) = self.oauth_client.authorize_url(Some(pkce_challenge));
        open_authorization_url(url.as_str(), &self.args.callback_url)?;

        let code = AuthServer::new(&self.args.callback_url)?
            .get_code(self.args.timeout, csrf)
            .await?;

        let token = self
            .oauth_client
            .exchange_code(&code, Some(pkce_verifier))
            .await?;

        return Ok(TokenInfo::from_token_response(token));
    }
}
