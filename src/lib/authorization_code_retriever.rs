use crate::lib::args::{Arguments, Flow};
use crate::lib::oauth_client::OAuthClient;
use crate::lib::token_retriever::TokenRetriever;
use crate::TokenInfo;
use async_trait::async_trait;
use reqwest::Url;
use std::io;
use std::process::Command;
use std::str::FromStr;
use tiny_http::{Header, Response, Server};

pub struct AuthorizationCodeRetriever<'a> {
    oauth_client: OAuthClient<'a>,
    args: &'a Arguments,
}

impl<'a> AuthorizationCodeRetriever<'a> {
    pub fn new(
        args: &Arguments,
    ) -> Result<AuthorizationCodeRetriever, Box<dyn std::error::Error>> {
        Ok(AuthorizationCodeRetriever {
            oauth_client: OAuthClient::new(args)?,
            args,
        })
    }

    fn get_port(args: &Arguments) -> u16 {
        match args.flow {
            Flow::AuthorizationCode { port } => port,
            _ => unreachable!(),
        }
    }

    fn open_token_url(&self) -> io::Result<()> {
        let (url, _) = self.oauth_client.authorize_url(None);

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

        let server = Server::http(format!("127.0.0.1:{}", port)).unwrap();

        self.open_token_url()?;

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
                        .exchange_code(&code, None)
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
