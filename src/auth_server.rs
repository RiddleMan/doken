use crate::TokenInfo;
use anyhow::{anyhow, Context, Result};
use oauth2::CsrfToken;
use std::borrow::Cow;
use std::ops::Add;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tiny_http::{Header, Method, Request, Response, Server as TinyServer};
use tokio::sync::oneshot;
use url::Url;

#[derive(Error, Debug)]
enum ServerError {
    #[error("No requests with required data. Timeout.")]
    Timeout,
}

pub struct AuthServer {
    server: Arc<TinyServer>,
}

impl AuthServer {
    pub fn new(callback_url: &str) -> Result<AuthServer> {
        let url = Url::parse(callback_url)?;
        let port = url.port_or_known_default().unwrap();
        log::debug!("Creating http server on port {}", port);
        let server = TinyServer::http(format!("127.0.0.1:{}", port))
            .map_err(|e| anyhow!(e))
            .with_context(|| {
            let base_str = format!("Couldn't create a server on port: {}.", port);
            if port <= 1024 {
                format!("{}\nYou're trying to listen on restricted port. Make sure you have required permissions or run it using root privileges `sudo doken`", base_str)
            } else {
                base_str
            }
        })?;

        log::info!("Waiting for connections...");
        Ok(AuthServer {
            server: Arc::new(server),
        })
    }

    fn response_with_default_message(request: Request) -> Result<()> {
        let html_header = Header::from_str("Content-Type: text/html; charset=UTF-8").unwrap();
        let mut response = Response::from_string("<!doctype html><html lang=\"en\"><script>window.close();</script><head><meta charset=utf-8><title>Doken</title></head><body>Successfully signed in. Close current tab.</body></html>");
        response.add_header(html_header);

        log::debug!("Responding to the user browser..");
        request.respond(response)?;
        Ok(())
    }

    async fn process_request<TResponse, F>(&self, timeout: u64, f: F) -> Result<TResponse>
    where
        TResponse: Send + Clone + Sync + 'static,
        F: Send + Fn(Request) -> Option<TResponse> + 'static,
    {
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

                match f(request) {
                    Some(response) => {
                        let _ = tx_server.send(response);
                        break;
                    }
                    None => {
                        log::debug!("Unsupported request. Ignoring...");
                    }
                }
            }
        });

        tokio::select! {
            _ = rx_sleep => {
                self.server.unblock();
                Err::<TResponse, anyhow::Error>(ServerError::Timeout.into())
            }
            Ok(response) = rx_server => {
                Ok::<TResponse, anyhow::Error>(response)
            }
        }
    }

    pub async fn get_code(&self, timeout: u64, csrf_token: CsrfToken) -> Result<String> {
        self.process_request(timeout, move |request| {
            let url = Url::parse(format!("http://localhost{}", request.url()).as_str()).unwrap();
            let state = url.query_pairs().find(|qp| qp.0.eq("state"));
            let code = url.query_pairs().find(|qp| qp.0.eq("code"));

            match (state, code) {
                (Some((_, state)), Some((_, code))) => {
                    if state == *csrf_token.secret() {
                        let code = code.to_string();
                        log::debug!("Given code {}", code);

                        Self::response_with_default_message(request).unwrap();

                        Some(code)
                    } else {
                        log::debug!("Incorrect CSRF token. Ignoring...");

                        None
                    }
                }
                _ => {
                    log::debug!(
                        "Call to server without a state and/or a code parameter. Ignoring..."
                    );

                    None
                }
            }
        })
        .await
    }

    pub async fn get_token_data(&self, timeout: u64, csrf_token: CsrfToken) -> Result<TokenInfo> {
        self.process_request(timeout, move |mut request| {
            let mut body = String::new();
            match request.method() {
                Method::Post => {
                    request.as_reader().read_to_string(&mut body).unwrap();

                    let form_params =
                        form_urlencoded::parse(body.as_bytes())
                            .collect::<Vec<(Cow<str>, Cow<str>)>>();

                    let (_, access_token) = form_params
                        .iter()
                        .find(|(name, _value)| name == "access_token")
                        .expect("Cannot find access_token in the HTTP Post request.");

                    let (_, expires_in) = form_params
                        .iter()
                        .find(|(name, _value)| name == "expires_in")
                        .expect("Cannot find expires_in in the HTTP Post request.");

                    let (_, state) = form_params
                        .iter()
                        .find(|(name, _value)| name == "state")
                        .expect("Cannot find state in the HTTP Post request.");

                    if state == csrf_token.secret() {
                        Self::response_with_default_message(request).unwrap();

                        Some(TokenInfo {
                            access_token: access_token.to_string(),
                            refresh_token: None,
                            expires: Some(
                                SystemTime::now().add(Duration::from_secs(
                                    expires_in
                                        .parse::<u64>()
                                        .expect("expires_in is an incorrect number"),
                                )),
                            ),
                            scope: None,
                        })
                    } else {
                        log::debug!("Incorrect CSRF token. Ignoring...");

                        None
                    }
                }
                _ => {
                    log::debug!(
                        "Call to server without a state and/or a code parameter. Ignoring..."
                    );

                    None
                }
            }
        })
        .await
    }
}