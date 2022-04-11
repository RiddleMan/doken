use crate::lib::args::Flow;
use crate::lib::authorization_code_with_pkce_retriever::AuthorizationCodeWithPKCERetriever;
use crate::lib::file_state::FileState;
use crate::lib::token_retriever::TokenRetriever;

mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = lib::args::Args::parse();
    let file_state = FileState::new();

    match args.flow {
        Flow::AuthorizationCodeWithPKCE { port: _port } => {
            let token = AuthorizationCodeWithPKCERetriever::new(&args)?
                .retrieve()
                .await?;

            file_state.save(args.client_id, &token).await?;

            Ok(())
        }
        _ => panic!("Not implemented"),
    }
}
