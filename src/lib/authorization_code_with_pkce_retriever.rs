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

pub struct AuthorizationCodeWithPKCERetriever;

#[get("/")]
async fn root(auth_params: web::Query<CallbackQuery>) -> impl Responder {
    HttpResponse::Ok()
        .content_type(header::ContentType(mime::TEXT_HTML))
        .body(format!("<!doctype html><html lang=\"en\"><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab. {}</body></html>", auth_params.code))
}

impl AuthorizationCodeWithPKCERetriever {
    fn get_port(args: &Arguments) -> u16 {
        match args.flow {
            Flow::AuthorizationCodeWithPKCE { port } => port,
            _ => unreachable!(),
        }
    }

    fn open_token_url(args: &Arguments) -> io::Result<()> {
        let port = Self::get_port(args);
        let mut url = Url::parse(&args.authorization_url).unwrap();
        let verifier = pkce::code_verifier(128);

        {
            let mut qs = url.query_pairs_mut();

            qs.append_pair("response_type", "code")
                .append_pair("code_challenge", &pkce::code_challenge(&verifier))
                .append_pair("code_challenge_method", "S256")
                .append_pair("client_id", &args.client_id)
                .append_pair("scope", &args.scope)
                .append_pair("redirect_uri", &format!("http://localhost:{}", port));

            match &args.audience {
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

    pub async fn retrieve(args: &Arguments) -> io::Result<()> {
        let port = Self::get_port(args);

        let server = HttpServer::new(|| App::new().service(root))
            .bind(("127.0.0.1", port))?
            .run();

        // let server_handle = server.handle();
        Self::open_token_url(args)?;

        server.await
    }
}
