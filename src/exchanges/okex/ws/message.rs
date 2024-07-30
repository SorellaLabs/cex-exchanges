use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::channels::{OkexTicker, OkexTrade};
use crate::{clients::ws::CriticalWsMessage, exchanges::normalized::ws::NormalizedWsDataTypes, okex::OkexTradingPair, CexExchange};

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "data")]
pub enum OkexWsMessage {
    TradesAll(OkexTrade),
    Tickers(OkexTicker),
    Subscribe(serde_json::Value),
    Error { error: String, raw_msg: String, bad_pair: Option<OkexTradingPair> }
}

impl OkexWsMessage {
    fn try_deserialize(value: Value) -> eyre::Result<Self> {
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
                    .get("msg")
                    .ok_or(eyre::ErrReport::msg("Could not find 'msg' (error message) field in Okex ws message".to_string()))?
                    .as_str()
                    .ok_or(eyre::ErrReport::msg("Could not convert 'msg' (error message) field in Okex ws message to &str".to_string()))?;

                Ok(Self::Error { error: msg.to_string(), raw_msg: String::new(), bad_pair: OkexTradingPair::parse_for_bad_pair(msg) })
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
            OkexWsMessage::Tickers(v) => NormalizedWsDataTypes::Quotes(v.normalize().map(|val| vec![val]).unwrap_or_default()),
            OkexWsMessage::Subscribe(v) => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Okex, kind: "Subscribe".to_string(), value: format!("{:?}", v) }
            }
            OkexWsMessage::Error { error, raw_msg, bad_pair } => {
                if let Some(bp) = bad_pair {
                    NormalizedWsDataTypes::RemovedPair {
                        exchange:    CexExchange::Okex,
                        bad_pair:    bp.normalize(),
                        raw_message: format!("{error} - {raw_msg}")
                    }
                } else {
                    NormalizedWsDataTypes::Other { exchange: CexExchange::Okex, kind: error, value: String::new() }
                }
            }
        }
    }
}

impl PartialEq<NormalizedWsDataTypes> for OkexWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match (self, other) {
            (OkexWsMessage::TradesAll(this), NormalizedWsDataTypes::Trade(that)) => this == that,
            (OkexWsMessage::Tickers(this), NormalizedWsDataTypes::Quotes(that)) => &vec![this.clone()] == that,
            (OkexWsMessage::Subscribe(_), NormalizedWsDataTypes::Other { .. }) => true,
            (OkexWsMessage::Error { .. }, NormalizedWsDataTypes::Other { .. }) => true,
            (OkexWsMessage::Error { .. }, NormalizedWsDataTypes::RemovedPair { .. }) => true,
            _ => false
        }
    }
}

impl CriticalWsMessage for OkexWsMessage {
    fn make_critical(&mut self, msg: String) {
        if let OkexWsMessage::Error { raw_msg, bad_pair, .. } = self {
            if bad_pair.is_none() {
                *bad_pair = OkexTradingPair::parse_for_bad_pair(&msg);
            }

            *raw_msg = msg;
        }
    }
}
