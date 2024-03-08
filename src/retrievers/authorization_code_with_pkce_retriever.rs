use crate::args::Arguments;
use crate::auth_browser::auth_browser::AuthPage;
use crate::oauth_client::OAuthClient;
use crate::token_info::TokenInfo;
use anyhow::Result;
use async_trait::async_trait;
use oauth2::PkceCodeChallenge;
use url::Url;

use super::token_retriever::TokenRetriever;

pub struct AuthorizationCodeWithPKCERetriever<'a> {
    oauth_client: &'a OAuthClient<'a>,
    auth_page: AuthPage,
    args: &'a Arguments,
}

impl<'a> AuthorizationCodeWithPKCERetriever<'a> {
    pub fn new<'b>(
        args: &'b Arguments,
        oauth_client: &'b OAuthClient<'b>,
        auth_page: AuthPage,
    ) -> AuthorizationCodeWithPKCERetriever<'b> {
        AuthorizationCodeWithPKCERetriever {
            oauth_client,
            auth_page,
            args,
        }
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for AuthorizationCodeWithPKCERetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (url, csrf, _nonce) = self.oauth_client.authorize_url(Some(pkce_challenge));

        let code = self
            .auth_page
            .get_code(
                self.args.timeout,
                url,
                Url::parse(self.args.callback_url.as_deref().unwrap())?,
                csrf,
            )
            .await?;

        let token = self
            .oauth_client
            .exchange_code(&code, Some(pkce_verifier))
            .await?;

        return Ok(TokenInfo::from_token_response(token));
    }
}
