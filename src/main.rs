#![deny(warnings)]

use crate::args::Args;
use crate::args::Grant;
use crate::file_state::FileState;
use crate::oauth_client::OAuthClient;
use crate::retrievers::authorization_code_retriever::AuthorizationCodeRetriever;
use crate::retrievers::authorization_code_with_pkce_retriever::AuthorizationCodeWithPKCERetriever;
use crate::retrievers::client_credentials_retriever::ClientCredentialsRetriever;
use crate::retrievers::file_retriever::FileRetriever;
use crate::retrievers::implicit_retriever::ImplicitRetriever;
use crate::retrievers::resource_owner_password_client_credentials_retriever::ResourceOwnerPasswordClientCredentialsRetriever;
use crate::retrievers::token_retriever::TokenRetriever;
use anyhow::Context;
use anyhow::Result;
use std::env;
use std::process::exit;
use token_info::TokenInfo;

mod args;
mod auth_browser;
mod auth_server;
mod file_state;
mod oauth_client;
mod openidc_discovery;
mod retrievers;
mod token_info;

fn enable_debug_via_args() {
    let has_debug_flag = env::args().any(|s| s.eq("--debug") || s.eq("-d"));

    if env::var("RUST_LOG").is_err() && has_debug_flag {
        env::set_var("RUST_LOG", "debug")
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    enable_debug_via_args();
    env_logger::init();

    let args = Args::parse();

    let file_state = FileState::new();
    let oauth_client = OAuthClient::new(&args).await?;

    if !args.force {
        let file_retriever = FileRetriever::new(&args, &oauth_client);

        let file_token_info = file_retriever.retrieve().await;

        if file_token_info.is_ok() {
            println!("{}", file_token_info.unwrap().access_token);
            exit(0);
        }
    }

    let retriever: Box<dyn TokenRetriever> = match args.grant {
        Grant::AuthorizationCodeWithPKCE { .. } => Box::new(
            AuthorizationCodeWithPKCERetriever::new(&args, &oauth_client),
        ),
        Grant::AuthorizationCode { .. } => {
            Box::new(AuthorizationCodeRetriever::new(&args, &oauth_client))
        }
        Grant::Implicit => Box::new(ImplicitRetriever::new(&args, &oauth_client)),
        Grant::ResourceOwnerPasswordClientCredentials => Box::new(
            ResourceOwnerPasswordClientCredentialsRetriever::new(&oauth_client),
        ),
        Grant::ClientCredentials => Box::new(ClientCredentialsRetriever::new(&oauth_client)),
    };

    let token_info = retriever
        .retrieve()
        .await
        .context("Failed to retrieve a token")?;

    file_state
        .upsert_token_info(args.client_id.to_owned(), token_info.to_owned())
        .await?;

    println!("{}", token_info.access_token);
    exit(0);
}
