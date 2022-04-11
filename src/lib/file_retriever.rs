use crate::{TokenInfo, TokenRetriever};
use async_trait::async_trait;
use std::error::Error;

struct FileRetriever {}

#[async_trait]
impl TokenRetriever for FileRetriever {
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn Error>> {
        todo!()
    }
}
