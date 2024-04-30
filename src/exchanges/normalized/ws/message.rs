use super::NormalizedWsDataTypes;
use crate::CexExchange;
#[cfg(feature = "non-us")]
use crate::{binance::ws::BinanceWsMessage, bybit::ws::BybitWsMessage, kucoin::ws::KucoinWsMessage};
#[cfg(feature = "us")]
use crate::{exchanges::coinbase::ws::CoinbaseWsMessage, exchanges::okex::ws::OkexWsMessage};

#[derive(Debug, Clone)]
pub enum CombinedWsMessage {
    #[cfg(feature = "us")]
    Coinbase(CoinbaseWsMessage),
    #[cfg(feature = "us")]
    Okex(OkexWsMessage),
    #[cfg(feature = "non-us")]
    Binance(BinanceWsMessage),
    #[cfg(feature = "non-us")]
    Kucoin(KucoinWsMessage),
    #[cfg(feature = "non-us")]
    Bybit(BybitWsMessage),
    Disconnect {
        exchange:    CexExchange,
        message:     String,
        raw_message: String
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
            #[cfg(feature = "non-us")]
            CombinedWsMessage::Kucoin(c) => c.normalize(),
            #[cfg(feature = "non-us")]
            CombinedWsMessage::Bybit(c) => c.normalize(),
            CombinedWsMessage::Disconnect { exchange, message, raw_message } => NormalizedWsDataTypes::Disconnect { exchange, message, raw_message }
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

#[cfg(feature = "non-us")]
combined_ws!(Kucoin);

#[cfg(feature = "non-us")]
combined_ws!(Bybit);

impl PartialEq<NormalizedWsDataTypes> for CombinedWsMessage {
    fn eq(&self, other: &NormalizedWsDataTypes) -> bool {
        match self {
            #[cfg(feature = "us")]
            CombinedWsMessage::Coinbase(vals) => vals == other,
            #[cfg(feature = "us")]
            CombinedWsMessage::Okex(vals) => vals == other,
            #[cfg(feature = "non-us")]
            CombinedWsMessage::Binance(vals) => vals == other,
            #[cfg(feature = "non-us")]
            CombinedWsMessage::Kucoin(vals) => vals == other,
            #[cfg(feature = "non-us")]
            CombinedWsMessage::Bybit(vals) => vals == other,
            CombinedWsMessage::Disconnect { .. } => true
        }
    }
}
