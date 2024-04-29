use eyre::Ok;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ticker::BybitTicker, trades::BybitTrade};
use crate::{exchanges::normalized::ws::NormalizedWsDataTypes, CexExchange};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "data")]
pub enum BybitWsMessage {
    Trade(BybitTrade),
    Ticker(BybitTicker),
    SuscriptionResponse { result: Option<String>, id: u64 }
}

impl BybitWsMessage {
    fn try_deserialize(value: Value) -> eyre::Result<Self> {
        let data_val: Result<&Value, eyre::Error> = value
            .get("data")
            .ok_or(eyre::ErrReport::msg("Could not find 'data' field in Bybit ws message".to_string()));
        if data_val.is_ok() {
            let data = data_val?.clone();
            let data_type = value
                .get("stream")
                .ok_or(eyre::ErrReport::msg("Could not find nested 'stream' (event type) field in Bybit ws message".to_string()))?
                .as_str()
                .ok_or(eyre::ErrReport::msg("Could not convert 'stream' (event type) field in Bybit ws message to &str".to_string()))?;

            if data_type.contains("@trade") {
                let trade: BybitTrade = serde_json::from_value(data.clone())?;
                Ok(Self::Trade(trade))
            } else if data_type.contains("@bookTicker") {
                let book_ticker: BybitTicker = serde_json::from_value(data.clone())?;
                Ok(Self::Ticker(book_ticker))
            } else {
                Err(eyre::ErrReport::msg(format!("Event type '{data_type}' cannot be deserialized")))
            }
        } else {
            let result_val = value
                .get("result")
                .ok_or(eyre::ErrReport::msg("Could not find 'result' field in Bybit ws message".to_string()))?;
            let result = serde_json::from_value(result_val.clone())?;

            let id_val = value
                .get("id")
                .ok_or(eyre::ErrReport::msg("Could not find 'id' field in Bybit ws message".to_string()))?;
            let id = serde_json::from_value(id_val.clone())?;

            Ok(Self::SuscriptionResponse { result, id })
        }
    }
}

impl<'de> Deserialize<'de> for BybitWsMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let map = Value::deserialize(deserializer)?;

        Self::try_deserialize(map).map_err(serde::de::Error::custom)
    }
}

impl BybitWsMessage {
    pub fn normalize(self) -> NormalizedWsDataTypes {
        match self {
            BybitWsMessage::Trade(v) => NormalizedWsDataTypes::Trade(v.normalize()),
            BybitWsMessage::Ticker(v) => NormalizedWsDataTypes::Quote(v.normalize()),
            BybitWsMessage::SuscriptionResponse { result, id } => NormalizedWsDataTypes::Other {
                exchange: CexExchange::Bybit,
                kind:     "SUBSCRIBE".to_string(),
                value:    format!("result: {:?} -- id: {}", result, id)
            }
        }
    }
}

impl PartialEq<NormalizedWsDataTypes> for BybitWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match (self, other) {
            (BybitWsMessage::Trade(this), NormalizedWsDataTypes::Trade(that)) => this == that,
            (BybitWsMessage::Ticker(this), NormalizedWsDataTypes::Quote(that)) => this == that,
            (BybitWsMessage::SuscriptionResponse { .. }, NormalizedWsDataTypes::Other { .. }) => true,
            _ => false
        }
    }
}
