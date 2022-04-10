use crate::lib::args::{Arguments, Flow};
use actix_web::{get, http::header, App, HttpResponse, HttpServer, Responder};
use std::io;

pub struct AuthorizationCodeWithPKCERetriever;

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok()
        .content_type(header::ContentType(mime::TEXT_HTML))
        .body("<!doctype html><html lang=\"en\"><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab.</body></html>")
}

impl AuthorizationCodeWithPKCERetriever {
    pub async fn retrieve(args: Arguments) -> io::Result<()> {
        let port = match args.flow {
            Flow::AuthorizationCodeWithPKCE { port } => port,
            _ => unreachable!(),
        };

        // Show an error if port is occupied
        HttpServer::new(|| App::new().service(root))
            .bind(("127.0.0.1", port))?
            .run()
            .await
    }
}
