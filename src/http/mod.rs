use self::errors::HttpError;
use crate::{exchanges::Exchange, types::normalized::http::combined::CombinedHttpResponse};

pub mod errors;

pub struct ExchangeApi {
    web_client: reqwest::Client
}

impl Default for ExchangeApi {
    fn default() -> Self {
        Self::new()
    }
}

impl ExchangeApi {
    pub fn new() -> Self {
        Self { web_client: reqwest::Client::new() }
    }

    pub async fn all_symbols<E: Exchange>(&self) -> Result<CombinedHttpResponse, HttpError> {
        Ok(E::all_symbols(&self.web_client).await?.into())
    }
}
