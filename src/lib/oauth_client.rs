use crate::lib::args::Arguments;
use crate::Flow;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenUrl,
};
use std::error::Error;
use url::Url;

pub struct OAuthClient<'a> {
    args: &'a Arguments,
    inner: BasicClient,
}

impl<'a> OAuthClient<'a> {
    fn get_port(args: &Arguments) -> u16 {
        match args.flow {
            Flow::AuthorizationCodeWithPKCE { port } => port,
            _ => unreachable!(),
        }
    }

    pub fn new(args: &Arguments) -> Result<OAuthClient, Box<dyn Error>> {
        let port = Self::get_port(args);

        let client = BasicClient::new(
            ClientId::new(args.client_id.to_string()),
            args.client_secret.clone().map(ClientSecret::new),
            AuthUrl::new(args.authorization_url.to_string())?,
            Some(TokenUrl::new(args.token_url.to_string())?),
        )
        .set_redirect_uri(RedirectUrl::new(format!("http://localhost:{}", port)).unwrap());

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
