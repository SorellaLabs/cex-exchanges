use eyre::Ok;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{matches::KucoinMatch, ticker::KucoinTicker};
use crate::{exchanges::normalized::ws::NormalizedWsDataTypes, CexExchange};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "data")]
pub enum KucoinWsMessage {
    Match(KucoinMatch),
    Ticker(KucoinTicker),
    SuscriptionResponse { id: String, msg: String }
}

impl KucoinWsMessage {
    fn try_deserialize(value: Value) -> eyre::Result<Self> {
        let try_match = serde_json::from_value(value.clone());
        if try_match.is_ok() {
            return Ok(Self::Match(try_match?))
        }

        let try_ticker = serde_json::from_value(value.clone());
        if try_ticker.is_ok() {
            return Ok(Self::Ticker(try_ticker?))
        }

        let id = value.get("id");
        let msg = value.get("type");
        if let (Some(i), Some(m)) = (id, msg) {
            return Ok(Self::SuscriptionResponse { id: i.as_str().unwrap().to_string(), msg: m.as_str().unwrap().to_string() })
        }

        Err(eyre::ErrReport::msg(format!("Could not deserialize kucoin ws message: {:?}", value)))
    }
}

impl<'de> Deserialize<'de> for KucoinWsMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let map = Value::deserialize(deserializer)?;

        Self::try_deserialize(map).map_err(serde::de::Error::custom)
    }
}

impl KucoinWsMessage {
    pub fn normalize(self) -> NormalizedWsDataTypes {
        match self {
            KucoinWsMessage::Match(v) => NormalizedWsDataTypes::Trade(v.normalize()),
            KucoinWsMessage::Ticker(v) => NormalizedWsDataTypes::Quote(v.normalize()),
            KucoinWsMessage::SuscriptionResponse { id, msg } => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Kucoin, kind: msg, value: id }
            }
        }
    }
}

impl PartialEq<NormalizedWsDataTypes> for KucoinWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match (self, other) {
            (KucoinWsMessage::Match(this), NormalizedWsDataTypes::Trade(that)) => this == that,
            (KucoinWsMessage::Ticker(this), NormalizedWsDataTypes::Quote(that)) => this == that,
            (KucoinWsMessage::SuscriptionResponse { .. }, NormalizedWsDataTypes::Other { .. }) => true,
            _ => false
        }
    }
}
