use crate::lib::args::{Arguments, Flow};
use actix_web::{get, http::header, web, App, HttpResponse, HttpServer, Responder};
use reqwest::Url;
use serde::Deserialize;
use std::io;
use std::process::Command;

#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
}

pub struct AuthorizationCodeWithPKCERetriever<'a> {
    code_verifier: Vec<u8>,
    args: &'a Arguments,
}

#[get("/")]
async fn root(auth_params: web::Query<CallbackQuery>) -> impl Responder {
    HttpResponse::Ok()
        .content_type(header::ContentType(mime::TEXT_HTML))
        .body(format!("<!doctype html><html lang=\"en\"><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab. {}</body></html>", auth_params.code))
}

impl<'a> AuthorizationCodeWithPKCERetriever<'a> {
    pub fn new(args: &Arguments) -> AuthorizationCodeWithPKCERetriever {
        AuthorizationCodeWithPKCERetriever {
            code_verifier: pkce::code_verifier(128),
            args,
        }
    }

    fn get_port(&self) -> u16 {
        match self.args.flow {
            Flow::AuthorizationCodeWithPKCE { port } => port,
            _ => unreachable!(),
        }
    }

    // async fn exchange_code(&self) -> io::Result<String> {
    //     // reqwest::get(self.args.token_url)
    // }

    fn open_token_url(&self) -> io::Result<()> {
        let port = self.get_port();
        let mut url = Url::parse(&self.args.authorization_url).unwrap();

        {
            let mut qs = url.query_pairs_mut();

            qs.append_pair("response_type", "code")
                .append_pair("code_challenge", &pkce::code_challenge(&self.code_verifier))
                .append_pair("code_challenge_method", "S256")
                .append_pair("client_id", &self.args.client_id)
                .append_pair("scope", &self.args.scope)
                .append_pair("redirect_uri", &format!("http://localhost:{}", port));

            match &self.args.audience {
                Some(audience) => {
                    qs.append_pair("audience", audience);
                }
                None => {}
            };
        }

        let status = Command::new("open").arg(url.as_str()).status()?;

        if !status.success() {
            panic!("Url couldn't be opened.")
        }

        Ok(())
    }

    pub async fn retrieve(&self) -> io::Result<()> {
        let port = self.get_port();

        let server = HttpServer::new(|| App::new().service(root))
            .bind(("127.0.0.1", port))?
            .run();

        // let server_handle = server.handle();
        self.open_token_url()?;

        server.await
    }
}
