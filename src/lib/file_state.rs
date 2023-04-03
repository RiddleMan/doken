use crate::lib::token_info::TokenInfo;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
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

            None => panic!("Couldn't access $HOME_DIR"),
        };

        FileState {
            file_path: home_path,
        }
    }

    async fn read(&self) -> DokenState {
        log::debug!("Reading the state file");
        let text = fs::read_to_string(&self.file_path)
            .await
            .unwrap_or_default();

        let data = HashMap::new();
        serde_json::from_str::<DokenState>(&text).unwrap_or(DokenState { version: 1, data })
    }

    async fn write(&self, state: &DokenState) -> Result<()> {
        log::debug!("Writing the state file");
        let state_str = serde_json::to_string(state).unwrap();

        fs::write(&self.file_path, state_str)
            .await
            .with_context(|| {
                format!(
                    "Failed to write to {} file",
                    &self.file_path.as_os_str().to_string_lossy()
                )
            })?;

        Ok(())
    }

    pub async fn read_token_info(&self, client_id: &String) -> Option<TokenInfo> {
        log::debug!(
            "Reading token info for client_id: {} from the state",
            client_id
        );
        let state = self.read().await;

        state.data.get(client_id).cloned()
    }

    pub async fn upsert_token_info(&self, client_id: String, token_info: TokenInfo) -> Result<()> {
        log::debug!(
            "Saving token info: {:#?} for client_id: {} to the state",
            token_info,
            client_id
        );
        let mut state = self.read().await;

        state.data.insert(client_id, token_info);

        self.write(&state).await?;

        Ok(())
    }

    pub async fn clear_token_info(&self, client_id: String) -> Result<()> {
        log::debug!(
            "Clearing token info for client_id: {} in the state",
            client_id
        );
        let mut state = self.read().await;

        state.data.remove(&client_id);

        self.write(&state).await?;

        Ok(())
    }
}
