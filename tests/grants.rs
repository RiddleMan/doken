use home::home_dir;
use std::time::Duration;

use doken::{args::Arguments, auth_browser::auth_browser::AuthBrowser, get_token, grant::Grant};
use lazy_static::lazy_static;
use serial_test::serial;
use std::fs::remove_file;
use std::sync::Arc;
use testcontainers::clients;
use tokio::sync::{Mutex, OnceCell};
use tokio::time::sleep;

use crate::common::keycloak_client::KeycloakClient;

mod common;

const REALM_NAME: &str = "test-realm";
const USERNAME: &str = "test-user";
const PASSWORD: &str = "test-password";
const CLIENT_ID: &str = "auth-client-id";

struct IdentityProviderInfo {
    _client: KeycloakClient<'static>,
    client_secret: String,
    discovery_url: String,
    _token_url: String,
    _authorize_url: String,
}

lazy_static! {
    static ref DOCKER_CLIENT: clients::Cli = clients::Cli::default();
    static ref REDIRECT_URIS: Vec<String> = vec![
        "http://localhost:3000/oauth/callback".to_owned(),
        "https://wykop.pl/this/is/test/string/that/should/be/checked".to_owned(),
    ];
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
    static ref IDP_INFO: OnceCell<IdentityProviderInfo> = OnceCell::new();
    static ref AUTH_BROWSER: Arc<Mutex<AuthBrowser>> = {
        let browser = Arc::new(Mutex::new(AuthBrowser::new(false)));
        let temp = browser.clone();
        TOKIO_RUNTIME.spawn(async move {
            loop {
                let browser = temp.lock().await;
                let pages = browser.pages().await.unwrap();
                for page in pages {
                    let _ = page.bring_to_front().await;
                    let username_element = page.find_element(r#"input[name="username"]"#).await;
                    let password_element = page.find_element(r#"input[name="password"]"#).await;
                    let submit_element = page.find_element(r#"input[type="submit"]"#).await;

                    match (username_element, password_element, submit_element) {
                        (Ok(username_element), Ok(password_element), Ok(submit_element)) => {
                            username_element
                                .click()
                                .await
                                .unwrap()
                                .type_str(USERNAME)
                                .await
                                .unwrap();
                            password_element
                                .click()
                                .await
                                .unwrap()
                                .type_str(PASSWORD)
                                .await
                                .unwrap();
                            submit_element.click().await.unwrap();
                        }
                        _ => {
                            log::debug!("Waiting for the login page to load");
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        });
        browser
    };
}

async fn get_idp_info() -> &'static IdentityProviderInfo {
    IDP_INFO
        .get_or_init(|| async {
            let kc = KeycloakClient::new(&DOCKER_CLIENT).await.unwrap();
            kc.create_realm(REALM_NAME, USERNAME, PASSWORD, CLIENT_ID, &REDIRECT_URIS)
                .await
                .unwrap();
            let client_secret = kc.get_client_secret(REALM_NAME, CLIENT_ID).await.unwrap();

            let discovery_url = kc.discovery_url(REALM_NAME);
            let token_url = kc.token_url(REALM_NAME);
            let authorize_url = kc.authorize_url(REALM_NAME);
            IdentityProviderInfo {
                _client: kc,
                client_secret,
                discovery_url,
                _token_url: token_url,
                _authorize_url: authorize_url,
            }
        })
        .await
}

fn remove_config_if_available() -> () {
    let mut doken_config = home_dir().unwrap();
    doken_config.push(".doken.json");
    let _ = remove_file(doken_config);
}

fn assert_token_like(s: String) -> () {
    let token_parts: Vec<&str> = s.split('.').collect();

    assert_eq!(token_parts.len(), 3);
}

#[test]
#[serial]
fn it_authenticates_with_authorization_code_with_pkce_flow() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::AuthorizationCodeWithPkce,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                callback_url: Some(REDIRECT_URIS[0].to_owned()),
                client_id: CLIENT_ID.to_owned(),
                client_secret: Some(idp_info.client_secret.to_owned()),

                timeout: 1_000,
                ..Default::default()
            },
            browser,
        )
        .await
        .unwrap();

        assert_token_like(pkce_token);
    });
}

#[test]
#[serial]
fn it_authenticates_with_authorization_code_flow() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::AuthorizationCode,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                callback_url: Some(REDIRECT_URIS[1].to_owned()),
                client_id: CLIENT_ID.to_owned(),
                client_secret: Some(idp_info.client_secret.to_owned()),
                timeout: 1_000,
                ..Default::default()
            },
            browser,
        )
        .await
        .unwrap();

        assert_token_like(pkce_token);
    });
}
