use crate::args::Arguments;
use crate::lib::oauth_client::OAuthClient;
use crate::lib::token_info::TokenInfo;
use crate::FileState;
use anyhow::Result;
use async_trait::async_trait;
use std::time::SystemTime;
use thiserror::Error;

use super::token_retriever::TokenRetriever;

#[derive(Error, Debug)]
enum FileRetrieverError {
    #[error("Token not found in state file")]
    TokenInfoNotFound,
}

pub struct FileRetriever<'a> {
    oauth_client: &'a OAuthClient<'a>,
    file_state: FileState,
    args: &'a Arguments,
}

impl<'a> FileRetriever<'a> {
    pub fn new<'b>(args: &'b Arguments, oauth_client: &'b OAuthClient<'b>) -> FileRetriever<'b> {
        FileRetriever {
            oauth_client,
            file_state: FileState::new(),
            args,
        }
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenInfo> {
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

                Err(FileRetrieverError::TokenInfoNotFound.into())
            }
        }
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for FileRetriever<'a> {
    async fn retrieve(&self) -> Result<TokenInfo> {
        let token_info = self.file_state.read_token_info(&self.args.client_id).await;

        if token_info.is_none() {
            return Err(FileRetrieverError::TokenInfoNotFound.into());
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

                Err(FileRetrieverError::TokenInfoNotFound.into())
            }
        }
    }
}
