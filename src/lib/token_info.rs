use oauth2::basic::BasicTokenResponse;
use oauth2::TokenResponse;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::time::SystemTime;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TokenInfo {
    pub access_token: String,

    pub expires: Option<SystemTime>,

    pub scope: Option<String>,

    pub refresh_token: Option<String>,
}

impl TokenInfo {
    pub fn from_token_response(response: BasicTokenResponse) -> TokenInfo {
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
