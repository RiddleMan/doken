use crate::lib::args::Flow;
use crate::lib::authorization_code_retriever::AuthorizationCodeRetriever;
use crate::lib::authorization_code_with_pkce_retriever::AuthorizationCodeWithPKCERetriever;
use crate::lib::file_retriever::FileRetriever;
use crate::lib::file_state::FileState;
use crate::lib::oauth_client::OAuthClient;
use crate::lib::token_retriever::TokenRetriever;
use lib::token_info::TokenInfo;
use log::LevelFilter;
use std::process::exit;

mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = lib::args::Args::parse()?;

    if args.debug {
        log::set_max_level(LevelFilter::Debug);
    }

    let file_state = FileState::new();
    let oauth_client = OAuthClient::new(&args).await?;

    if !args.force {
        let file_retriever = FileRetriever::new(&args, &oauth_client).await?;

        let file_token_info = file_retriever.retrieve().await;

        if file_token_info.is_ok() {
            println!("{}", file_token_info.unwrap().access_token);
            exit(0);
        }
    }

    match args.flow {
        Flow::AuthorizationCodeWithPKCE { port: _port } => {
            let token = AuthorizationCodeWithPKCERetriever::new(&args, &oauth_client)
                .await?
                .retrieve()
                .await?;

            file_state
                .upsert_token_info(args.client_id, token.clone())
                .await?;

            println!("{}", token.access_token);
            exit(0);
        }
        Flow::AuthorizationCode { port: _port } => {
            let token = AuthorizationCodeRetriever::new(&args, &oauth_client)
                .await?
                .retrieve()
                .await?;

            file_state
                .upsert_token_info(args.client_id, token.clone())
                .await?;

            println!("{}", token.access_token);
            exit(0);
        }
    }
}
