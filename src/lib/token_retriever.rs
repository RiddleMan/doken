use async_trait::async_trait;
use oauth2::basic::BasicTokenResponse;

#[async_trait]
pub trait TokenRetriever {
    async fn retrieve(&self) -> Result<BasicTokenResponse, Box<dyn std::error::Error>>;
}
