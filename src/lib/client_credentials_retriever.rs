use crate::lib::args::Arguments;
use crate::lib::token_retriever::TokenRetriever;
use crate::TokenInfo;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TokenEndpointResponse {
    access_token: String,
    expires_in: u32,
    token_type: String,
}

pub struct ClientCredentialsRetriever<'a> {
    args: &'a Arguments,
}

impl<'a> ClientCredentialsRetriever<'a> {
    pub async fn new(
        args: &Arguments,
    ) -> Result<ClientCredentialsRetriever, Box<dyn std::error::Error>> {
        Ok(ClientCredentialsRetriever { args })
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for ClientCredentialsRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn std::error::Error>> {
        panic!("not implemented")
    }
}
