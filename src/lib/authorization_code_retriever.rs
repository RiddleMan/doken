use crate::lib::args::Arguments;
use crate::lib::oauth_client::OAuthClient;
use crate::lib::server::get_code;
use crate::lib::token_retriever::TokenRetriever;
use crate::TokenInfo;
use async_trait::async_trait;
use std::io;
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

    fn open_token_url(&self) -> io::Result<()> {
        let (url, _) = self.oauth_client.authorize_url(None);
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
impl<'a> TokenRetriever for AuthorizationCodeRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn std::error::Error>> {
        let port = self.args.port;

        self.open_token_url()?;

        let code = get_code(port).await?;

        let token = self.oauth_client.exchange_code(&code, None).await?;

        Ok(TokenInfo::from_token_response(token))
    }
}
