use crate::{token_info::TokenInfo, OAuthClient};
use anyhow::Result;
use async_trait::async_trait;

use super::token_retriever::TokenRetriever;

pub struct ClientCredentialsRetriever<'a> {
    oauth_client: &'a OAuthClient<'a>,
}

impl<'a> ClientCredentialsRetriever<'a> {
    pub fn new<'b>(oauth_client: &'b OAuthClient<'b>) -> ClientCredentialsRetriever<'b> {
        ClientCredentialsRetriever { oauth_client }
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for ClientCredentialsRetriever<'a> {
    async fn retrieve(&mut self) -> Result<TokenInfo> {
        Ok(TokenInfo::from_token_response(
            self.oauth_client.exchange_client_credentials().await?,
        ))
    }
}
