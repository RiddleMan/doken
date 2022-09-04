use crate::lib::args::Flow;
use crate::lib::authorization_code_retriever::AuthorizationCodeRetriever;
use crate::lib::authorization_code_with_pkce_retriever::AuthorizationCodeWithPKCERetriever;
use crate::lib::client_credentials_retriever::ClientCredentialsRetriever;
use crate::lib::file_retriever::FileRetriever;
use crate::lib::file_state::FileState;
use crate::lib::implicit_retriever::ImplicitRetriever;
use crate::lib::oauth_client::OAuthClient;
use crate::lib::token_retriever::TokenRetriever;
use lib::token_info::TokenInfo;
use std::env;
use std::process::exit;

mod lib;

fn enable_debug_via_args() {
    let has_debug_flag = env::args().any(|s| s.eq("--debug") || s.eq("-d"));

    if env::var("RUST_LOG").is_err() && has_debug_flag {
        env::set_var("RUST_LOG", "debug")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_debug_via_args();
    env_logger::init();

    let args = lib::args::Args::parse()?;

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

    let retriever: Box<dyn TokenRetriever> = match args.flow {
        Flow::AuthorizationCodeWithPKCE { .. } => Box::new(
            AuthorizationCodeWithPKCERetriever::new(&args, &oauth_client),
        ),
        Flow::AuthorizationCode { .. } => {
            Box::new(AuthorizationCodeRetriever::new(&args, &oauth_client))
        }
        Flow::Implicit => Box::new(ImplicitRetriever::new(&args)),
        Flow::ClientCredentials => Box::new(ClientCredentialsRetriever::new(&args)),
    };

    let token_info = retriever.retrieve().await?;

    file_state
        .upsert_token_info(args.client_id.to_owned(), token_info.to_owned())
        .await?;

    println!("{}", token_info.access_token);
    exit(0);
}
