use doken::{get_token, args::Arguments, grant::Grant};
use testcontainers::clients;

use crate::common::keycloak_client::KeycloakClient;

mod common;

#[tokio::test]
async fn it_authenticates_with_implicit_flow() {
    env_logger::init();
    let docker = clients::Cli::default();
    const REALM_NAME: &str = "test-realm";
    const USERNAME: &str = "test-user";
    const PASSWORD: &str = "test-password";
    const CLIENT_ID: &str = "auth-client-id";
    let redirect_uris = vec!["http://localhost:3000/oauth/callback".to_owned(), "http://google.com/test/auth".to_owned()];

    let admin = KeycloakClient::new(&docker).await.unwrap();
    admin.create_realm(REALM_NAME, USERNAME, PASSWORD, CLIENT_ID, &redirect_uris).await.unwrap();
    let client_secret = admin.get_client_secret(REALM_NAME, CLIENT_ID).await.unwrap();


    let pkce_token = get_token(Arguments { 
        grant: Grant::AuthorizationCodeWithPkce,
        discovery_url: Some(admin.discovery_url(REALM_NAME)), 
        callback_url: Some(redirect_uris[0].to_owned()),
        client_id: CLIENT_ID.to_owned(), 
        client_secret: Some(client_secret), 
        ..Default::default()
    }).await.unwrap();

    assert!(!pkce_token.is_empty());
}
