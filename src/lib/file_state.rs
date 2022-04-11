use crate::lib::token_info::TokenInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

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

    async fn read(&self) -> DokenState {
        let text = fs::read_to_string(&self.file_path)
            .await
            .unwrap_or_else(|_| "".to_string());

        serde_json::from_str::<DokenState>(&text).unwrap_or_else(|_| {
            let data = HashMap::new();

            DokenState { version: 1, data }
        })
    }

    async fn write(&self, state: &DokenState) -> Result<(), Box<dyn std::error::Error>> {
        let state_str = serde_json::to_string(state).unwrap();

        fs::write(&self.file_path, state_str).await?;

        Ok(())
    }

    pub async fn read_token_info(&self, client_id: String) -> Option<TokenInfo> {
        let state = self.read().await;

        state.data.get(&client_id).cloned()
    }

    pub async fn upsert_token_info(
        &self,
        client_id: String,
        token_info: TokenInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.read().await;

        state.data.insert(client_id, token_info);

        self.write(&state).await?;

        Ok(())
    }
}
