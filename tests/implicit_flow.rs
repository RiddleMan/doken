use std::time::Duration;

use doken::{args::Arguments, auth_browser::AuthBrowser, get_token, grant::Grant};
use testcontainers::clients;
use tokio::time::sleep;

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
    let redirect_uris = vec![
        "http://localhost:3000/oauth/callback".to_owned(),
        "http://google.com/test/auth".to_owned(),
    ];

    let admin = KeycloakClient::new(&docker).await.unwrap();
    admin
        .create_realm(REALM_NAME, USERNAME, PASSWORD, CLIENT_ID, &redirect_uris)
        .await
        .unwrap();
    let client_secret = admin
        .get_client_secret(REALM_NAME, CLIENT_ID)
        .await
        .unwrap();

    let mut auth_browser = AuthBrowser::new().await.unwrap();

    let page = auth_browser.page().clone();

    tokio::spawn(async move {
        loop {
            let username_element = page.find_element(r#"input[name="username"]"#).await;

            match username_element {
                Ok(username_element) => {
                    username_element
                        .click()
                        .await
                        .unwrap()
                        .type_str(USERNAME)
                        .await
                        .unwrap();
                    page.find_element(r#"input[name="password"]"#)
                        .await
                        .unwrap()
                        .click()
                        .await
                        .unwrap()
                        .type_str(PASSWORD)
                        .await
                        .unwrap();
                    page.find_element(r#"input[type="submit"]"#)
                        .await
                        .unwrap()
                        .click()
                        .await
                        .unwrap()
                        .click()
                        .await
                        .unwrap();
                    break;
                }
                Err(_) => {
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    let pkce_token = get_token(
        Arguments {
            grant: Grant::AuthorizationCodeWithPkce,
            discovery_url: Some(admin.discovery_url(REALM_NAME)),
            callback_url: Some(redirect_uris[0].to_owned()),
            client_id: CLIENT_ID.to_owned(),
            client_secret: Some(client_secret),
            ..Default::default()
        },
        &mut auth_browser,
    )
    .await
    .unwrap();

    print!("{}", pkce_token);

    assert!(!pkce_token.is_empty());
}
