use crate::args::Arguments;
use crate::openidc_discovery::get_endpoints_from_discovery_url;
use anyhow::{Context, Result};
use oauth2::basic::{
    BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenResponse,
};
use oauth2::{
    AuthUrl, AuthorizationCode, AuthorizationRequest, Client, ClientId, ClientSecret, CsrfToken,
    EndpointNotSet, EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken,
    ResourceOwnerPassword, ResourceOwnerUsername, Scope, StandardRevocableToken, TokenUrl,
};
use rand::distr::{Alphanumeric, SampleString};
use reqwest::redirect::Policy;
use url::Url;

type BaseClient<
    HasAuthUrl = EndpointSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointSet,
> = Client<
    BasicErrorResponse,
    BasicTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
>;
pub struct OAuthClient<'a> {
    args: &'a Arguments,
    inner: BaseClient,
    http: reqwest::Client,
}
impl OAuthClient<'_> {
    fn get_client(
        args: &Arguments,
        token_url: Option<&str>,
        authorization_url: &str,
    ) -> Result<BaseClient> {
        let token = match token_url {
            Some(url) => Some(TokenUrl::new(url.to_owned()).with_context(|| {
                format!("`--token-url` is not a correct absolute URL. Provided value: {url}",)
            })?),
            None => None,
        };

        let mut client: BaseClient = BaseClient::new(ClientId::new(args.client_id.to_owned()))
            .set_auth_uri(AuthUrl::new(authorization_url.to_owned()).with_context(|| {
                format!(
                    "`--authorization-url` is not a correct absolute URL. Provided value: {authorization_url}",
                )
            })?)
            .set_token_uri(token.unwrap());

        let client_secret = args.client_secret.to_owned().map(ClientSecret::new);

        if client_secret.is_some() {
            client = client.set_client_secret(client_secret.unwrap());
        }

        if let Some(callback_url) = &args.callback_url {
            client = client.set_redirect_uri(RedirectUrl::new(callback_url.to_owned()).unwrap())
        }

        Ok(client)
    }

    pub async fn new(args: &Arguments) -> Result<OAuthClient> {
        log::debug!("Creating OAuthClient...");

        let (token_url, authorization_url) = if let Some(discovery_url) =
            args.discovery_url.to_owned()
        {
            log::debug!(
                "Using `--discovery-url`={discovery_url} to get token_url and authorization_url ",
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

        log::debug!("Resolved token_url={token_url:?} and authorization_url={authorization_url}",);

        let client = Self::get_client(args, token_url.as_deref(), &authorization_url)
            .context("Failed to create a OAuthClient")?;

        log::debug!("OAuthClient created");

        let http_client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()?;

        Ok(OAuthClient {
            args,
            inner: client,
            http: http_client,
        })
    }

    fn authorization_url_builder(&self) -> AuthorizationRequest {
        let mut builder = self
            .inner
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(self.args.scope.to_string()));

        if let Some(ref aud) = self.args.audience {
            builder = builder.add_extra_param("audience", aud);
        }

        builder
    }

    pub fn authorize_url(
        &self,
        pkce_challenge: Option<PkceCodeChallenge>,
    ) -> (Url, CsrfToken, String) {
        let nonce = Alphanumeric.sample_string(&mut rand::rng(), 16);
        let mut builder = self.authorization_url_builder();

        builder = builder.add_extra_param("nonce", nonce.to_owned());

        if let Some(challenge) = pkce_challenge {
            builder = builder.set_pkce_challenge(challenge);
        }

        let (url, csrf) = builder.url();

        (url, csrf, nonce)
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
            .request_async(&self.http)
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
            .request_async(&self.http)
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
            .request_async(&self.http)
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
            .request_async(&self.http)
            .await
            .context("Failed to exchange refresh token to a new token")?;

        log::debug!("Refresh done");
        Ok(response)
    }
}
