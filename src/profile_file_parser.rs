use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
