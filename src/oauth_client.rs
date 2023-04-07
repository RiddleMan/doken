use crate::args::Arguments;
use crate::openidc_discovery::get_endpoints_from_discovery_url;
use anyhow::{Context, Result};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, AuthorizationRequest, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, ResourceOwnerPassword,
    ResourceOwnerUsername, Scope, TokenUrl,
};
use url::Url;

pub struct OAuthClient<'a> {
    args: &'a Arguments,
    inner: BasicClient,
}
impl<'a> OAuthClient<'a> {
    fn get_client(
        args: &Arguments,
        token_url: Option<&str>,
        authorization_url: &str,
    ) -> Result<BasicClient> {
        let token = match token_url {
            Some(url) => Some(TokenUrl::new(url.to_owned()).with_context(|| {
                format!(
                    "`--token-url` is not a correct absolute URL. Provided value: {}",
                    url
                )
            })?),
            None => None,
        };

        Ok(BasicClient::new(
            ClientId::new(args.client_id.to_owned()),
            args.client_secret.clone().map(ClientSecret::new),
            AuthUrl::new(authorization_url.to_owned()).with_context(|| {
                format!(
                    "`--authorization-url` is not a correct absolute URL. Provided value: {}",
                    authorization_url
                )
            })?,
            token,
        )
        .set_redirect_uri(RedirectUrl::new(args.callback_url.to_owned()).unwrap()))
    }

    pub async fn new(args: &Arguments) -> Result<OAuthClient> {
        log::debug!("Creating OAuthClient...");

        let (token_url, authorization_url) =
            if let Some(discovery_url) = args.discovery_url.to_owned() {
                log::debug!(
                    "Using `--discovery-url`={} to get token_url and authorization_url ",
                    discovery_url
                );

                let (token_url, authorization_url) =
                    get_endpoints_from_discovery_url(discovery_url).await?;

                (Some(token_url), authorization_url)
            } else {
                (
                    args.token_url.to_owned(),
                    args.authorization_url.to_owned().unwrap(),
                )
            };

        log::debug!(
            "Resolved token_url={:?} and authorization_url={}",
            token_url,
            authorization_url
        );

        let client = Self::get_client(args, token_url.as_deref(), &authorization_url)
            .context("Failed to create a OAuthClient")?;

        log::debug!("OAuthClient created");

        Ok(OAuthClient {
            args,
            inner: client,
        })
    }

    fn authorization_url_builder(&self) -> AuthorizationRequest {
        let mut builder = self
            .inner
            .authorize_url(CsrfToken::new_random)
            // TODO: Generate cryptographic strong NONCE
            .add_extra_param("nonce", "NONCE")
            .add_scope(Scope::new(self.args.scope.to_string()));

        if let Some(ref aud) = self.args.audience {
            builder = builder.add_extra_param("audience", aud);
        }

        builder
    }

    pub fn authorize_url(&self, pkce_challenge: Option<PkceCodeChallenge>) -> (Url, CsrfToken) {
        let mut builder = self.authorization_url_builder();

        if let Some(challenge) = pkce_challenge {
            builder = builder.set_pkce_challenge(challenge);
        }

        builder.url()
    }

    pub fn implicit_url(&self) -> (Url, CsrfToken) {
        self.authorization_url_builder()
            .add_extra_param("response_mode", "form_post")
            .use_implicit_flow()
            .url()
    }

    pub async fn exchange_client_credentials(&self) -> Result<BasicTokenResponse> {
        log::debug!("Exchanging credentials for a token...");

        // NOTE: offline_mode doesn't make any sense for Client Credentials.
        // Replaces any usages for this scope even if provided by user
        let scope = Scope::new(self.args.scope.to_string().replace("offline_access", ""));

        let mut builder = self.inner.exchange_client_credentials().add_scope(scope);

        if let Some(aud) = &self.args.audience {
            builder = builder.add_extra_param("audience", aud);
        }

        let token = builder
            .request_async(async_http_client)
            .await
            .context("Failed to exchange of client credentials for a token")?;
        log::debug!("Exchange done");
        Ok(token)
    }

    pub async fn exchange_resource_owner_password_client_credentials(
        &self,
    ) -> Result<BasicTokenResponse> {
        log::debug!("Exchanging credentials for a token...");

        let username =
            &ResourceOwnerUsername::new(self.args.username.as_deref().unwrap().to_owned());
        let password =
            &ResourceOwnerPassword::new(self.args.password.as_deref().unwrap().to_owned());
        let mut builder = self
            .inner
            .exchange_password(username, password)
            .add_scope(Scope::new(self.args.scope.to_string()));

        if let Some(aud) = &self.args.audience {
            builder = builder.add_extra_param("audience", aud);
        }

        let token = builder
            .request_async(async_http_client)
            .await
            .context("Failed to exchange client credentials for a token")?;
        log::debug!("Exchange done");
        Ok(token)
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: Option<PkceCodeVerifier>,
    ) -> Result<BasicTokenResponse> {
        log::debug!("Exchanging code for a token...");
        let mut builder = self
            .inner
            .exchange_code(AuthorizationCode::new(code.to_string()));

        if let Some(verifier) = code_verifier {
            builder = builder.set_pkce_verifier(verifier);
        }

        let token: BasicTokenResponse = builder
            .request_async(async_http_client)
            .await
            .context("Failed to exchange code for a token")?;
        log::debug!("Exchange done");

        Ok(token)
    }

    pub async fn refresh_token(&self, refresh_token: String) -> Result<BasicTokenResponse> {
        log::debug!("Refreshing token...");

        let refresh_token = RefreshToken::new(refresh_token);

        let response = self
            .inner
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await
            .context("Failed to exchange refresh token to a new token")?;

        log::debug!("Refresh done");
        Ok(response)
    }
}
