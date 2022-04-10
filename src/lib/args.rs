use clap::{ArgEnum, Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Flow {
    AuthorizationCodeWithPKCE,
    AuthorizationCode,
    Implicit,
    ClientCredentials,
}

#[derive(Debug, ArgEnum, Clone)]
pub enum Token {
    IdToken,
    AccessToken,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Arguments {
    #[clap(subcommand)]
    pub flow: Flow,

    #[clap(long)]
    pub issuer_url: String,

    #[clap(long)]
    pub client_id: String,

    #[clap(long, default_value = "offline_access")]
    pub scope: String,

    #[clap(long, arg_enum, default_value_t = Token::AccessToken)]
    pub token: Token,
}

pub struct Args;

impl Args {
    pub fn parse() -> Arguments {
        Arguments::parse()
    }
}
