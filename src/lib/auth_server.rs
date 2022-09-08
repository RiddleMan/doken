use crate::TokenInfo;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tiny_http::{Header, Method, Request, Response, Server as TinyServer};
use tokio::sync::oneshot;
use url::Url;

#[derive(Debug)]
struct TokenError {}

impl Display for TokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Can't get a token")
    }
}

impl Error for TokenError {}

pub struct AuthServer {
    server: Arc<TinyServer>,
}

impl AuthServer {
    pub fn new(port: u16) -> AuthServer {
        log::debug!("Creating http server on port {}", port);
        let server = TinyServer::http(format!("127.0.0.1:{}", port)).unwrap();

        log::info!("Waiting for connections...");
        AuthServer {
            server: Arc::new(server),
        }
    }

    fn response_with_default_message(request: Request) -> Result<(), Box<dyn Error>> {
        let html_header = Header::from_str("Content-Type: text/html; charset=UTF-8").unwrap();
        let mut response = Response::from_string("<!doctype html><html lang=\"en\"><script>window.close();</script><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab.</body></html>");
        response.add_header(html_header);

        log::debug!("Responding to the user browser..");
        request.respond(response)?;
        Ok(())
    }

    pub async fn get_code(&self, timeout: u64) -> Result<String, Box<dyn Error>> {
        let (tx_server, rx_server) = oneshot::channel();
        let (tx_sleep, rx_sleep) = oneshot::channel();
        let server = self.server.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(timeout)).await;

            let _ = tx_sleep.send("timeout");
        });

        tokio::spawn(async move {
            for request in server.incoming_requests() {
                log::debug!("Request received");
                let url =
                    Url::parse(format!("http://localhost{}", request.url()).as_str()).unwrap();
                let code = url.query_pairs().find(|qp| qp.0.eq("code"));

                match code {
                    Some(x) => {
                        let code = x.1.to_string();
                        log::debug!("Given code {}", code);

                        Self::response_with_default_message(request).unwrap();

                        let _ = tx_server.send(code);
                        break;
                    }
                    None => {
                        log::debug!("Call to server without a code parameter. Ignoring...");
                        println!("Ignoring");
                    }
                }
            }
        });

        tokio::select! {
            _ = rx_sleep => {
                self.server.unblock();
                Err::<String, Box<dyn Error>>(Box::new(TokenError {}))
            }
            Ok(code) = rx_server => {
                Ok::<String, Box<dyn Error>>(code)
            }
        }
    }

    pub async fn get_token_data(&self, timeout: u64) -> Result<TokenInfo, Box<dyn Error>> {
        let (tx_server, rx_server) = oneshot::channel();
        let (tx_sleep, rx_sleep) = oneshot::channel();
        let mut body = String::new();
        let server = self.server.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(timeout)).await;

            let _ = tx_sleep.send("timeout");
        });

        tokio::spawn(async move {
            let mut token_info = TokenInfo::new();
            for mut request in server.incoming_requests() {
                log::debug!("Request received");

                match request.method() {
                    Method::Post => {
                        request.as_reader().read_to_string(&mut body).unwrap();

                        Self::response_with_default_message(request).unwrap();

                        let form_params =
                            form_urlencoded::parse(body.as_bytes())
                                .collect::<Vec<(Cow<str>, Cow<str>)>>();

                        token_info.access_token = form_params
                            .iter()
                            .find(|(name, _value)| name == "access_token")
                            .expect("Cannot find access_token in the HTTP Post request.")
                            .1
                            .to_string();

                        token_info.expires = Some(
                            SystemTime::now().add(Duration::from_secs(
                                form_params
                                    .iter()
                                    .find(|(name, _value)| name == "expires_in")
                                    .expect("Cannot find expires_in in the HTTP Post request.")
                                    .1
                                    .parse::<u64>()
                                    .expect("expires_in is an incorrect number"),
                            )),
                        );
                        break;
                    }
                    _ => {
                        log::debug!("Call to server without a code parameter. Ignoring...");
                        println!("Ignoring");
                    }
                }
            }

            let _ = tx_server.send(token_info);
        });

        tokio::select! {
            _ = rx_sleep => {
                self.server.unblock();
                Err::<TokenInfo, Box<dyn Error>>(Box::new(TokenError {}))
            }
            Ok(token_info) = rx_server => {
                Ok::<TokenInfo, Box<dyn Error>>(token_info)
            }
        }
    }
}
