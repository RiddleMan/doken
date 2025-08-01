use anyhow::{Context, Result, anyhow};
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
        let text = fs::read_to_string(&self.file_path).await.context(format!(
            "Cannot access {}",
            self.file_path.to_string_lossy()
        ));

        let profile = HashMap::new();
        match text {
            Ok(text) => toml::from_str::<Config>(&text).unwrap_or_else(|e| {
                log::warn!(
                    "Cannot parse config file {}. Error: {:?}",
                    &self.file_path.to_string_lossy(),
                    anyhow!(e)
                );

                Config { profile }
            }),
            Err(e) => {
                log::warn!("{e}");
                Config { profile }
            }
        }
    }

    pub async fn apply_profile(&self, profile: Option<String>) -> Result<()> {
        let config = self.read().await;

        if let Some(profile) = profile {
            let profile = config
                .profile
                .get(&profile)
                .context(format!("The given profile `{profile:?}` doesn't exist"))?;

            // TODO: Some macro?
            if let Some(grant) = &profile.grant {
                unsafe {
                    env::set_var("DOKEN_GRANT", to_variant_name(&grant).unwrap());
                }
            }

            if let Some(discovery_url) = &profile.discovery_url {
                unsafe {
                    env::set_var("DOKEN_DISCOVERY_URL", discovery_url);
                }
            }

            if let Some(token_url) = &profile.token_url {
                unsafe {
                    env::set_var("DOKEN_TOKEN_URL", token_url);
                }
            }

            if let Some(authorization_url) = &profile.authorization_url {
                unsafe {
                    env::set_var("DOKEN_AUTHORIZATION_URL", authorization_url);
                }
            }

            if let Some(callback_url) = &profile.callback_url {
                unsafe {
                    env::set_var("DOKEN_CALLBACK_URL", callback_url);
                }
            }

            if let Some(client_id) = &profile.client_id {
                unsafe {
                    env::set_var("DOKEN_CLIENT_ID", client_id);
                }
            }

            if let Some(client_secret) = &profile.client_secret {
                unsafe {
                    env::set_var("DOKEN_CLIENT_SECRET", client_secret);
                }
            }

            if let Some(username) = &profile.username {
                unsafe {
                    env::set_var("DOKEN_USERNAME", username);
                }
            }

            if let Some(password) = &profile.password {
                unsafe {
                    env::set_var("DOKEN_PASSWORD", password);
                }
            }

            if let Some(scope) = &profile.scope {
                unsafe {
                    env::set_var("DOKEN_SCOPE", scope);
                }
            }

            if let Some(audience) = &profile.audience {
                unsafe {
                    env::set_var("DOKEN_AUDIENCE", audience);
                }
            }

            if let Some(timeout) = &profile.timeout {
                unsafe {
                    env::set_var("DOKEN_TIMEOUT", timeout.to_string());
                }
            }
        }

        Ok(())
    }
}
