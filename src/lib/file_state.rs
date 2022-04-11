use oauth2::basic::BasicTokenResponse;
use oauth2::TokenResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;

#[derive(Deserialize, Serialize)]
pub struct TokenInfo {
    access_token: String,

    expires: Option<SystemTime>,

    scope: Option<String>,

    refresh_token: Option<String>,
}

impl TokenInfo {
    pub fn from_token_response(response: &BasicTokenResponse) -> TokenInfo {
        TokenInfo {
            access_token: response.access_token().secret().to_owned(),
            expires: response
                .expires_in()
                .map(|duration| SystemTime::now().add(duration)),
            scope: response
                .scopes()
                .map(|v| v.iter().map(|scope| scope.to_string()).collect()),
            refresh_token: response
                .refresh_token()
                .map(|token| token.secret().to_owned()),
        }
    }
}

type ClientId = String;

#[derive(Deserialize, Serialize)]
struct DokenState {
    version: u32,
    data: HashMap<ClientId, TokenInfo>,
}

pub struct FileState {
    file_path: PathBuf,
}

impl FileState {
    pub fn new() -> FileState {
        let home_path = match home::home_dir() {
            Some(mut home_dir) => {
                home_dir.push(".doken.json");
                home_dir
            }

            // TODO: Fallback path if cannot get home_dir
            None => Path::new("asdf").to_path_buf(),
        };

        FileState {
            file_path: home_path,
        }
    }

    pub async fn save(
        &self,
        client_id: String,
        token_info: TokenInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text = fs::read_to_string(&self.file_path)
            .await
            .unwrap_or_else(|_| "".to_string());

        let mut state = serde_json::from_str::<DokenState>(&text).unwrap_or_else(|_| {
            let data = HashMap::new();

            DokenState { version: 1, data }
        });

        state.data.insert(client_id, token_info);

        let state_str = serde_json::to_string(&state).unwrap();

        fs::write(&self.file_path, state_str).await?;

        Ok(())
    }
}
