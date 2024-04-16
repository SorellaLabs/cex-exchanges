use crate::{exchanges::CexExchange, types::normalized::ws::NormalizedWsDataTypes};

pub mod matches;
pub mod status;
pub mod ticker;

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoinbaseWsMessage {
    #[serde(alias = "last_match")]
    Match(matches::CoinbaseMatchesMessage),
    Ticker(ticker::CoinbaseTickerMessage),
    Status(status::CoinbaseStatusMessage),
    Subscriptions(serde_json::Value),
    Error(String)
}

impl CoinbaseWsMessage {
    pub fn normalize(self) -> NormalizedWsDataTypes {
        match self {
            CoinbaseWsMessage::Match(v) => NormalizedWsDataTypes::Trades(v.normalize()),
            CoinbaseWsMessage::Ticker(v) => NormalizedWsDataTypes::Quotes(v.normalize()),
            CoinbaseWsMessage::Status(v) => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Coinbase, kind: "Status".to_string(), value: format!("{:?}", v) }
            }
            CoinbaseWsMessage::Subscriptions(v) => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Coinbase, kind: "Subscriptions".to_string(), value: format!("{:?}", v) }
            }
            CoinbaseWsMessage::Error(v) => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Coinbase, kind: "Error".to_string(), value: format!("{:?}", v) }
            }
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for CoinbaseWsMessage {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        match self {
            CoinbaseWsMessage::Match(vals) => matches!(normalized, NormalizedWsDataTypes::Trades(_)) && vals.equals_normalized(),
            CoinbaseWsMessage::Ticker(vals) => matches!(normalized, NormalizedWsDataTypes::Quotes(_)) && vals.equals_normalized(),
            CoinbaseWsMessage::Status(_) => matches!(normalized, NormalizedWsDataTypes::Other { .. }),
            CoinbaseWsMessage::Subscriptions(_) => matches!(normalized, NormalizedWsDataTypes::Other { .. }),
            CoinbaseWsMessage::Error(_) => matches!(normalized, NormalizedWsDataTypes::Other { .. })
        }
    }
}
