use std::time::Duration;

use anyhow::{anyhow, Result};
use doken::{args::Arguments, auth_browser::auth_browser::AuthBrowser, get_token, grant::Grant};
use lazy_static::lazy_static;
use std::sync::{Arc, OnceLock};
use testcontainers::clients;
use tokio::time::sleep;

use crate::common::keycloak_client::KeycloakClient;

mod common;

const REALM_NAME: &str = "test-realm";
const USERNAME: &str = "test-user";
const PASSWORD: &str = "test-password";
const CLIENT_ID: &str = "auth-client-id";

struct IdentityProviderInfo {
    client: KeycloakClient<'static>,
    client_secret: String,
    discovery_url: String,
    token_url: String,

    authorize_url: String,
}

lazy_static! {
    static ref DOCKER_CLIENT: clients::Cli = clients::Cli::default();
    static ref REDIRECT_URIS: Vec<String> = vec![
        "http://localhost:3000/oauth/callback".to_owned(),
        "http://google.com/test/auth".to_owned(),
    ];
    static ref IDP_INFO: OnceLock<IdentityProviderInfo> = OnceLock::new();
    static ref AUTH_BROWSER: Arc<AuthBrowser> = {
        let browser = Arc::new(AuthBrowser::new(false));
        let temp = browser.clone();
        tokio::spawn(async move {
                let browser = temp;
            log::info!("==============================");
            loop {
                log::info!("=======11111111111111111111111======");
                log::info!("=======3333333333333333333333333======");
                let pages = browser.pages().await.unwrap();

                log::info!("=======222222222222222=====");
                log::info!("Pages: {:?}", pages.len());
                for page in pages {
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
                            log::info!("Not found once again!: {:?}", page.url().await.unwrap());
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        });
        browser
    };
}

async fn get_idp_info() -> Result<&'static IdentityProviderInfo> {
    match IDP_INFO.get() {
        Some(idp) => Ok(idp),
        None => {
            let kc = KeycloakClient::new(&DOCKER_CLIENT)
                .await
                .map_err(|e| anyhow!(e))?;
            kc.create_realm(REALM_NAME, USERNAME, PASSWORD, CLIENT_ID, &REDIRECT_URIS)
                .await
                .map_err(|e| anyhow!(e))?;
            let client_secret = kc
                .get_client_secret(REALM_NAME, CLIENT_ID)
                .await
                .map_err(|e| anyhow!(e))?;

            let discovery_url = kc.discovery_url(REALM_NAME);
            let token_url = kc.token_url(REALM_NAME);
            let authorize_url = kc.authorize_url(REALM_NAME);
            let _ = IDP_INFO.set(IdentityProviderInfo {
                client: kc,
                client_secret,
                discovery_url,
                token_url,
                authorize_url,
            });

            Ok(IDP_INFO.get().unwrap())
        }
    }
}

#[tokio::test]
async fn it_authenticates_with_implicit_flow() {
    env_logger::init();
    let idp_info = get_idp_info().await.unwrap();

    let browser = AUTH_BROWSER.clone();
    let pkce_token = get_token(
        Arguments {
            grant: Grant::AuthorizationCodeWithPkce,
            discovery_url: Some(idp_info.discovery_url.to_owned()),
            callback_url: Some(REDIRECT_URIS[0].to_owned()),
            client_id: CLIENT_ID.to_owned(),
            client_secret: Some(idp_info.client_secret.to_owned()),
            ..Default::default()
        },
        browser.as_ref(),
    )
    .await
    .unwrap();

    print!("{}", pkce_token);

    assert!(!pkce_token.is_empty());
}
