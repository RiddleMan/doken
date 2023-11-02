use anyhow::anyhow;
use std::{collections::HashMap, ffi::OsString, path::PathBuf};
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

impl Into<Vec<OsString>> for Profile {
    fn into(self) -> Vec<OsString> {
        let mut res: Vec<OsString> = Vec::new();

        // TODO: Some macro?
        if let Some(grant) = self.grant {
            res.append(&mut vec![
                "--grant".to_string().into(),
                to_variant_name(&grant).unwrap().to_string().into(),
            ]);
        }

        if let Some(token_url) = self.token_url {
            res.append(&mut vec![
                "--token-url".to_string().into(),
                token_url.into(),
            ]);
        }

        if let Some(authorization_url) = self.authorization_url {
            res.append(&mut vec![
                "--authorization-url".to_string().into(),
                authorization_url.into(),
            ]);
        }

        if let Some(callback_url) = self.callback_url {
            res.append(&mut vec![
                "--callback-url".to_string().into(),
                callback_url.into(),
            ]);
        }

        if let Some(client_id) = self.client_id {
            res.append(&mut vec![
                "--client-id".to_string().into(),
                client_id.into(),
            ]);
        }

        if let Some(client_secret) = self.client_secret {
            res.append(&mut vec![
                "--client-secret".to_string().into(),
                client_secret.into(),
            ]);
        }

        if let Some(username) = self.username {
            res.append(&mut vec!["--username".to_string().into(), username.into()]);
        }

        if let Some(password) = self.password {
            res.append(&mut vec!["--password".to_string().into(), password.into()]);
        }

        if let Some(scope) = self.scope {
            res.append(&mut vec!["--scope".to_string().into(), scope.into()]);
        }

        if let Some(audience) = self.audience {
            res.append(&mut vec!["--audience".to_string().into(), audience.into()]);
        }

        if let Some(timeout) = self.timeout {
            res.append(&mut vec![
                "--timeout".to_string().into(),
                timeout.to_string().into(),
            ]);
        }

        res
    }
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

    pub async fn read(&self) -> Config {
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
}
