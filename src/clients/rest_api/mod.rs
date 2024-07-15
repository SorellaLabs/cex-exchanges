mod errors;
pub use errors::*;

use crate::{exchanges::normalized::rest_api::CombinedRestApiResponse, normalized::rest_api::NormalizedRestApiRequest, Exchange};

#[derive(Debug, Default)]
pub struct ExchangeApi {
    web_client: reqwest::Client
}

impl ExchangeApi {
    pub fn new() -> Self {
        Self { web_client: reqwest::Client::new() }
    }

    pub async fn all_currencies<E: Exchange>(&self) -> Result<CombinedRestApiResponse, RestApiError> {
        Ok(E::rest_api_call(&E::default(), &self.web_client, NormalizedRestApiRequest::AllCurrencies)
            .await?
            .into())
    }

    pub async fn all_instruments<E: Exchange>(&self) -> Result<CombinedRestApiResponse, RestApiError> {
        Ok(E::rest_api_call(&E::default(), &self.web_client, NormalizedRestApiRequest::AllInstruments)
            .await?
            .into())
    }

    pub async fn all_trade_fees<E: Exchange>(&self) -> Result<CombinedRestApiResponse, RestApiError> {
        Ok(E::rest_api_call(&E::default(), &self.web_client, NormalizedRestApiRequest::AllTradeFees)
            .await?
            .into())
    }
}
