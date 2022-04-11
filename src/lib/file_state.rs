use oauth2::basic::BasicTokenResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Deserialize, Serialize, Clone)]
struct DokenState {
    version: u32,
    // K: client_id, V: BasicTokenResponse
    data: HashMap<String, BasicTokenResponse>,
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
        token_info: &BasicTokenResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text = fs::read_to_string(&self.file_path)
            .await
            .unwrap_or_else(|_| "".to_string());

        let mut state = serde_json::from_str::<DokenState>(&text).unwrap_or_else(|_| {
            let data = HashMap::new();

            DokenState { version: 1, data }
        });

        state.data.insert(client_id, token_info.clone());

        let state_str = serde_json::to_string(&state).unwrap();

        fs::write(&self.file_path, state_str).await?;

        Ok(())
    }
}
