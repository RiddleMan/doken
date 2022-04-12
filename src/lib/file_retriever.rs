use crate::lib::args::Arguments;
use crate::lib::oauth_client::OAuthClient;
use crate::{FileState, TokenInfo, TokenRetriever};
use async_trait::async_trait;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
struct TokenInfoNotFoundError;

impl Display for TokenInfoNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token not found in state file")
    }
}

impl Error for TokenInfoNotFoundError {}

pub struct FileRetriever<'a> {
    oauth_client: OAuthClient<'a>,
    args: &'a Arguments,
}

impl<'a> FileRetriever<'a> {
    pub fn new(args: &Arguments) -> Result<FileRetriever, Box<dyn Error>> {
        Ok(FileRetriever {
            oauth_client: OAuthClient::new(args)?,
            args,
        })
    }
}

#[async_trait]
impl<'a> TokenRetriever for FileRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn Error>> {
        let file_state = FileState::new();

        let token_info = file_state.read_token_info(&self.args.client_id).await;

        if token_info.is_none() {
            return Err(Box::new(TokenInfoNotFoundError {}));
        }

        Ok(token_info.unwrap())
    }
}
