use crate::lib::args::Flow;
use crate::lib::authorization_code_with_pkce_retriever::AuthorizationCodeWithPKCERetriever;

mod lib;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = lib::args::Args::parse();

    println!("{:#?}", args);

    match args.flow {
        Flow::AuthorizationCodeWithPKCE { port: _port } => {
            AuthorizationCodeWithPKCERetriever::retrieve(args).await
        }
        _ => panic!("Not implemented"),
    }
}
