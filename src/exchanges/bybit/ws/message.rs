use eyre::Ok;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::channels::{BybitOrderbook, BybitTrade};
use crate::{clients::ws::CriticalWsMessage, exchanges::normalized::ws::NormalizedWsDataTypes, CexExchange};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "data")]
pub enum BybitWsMessage {
    Trade(BybitTrade),
    OrderbookL1(BybitOrderbook),
    SuscriptionResponse { id: String, msg: String }
}

impl BybitWsMessage {
    fn try_deserialize(value: Value) -> eyre::Result<Self> {
        let try_trade = serde_json::from_value(value.clone());
        if try_trade.is_ok() {
            return Ok(Self::Trade(try_trade?))
        }

        let try_ticker = serde_json::from_value(value.clone());
        if try_ticker.is_ok() {
            return Ok(Self::OrderbookL1(try_ticker?))
        }

        let conn_id = value.get("conn_id");
        let success = value.get("success");
        if let (Some(c), Some(s)) = (conn_id, success) {
            let success_val = s.as_bool().unwrap();
            if success_val {
                return Ok(Self::SuscriptionResponse { id: c.as_str().unwrap().to_string(), msg: success_val.to_string() })
            }
        }

        Err(eyre::ErrReport::msg(format!(
            "Could not deserialize kucoin ws
        message: {:?}",
            value
        )))
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
            BybitWsMessage::Trade(v) => NormalizedWsDataTypes::Trades(v.normalize()),
            BybitWsMessage::OrderbookL1(v) => NormalizedWsDataTypes::Quotes(v.normalize().map(|v| vec![v]).unwrap_or_default()),
            BybitWsMessage::SuscriptionResponse { id, msg } => NormalizedWsDataTypes::Other {
                exchange: CexExchange::Bybit,
                kind:     "subscribe".to_string(),
                value:    format!("result: {} -- id: {}", msg, id)
            }
        }
    }
}

impl PartialEq<NormalizedWsDataTypes> for BybitWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match (self, other) {
            (BybitWsMessage::Trade(this), NormalizedWsDataTypes::Trades(that)) => this == that,
            (BybitWsMessage::OrderbookL1(this), NormalizedWsDataTypes::Quotes(that)) => this == that,
            (BybitWsMessage::SuscriptionResponse { .. }, NormalizedWsDataTypes::Other { .. }) => true,
            _ => false
        }
    }
}

impl CriticalWsMessage for BybitWsMessage {
    fn make_critical(&mut self, _msg: String) {}
}
