pub mod trades;

use self::trades::Trade;
use crate::{exchanges::CexExchange, types::normalized::ws::NormalizedWsDataTypes};

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BinanceWsMessage {
    Trade(Trade),
    Error(String)
}

impl BinanceWsMessage {
    pub fn normalize(self) -> NormalizedWsDataTypes {
        match self {
            BinanceWsMessage::Trade(v) => NormalizedWsDataTypes::Trades(v.normalize()),
            BinanceWsMessage::Error(v) => {
                NormalizedWsDataTypes::Other { exchange: CexExchange::Binance, kind: "Error".to_string(), value: format!("{:?}", v) }
            }
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for BinanceWsMessage {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        match self {
            BinanceWsMessage::Trade(vals) => matches!(normalized, NormalizedWsDataTypes::Trades(_)) && vals.equals_normalized(),
            BinanceWsMessage::Error(_) => matches!(normalized, NormalizedWsDataTypes::Other { .. })
        }
    }
}
