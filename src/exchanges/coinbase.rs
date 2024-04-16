use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use super::{normalized::CexExchange, Exchange};
use crate::{
    http::errors::HttpError,
    types::coinbase::{
        channels::{CoinbaseChannel, CoinbaseSubscription},
        messages::CoinbaseWsMessage,
        responses::{
            all_currencies::{CoinbaseAllCurrenciesProperties, CoinbaseAllCurrenciesResponse},
            CoinbaseHttpResponse
        }
    },
    ws::{errors::WsError, mutli::MutliWsStreamBuilder}
};

const WSS_URL: &str = "wss://ws-feed.exchange.coinbase.com";
const BASE_REST_API_URL: &str = "https://api.exchange.coinbase.com";

#[derive(Debug, Clone)]
pub struct Coinbase {
    subscription: CoinbaseSubscription
}

impl Coinbase {
    pub fn new_ws_subscription(sub: CoinbaseSubscription) -> Self {
        Self { subscription: sub }
    }
}

#[async_trait::async_trait]
impl Exchange for Coinbase {
    type HttpMessage = CoinbaseHttpResponse;
    type WsMessage = CoinbaseWsMessage;

    const EXCHANGE: CexExchange = CexExchange::Coinbase;

    async fn make_ws_connection(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
        let (mut ws, _) = tokio_tungstenite::connect_async(WSS_URL).await?;

        let sub_message = serde_json::to_string(&self.subscription)?;
        ws.send(Message::Text(sub_message)).await?;

        Ok(ws)
    }

    async fn all_symbols(web_client: &reqwest::Client) -> Result<CoinbaseHttpResponse, HttpError> {
        let url = format!("{BASE_REST_API_URL}/currencies");

        let response = web_client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await?
            .text()
            .await?;

        // println!("MSG: {}", response);

        let currencies: Vec<CoinbaseAllCurrenciesProperties> = serde_json::from_str(&response)?;

        Ok(CoinbaseHttpResponse::Currencies(CoinbaseAllCurrenciesResponse { currencies }))
    }
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct CoinbaseWsBuilder {
    pub channels:            Vec<CoinbaseChannel>,
    pub channels_per_stream: Option<usize>
}

impl CoinbaseWsBuilder {
    /// adds a channel to the builder
    pub fn add_channel(mut self, channel: CoinbaseChannel) -> Self {
        self.channels.push(channel);
        self
    }

    /// splits a [CoinbaseChannel] (with values) into mutliple instance of the
    /// same [CoinbaseChannel], each with fewer trading pairs
    ///
    /// if 'split_channel_size' is not passed, each trading pair will have it's
    /// own stream
    pub fn add_split_channel(mut self, channel: CoinbaseChannel, split_channel_size: Option<usize>) -> Self {
        match channel {
            CoinbaseChannel::Status => self.channels.push(channel),
            CoinbaseChannel::Match(Some(vals)) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(vals.len()), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| CoinbaseChannel::Match(Some(chk.to_vec())))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }

            CoinbaseChannel::Ticker(Some(vals)) => {
                let split_size = std::cmp::min(split_channel_size.unwrap_or(vals.len()), vals.len());
                let chunks = vals.chunks(split_size).collect::<Vec<_>>();
                let split_channels = chunks
                    .into_iter()
                    .map(|chk| CoinbaseChannel::Ticker(Some(chk.to_vec())))
                    .collect::<Vec<_>>();
                self.channels.extend(split_channels)
            }
            CoinbaseChannel::Match(None) | CoinbaseChannel::Ticker(None) => unreachable!("if passing with no symbols, use 'add_channel()' instead")
        }

        self
    }

    /// sets the number of channels
    pub fn set_channels_per_stream(mut self, channels_per_stream: usize) -> Self {
        self.channels_per_stream = Some(channels_per_stream);
        self
    }

    /// builds a single ws instance of [Coinbase], handling all channels on 1
    /// stream
    pub fn build(self) -> Coinbase {
        let mut sub = CoinbaseSubscription::new();
        self.channels.into_iter().for_each(|c| sub.add_channel(c));

        Coinbase::new_ws_subscription(sub)
    }

    /// builds many ws instances of the [Coinbase] as the inner streams of
    /// [MutliWsStreamBuilder] IFF 'channels_per_stream' is set, splitting
    /// channels by the specified number
    pub fn build_many(self) -> MutliWsStreamBuilder<Coinbase> {
        if let Some(per_stream) = self.channels_per_stream {
            let chunks = self.channels.chunks(per_stream).collect::<Vec<_>>();
            let split_exchange = chunks
                .into_iter()
                .map(|chk| {
                    let mut sub = CoinbaseSubscription::new();

                    chk.iter().for_each(|c| sub.add_channel(c.clone()));

                    Coinbase::new_ws_subscription(sub)
                })
                .collect();

            MutliWsStreamBuilder::new(split_exchange)
        } else {
            panic!("'channels_per_stream' was not set")
        }
    }
}


