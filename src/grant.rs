use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ValueEnum, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Grant {
    /// Authorization code with PKCE Grant. More: <https://www.rfc-editor.org/rfc/rfc7636>
    AuthorizationCodeWithPkce,
    /// Authorization Code Grant. More: <https://www.rfc-editor.org/rfc/rfc6749#section-4.1>
    AuthorizationCode,
    /// Implicit Grant. More: <https://www.rfc-editor.org/rfc/rfc6749#section-4.2>
    Implicit,
    /// Resource Owner Client Credentials Grant. More: <https://www.rfc-editor.org/rfc/rfc6749#section-4.3>
    ResourceOwnerPasswordClientCredentials,
    /// Client credentials Grant. More: <https://www.rfc-editor.org/rfc/rfc6749#section-4.4>
    ClientCredentials,
}
