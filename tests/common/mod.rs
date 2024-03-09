use home::home_dir;
use std::fs::remove_file;

pub mod keycloak;
pub mod keycloak_client;

pub fn remove_config_if_available() {
    let mut doken_config = home_dir().unwrap();
    doken_config.push(".doken.json");
    let _ = remove_file(doken_config);
}

pub fn assert_token_like(s: String) {
    let token_parts: Vec<&str> = s.split('.').collect();

    assert_eq!(token_parts.len(), 3);
}
