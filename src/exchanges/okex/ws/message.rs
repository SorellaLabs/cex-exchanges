use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{tickers::OkexTicker, trades::OkexTrade};
use crate::{clients::ws::CriticalWsMessage, exchanges::normalized::ws::NormalizedWsDataTypes, CexExchange};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "data")]
pub enum OkexWsMessage {
    TradesAll(OkexTrade),
    Tickers(OkexTicker),
    Subscribe(serde_json::Value),
    Error { error: String, raw_msg: String }
}

impl OkexWsMessage {
    fn try_deserialize(value: Value) -> eyre::Result<Self> {
        println!("\n\n\n\n\nVALUE: {:?}\n\n\n\n\n", value);
        if let Some(data) = value.get("data") {
            let channel = value
                .get("arg")
                .ok_or(eyre::ErrReport::msg("Could not find 'arg' field in Okex ws message".to_string()))?
                .get("channel")
                .ok_or(eyre::ErrReport::msg("Could not find nest 'channel' field in Okex ws message".to_string()))?
                .as_str()
                .ok_or(eyre::ErrReport::msg("Could not convert 'channel' field in Okex ws message to &str".to_string()))?;
            if channel == "trades-all" {
                let data: Vec<OkexTrade> = serde_json::from_value(data.clone())?;
                Ok(Self::TradesAll(data.first().unwrap().clone()))
            } else if channel == "tickers" {
                let data: Vec<OkexTicker> = serde_json::from_value(data.clone())?;
                Ok(Self::Tickers(data.first().unwrap().clone()))
            } else {
                Err(eyre::ErrReport::msg(format!("Channel type '{channel}' cannot be deserialized")))
            }
        } else {
            let event = value
                .get("event")
                .ok_or(eyre::ErrReport::msg("Could not find 'event' field in Okex ws message".to_string()))?
                .as_str()
                .ok_or(eyre::ErrReport::msg("Could not convert 'event' field in Okex ws message to &str".to_string()))?;

            if event == "subscribe" {
                Ok(Self::Subscribe(value))
            } else if event == "error" {
                let msg = value
                    .get("message")
                    .ok_or(eyre::ErrReport::msg("Could not find 'msg' (error message) field in Okex ws message".to_string()))?
                    .as_str()
                    .ok_or(eyre::ErrReport::msg("Could not convert 'msg' (error message) field in Okex ws message to &str".to_string()))?;

                Ok(Self::Error { error: msg.to_string(), raw_msg: String::new() })
            } else {
                Err(eyre::ErrReport::msg(format!("Event type '{event}' cannot be deserialized")))
            }
        }
    }
}

impl<'de> Deserialize<'de> for OkexWsMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let map = Value::deserialize(deserializer)?;

        Self::try_deserialize(map).map_err(serde::de::Error::custom)
    }
}

impl OkexWsMessage {
    pub fn normalize(self) -> NormalizedWsDataTypes {
        match self {
            OkexWsMessage::TradesAll(v) => NormalizedWsDataTypes::Trade(v.normalize()),
            OkexWsMessage::Tickers(v) => NormalizedWsDataTypes::Quote(v.normalize()),
            OkexWsMessage::Subscribe(v) => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Okex, kind: "Subscribe".to_string(), value: format!("{:?}", v) }
            }
            OkexWsMessage::Error { error, raw_msg } => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Okex, kind: error, value: raw_msg }
            }
        }
    }
}

impl PartialEq<NormalizedWsDataTypes> for OkexWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match (self, other) {
            (OkexWsMessage::TradesAll(this), NormalizedWsDataTypes::Trade(that)) => this == that,
            (OkexWsMessage::Tickers(this), NormalizedWsDataTypes::Quote(that)) => this == that,
            (OkexWsMessage::Subscribe(_), NormalizedWsDataTypes::Other { .. }) => true,
            (OkexWsMessage::Error { .. }, NormalizedWsDataTypes::Other { .. }) => true,
            _ => false
        }
    }
}

impl CriticalWsMessage for OkexWsMessage {
    fn make_critical(&mut self, msg: String) {
        match self {
            OkexWsMessage::Error { error: _, raw_msg } => *raw_msg = msg,
            _ => ()
        }
    }
}
