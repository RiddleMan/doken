use std::time::Duration;

use anyhow::Result;
use keycloak::{
    KeycloakAdmin, KeycloakAdminToken,
    types::{
        ClientRepresentation, CredentialRepresentation, RealmRepresentation, UserRepresentation,
    },
};
use testcontainers::{ContainerAsync, runners::AsyncRunner};

use super::keycloak::Keycloak;

pub struct KeycloakClient {
    inner: KeycloakAdmin,
    _container: ContainerAsync<Keycloak>,
    url: String,
}

pub const ACCESS_TOKEN_LIFESPAN: Duration = Duration::from_secs(30);

impl KeycloakClient {
    pub async fn new() -> Result<KeycloakClient> {
        let kc_image = Keycloak::default();
        let username = kc_image.username().to_owned();
        let password = kc_image.password().to_owned();

        let kc = kc_image.start().await?;

        let url = format!("http://localhost:{}", kc.get_host_port_ipv4(8080).await?);

        let client = reqwest::Client::new();
        let admin_token = KeycloakAdminToken::acquire(&url, &username, &password, &client).await?;

        Ok(Self {
            inner: KeycloakAdmin::new(&url, admin_token, client),
            _container: kc,
            url,
        })
    }

    pub async fn create_realm(
        &self,
        realm_name: &str,
        username: &str,
        password: &str,
        clients: &[(String, String, bool)],
    ) -> Result<()> {
        self.inner
            .post(RealmRepresentation {
                realm: Some(realm_name.to_owned()),
                enabled: Some(true),
                access_token_lifespan: Some(ACCESS_TOKEN_LIFESPAN.as_secs().try_into().unwrap()),
                users: Some(vec![UserRepresentation {
                    username: Some(username.to_owned()),
                    enabled: Some(true),
                    credentials: Some(vec![CredentialRepresentation {
                        temporary: Some(false),
                        value: Some(password.to_owned()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }]),
                clients: Some(
                    clients
                        .iter()
                        .map(
                            |(client_id, redirect_uri, public_client)| ClientRepresentation {
                                id: Some(client_id.to_owned()),
                                enabled: Some(true),
                                public_client: Some(*public_client),
                                implicit_flow_enabled: Some(true),
                                direct_access_grants_enabled: Some(true),
                                standard_flow_enabled: Some(true),
                                service_accounts_enabled: Some(!(*public_client)),
                                redirect_uris: Some(vec![redirect_uri.to_owned()]),
                                ..Default::default()
                            },
                        )
                        .collect(),
                ),
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    pub async fn get_client_secret(&self, realm_name: &str, client_id: &str) -> Result<String> {
        let credentials = self
            .inner
            .realm_clients_with_client_uuid_client_secret_get(realm_name, client_id)
            .await?;

        Ok(credentials.value.unwrap())
    }

    pub fn discovery_url(&self, realm_name: &str) -> String {
        format!(
            "{}/realms/{}/.well-known/openid-configuration",
            &self.url, realm_name
        )
    }

    pub fn token_url(&self, realm_name: &str) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/token",
            &self.url, realm_name
        )
    }

    pub fn authorize_url(&self, realm_name: &str) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect/auth",
            &self.url, realm_name
        )
    }
}
