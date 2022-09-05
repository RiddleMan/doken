use crate::lib::args::Arguments;
use crate::lib::auth_server::AuthServer;
use crate::lib::oauth_client::OAuthClient;
use crate::lib::token_retriever::TokenRetriever;
use crate::TokenInfo;
use async_trait::async_trait;
use oauth2::PkceCodeChallenge;
use std::io;
use std::process::Command;

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

    fn open_token_url(&self, pkce_challenge: PkceCodeChallenge) -> io::Result<()> {
        let (url, _) = self.oauth_client.authorize_url(Some(pkce_challenge));
        log::debug!("Using `{}` url to initiate user session", url);

        log::debug!("Opening a browser...");
        let status = Command::new("open").arg(url.as_str()).status()?;

        if !status.success() {
            panic!("Url couldn't be opened.")
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for AuthorizationCodeWithPKCERetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn std::error::Error>> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        self.open_token_url(pkce_challenge)?;

        let code = AuthServer::new(self.args.port).get_code()?;

        let token = self
            .oauth_client
            .exchange_code(&code, Some(pkce_verifier))
            .await?;

        return Ok(TokenInfo::from_token_response(token));
    }
}
