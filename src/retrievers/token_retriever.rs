use crate::token_info::TokenInfo;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait TokenRetriever {
    async fn retrieve(&mut self) -> Result<TokenInfo>;
}
