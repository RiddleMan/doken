use crate::TokenInfo;
use async_trait::async_trait;

#[async_trait]
pub trait TokenRetriever {
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn std::error::Error>>;
}
