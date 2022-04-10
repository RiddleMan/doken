use crate::lib::args::{Arguments, Flow};
use actix_web::{get, http::header, App, HttpResponse, HttpServer, Responder};
use clap::Arg;
use reqwest::Url;
use std::io;
use std::process::Command;

pub struct AuthorizationCodeWithPKCERetriever;

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok()
        .content_type(header::ContentType(mime::TEXT_HTML))
        .body("<!doctype html><html lang=\"en\"><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab.</body></html>")
}

impl AuthorizationCodeWithPKCERetriever {
    fn get_port(args: &Arguments) -> u16 {
        match args.flow {
            Flow::AuthorizationCodeWithPKCE { port } => port,
            _ => unreachable!(),
        }
    }

    fn open_token_url(args: &Arguments) {
        let port = Self::get_port(args);
        let mut url = Url::parse(&args.issuer_url).unwrap();
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

        // Command::new("open")
        //     .arg("")
        println!("{}", url.as_str());
    }
    pub async fn retrieve(args: &Arguments) -> io::Result<()> {
        let port = Self::get_port(args);

        let server = HttpServer::new(|| App::new().service(root))
            .bind(("127.0.0.1", port))?
            .run();

        // let server_handle = server.handle();
        Self::open_token_url(args);

        server.await
    }
}
