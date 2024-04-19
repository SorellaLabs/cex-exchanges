use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::{coinbase::pairs::CoinbaseTradingPair, normalized::types::NormalizedTrade},
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseMatches {
    pub trade_id:       u64,
    pub sequence:       u64,
    pub maker_order_id: String,
    pub taker_order_id: String,
    pub time:           DateTime<Utc>,
    pub product_id:     CoinbaseTradingPair,
    #[serde_as(as = "DisplayFromStr")]
    pub size:           f64,
    #[serde_as(as = "DisplayFromStr")]
    pub price:          f64,
    pub side:           String
}

impl CoinbaseMatches {
    pub fn normalize(self) -> NormalizedTrade {
        NormalizedTrade {
            exchange: CexExchange::Coinbase,
            pair:     self.product_id.normalize(),
            time:     self.time,
            side:     self.side.to_lowercase(),
            price:    self.price,
            amount:   self.size,
            trade_id: Some(self.trade_id.to_string())
        }
    }
}

impl PartialEq<NormalizedTrade> for CoinbaseMatches {
    fn eq(&self, other: &NormalizedTrade) -> bool {
        let equals = other.exchange == CexExchange::Coinbase
            && other.pair == self.product_id.normalize()
            && other.time == self.time
            && other.side == self.side.to_lowercase()
            && other.price == self.price
            && other.amount == self.size
            && other.trade_id.as_ref().unwrap() == &self.trade_id.to_string();

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
