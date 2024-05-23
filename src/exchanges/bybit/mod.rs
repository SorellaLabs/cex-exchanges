mod pairs;

use std::collections::HashSet;

use futures::SinkExt;
pub use pairs::*;

pub mod rest_api;
pub mod ws;

use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use self::{
    rest_api::{BybitAllCoins, BybitAllInstruments, BybitRestApiResponse},
    ws::{BybitSubscription, BybitWsMessage}
};
use crate::{
    binance::Binance,
    clients::{rest_api::RestApiError, ws::WsError},
    exchanges::Exchange,
    normalized::{rest_api::NormalizedRestApiRequest, types::NormalizedTradingPair},
    CexExchange
};

const WSS_URL: &str = "wss://stream.bybit.com/v5/public/spot";
const BASE_REST_API_URL: &str = "https://api.bybit.com";

#[derive(Debug, Default, Clone)]
pub struct Bybit {
    subscription: BybitSubscription
}

impl Bybit {
    pub fn new_ws_subscription(subscription: BybitSubscription) -> Self {
        Self { subscription }
    }

    pub async fn get_all_instruments(web_client: &reqwest::Client) -> Result<BybitAllInstruments, RestApiError> {
        let categories = ["linear", "inverse", "option", "spot"];

        let mut instruments = Vec::new();
        for cat in categories {
            let url = format!("{BASE_REST_API_URL}/v5/market/instruments-info?category={cat}");
            let cat_instruments: BybitAllInstruments = Self::simple_rest_api_request(web_client, url).await?;
            instruments.extend(cat_instruments.instruments);
        }

        Ok(BybitAllInstruments { instruments })
    }

    // pub async fn get_all_coins(web_client: &reqwest::Client) ->
    // Result<BybitAllCoins, RestApiError> {     let url = format!("https://api.bybit.com/v5/asset/coin/query-info");
    //     let val = web_client
    //         .get(url)
    //         .header("X-BAPI-API-KEY", "CFEJUGQEQPPHGOHGHM")
    //         .header("X-BAPI-TIMESTAMP", (Utc::now().timestamp_millis() -
    // 100).to_string())         .header("X-BAPI-RECV-WINDOW", "20000")
    //         .header("X-BAPI-SIGN",
    // "ade795201a7920cf821ff3dba38d9ba44a475cbd2cc1e8f24662d8865e5bff65")
    //         .send()
    //         .await?
    //         .json()
    //         .await?;

    //     Ok(val)
    // }

    pub async fn get_all_coins(web_client: &reqwest::Client) -> Result<BybitAllCoins, RestApiError> {
        let mut binance_coins = Binance::default()
            .rest_api_call(web_client, NormalizedRestApiRequest::AllCurrencies)
            .await?
            .take_symbols()
            .unwrap();

        let bybit_instrument_symbols = Self::get_all_instruments(web_client)
            .await?
            .instruments
            .into_iter()
            .flat_map(|s| {
                let norm = s.normalize();
                vec![norm.base_asset_symbol, norm.quote_asset_symbol]
            })
            .collect::<HashSet<_>>();

        binance_coins.retain(|c| bybit_instrument_symbols.contains(&c.symbol));

        Ok(BybitAllCoins { coins: binance_coins.into_iter().map(Into::into).collect() })
    }

    pub async fn simple_rest_api_request<T>(web_client: &reqwest::Client, url: String) -> Result<T, RestApiError>
    where
        T: for<'de> Deserialize<'de>
    {
        let data = web_client.get(&url).send().await?.json().await?;
        Ok(data)
    }
}

#[async_trait::async_trait]
impl Exchange for Bybit {
    type RestApiResult = BybitRestApiResponse;
    type WsMessage = BybitWsMessage;

    const EXCHANGE: CexExchange = CexExchange::Bybit;

    fn remove_bad_pair(&mut self, bad_pair: NormalizedTradingPair) -> bool {
        let pair = bad_pair.try_into().unwrap();
        self.subscription.remove_pair(&pair)
    }

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        let (mut ws, _) = tokio_tungstenite::connect_async(WSS_URL).await?;

        let sub_message = serde_json::to_string(&self.subscription)?;
        ws.send(Message::Text(sub_message)).await?;

        Ok(ws)
    }

    async fn rest_api_call(&self, web_client: &reqwest::Client, api_channel: NormalizedRestApiRequest) -> Result<BybitRestApiResponse, RestApiError> {
        let api_response = match api_channel {
            NormalizedRestApiRequest::AllCurrencies => BybitRestApiResponse::Coins(Self::get_all_coins(web_client).await?),
            NormalizedRestApiRequest::AllInstruments => BybitRestApiResponse::Instruments(Self::get_all_instruments(web_client).await?)
        };

        Ok(api_response)
    }
}
