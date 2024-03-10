use common::keycloak_client::ACCESS_TOKEN_LIFESPAN;
use common::{assert_token_like, remove_config_if_available};
use std::time::Duration;

use doken::{args::Arguments, auth_browser::browser::Browser, get_token, grant::Grant};
use lazy_static::lazy_static;
use serial_test::serial;
use std::sync::Arc;
use testcontainers::clients;
use tokio::sync::{Mutex, OnceCell};
use tokio::time::sleep;

use crate::common::keycloak_client::KeycloakClient;

mod common;

const REALM_NAME: &str = "test-realm";
const USERNAME: &str = "test-user";
const PASSWORD: &str = "test-password";
const TIMEOUT: u64 = 5_000;

struct ClientInfo {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

struct IdentityProviderInfo {
    _client: KeycloakClient<'static>,
    clients: Vec<ClientInfo>,
    discovery_url: String,
    _token_url: String,
    _authorize_url: String,
}

lazy_static! {
    static ref DOCKER_CLIENT: clients::Cli = clients::Cli::default();
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
    static ref IDP_INFO: OnceCell<IdentityProviderInfo> = OnceCell::new();
    static ref AUTH_BROWSER: Arc<Mutex<Browser>> = {
        let browser = Arc::new(Mutex::new(Browser::new(true)));
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
            const CLIENT_ID_1: &str = "test-client-id1";
            const REDIRECT_URI_1: &str = "http://localhost:3000/oauth/callback";
            const CLIENT_ID_2: &str = "test-client-id2";
            const REDIRECT_URI_2: &str =
                "https://wykop.pl/this/is/test/string/that/should/be/checked";
            const CLIENT_ID_3: &str = "test-client-id3";
            const REDIRECT_URI_3: &str = "https://localhost:1234/test/callback";
            let clients = vec![
                (CLIENT_ID_1.to_owned(), REDIRECT_URI_1.to_owned(), false),
                (CLIENT_ID_2.to_owned(), REDIRECT_URI_2.to_owned(), false),
                (CLIENT_ID_3.to_owned(), REDIRECT_URI_3.to_owned(), true),
            ];
            kc.create_realm(REALM_NAME, USERNAME, PASSWORD, &clients)
                .await
                .unwrap();
            let discovery_url = kc.discovery_url(REALM_NAME);
            let token_url = kc.token_url(REALM_NAME);
            let authorize_url = kc.authorize_url(REALM_NAME);
            let client_secret_1 = kc.get_client_secret(REALM_NAME, CLIENT_ID_1).await.unwrap();
            let client_secret_2 = kc.get_client_secret(REALM_NAME, CLIENT_ID_2).await.unwrap();
            IdentityProviderInfo {
                _client: kc,
                clients: vec![
                    ClientInfo {
                        client_id: CLIENT_ID_1.to_owned(),
                        client_secret: client_secret_1,
                        redirect_uri: REDIRECT_URI_1.to_owned(),
                    },
                    ClientInfo {
                        client_id: CLIENT_ID_2.to_owned(),
                        client_secret: client_secret_2,
                        redirect_uri: REDIRECT_URI_2.to_owned(),
                    },
                    ClientInfo {
                        client_id: CLIENT_ID_3.to_owned(),
                        client_secret: String::new(),
                        redirect_uri: REDIRECT_URI_3.to_owned(),
                    },
                ],
                discovery_url,
                _token_url: token_url,
                _authorize_url: authorize_url,
            }
        })
        .await
}

#[test]
#[serial]
fn it_authenticates_with_authorization_code_with_pkce_grant() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.first().unwrap();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::AuthorizationCodeWithPkce,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                callback_url: Some(client_info.redirect_uri.to_owned()),
                client_id: client_info.client_id.to_owned(),
                client_secret: Some(client_info.client_secret.to_owned()),
                timeout: TIMEOUT,
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
fn it_authenticates_with_authorization_code_with_pkce_grant_without_client_secret() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.get(2).unwrap();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::AuthorizationCodeWithPkce,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                callback_url: Some(client_info.redirect_uri.to_owned()),
                client_id: client_info.client_id.to_owned(),
                timeout: TIMEOUT,
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
fn it_returns_the_same_access_token_when_authenticating_once_again() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser_lock = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.first().unwrap();
        let args = Arguments {
            grant: Grant::AuthorizationCodeWithPkce,
            discovery_url: Some(idp_info.discovery_url.to_owned()),
            callback_url: Some(client_info.redirect_uri.to_owned()),
            client_id: client_info.client_id.to_owned(),
            client_secret: Some(client_info.client_secret.to_owned()),
            timeout: TIMEOUT,
            ..Default::default()
        };
        let token_before = get_token(args.to_owned(), browser_lock).await.unwrap();
        let browser_lock = browser.lock().await;
        let page_len_before = browser_lock.pages().await.unwrap().len();

        let token_after = get_token(args.to_owned(), browser_lock).await.unwrap();
        let browser_lock = browser.lock().await;
        let page_len_after = browser_lock.pages().await.unwrap().len();

        assert_eq!(token_before, token_after);
        // Checks whether it opened a browser
        assert_eq!(page_len_before, page_len_after);
    });
}

#[test]
#[serial]
fn it_reuses_refresh_token_provided_by_idp_when_authenticating_once_again() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser_lock = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.first().unwrap();
        let args = Arguments {
            grant: Grant::AuthorizationCodeWithPkce,
            discovery_url: Some(idp_info.discovery_url.to_owned()),
            callback_url: Some(client_info.redirect_uri.to_owned()),
            client_id: client_info.client_id.to_owned(),
            client_secret: Some(client_info.client_secret.to_owned()),
            timeout: TIMEOUT,
            ..Default::default()
        };
        let token_before = get_token(args.to_owned(), browser_lock).await.unwrap();
        let browser_lock = browser.lock().await;
        let page_len_before = browser_lock.pages().await.unwrap().len();

        sleep(ACCESS_TOKEN_LIFESPAN).await;

        let token_after = get_token(args.to_owned(), browser_lock).await.unwrap();
        let browser_lock = browser.lock().await;
        let page_len_after = browser_lock.pages().await.unwrap().len();

        assert_ne!(token_before, token_after);
        // Checks whether it opened a browser
        assert_eq!(page_len_before, page_len_after);
    });
}

#[test]
#[serial]
fn it_opens_new_tab_if_client_ids_does_not_match() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser_lock = browser.lock().await;
        remove_config_if_available();

        let client_info = idp_info.clients.first().unwrap();
        let args = Arguments {
            grant: Grant::AuthorizationCodeWithPkce,
            discovery_url: Some(idp_info.discovery_url.to_owned()),
            callback_url: Some(client_info.redirect_uri.to_owned()),
            client_id: client_info.client_id.to_owned(),
            client_secret: Some(client_info.client_secret.to_owned()),
            timeout: TIMEOUT,
            ..Default::default()
        };
        let _ = get_token(args.to_owned(), browser_lock).await.unwrap();

        let browser_lock = browser.lock().await;
        let page_len_before = browser_lock.pages().await.unwrap().len();

        let client_info = idp_info.clients.get(1).unwrap();
        let args = Arguments {
            grant: Grant::AuthorizationCodeWithPkce,
            discovery_url: Some(idp_info.discovery_url.to_owned()),
            callback_url: Some(client_info.redirect_uri.to_owned()),
            client_id: client_info.client_id.to_owned(),
            client_secret: Some(client_info.client_secret.to_owned()),
            timeout: TIMEOUT,
            ..Default::default()
        };
        let _ = get_token(args.to_owned(), browser_lock).await.unwrap();

        let browser_lock = browser.lock().await;
        let page_len_after = browser_lock.pages().await.unwrap().len();

        assert_eq!(page_len_after - page_len_before, 1);
    });
}

#[test]
#[serial]
fn it_authenticates_with_authorization_code_grant() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.get(1).unwrap();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::AuthorizationCode,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                callback_url: Some(client_info.redirect_uri.to_owned()),
                client_id: client_info.client_id.to_owned(),
                client_secret: Some(client_info.client_secret.to_owned()),
                timeout: TIMEOUT,
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
fn it_authenticates_with_authorization_code_grant_without_client_secret() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.get(2).unwrap();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::AuthorizationCode,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                callback_url: Some(client_info.redirect_uri.to_owned()),
                client_id: client_info.client_id.to_owned(),
                timeout: TIMEOUT,
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
fn it_authenticates_with_implicit_grant() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.get(1).unwrap();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::Implicit,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                callback_url: Some(client_info.redirect_uri.to_owned()),
                client_id: client_info.client_id.to_owned(),
                client_secret: Some(client_info.client_secret.to_owned()),
                timeout: TIMEOUT,
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
fn it_authenticates_with_implicit_grant_without_client_secret() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.get(2).unwrap();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::Implicit,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                callback_url: Some(client_info.redirect_uri.to_owned()),
                client_id: client_info.client_id.to_owned(),
                timeout: TIMEOUT,
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
fn it_authenticates_with_client_credentials_grant() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.get(1).unwrap();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::ClientCredentials,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                client_id: client_info.client_id.to_owned(),
                client_secret: Some(client_info.client_secret.to_owned()),
                timeout: TIMEOUT,
                scope: "email".to_owned(),
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
fn it_authenticates_with_resource_owner_password_client_credentials_grant() {
    let _ = env_logger::try_init();
    TOKIO_RUNTIME.block_on(async {
        let idp_info = get_idp_info().await;

        let browser = AUTH_BROWSER.clone();
        let browser = browser.lock().await;
        remove_config_if_available();
        let client_info = idp_info.clients.first().unwrap();
        let pkce_token = get_token(
            Arguments {
                grant: Grant::ResourceOwnerPasswordClientCredentials,
                discovery_url: Some(idp_info.discovery_url.to_owned()),
                client_id: client_info.client_id.to_owned(),
                client_secret: Some(client_info.client_secret.to_owned()),
                username: Some(USERNAME.to_owned()),
                password: Some(PASSWORD.to_owned()),
                scope: "email".to_owned(),
                timeout: TIMEOUT,
                ..Default::default()
            },
            browser,
        )
        .await
        .unwrap();

        assert_token_like(pkce_token);
    });
}
