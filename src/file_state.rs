use crate::token_info::TokenInfo;
use anyhow::{Context, Result};
use file_guard::{FileGuard, Lock};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Write, Seek};
use std::path::PathBuf;
use std::sync::Arc;
use std::{collections::HashMap, fs::File};

type ClientId = String;

#[derive(Deserialize, Serialize)]
struct DokenState {
    version: u32,
    data: HashMap<ClientId, TokenInfo>,
}

pub struct FileState {
    file: Arc<File>,
    _guard1: FileGuard<Arc<File>>,
    _guard2: FileGuard<Arc<File>>,
}

impl FileState {
    pub fn new() -> Result<FileState> {
        let home_path = match home::home_dir() {
            Some(mut home_dir) => {
                home_dir.push(".doken.json");
                home_dir
            }

            None => panic!("Couldn't access $HOME_DIR"),
        };

        let file = Arc::new(OpenOptions::new()
                            .write(true)
            .read(true)
            .create(true)
            .open(home_path)?);

        let guard1 = file_guard::lock(file.clone(), Lock::Exclusive, 0, 1)?;
        let guard2 = file_guard::lock(file.clone(), Lock::Shared, 0, 1)?;

        Ok(FileState { file, _guard1: guard1, _guard2: guard2 })
    }

    pub fn _from(file_path: PathBuf) -> Result<FileState> {
        let file = Arc::new(OpenOptions::new()
                            .write(true)
            .read(true)
            .create(true)
            .open(file_path)?);

        let guard1 = file_guard::lock(file.clone(), Lock::Exclusive, 0, 1)?;
        let guard2 = file_guard::lock(file.clone(), Lock::Shared, 0, 1)?;

        Ok(FileState { file, _guard1: guard1, _guard2: guard2 })
    }

    fn read(&mut self) -> DokenState {
        log::debug!("Reading the state file");
        let mut text = String::new();
        let _ = self.file.seek(std::io::SeekFrom::Start(0));
        let _ = self.file.read_to_string(&mut text);

        let data = HashMap::new();
        serde_json::from_str::<DokenState>(&text).unwrap_or(DokenState { version: 1, data })
    }

    fn write(&mut self, state: &DokenState) -> Result<()> {
        log::debug!("Writing the state file");
        let state_str = serde_json::to_string(state).unwrap();

        self.file.seek(std::io::SeekFrom::Start(0))?;
        self.file.set_len(0)?;
        self.file
            .write_all(state_str.as_bytes())
            .context("Failed to write to a file")?;
        self.file.flush()?;

        Ok(())
    }

    pub fn read_token_info(&mut self, client_id: &String) -> Option<TokenInfo> {
        log::debug!(
            "Reading token info for client_id: {} from the state",
            client_id
        );
        let state = self.read();

        state.data.get(client_id).cloned()
    }

    pub fn upsert_token_info(&mut self, client_id: String, token_info: TokenInfo) -> Result<()> {
        log::debug!(
            "Saving token info: {:#?} for client_id: {} to the state",
            token_info,
            client_id
        );
        let mut state = self.read();

        state.data.insert(client_id, token_info);

        self.write(&state)?;

        Ok(())
    }

    pub fn clear_token_info(&mut self, client_id: String) -> Result<()> {
        log::debug!(
            "Clearing token info for client_id: {} in the state",
            client_id
        );
        let mut state = self.read();

        state.data.remove(&client_id);

        self.write(&state)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{Duration, SystemTime},
    };

    use tempfile::TempDir;
    use tokio::{time::sleep, join};

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

    #[test]
    fn it_writes_state_to_file() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let mut file_state = FileState::_from(tmp_path.to_owned()).unwrap();
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
            .unwrap();

        let content = fs::read_to_string(tmp_path).unwrap_or_default();

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

    #[test]
    fn it_writes_state_to_file_with_all_possible_values() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let mut file_state = FileState::_from(tmp_path.to_owned()).unwrap();
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
            .unwrap();

        let content = fs::read_to_string(tmp_path).unwrap_or_default();

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

    #[test]
    fn it_overwrites_state_of_client_id() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let mut file_state = FileState::_from(tmp_path.to_owned()).unwrap();
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
            .unwrap();

        let content = fs::read_to_string(tmp_path).unwrap_or_default();

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

    #[test]
    fn it_removes_client_id_data() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let mut file_state = FileState::_from(tmp_path.to_owned()).unwrap();
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
            .unwrap();

        file_state.clear_token_info(CLIENT_ID.to_owned()).unwrap();

        let content = fs::read_to_string(tmp_path).unwrap_or_default();

        let expected = r#"{
  "version": 1,
  "data": {}
}"#;

        assert_eq!(content, uglify(expected));
    }

    #[test]
    fn it_does_not_fail_on_clearing_non_existent_state() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let mut file_state = FileState::_from(tmp_path.to_owned()).unwrap();

        file_state
            .clear_token_info("test-client-id".to_owned())
            .unwrap();

        let content = fs::read_to_string(tmp_path).unwrap_or_default();

        let expected = r#"{
  "version": 1,
  "data": {}
}"#;

        assert_eq!(content, uglify(expected));
    }

    #[test]
    fn it_does_not_change_other_client_id_state() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let mut file_state = FileState::_from(tmp_path.to_owned()).unwrap();
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
            .unwrap();

        file_state
            .clear_token_info("test-client-id-that-does-not-exist".to_owned())
            .unwrap();

        let content = fs::read_to_string(tmp_path).unwrap_or_default();

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

    #[test]
    fn it_reads_state_of_correct_client_id() {
        let (_tmp_dir, tmp_path) = get_tmp_path().unwrap();
        let mut file_state = FileState::_from(tmp_path.to_owned()).unwrap();
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
            .unwrap();
        file_state
            .upsert_token_info(CLIENT_ID.to_owned(), expected_token_info.to_owned())
            .unwrap();

        let actual_token_info = file_state.read_token_info(&CLIENT_ID.to_owned()).unwrap();

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
        let handle1 = tokio::spawn(async move {
            let mut file_state = FileState::_from(tmp_path_1.to_owned()).unwrap();
            sleep(Duration::from_millis(2_000)).await;
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
                .unwrap();
        });
        let token = expected_token_info.to_owned();
        let client_id = CLIENT_ID.to_owned();
        let tmp_path_1 = tmp_path.to_owned();
        let handle2 = tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            let mut file_state = FileState::_from(tmp_path_1.to_owned()).unwrap();
            file_state
                .upsert_token_info(client_id, token.to_owned())
                .unwrap();
        });

        let _ = join!(handle1, handle2);

        let mut file_state = FileState::_from(tmp_path.to_owned()).unwrap();
        let actual_token_info = file_state.read_token_info(&CLIENT_ID.to_owned()).unwrap();

        assert_eq!(
            actual_token_info.access_token,
            expected_token_info.access_token
        );
    }
}
