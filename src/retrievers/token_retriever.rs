use anyhow::Result;
use async_trait::async_trait;

use crate::lib::token_info::TokenInfo;

#[async_trait(?Send)]
pub trait TokenRetriever {
    async fn retrieve(&self) -> Result<TokenInfo>;
}
