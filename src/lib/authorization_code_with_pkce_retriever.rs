use crate::lib::args::{Arguments, Flow};
use crate::lib::oauth_client::OAuthClient;
use crate::lib::token_retriever::TokenRetriever;
use crate::TokenInfo;
use async_trait::async_trait;
use oauth2::PkceCodeChallenge;
use reqwest::Url;
use std::io;
use std::process::Command;
use std::str::FromStr;
use tiny_http::{Header, Response, Server};

pub struct AuthorizationCodeWithPKCERetriever<'a> {
    oauth_client: OAuthClient<'a>,
    args: &'a Arguments,
}

impl<'a> AuthorizationCodeWithPKCERetriever<'a> {
    pub fn new(
        args: &Arguments,
    ) -> Result<AuthorizationCodeWithPKCERetriever, Box<dyn std::error::Error>> {
        Ok(AuthorizationCodeWithPKCERetriever {
            oauth_client: OAuthClient::new(args)?,
            args,
        })
    }

    fn get_port(args: &Arguments) -> u16 {
        match args.flow {
            Flow::AuthorizationCodeWithPKCE { port } => port,
            _ => unreachable!(),
        }
    }

    fn open_token_url(&self, pkce_challenge: PkceCodeChallenge) -> io::Result<()> {
        let (url, _) = self.oauth_client.authorize_url(Some(pkce_challenge));

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
        let port = Self::get_port(self.args);

        let server = Server::http(format!("127.0.0.1:{}", port)).unwrap();
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        self.open_token_url(pkce_challenge)?;

        for request in server.incoming_requests() {
            let url = Url::parse(format!("http://localhost{}", request.url()).as_str()).unwrap();
            let code = url.query_pairs().find(|qp| qp.0.eq("code"));

            match code {
                Some(x) => {
                    let code = x.1.to_string();

                    let html_header =
                        Header::from_str("Content-Type: text/html; charset=UTF-8").unwrap();
                    let mut response = Response::from_string("<!doctype html><html lang=\"en\"><script>window.close();</script><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab.</body></html>");
                    response.add_header(html_header);

                    request.respond(response)?;

                    let token = self
                        .oauth_client
                        .exchange_code(&code, Some(pkce_verifier))
                        .await?;

                    return Ok(TokenInfo::from_token_response(token));
                }
                None => {
                    println!("Ignoring");
                }
            }
        }

        panic!("Cannot get token")
    }
}
