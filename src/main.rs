use crate::lib::args::Flow;
use crate::lib::authorization_code_with_pkce_retriever::AuthorizationCodeWithPKCERetriever;
use crate::lib::file_retriever::FileRetriever;
use crate::lib::file_state::FileState;
use crate::lib::token_retriever::TokenRetriever;
use lib::token_info::TokenInfo;
use std::process::exit;

mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = lib::args::Args::parse();
    let file_state = FileState::new();
    let file_retriever = FileRetriever::new(&args);

    let file_token_info = file_retriever.retrieve().await;

    if file_token_info.is_ok() {
        println!("{}", file_token_info.unwrap().access_token);
        exit(0);
    }

    match args.flow {
        Flow::AuthorizationCodeWithPKCE { port: _port } => {
            let token = AuthorizationCodeWithPKCERetriever::new(&args)?
                .retrieve()
                .await?;

            file_state
                .upsert_token_info(args.client_id, token.clone())
                .await?;

            println!("{}", token.access_token);
            exit(0);
        }
        _ => panic!("Not implemented"),
    }
}
