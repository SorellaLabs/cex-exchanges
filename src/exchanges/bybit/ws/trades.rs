use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::{kucoin::pairs::KucoinTradingPair, normalized::types::NormalizedTrade},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct KucoinTrade {
    #[serde(rename = "s")]
    pub pair:                  KucoinTradingPair,
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

impl KucoinTrade {
    pub fn normalize(self) -> NormalizedTrade {
        NormalizedTrade {
            exchange: CexExchange::Kucoin,
            pair:     self.pair.normalize(),
            time:     DateTime::from_timestamp_millis(self.trade_time as i64).unwrap(),
            side:     if self.is_buyer_market_maker { "buy".to_string() } else { "sell".to_string() },
            price:    self.price,
            amount:   self.quantity,
            trade_id: Some(self.trade_id.to_string())
        }
    }
}

impl PartialEq<NormalizedTrade> for KucoinTrade {
    fn eq(&self, other: &NormalizedTrade) -> bool {
        let equals = other.exchange == CexExchange::Kucoin
            && other.pair == self.pair.normalize()
            && other.time == DateTime::from_timestamp_millis(self.trade_time as i64).unwrap()
            && other.side == if self.is_buyer_market_maker { "buy".to_string() } else { "sell".to_string() }
            && other.price == self.price
            && other.amount == self.quantity
            && other.trade_id.as_ref().unwrap() == &self.trade_id.to_string();

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
