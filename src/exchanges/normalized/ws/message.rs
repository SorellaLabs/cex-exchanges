use super::NormalizedWsDataTypes;
#[cfg(feature = "non-us")]
use crate::exchanges::binance::ws::BinanceWsMessage;
#[cfg(feature = "us")]
use crate::exchanges::coinbase::ws::CoinbaseWsMessage;
use crate::{exchanges::okex::ws::OkexWsMessage, CexExchange};

#[derive(Debug, Clone)]
pub enum CombinedWsMessage {
    #[cfg(feature = "us")]
    Coinbase(CoinbaseWsMessage),
    #[cfg(feature = "us")]
    Okex(OkexWsMessage),
    #[cfg(feature = "non-us")]
    Binance(BinanceWsMessage),
    Disconnect {
        exchange: CexExchange,
        message:  String
    }
}

impl CombinedWsMessage {
    pub fn normalize(self) -> NormalizedWsDataTypes {
        match self {
            #[cfg(feature = "us")]
            CombinedWsMessage::Coinbase(c) => c.normalize(),
            #[cfg(feature = "us")]
            CombinedWsMessage::Okex(c) => c.normalize(),
            #[cfg(feature = "non-us")]
            CombinedWsMessage::Binance(c) => c.normalize(),
            CombinedWsMessage::Disconnect { exchange, message } => NormalizedWsDataTypes::Disconnect { exchange, message }
        }
    }

    pub fn is_ok(&self) -> bool {
        !self.is_err()
    }

    pub fn is_err(&self) -> bool {
        matches!(self, CombinedWsMessage::Disconnect { .. })
    }
}

macro_rules! combined_ws {
    ($exchange:ident) => {
        paste::paste! {
            impl From<[<$exchange WsMessage>]> for CombinedWsMessage {
                fn from(value: [<$exchange WsMessage>]) -> Self {
                    Self::$exchange(value)
                }
            }
        }
    };
}

#[cfg(feature = "us")]
combined_ws!(Coinbase);

#[cfg(feature = "us")]
combined_ws!(Okex);

#[cfg(feature = "non-us")]
combined_ws!(Binance);

impl PartialEq<NormalizedWsDataTypes> for CombinedWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match self {
            #[cfg(feature = "us")]
            CombinedWsMessage::Coinbase(vals) => vals == other,
            #[cfg(feature = "us")]
            CombinedWsMessage::Okex(vals) => vals == other,
            #[cfg(feature = "non-us")]
            CombinedWsMessage::Binance(vals) => vals == other,
            CombinedWsMessage::Disconnect { exchange, message } => true
        }
    }
}
