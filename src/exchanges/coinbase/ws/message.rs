use super::{CoinbaseMatches, CoinbaseStatus, CoinbaseTicker};
use crate::{clients::ws::CriticalWsMessage, coinbase::CoinbaseTradingPair, exchanges::normalized::ws::NormalizedWsDataTypes, CexExchange};

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoinbaseWsMessage {
    #[serde(alias = "last_match", alias = "match")]
    Matches(CoinbaseMatches),
    Ticker(CoinbaseTicker),
    Status(CoinbaseStatus),
    Subscriptions(serde_json::Value),
    Error(CoinbaseError)
}

impl CoinbaseWsMessage {
    pub fn normalize(self) -> NormalizedWsDataTypes {
        match self {
            CoinbaseWsMessage::Matches(v) => NormalizedWsDataTypes::Trade(v.normalize()),
            CoinbaseWsMessage::Ticker(v) => NormalizedWsDataTypes::Quote(v.normalize()),
            CoinbaseWsMessage::Status(v) => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Coinbase, kind: "Status".to_string(), value: format!("{:?}", v) }
            }
            CoinbaseWsMessage::Subscriptions(v) => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Coinbase, kind: "Subscriptions".to_string(), value: format!("{:?}", v) }
            }
            CoinbaseWsMessage::Error(err) => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Coinbase, kind: err.reason, value: err.message }
            }
        }
    }
}

impl PartialEq<NormalizedWsDataTypes> for CoinbaseWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match (self, other) {
            (CoinbaseWsMessage::Matches(this), NormalizedWsDataTypes::Trade(that)) => this == that,
            (CoinbaseWsMessage::Ticker(this), NormalizedWsDataTypes::Quote(that)) => this == that,
            (CoinbaseWsMessage::Status(_), NormalizedWsDataTypes::Other { .. }) => true,
            (CoinbaseWsMessage::Subscriptions(_), NormalizedWsDataTypes::Other { .. }) => true,
            (CoinbaseWsMessage::Error { .. }, NormalizedWsDataTypes::Other { .. }) => true,
            _ => false
        }
    }
}

impl CriticalWsMessage for CoinbaseWsMessage {
    fn make_critical(&mut self, msg: String) {
        match self {
            CoinbaseWsMessage::Error(err) => {
                err.parse_for_bad_pair();
                err.message = msg;
            }
            _ => ()
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoinbaseError {
    pub message:  String,
    pub reason:   String,
    pub bad_pair: Option<CoinbaseTradingPair>
}

impl CoinbaseError {
    pub fn parse_for_bad_pair(&mut self) {
        self.bad_pair = CoinbaseTradingPair::parse_for_bad_pair(&self.reason);
    }
}
