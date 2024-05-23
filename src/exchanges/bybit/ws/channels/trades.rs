use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::{bybit::pairs::BybitTradingPair, normalized::types::NormalizedTrade},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BybitTrade {
    pub topic:             String,
    #[serde(rename = "type")]
    pub kind:              String,
    #[serde(rename = "ts")]
    pub request_timestamp: u64,
    pub data:              Vec<BybitTradeInner>
}

impl BybitTrade {
    pub fn normalize(self) -> Vec<NormalizedTrade> {
        self.data
            .into_iter()
            .map(|inner| NormalizedTrade {
                exchange: CexExchange::Bybit,
                pair:     inner.pair.normalize(),
                time:     DateTime::<Utc>::from_timestamp_millis(inner.timestamp as i64).unwrap(),
                side:     inner.side.to_lowercase(),
                price:    inner.price,
                amount:   inner.amount,
                trade_id: Some(inner.trade_id.to_string())
            })
            .collect()
    }
}

impl PartialEq<Vec<NormalizedTrade>> for BybitTrade {
    fn eq(&self, other: &Vec<NormalizedTrade>) -> bool {
        let all_equals = self.data.iter().all(|inner| {
            other.iter().any(|other_data| {
                let equals = other_data.exchange == CexExchange::Bybit
                    && other_data.pair == inner.pair.normalize()
                    && other_data.time == DateTime::<Utc>::from_timestamp_millis(inner.timestamp as i64).unwrap()
                    && other_data.side == inner.side.to_lowercase()
                    && other_data.price == inner.price
                    && other_data.amount == inner.amount
                    && other_data.trade_id.as_ref().unwrap() == &inner.trade_id.to_string();
                equals
            })
        });

        if !all_equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        all_equals
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct BybitTradeInner {
    #[serde(rename = "T")]
    pub timestamp: u64,
    #[serde(rename = "s")]
    pub pair: BybitTradingPair,
    #[serde(rename = "S")]
    pub side: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "v")]
    pub amount: f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "L")]
    pub direction_of_price_change: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "i")]
    pub trade_id: u64,
    #[serde(rename = "BT")]
    pub is_block_trade_order: bool
}
