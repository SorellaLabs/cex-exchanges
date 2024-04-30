use eyre::Ok;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{book_ticker::BinanceBookTicker, trades::BinanceTrade};
use crate::{clients::ws::CriticalWsMessage, exchanges::normalized::ws::NormalizedWsDataTypes, CexExchange};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "data")]
pub enum BinanceWsMessage {
    Trade(BinanceTrade),
    BookTicker(BinanceBookTicker),
    SuscriptionResponse { result: Option<String>, id: u64 }
}

impl BinanceWsMessage {
    fn try_deserialize(value: Value) -> eyre::Result<Self> {
        let data_val: Result<&Value, eyre::Error> = value
            .get("data")
            .ok_or(eyre::ErrReport::msg("Could not find 'data' field in Binance ws message".to_string()));
        if data_val.is_ok() {
            let data = data_val?.clone();
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
        } else {
            let result_val = value
                .get("result")
                .ok_or(eyre::ErrReport::msg("Could not find 'result' field in Binance ws message".to_string()))?;
            let result = serde_json::from_value(result_val.clone())?;

            let id_val = value
                .get("id")
                .ok_or(eyre::ErrReport::msg("Could not find 'id' field in Binance ws message".to_string()))?;
            let id = serde_json::from_value(id_val.clone())?;

            Ok(Self::SuscriptionResponse { result, id })
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
            BinanceWsMessage::BookTicker(v) => NormalizedWsDataTypes::Quote(v.normalize()),
            BinanceWsMessage::SuscriptionResponse { result, id } => NormalizedWsDataTypes::Other {
                exchange: CexExchange::Binance,
                kind:     "SUBSCRIBE".to_string(),
                value:    format!("result: {:?} -- id: {}", result, id)
            }
        }
    }
}

impl PartialEq<NormalizedWsDataTypes> for BinanceWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match (self, other) {
            (BinanceWsMessage::Trade(this), NormalizedWsDataTypes::Trade(that)) => this == that,
            (BinanceWsMessage::BookTicker(this), NormalizedWsDataTypes::Quote(that)) => this == that,
            (BinanceWsMessage::SuscriptionResponse { .. }, NormalizedWsDataTypes::Other { .. }) => true,
            _ => false
        }
    }
}
impl CriticalWsMessage for BinanceWsMessage {
    fn make_critical(&mut self, _msg: String) {}
}
