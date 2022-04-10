use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Flows {
    AuthorizationCodeWithPKCE { client_id: String },
    AuthorizationCode,
    Implicit,
    ClientCredentials,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Arguments {
    #[clap(subcommand)]
    pub flow: Flows,

    #[clap(long)]
    pub issuer_url: String,
}

pub struct Args;

impl Args {
    pub fn parse() -> Arguments {
        Arguments::parse()
    }
}
