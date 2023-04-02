use crate::TokenInfo;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait(?Send)]
pub trait TokenRetriever {
    async fn retrieve(&self) -> Result<TokenInfo>;
}
