use crate::lib::args::Flow;
use crate::lib::authorization_code_with_pkce_retriever::AuthorizationCodeWithPKCERetriever;
use crate::lib::token_retriever::TokenRetriever;

mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = lib::args::Args::parse();

    match args.flow {
        Flow::AuthorizationCodeWithPKCE { port: _port } => {
            AuthorizationCodeWithPKCERetriever::new(&args)?
                .retrieve()
                .await?;

            Ok(())
        }
        _ => panic!("Not implemented"),
    }
}
