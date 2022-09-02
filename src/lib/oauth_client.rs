use crate::lib::args::Arguments;
use crate::Flow;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use url::Url;

pub struct OAuthClient<'a> {
    args: &'a Arguments,
    inner: BasicClient,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OpenIDProviderMetadata {
    token_endpoint: String,

    authorization_endpoint: String,
}

impl<'a> OAuthClient<'a> {
    fn get_port(args: &Arguments) -> u16 {
        match args.flow {
            Flow::AuthorizationCodeWithPKCE { port } => port,
            Flow::AuthorizationCode { port } => port,
        }
    }

    async fn get_endpoints_from_discovery_url(
        discovery_url: String,
    ) -> Result<(String, String), Box<dyn Error>> {
        let result = reqwest::get(discovery_url.to_owned())
            .await
            .expect("Couldn't reach out to provided `--discovery-url`")
            .json::<OpenIDProviderMetadata>()
            .await
            .expect("Couldn't process json given by `--discovery-url`");

        println!("{:?}", result);

        Ok((result.token_endpoint, result.authorization_endpoint))
    }

    fn get_client(
        args: &Arguments,
        token_url: String,
        authorization_url: String,
    ) -> Result<BasicClient, Box<dyn Error>> {
        let port = Self::get_port(args);

        Ok(BasicClient::new(
            ClientId::new(args.client_id.to_owned()),
            args.client_secret.clone().map(ClientSecret::new),
            AuthUrl::new(authorization_url)?,
            Some(TokenUrl::new(token_url)?),
        )
        .set_redirect_uri(RedirectUrl::new(format!("http://localhost:{}", port)).unwrap()))
    }

    pub async fn new(args: &Arguments) -> Result<OAuthClient, Box<dyn Error>> {
        let client = if let Some(discovery_url) = args.discovery_url.to_owned() {
            let (token_url, authorization_url) =
                Self::get_endpoints_from_discovery_url(discovery_url).await?;

            Self::get_client(args, token_url, authorization_url)
        } else {
            Self::get_client(
                args,
                args.token_url.to_owned().unwrap(),
                args.authorization_url.to_owned().unwrap(),
            )
        }?;

        Ok(OAuthClient {
            args,
            inner: client,
        })
    }

    pub fn authorize_url(&self, pkce_challenge: Option<PkceCodeChallenge>) -> (Url, CsrfToken) {
        let mut builder = self
            .inner
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(self.args.scope.to_string()));

        if let Some(challenge) = pkce_challenge {
            builder = builder.set_pkce_challenge(challenge);
        }

        if let Some(aud) = &self.args.audience {
            builder = builder.add_extra_param("audience", aud);
        }

        builder.url()
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: Option<PkceCodeVerifier>,
    ) -> Result<BasicTokenResponse, Box<dyn Error>> {
        let mut builder = self
            .inner
            .exchange_code(AuthorizationCode::new(code.to_string()));

        if let Some(verifier) = code_verifier {
            builder = builder.set_pkce_verifier(verifier);
        }

        let token = builder.request_async(async_http_client).await?;

        Ok(token)
    }

    pub async fn refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<BasicTokenResponse, Box<dyn Error>> {
        let refresh_token = RefreshToken::new(refresh_token);

        Ok(self
            .inner
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await?)
    }
}
