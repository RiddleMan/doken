use crate::{OAuthClient, token_info::TokenInfo};
use anyhow::Result;
use async_trait::async_trait;

use super::token_retriever::TokenRetriever;

pub struct ResourceOwnerPasswordClientCredentialsRetriever<'a> {
    oauth_client: &'a OAuthClient<'a>,
}

impl ResourceOwnerPasswordClientCredentialsRetriever<'_> {
    pub fn new<'b>(
        oauth_client: &'b OAuthClient<'b>,
    ) -> ResourceOwnerPasswordClientCredentialsRetriever<'b> {
        ResourceOwnerPasswordClientCredentialsRetriever { oauth_client }
    }
}

#[async_trait(?Send)]
impl TokenRetriever for ResourceOwnerPasswordClientCredentialsRetriever<'_> {
    async fn retrieve(&mut self) -> Result<TokenInfo> {
        Ok(TokenInfo::from_token_response(
            self.oauth_client
                .exchange_resource_owner_password_client_credentials()
                .await?,
        ))
    }
}
