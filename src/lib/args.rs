use clap::{ArgEnum, Parser, Subcommand};
use dotenv::dotenv;

#[derive(Subcommand, Debug)]
pub enum Flow {
    /// Authorization code with PKCE flow. More: https://www.rfc-editor.org/rfc/rfc7636
    AuthorizationCodeWithPKCE {
        /// Port for callback url
        #[clap(long, default_value_t = 8081, env = "DOKEN_PORT")]
        port: u16,
    },
    /// Authorization code flow. More: https://www.rfc-editor.org/rfc/rfc6749#section-1.3.1
    AuthorizationCode {
        /// Port for callback url
        #[clap(long, default_value_t = 8081, env = "DOKEN_PORT")]
        port: u16,
    },
    // TODO: Implement flows
    // /// Implicit flow. More: https://www.rfc-editor.org/rfc/rfc6749#section-1.3.2
    // Implicit,
    // /// Client credentials flow. More: https://www.rfc-editor.org/rfc/rfc6749#section-1.3.4
    // ClientCredentials,
}

#[derive(Debug, ArgEnum, Clone)]
pub enum TokenType {
    IdToken,
    AccessToken,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Arguments {
    /// Authentication flow
    #[clap(subcommand)]
    pub flow: Flow,

    /// OAuth 2.0 token exchange url
    #[clap(long, env = "DOKEN_TOKEN_URL")]
    pub token_url: String,

    /// OAuth 2.0 authorization initiation url
    #[clap(long, env = "DOKEN_AUTHORIZATION_URL")]
    pub authorization_url: String,

    /// OAuth 2.0 Client Identifier https://www.rfc-editor.org/rfc/rfc6749#section-2.2
    #[clap(long, env = "DOKEN_CLIENT_ID")]
    pub client_id: String,

    /// OAuth 2.0 Client Secret https://www.rfc-editor.org/rfc/rfc6749#section-2.3.1
    #[clap(long, env = "DOKEN_CLIENT_SECRET")]
    pub client_secret: Option<String>,

    /// OAuth 2.0 Scope https://www.rfc-editor.org/rfc/rfc6749#section-3.3
    #[clap(long, default_value = "offline_access", env = "DOKEN_SCOPE")]
    pub scope: String,

    /// OpenID Connect requested aud
    #[clap(long, env = "DOKEN_AUDIENCE")]
    pub audience: Option<String>,

    /// Token type: OpenID Connect ID Token or OAuth 2.0 Access Token
    #[clap(long, arg_enum, default_value_t = TokenType::AccessToken, env = "DOKEN_TOKEN_TYPE")]
    pub token_type: TokenType,
}

pub struct Args;

impl Args {
    pub fn parse() -> Arguments {
        if dotenv().is_ok() {}

        Arguments::parse()
    }
}
