#![deny(warnings)]

use crate::file_state::FileState;
use crate::grant::Grant;
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
use auth_browser::AuthBrowser;
use crate::args::Arguments;
use std::process::exit;
use crate::token_info::TokenInfo;

pub mod args;
pub mod auth_browser;
mod config_file;
mod file_state;
pub mod grant;
mod oauth_client;
mod openidc_discovery;
mod retrievers;
mod token_info;

pub async fn get_token(args: Arguments, auth_browser: &mut AuthBrowser) -> Result<String> {
    let file_state = FileState::new();
    let oauth_client = OAuthClient::new(&args).await?;

    if !args.force {
        let mut file_retriever = FileRetriever::new(&args, &oauth_client);

        let file_token_info = file_retriever.retrieve().await;

        if file_token_info.is_ok() {
            println!("{}", file_token_info.unwrap().access_token);
            exit(0);
        }
    }

    let mut retriever: Box<dyn TokenRetriever> = match args.grant {
        Grant::AuthorizationCodeWithPkce { .. } => Box::new(
            AuthorizationCodeWithPKCERetriever::new(&args, &oauth_client, auth_browser),
        ),
        Grant::AuthorizationCode { .. } => {
            Box::new(AuthorizationCodeRetriever::new(&args, &oauth_client, auth_browser))
        }
        Grant::Implicit => Box::new(ImplicitRetriever::new(&args, &oauth_client, auth_browser)),
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

    Ok(token_info.access_token)
}
