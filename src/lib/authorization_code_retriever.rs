use crate::lib::args::{Arguments, Flow};
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
    pub async fn new<'b>(
        args: &'b Arguments,
        oauth_client: &'b OAuthClient<'b>,
    ) -> Result<AuthorizationCodeRetriever<'b>, Box<dyn std::error::Error>> {
        Ok(AuthorizationCodeRetriever { oauth_client, args })
    }

    fn get_port(args: &Arguments) -> u16 {
        match args.flow {
            Flow::AuthorizationCode { port } => port,
            _ => unreachable!(),
        }
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
        let port = Self::get_port(self.args);

        self.open_token_url()?;

        let code = get_code(port).await?;

        let token = self.oauth_client.exchange_code(&code, None).await?;

        return Ok(TokenInfo::from_token_response(token));
    }
}
