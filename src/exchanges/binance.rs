use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use super::{CexExchange, Exchange};
use crate::{
    clients::{
        http::errors::HttpError,
        ws::{errors::WsError, mutli::MutliWsStreamBuilder}
    },
    types::binance::{
        channels::BinanceChannel,
        messages::BinanceWsMessage,
        responses::{all_symbols::BinanceAllSymbolsResponse, BinanceHttpResponse}
    }
};

const WSS_URL: &str = "wss://stream.binance.com:443/stream?";
const BASE_REST_API_URL: &str = "https://api.binance.com/api/v3";
const ALL_SYMBOLS_URL: &str = "https://www.binance.com/bapi/composite/v1/public/promo/cmc/cryptocurrency/listings/latest";

#[derive(Debug, Clone)]
pub struct Binance {
    ws_url: String
}

impl Binance {
    pub fn new_ws_subscription(ws_url: String) -> Self {
        Self { ws_url }
    }
}

#[async_trait::async_trait]
impl Exchange for Binance {
    type HttpMessage = BinanceHttpResponse;
    type WsMessage = BinanceWsMessage;

    const EXCHANGE: CexExchange = CexExchange::Binance;

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        let (ws, _) = tokio_tungstenite::connect_async(&self.ws_url).await?;

        Ok(ws)
    }

    async fn all_symbols(web_client: &reqwest::Client) -> Result<BinanceHttpResponse, HttpError> {
        let currencies: BinanceAllSymbolsResponse = web_client.get(ALL_SYMBOLS_URL).send().await?.json().await?;

        Ok(BinanceHttpResponse::Symbols(currencies))
    }
}

#[derive(Debug, Clone, Default)]
pub struct BinanceWsBuilder {
    pub channels:            Vec<BinanceChannel>,
    pub channels_per_stream: Option<usize>
}

impl BinanceWsBuilder {
    /// adds a channel to the builder
    pub fn add_channel(mut self, channel: BinanceChannel) -> Self {
        self.channels.push(channel);
        self
    }

    /// sets the number of channels
    pub fn set_channels_per_stream(mut self, channels_per_stream: usize) -> Self {
        self.channels_per_stream = Some(channels_per_stream);
        self
    }

    /// builds a single ws instance of [Binance], handling all channels on 1
    /// stream
    pub fn build(self) -> Binance {
        let base_url = WSS_URL.to_string();
        let channel_urls = self
            .channels
            .into_iter()
            .map(|c| c.build_url())
            .collect::<Vec<_>>();

        let url = format!("{base_url}{}", channel_urls.join("/"));

        Binance::new_ws_subscription(url)
    }

    /// builds many ws instances of the [Binance] as the inner streams of
    /// [MutliWsStreamBuilder] IFF 'channels_per_stream' is set, splitting
    /// channels by the specified number
    pub fn build_many(self) -> MutliWsStreamBuilder<Binance> {
        let base_url = WSS_URL.to_string();
        if let Some(per_stream) = self.channels_per_stream {
            let chunks = self.channels.chunks(per_stream).collect::<Vec<_>>();
            let split_exchange = chunks
                .into_iter()
                .map(|chk| {
                    let channel_urls = chk.iter().map(|c| c.build_url()).collect::<Vec<_>>();

                    let url = format!("{base_url}{}", channel_urls.join("/"));

                    Binance::new_ws_subscription(url)
                })
                .collect();

            MutliWsStreamBuilder::new(split_exchange)
        } else {
            panic!("'channels_per_stream' was not set")
        }
    }
}
