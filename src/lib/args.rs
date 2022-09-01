use clap::{ArgEnum, Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Flow {
    AuthorizationCodeWithPKCE {
        #[clap(long, default_value_t = 8081, env = "DOKEN_PORT")]
        port: u16,
    },
    AuthorizationCode {
        #[clap(long, default_value_t = 8081, env = "DOKEN_PORT")]
        port: u16,
    },
    Implicit,
    ClientCredentials,
}

#[derive(Debug, ArgEnum, Clone)]
pub enum TokenType {
    IdToken,
    AccessToken,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Arguments {
    #[clap(subcommand)]
    pub flow: Flow,

    #[clap(long, env = "DOKEN_TOKEN_URL")]
    pub token_url: String,

    #[clap(long, env = "DOKEN_AUTHORIZATION_URL")]
    pub authorization_url: String,

    #[clap(long, env = "DOKEN_CLIENT_ID")]
    pub client_id: String,

    #[clap(long, env = "DOKEN_CLIENT_SECRET")]
    pub client_secret: Option<String>,

    #[clap(long, default_value = "offline_access", env = "DOKEN_SCOPE")]
    pub scope: String,

    #[clap(long, env = "DOKEN_AUDIENCE")]
    pub audience: Option<String>,

    #[clap(long, arg_enum, default_value_t = TokenType::AccessToken, env = "DOKEN_TOKEN_TYPE")]
    pub token_type: TokenType,
}

pub struct Args;

impl Args {
    pub fn parse() -> Arguments {
        Arguments::parse()
    }
}
