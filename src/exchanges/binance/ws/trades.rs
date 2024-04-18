use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::{binance::pairs::BinanceTradingPair, normalized::types::NormalizedTrade},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceTradeMessage {
    #[serde(rename = "s")]
    pub pair:                  BinanceTradingPair,
    #[serde(rename = "p")]
    #[serde_as(as = "DisplayFromStr")]
    pub price:                 f64,
    #[serde(rename = "q")]
    #[serde_as(as = "DisplayFromStr")]
    pub quantity:              f64,
    #[serde(rename = "t")]
    pub trade_id:              u64,
    #[serde(rename = "b")]
    pub buyer_order_id:        u64,
    #[serde(rename = "a")]
    pub seller_order_id:       u64,
    #[serde(rename = "m")]
    pub is_buyer_market_maker: bool,
    #[serde(rename = "T")]
    pub trade_time:            u64
}

impl BinanceTradeMessage {
    pub fn normalize(self) -> NormalizedTrade {
        NormalizedTrade {
            exchange: CexExchange::Binance,
            pair:     self.pair.normalize(),
            time:     DateTime::from_timestamp_millis(self.trade_time as i64).unwrap(),
            side:     if self.is_buyer_market_maker { "buy".to_string() } else { "sell".to_string() },
            price:    self.price,
            amount:   self.quantity,
            trade_id: Some(self.trade_id.to_string())
        }
    }
}

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for BinanceTradeMessage {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();

        normalized.exchange == CexExchange::Binance
            && normalized.pair == self.pair.normalize()
            && normalized.time == DateTime::from_timestamp_millis(self.trade_time as i64).unwrap()
            && normalized.side == if self.is_buyer_market_maker { "buy".to_string() } else { "sell".to_string() }
            && normalized.price == self.price
            && normalized.amount == self.quantity
            && normalized.trade_id.unwrap() == self.trade_id.to_string()
    }
}
