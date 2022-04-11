use crate::lib::args::{Arguments, Flow};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
};
use reqwest::Url;
use std::io;
use std::process::Command;
use std::str::FromStr;
use tiny_http::{Header, Response, Server};

pub struct AuthorizationCodeWithPKCERetriever<'a> {
    oauth2_client: oauth2::basic::BasicClient,
    args: &'a Arguments,
}

impl<'a> AuthorizationCodeWithPKCERetriever<'a> {
    pub fn new(
        args: &Arguments,
    ) -> Result<AuthorizationCodeWithPKCERetriever, Box<dyn std::error::Error>> {
        let port = Self::get_port(args);

        let client = BasicClient::new(
            ClientId::new(args.client_id.to_string()),
            args.client_secret.clone().map(ClientSecret::new),
            AuthUrl::new(args.authorization_url.to_string())?,
            Some(TokenUrl::new(args.token_url.to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(format!("http://localhost:{}", port)).unwrap());

        Ok(AuthorizationCodeWithPKCERetriever {
            oauth2_client: client,
            args,
        })
    }

    fn get_port(args: &Arguments) -> u16 {
        match args.flow {
            Flow::AuthorizationCodeWithPKCE { port } => port,
            _ => unreachable!(),
        }
    }

    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: PkceCodeVerifier,
    ) -> Result<BasicTokenResponse, Box<dyn std::error::Error>> {
        let token = self
            .oauth2_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(code_verifier)
            .request_async(async_http_client)
            .await?;

        Ok(token)
    }

    fn open_token_url(&self, pkce_challenge: PkceCodeChallenge) -> io::Result<()> {
        let (url, _) = self
            .oauth2_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(self.args.scope.to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let status = Command::new("open").arg(url.as_str()).status()?;

        if !status.success() {
            panic!("Url couldn't be opened.")
        }

        Ok(())
    }

    pub async fn retrieve(&self) -> Result<BasicTokenResponse, Box<dyn std::error::Error>> {
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
                    let mut response = Response::from_string("<!doctype html><html lang=\"en\"><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab.</body></html>");
                    response.add_header(html_header);

                    request.respond(response)?;

                    let token = self.exchange_code(&code, pkce_verifier).await?;

                    return Ok(token);
                }
                None => {
                    println!("Ignoring");
                }
            }
        }

        panic!("Cannot get token")
    }
}
