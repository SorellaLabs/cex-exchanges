mod pairs;

use futures::SinkExt;
pub use pairs::*;

pub mod rest_api;
pub mod ws;

use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use self::{
    rest_api::{BybitAllIntruments, BybitRestApiResponse},
    ws::{BybitSubscription, BybitWsMessage}
};
use crate::{
    clients::{rest_api::RestApiError, ws::WsError},
    exchanges::Exchange,
    normalized::rest_api::NormalizedRestApiRequest,
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

    pub async fn get_all_instruments(web_client: &reqwest::Client) -> Result<BybitAllIntruments, RestApiError> {
        let categories = ["linear", "inverse", "option", "spot"];

        let mut instruments = Vec::new();
        for cat in categories {
            println!("STARTING: {cat}");
            let url = format!("{BASE_REST_API_URL}/v5/market/instruments-info?category={cat}");
            let cat_instruments: BybitAllIntruments = Self::simple_rest_api_request(web_client, url).await?;
            instruments.extend(cat_instruments.instruments);
        }

        Ok(BybitAllIntruments { instruments })
    }

    // TODO: fix
    // pub async fn get_all_coins(web_client: &reqwest::Client) ->
    // Result<BybitAllCoins, RestApiError> {

    //     let url = format!("https://api.bybit.com/v5/asset/coin/query-info");
    //     let val = web_client
    //         .get(url)
    //         .header("X-BAPI-API-KEY", "CFEJUGQEQPPHGOHGHM")
    //         .header("X-BAPI-TIMESTAMP", (Utc::now().timestamp_millis() -
    // 100).to_string())         .header("X-BAPI-RECV-WINDOW", "20000")
    //         .header("X-BAPI-SIGN",
    // "baca4a6a799d7cfed763880bbb13a4a90f41fa3f125db2168ede203747ce5c95")
    //         .send()
    //         .await?
    //         .json()
    //         .await?;

    //     Ok(val)
    // }

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

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        let (mut ws, _) = tokio_tungstenite::connect_async(WSS_URL).await?;

        let sub_message = serde_json::to_string(&self.subscription)?;
        ws.send(Message::Text(sub_message)).await?;

        Ok(ws)
    }

    async fn rest_api_call(&self, web_client: &reqwest::Client, api_channel: NormalizedRestApiRequest) -> Result<BybitRestApiResponse, RestApiError> {
        let api_response = match api_channel {
            NormalizedRestApiRequest::AllCurrencies => unimplemented!("BybitAllCoins"), /* BybitRestApiResponse::Coins(Self::get_all_coins(web_client).await?), */
            NormalizedRestApiRequest::AllInstruments => BybitRestApiResponse::Instruments(Self::get_all_instruments(&web_client).await?)
        };

        Ok(api_response)
    }
}
