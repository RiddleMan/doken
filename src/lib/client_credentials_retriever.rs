use crate::lib::args::Arguments;
use crate::lib::token_retriever::TokenRetriever;
use crate::{lib, TokenInfo};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::ops::Add;
use std::time::{Duration, SystemTime};

#[derive(Serialize, Deserialize)]
struct TokenEndpointResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

pub struct ClientCredentialsRetriever<'a> {
    args: &'a Arguments,
}

impl<'a> ClientCredentialsRetriever<'a> {
    pub fn new(args: &Arguments) -> ClientCredentialsRetriever {
        ClientCredentialsRetriever { args }
    }

    async fn resolve_token_url(&self) -> Result<String, Box<dyn Error>> {
        let token_url: String = if let Some(discovery_url) = self.args.discovery_url.to_owned() {
            log::debug!(
                "Using `--discovery-url`={} to get token_url and authorization_url ",
                discovery_url
            );

            lib::openidc_discovery::get_endpoints_from_discovery_url(discovery_url)
                .await?
                .0
        } else {
            self.args.token_url.to_owned().unwrap()
        };

        log::debug!("Resolved token_url={}", token_url);

        Ok(token_url)
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for ClientCredentialsRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn Error>> {
        let token_url = self.resolve_token_url().await?;
        let mut params = vec![
            ("grant_type", "client_credentials"),
            ("client_id", self.args.client_id.as_str()),
            ("client_secret", self.args.client_secret.as_deref().unwrap()),
            ("scope", self.args.scope.as_str()),
        ];

        if let Some(ref audience) = self.args.audience {
            params.push(("audience", audience));
        }

        let json = reqwest::Client::new()
            .post(token_url)
            .form(&params)
            .send()
            .await
            .expect("Couldn't exchange credentials to a token provided by `--token-url`")
            .error_for_status()?
            .json::<TokenEndpointResponse>()
            .await
            .expect("Couldn't process json given by --token-url");

        Ok(TokenInfo {
            access_token: json.access_token,
            expires: Some(SystemTime::now().add(Duration::from_secs(json.expires_in))),
            refresh_token: None,
            scope: None,
        })
    }
}
