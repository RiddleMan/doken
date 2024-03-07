use crate::args::Arguments;
use crate::auth_browser::auth_browser::AuthPage;
use crate::token_info::TokenInfo;
use crate::OAuthClient;
use anyhow::Result;
use async_trait::async_trait;
use url::Url;

use super::token_retriever::TokenRetriever;

pub struct ImplicitRetriever<'a> {
    args: &'a Arguments,
    oauth_client: &'a OAuthClient<'a>,
    auth_page: AuthPage,
}

impl<'a> ImplicitRetriever<'a> {
    pub fn new<'b>(
        args: &'b Arguments,
        oauth_client: &'b OAuthClient<'b>,
        auth_page: AuthPage,
    ) -> ImplicitRetriever<'b> {
        ImplicitRetriever { args, oauth_client, auth_page }
    }
}

#[async_trait(?Send)]
impl<'a> TokenRetriever for ImplicitRetriever<'a> {
    async fn retrieve(&mut self) -> Result<TokenInfo> {
        let (url, csrf) = self.oauth_client.implicit_url();

        self.auth_page
            .get_token_data(self.args.timeout,url, Url::parse(self.args.callback_url.as_deref().unwrap())?, csrf)
            .await
    }
}
