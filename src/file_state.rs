use crate::token_info::TokenInfo;
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

    pub fn from(file_path: PathBuf) -> FileState {
        FileState { file_path }
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use tempfile::TempDir;
    use tokio::time::sleep;

    use super::*;

    fn get_tmp_path() -> Result<(TempDir, PathBuf)> {
        let tmp_dir = tempfile::tempdir()?;
        let mut path = tmp_dir.path().to_owned();
        path.push(".doken.json");
        Ok((tmp_dir, path))
    }

    fn uglify(s: &str) -> String {
        s.replace([' ', '\n'], "")
    }

    #[tokio::test]
    async fn it_writes_state_to_file() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let file_state = FileState::from(tmp_path.to_owned());
        const CLIENT_ID: &str = "test-client-id";
        const ACCESS_TOKEN: &str = "test-access-token";

        file_state
            .upsert_token_info(
                CLIENT_ID.to_owned(),
                TokenInfo {
                    access_token: ACCESS_TOKEN.to_owned(),
                    refresh_token: None,
                    expires: None,
                    scope: None,
                },
            )
            .await
            .unwrap();

        let content = fs::read_to_string(tmp_path).await.unwrap_or_default();

        let expected = r#"{
  "version": 1,
  "data": {
    "test-client-id": {
      "access_token": "test-access-token",
      "refresh_token": null,
      "expires": null,
      "scope": null
    }
  }
}"#;

        assert_eq!(content, uglify(expected));
    }

    #[tokio::test]
    async fn it_writes_state_to_file_with_all_possible_values() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let file_state = FileState::from(tmp_path.to_owned());
        const CLIENT_ID: &str = "test-client-id";
        const ACCESS_TOKEN: &str = "test-access-token";
        const REFRESH_TOKEN: &str = "test-refresh-token";
        const EXPIRES: SystemTime = SystemTime::UNIX_EPOCH;
        const SCOPE: &str = "email-profile";

        file_state
            .upsert_token_info(
                CLIENT_ID.to_owned(),
                TokenInfo {
                    access_token: ACCESS_TOKEN.to_owned(),
                    refresh_token: Some(REFRESH_TOKEN.to_owned()),
                    expires: Some(EXPIRES),
                    scope: Some(SCOPE.to_owned()),
                },
            )
            .await
            .unwrap();

        let content = fs::read_to_string(tmp_path).await.unwrap_or_default();

        let expected = r#"{
  "version": 1,
  "data": {
    "test-client-id": {
      "access_token": "test-access-token",
      "refresh_token": "test-refresh-token",
      "expires": {
        "secs_since_epoch": 0,
        "nanos_since_epoch": 0
      },
      "scope": "email-profile"
    }
  }
}"#;

        assert_eq!(content, uglify(expected));
    }

    #[tokio::test]
    async fn it_overwrites_state_of_client_id() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let file_state = FileState::from(tmp_path.to_owned());
        const CLIENT_ID: &str = "test-client-id";
        const ACCESS_TOKEN: &str = "test-access-token-overwrite";

        file_state
            .upsert_token_info(
                CLIENT_ID.to_owned(),
                TokenInfo {
                    access_token: "not-important".to_owned(),
                    refresh_token: None,
                    expires: None,
                    scope: None,
                },
            )
            .await
            .unwrap();
        file_state
            .upsert_token_info(
                CLIENT_ID.to_owned(),
                TokenInfo {
                    access_token: ACCESS_TOKEN.to_owned(),
                    refresh_token: None,
                    expires: None,
                    scope: None,
                },
            )
            .await
            .unwrap();

        let content = fs::read_to_string(tmp_path).await.unwrap_or_default();

        let expected = r#"{
  "version": 1,
  "data": {
    "test-client-id": {
      "access_token": "test-access-token-overwrite",
      "refresh_token": null,
      "expires": null,
      "scope": null
    }
  }
}"#;

        assert_eq!(content, uglify(expected));
    }

    #[tokio::test]
    async fn it_removes_client_id_data() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let file_state = FileState::from(tmp_path.to_owned());
        const CLIENT_ID: &str = "test-client-id";

        file_state
            .upsert_token_info(
                CLIENT_ID.to_owned(),
                TokenInfo {
                    access_token: "not-important".to_owned(),
                    refresh_token: None,
                    expires: None,
                    scope: None,
                },
            )
            .await
            .unwrap();

        file_state
            .clear_token_info(CLIENT_ID.to_owned())
            .await
            .unwrap();

        let content = fs::read_to_string(tmp_path).await.unwrap_or_default();

        let expected = r#"{
  "version": 1,
  "data": {}
}"#;

        assert_eq!(content, uglify(expected));
    }

    #[tokio::test]
    async fn it_does_not_fail_on_clearing_non_existent_state() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let file_state = FileState::from(tmp_path.to_owned());

        file_state
            .clear_token_info("test-client-id".to_owned())
            .await
            .unwrap();

        let content = fs::read_to_string(tmp_path).await.unwrap_or_default();

        let expected = r#"{
  "version": 1,
  "data": {}
}"#;

        assert_eq!(content, uglify(expected));
    }

    #[tokio::test]
    async fn it_does_not_change_other_client_id_state() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let file_state = FileState::from(tmp_path.to_owned());
        const CLIENT_ID: &str = "test-client-id-10";
        const ACCESS_TOKEN: &str = "test-access-token-another-one";

        file_state
            .upsert_token_info(
                CLIENT_ID.to_owned(),
                TokenInfo {
                    access_token: ACCESS_TOKEN.to_owned(),
                    refresh_token: None,
                    expires: None,
                    scope: None,
                },
            )
            .await
            .unwrap();

        file_state
            .clear_token_info("test-client-id-that-does-not-exist".to_owned())
            .await
            .unwrap();

        let content = fs::read_to_string(tmp_path).await.unwrap_or_default();

        let expected = r#"{
  "version": 1,
  "data": {
    "test-client-id-10": {
      "access_token": "test-access-token-another-one",
      "refresh_token": null,
      "expires": null,
      "scope": null
    }
  }
}"#;

        assert_eq!(content, uglify(expected));
    }

    #[tokio::test]
    async fn it_reads_state_of_correct_client_id() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let file_state = FileState::from(tmp_path.to_owned());
        const CLIENT_ID: &str = "test-client-id";
        const ACCESS_TOKEN: &str = "test-access-token";
        const REFRESH_TOKEN: &str = "test-refresh-token";
        const EXPIRES: SystemTime = SystemTime::UNIX_EPOCH;
        const SCOPE: &str = "email-profile";

        let expected_token_info = TokenInfo {
            access_token: ACCESS_TOKEN.to_owned(),
            refresh_token: Some(REFRESH_TOKEN.to_owned()),
            expires: Some(EXPIRES),
            scope: Some(SCOPE.to_owned()),
        };

        file_state
            .upsert_token_info(
                "not-important".to_owned(),
                TokenInfo {
                    access_token: "not-important".to_owned(),
                    refresh_token: None,
                    expires: None,
                    scope: None,
                },
            )
            .await
            .unwrap();
        file_state
            .upsert_token_info(CLIENT_ID.to_owned(), expected_token_info.to_owned())
            .await
            .unwrap();

        let actual_token_info = file_state
            .read_token_info(&CLIENT_ID.to_owned())
            .await
            .unwrap();

        assert_eq!(
            actual_token_info.access_token,
            expected_token_info.access_token
        );
        assert_eq!(
            actual_token_info.refresh_token,
            expected_token_info.refresh_token
        );
        assert_eq!(actual_token_info.expires, expected_token_info.expires);
        assert_eq!(actual_token_info.scope, expected_token_info.scope);
    }

    #[tokio::test]
    async fn it_locks_rw_access_to_file() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        const CLIENT_ID: &str = "test-client-id";
        let expected_token_info = TokenInfo {
            access_token: "expected_token_info".to_owned(),
            refresh_token: None,
            expires: None,
            scope: None,
        };

        let client_id = CLIENT_ID.to_owned();
        let tmp_path_1 = tmp_path.to_owned();
        let handle = tokio::spawn(async move {
            let file_state = FileState::from(tmp_path_1.to_owned());
            sleep(Duration::from_millis(10)).await;
            file_state
                .upsert_token_info(
                    client_id,
                    TokenInfo {
                        access_token: "not-important".to_owned(),
                        refresh_token: None,
                        expires: None,
                        scope: None,
                    },
                )
                .await
                .unwrap();
        });

        let file_state = FileState::from(tmp_path.to_owned());
        file_state
            .upsert_token_info(CLIENT_ID.to_owned(), expected_token_info.to_owned())
            .await
            .unwrap();

        let _ = handle.await;

        let actual_token_info = file_state
            .read_token_info(&CLIENT_ID.to_owned())
            .await
            .unwrap();

        assert_eq!(
            actual_token_info.access_token,
            expected_token_info.access_token
        );
    }
}
