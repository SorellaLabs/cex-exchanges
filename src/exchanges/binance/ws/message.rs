use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{book_ticker::BinanceBookTicker, trades::BinanceTrade};
use crate::exchanges::normalized::ws::NormalizedWsDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "data")]
pub enum BinanceWsMessage {
    Trade(BinanceTrade),
    BookTicker(BinanceBookTicker)
}

impl BinanceWsMessage {
    fn try_deserialize(value: Value) -> eyre::Result<Self> {
        let data = value
            .get("data")
            .ok_or(eyre::ErrReport::msg("Could not find 'data' field in Binance ws message".to_string()))?;

        let data_type = value
            .get("stream")
            .ok_or(eyre::ErrReport::msg("Could not find nested 'stream' (event type) field in Binance ws message".to_string()))?
            .as_str()
            .ok_or(eyre::ErrReport::msg("Could not convert 'stream' (event type) field in Binance ws message to &str".to_string()))?;

        if data_type.contains("@trade") {
            let trade: BinanceTrade = serde_json::from_value(data.clone())?;
            Ok(Self::Trade(trade))
        } else if data_type.contains("@bookTicker") {
            let book_ticker: BinanceBookTicker = serde_json::from_value(data.clone())?;
            Ok(Self::BookTicker(book_ticker))
        } else {
            Err(eyre::ErrReport::msg(format!("Event type '{data_type}' cannot be deserialized")))
        }
    }
}

impl<'de> Deserialize<'de> for BinanceWsMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let map = Value::deserialize(deserializer)?;

        Self::try_deserialize(map).map_err(serde::de::Error::custom)
    }
}

impl BinanceWsMessage {
    pub fn normalize(self) -> NormalizedWsDataTypes {
        match self {
            BinanceWsMessage::Trade(v) => NormalizedWsDataTypes::Trade(v.normalize()),
            BinanceWsMessage::BookTicker(v) => NormalizedWsDataTypes::Quote(v.normalize())
        }
    }
}

impl PartialEq<NormalizedWsDataTypes> for BinanceWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match (self, other) {
            (BinanceWsMessage::Trade(this), NormalizedWsDataTypes::Trade(that)) => this == that,
            (BinanceWsMessage::BookTicker(this), NormalizedWsDataTypes::Quote(that)) => this == that,
            _ => false
        }
    }
}
