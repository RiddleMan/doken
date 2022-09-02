use std::str::FromStr;
use tiny_http::{Header, Response, Server as TinyServer};
use url::Url;

pub async fn get_code(port: u16) -> Result<String, Box<dyn std::error::Error>> {
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
