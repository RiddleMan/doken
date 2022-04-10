use crate::lib::args::Flow;
use crate::lib::authorization_code_with_pkce_retriever::AuthorizationCodeWithPKCERetriever;

mod lib;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = lib::args::Args::parse();

    match args.flow {
        Flow::AuthorizationCodeWithPKCE { port: _port } => {
            AuthorizationCodeWithPKCERetriever::new(&args)
                .retrieve()
                .await
        }
        _ => panic!("Not implemented"),
    }
}
