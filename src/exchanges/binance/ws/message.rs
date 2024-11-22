use serde::{Deserialize, Serialize};

use super::{
    channels::{BinanceBookTicker, BinanceDiffDepth, BinancePartialBookDepth, BinanceTrade},
    BinanceSubscriptionResponse
};
use crate::{clients::ws::CriticalWsMessage, exchanges::normalized::ws::NormalizedWsDataTypes, CexExchange};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "data")]
pub enum BinanceWsMessage {
    Trade(BinanceTrade),
    BookTicker(BinanceBookTicker),
    PartialBookDepth(BinancePartialBookDepth),
    DiffDepth(BinanceDiffDepth),
    SubscriptionResponse(BinanceSubscriptionResponse)
}

impl<'de> Deserialize<'de> for BinanceWsMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        Ok(private::BinanceWsMessageRaw::deserialize(deserializer)?.into())
    }
}

impl BinanceWsMessage {
    pub fn normalize(self) -> NormalizedWsDataTypes {
        match self {
            BinanceWsMessage::Trade(v) => NormalizedWsDataTypes::Trade(v.normalize()),
            BinanceWsMessage::BookTicker(v) => NormalizedWsDataTypes::Quote(v.normalize()),
            BinanceWsMessage::DiffDepth(v) => NormalizedWsDataTypes::L2(v.normalize()),
            BinanceWsMessage::PartialBookDepth(v) => NormalizedWsDataTypes::L2(v.normalize()),
            BinanceWsMessage::SubscriptionResponse(v) => NormalizedWsDataTypes::Other {
                exchange: CexExchange::Binance,
                kind:     "SUBSCRIBE".to_string(),
                value:    format!("result: {:?} -- id: {}", v.result, v.id)
            }
        }
    }
}

impl PartialEq<NormalizedWsDataTypes> for BinanceWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match (self, other) {
            (BinanceWsMessage::Trade(this), NormalizedWsDataTypes::Trade(that)) => this == that,
            (BinanceWsMessage::BookTicker(this), NormalizedWsDataTypes::Quote(that)) => this == that,
            (BinanceWsMessage::DiffDepth(this), NormalizedWsDataTypes::L2(that)) => this == that,
            (BinanceWsMessage::SubscriptionResponse { .. }, NormalizedWsDataTypes::Other { .. }) => true,
            _ => false
        }
    }
}
impl CriticalWsMessage for BinanceWsMessage {
    fn make_critical(&mut self, _msg: String) {}
}

mod private {
    #![allow(private_interfaces)]

    use serde_with::{serde_as, DisplayFromStr};

    use super::*;
    use crate::binance::{ws::channels::BinancePartialBookDepth, BinanceTradingPair};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub(super) enum BinanceWsMessageRaw {
        DataMsg(BinanceDataStreamMsg),
        DiffDepth(BinanceDiffDepth),
        OtherMsg(BinanceSubscriptionResponse),
        BookTicker(BinanceBookTicker)
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct BinanceDataStreamMsg {
        data:   BinanceWsMessageDataRaw,
        stream: String
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    enum BinanceWsMessageDataRaw {
        Trade(BinanceTrade),
        BookTicker(BinanceBookTicker),
        DiffDepth(BinanceDiffDepth),
        PartialBookDepth(BinancePartialBookDepthRaw)
    }

    impl From<BinanceWsMessageRaw> for BinanceWsMessage {
        fn from(value: BinanceWsMessageRaw) -> Self {
            match value {
                BinanceWsMessageRaw::DataMsg(msg) => match msg.data {
                    BinanceWsMessageDataRaw::Trade(v) => BinanceWsMessage::Trade(v),
                    BinanceWsMessageDataRaw::BookTicker(v) => BinanceWsMessage::BookTicker(v),
                    BinanceWsMessageDataRaw::DiffDepth(v) => BinanceWsMessage::DiffDepth(v),
                    BinanceWsMessageDataRaw::PartialBookDepth(v) => BinanceWsMessage::PartialBookDepth((v, parse_stream_to_pair(msg.stream)).into())
                },
                BinanceWsMessageRaw::OtherMsg(msg) => BinanceWsMessage::SubscriptionResponse(msg),
                BinanceWsMessageRaw::DiffDepth(v) => BinanceWsMessage::DiffDepth(v),
                BinanceWsMessageRaw::BookTicker(v) => BinanceWsMessage::BookTicker(v)
            }
        }
    }

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
    struct BinancePartialBookDepthRaw {
        #[serde_as(as = "Vec<Vec<DisplayFromStr>>")]
        bids:                Vec<Vec<f64>>,
        #[serde_as(as = "Vec<Vec<DisplayFromStr>>")]
        asks:                Vec<Vec<f64>>,
        #[serde(rename = "lastUpdateId")]
        orderbook_update_id: u64
    }

    impl From<(BinancePartialBookDepthRaw, BinanceTradingPair)> for BinancePartialBookDepth {
        fn from(value: (BinancePartialBookDepthRaw, BinanceTradingPair)) -> Self {
            let (value, pair) = value;
            BinancePartialBookDepth { pair, bids: value.bids, asks: value.asks, orderbook_update_id: value.orderbook_update_id }
        }
    }

    fn parse_stream_to_pair(stream: String) -> BinanceTradingPair {
        let parts = stream.split("@").next().unwrap();
        BinanceTradingPair::new_checked(parts).unwrap()
    }
}
