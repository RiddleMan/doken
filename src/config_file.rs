use anyhow::{anyhow, Context, Result};
use std::{collections::HashMap, env, path::PathBuf};
use tokio::fs;

use serde::{Deserialize, Serialize};
use serde_variant::to_variant_name;

use crate::grant::Grant;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Profile {
    /// Authentication Grant
    pub grant: Option<Grant>,

    /// OAuth 2.0 token exchange url
    pub token_url: Option<String>,

    /// OAuth 2.0 authorization initiation url
    pub authorization_url: Option<String>,

    /// OpenID Connect discovery url
    pub discovery_url: Option<String>,

    /// Callback URL that's been set for your application
    pub callback_url: Option<String>,

    /// OAuth 2.0 Client Identifier <https://www.rfc-editor.org/rfc/rfc6749#section-2.2>
    pub client_id: Option<String>,

    /// OAuth 2.0 Client Secret. Please use `--client-secret-stdin`, because it's not get stored in a shell history.  <https://www.rfc-editor.org/rfc/rfc6749#section-2.3.1>
    pub client_secret: Option<String>,

    /// OAuth 2.0 Resource Owner Password Client Credentials Grant's username <https://www.rfc-editor.org/rfc/rfc6749#section-4.3.2>
    pub username: Option<String>,

    /// OAuth 2.0 Resource Owner Password Client Credentials Grant's password <https://www.rfc-editor.org/rfc/rfc6749#section-4.3.2>
    pub password: Option<String>,

    /// OAuth 2.0 Scope <https://www.rfc-editor.org/rfc/rfc6749#section-3.3>
    pub scope: Option<String>,

    /// OpenID Connect requested aud
    pub audience: Option<String>,

    /// Authorization Code, Authorization Code with PKCE and Implicit Grants' timeout,
    pub timeout: Option<u64>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    pub profile: HashMap<String, Profile>,
}

pub struct ConfigFile {
    file_path: PathBuf,
}

impl ConfigFile {
    pub fn new() -> ConfigFile {
        let home_path = match home::home_dir() {
            Some(mut home_dir) => {
                home_dir.push(".doken/config.toml");
                home_dir
            }

            None => panic!("Couldn't access $HOME_DIR"),
        };

        ConfigFile {
            file_path: home_path,
        }
    }

    async fn read(&self) -> Config {
        log::debug!("Reading the state file");
        let text = fs::read_to_string(&self.file_path)
            .await
            .unwrap_or_default();

        println!("File {:?}", text);

        let profile = HashMap::new();
        toml::from_str::<Config>(&text).unwrap_or_else(|e| {
            log::warn!(
                "Cannot parse config file {}. Error: {:?}",
                &self.file_path.to_string_lossy(),
                anyhow!(e)
            );

            Config { profile }
        })
    }

    pub async fn apply_profile(&self, profile: Option<String>) -> Result<()> {
        let config = self.read().await;

        match profile {
            Some(profile) => {
                let profile = config
                    .profile
                    .get(&profile)
                    .context(format!("The given profile `{:?}` doesn't exist", profile))?;

                // TODO: Some macro?
                if let Some(grant) = &profile.grant {
                    env::set_var("DOKEN_GRANT", to_variant_name(&grant).unwrap().to_string());
                }

                if let Some(token_url) = &profile.token_url {
                    env::set_var("DOKEN_TOKEN_URL", token_url);
                }

                if let Some(authorization_url) = &profile.authorization_url {
                    env::set_var("DOKEN_AUTHORIZATION_URL", authorization_url);
                }

                if let Some(callback_url) = &profile.callback_url {
                    env::set_var("DOKEN_CALLBACK_URL".to_string(), callback_url);
                }

                if let Some(client_id) = &profile.client_id {
                    env::set_var("DOKEN_CLIENT_ID".to_string(), client_id);
                }

                if let Some(client_secret) = &profile.client_secret {
                    env::set_var("DOKEN_CLIENT_SECRET".to_string(), client_secret);
                }

                if let Some(username) = &profile.username {
                    env::set_var("DOKEN_USERNAME".to_string(), username);
                }

                if let Some(password) = &profile.password {
                    env::set_var("DOKEN_PASSWORD".to_string(), password);
                }

                if let Some(scope) = &profile.scope {
                    env::set_var("DOKEN_SCOPE".to_string(), scope);
                }

                if let Some(audience) = &profile.audience {
                    env::set_var("DOKEN_AUDIENCE".to_string(), audience);
                }

                if let Some(timeout) = &profile.timeout {
                    env::set_var("DOKEN_TIMEOUT".to_string(), timeout.to_string());
                }
            }
            _ => {}
        }

        Ok(())
    }
}
