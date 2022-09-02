use crate::lib::args::Arguments;
use crate::lib::oauth_client::OAuthClient;
use crate::{FileState, TokenInfo, TokenRetriever};
use async_trait::async_trait;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::time::SystemTime;

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
    file_state: FileState,
    args: &'a Arguments,
}

impl<'a> FileRetriever<'a> {
    pub async fn new(args: &Arguments) -> Result<FileRetriever, Box<dyn Error>> {
        Ok(FileRetriever {
            oauth_client: OAuthClient::new(args).await?,
            file_state: FileState::new(),
            args,
        })
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenInfo, Box<dyn Error>> {
        let result = self
            .oauth_client
            .refresh_token(refresh_token.to_owned())
            .await;

        match result {
            Ok(token_response) => {
                let token_info = TokenInfo::from_token_response(token_response);

                self.file_state
                    .upsert_token_info(self.args.client_id.to_owned(), token_info.to_owned())
                    .await?;

                Ok(token_info)
            }
            Err(_) => {
                self.file_state
                    .clear_token_info(self.args.client_id.to_owned())
                    .await?;

                Err(Box::new(TokenInfoNotFoundError {}))
            }
        }
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for FileRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo, Box<dyn Error>> {
        let token_info = self.file_state.read_token_info(&self.args.client_id).await;

        if token_info.is_none() {
            return Err(Box::new(TokenInfoNotFoundError {}));
        }

        let token_info = token_info.unwrap();

        let expires = token_info.expires.unwrap_or_else(SystemTime::now);

        let is_token_expired = expires < SystemTime::now();

        if !is_token_expired {
            return Ok(token_info);
        }

        match token_info.refresh_token {
            Some(token) => {
                let token_info = self.refresh_token(&token).await?;

                Ok(token_info)
            }
            None => {
                self.file_state
                    .clear_token_info(self.args.client_id.to_owned())
                    .await?;

                Err(Box::new(TokenInfoNotFoundError {}))
            }
        }
    }
}
