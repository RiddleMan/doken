use crate::{lib::token_info::TokenInfo, OAuthClient};
use anyhow::Result;
use async_trait::async_trait;

use super::token_retriever::TokenRetriever;

pub struct ResourceOwnerPasswordClientCredentialsRetriever<'a> {
    oauth_client: &'a OAuthClient<'a>,
}

impl<'a> ResourceOwnerPasswordClientCredentialsRetriever<'a> {
    pub fn new<'b>(
        oauth_client: &'b OAuthClient<'b>,
    ) -> ResourceOwnerPasswordClientCredentialsRetriever<'b> {
        ResourceOwnerPasswordClientCredentialsRetriever { oauth_client }
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for ResourceOwnerPasswordClientCredentialsRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo> {
        Ok(TokenInfo::from_token_response(
            self.oauth_client
                .exchange_resource_owner_password_client_credentials()
                .await?,
        ))
    }
}
