use crate::TokenInfo;
use std::borrow::Cow;
use std::error::Error;
use std::ops::Add;
use std::str::FromStr;
use std::time::{Duration, UNIX_EPOCH};
use tiny_http::{Header, Method, Response, Server as TinyServer};
use url::Url;

// TODO: Refactor
pub async fn get_code(port: u16) -> Result<String, Box<dyn Error>> {
    log::debug!("Creating http server on port {}", port);
    let server = TinyServer::http(format!("127.0.0.1:{}", port)).unwrap();

    log::info!("Waiting for connections...");
    eprintln!("Opening browser and waiting for a authorization flow completion...");
    for request in server.incoming_requests() {
        log::debug!("Request received");
        let url = Url::parse(format!("http://localhost{}", request.url()).as_str()).unwrap();
        let code = url.query_pairs().find(|qp| qp.0.eq("code"));

        match code {
            Some(x) => {
                let code = x.1.to_string();
                log::debug!("Given code {}", code);

                let html_header =
                    Header::from_str("Content-Type: text/html; charset=UTF-8").unwrap();
                let mut response = Response::from_string("<!doctype html><html lang=\"en\"><script>window.close();</script><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab.</body></html>");
                response.add_header(html_header);

                log::debug!("Responding to the user browser..");
                request.respond(response)?;

                return Ok(code);
            }
            None => {
                log::debug!("Call to server without a code parameter. Ignoring...");
                println!("Ignoring");
            }
        }
    }

    panic!("Cannot get token")
}

pub async fn get_token_data(port: u16) -> Result<TokenInfo, Box<dyn Error>> {
    log::debug!("Creating http server on port {}", port);
    let server = TinyServer::http(format!("127.0.0.1:{}", port)).unwrap();

    log::info!("Waiting for connections...");
    eprintln!("Opening browser and waiting for a authorization flow completion...");
    for mut request in server.incoming_requests() {
        log::debug!("Request received");

        match request.method() {
            Method::Post => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body)?;

                let form_fields =
                    form_urlencoded::parse(body.as_bytes()).collect::<Vec<(Cow<str>, Cow<str>)>>();

                let html_header =
                    Header::from_str("Content-Type: text/html; charset=UTF-8").unwrap();
                let mut response = Response::from_string("<!doctype html><html lang=\"en\"><script>window.close();</script><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab.</body></html>");
                response.add_header(html_header);

                log::debug!("Responding to the user browser..");
                request.respond(response)?;

                return Ok(TokenInfo {
                    access_token: form_fields
                        .iter()
                        .find(|(name, _value)| name == "access_token")
                        .expect("Cannot find access_token in the HTTP Post request.")
                        .1
                        .to_string(),
                    refresh_token: None,
                    expires: Some(
                        UNIX_EPOCH.add(Duration::from_secs(
                            form_fields
                                .iter()
                                .find(|(name, _value)| name == "expires_in")
                                .expect("Cannot find expires_in in the HTTP Post request.")
                                .1
                                .parse::<u64>()
                                .expect("expires_in is an incorrect number"),
                        )),
                    ),
                    scope: None,
                });
            }
            _ => {
                log::debug!("Call to server without a code parameter. Ignoring...");
                println!("Ignoring");
            }
        }
    }

    panic!("Cannot get token")
}
