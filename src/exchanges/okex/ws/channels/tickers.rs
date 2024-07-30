use std::f64::EPSILON;

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError, DisplayFromStr};
use tracing::warn;

use crate::{
    exchanges::{normalized::types::NormalizedQuote, okex::pairs::OkexTradingPair},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct OkexTicker {
    /// SWAP, PERP, OPTION, ..
    #[serde(rename = "instType")]
    pub pair_type:         String,
    #[serde(rename = "instId")]
    pub pair:              OkexTradingPair,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "last")]
    pub last_price:        f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "lastSz")]
    pub last_size:         f64,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(rename = "askPx")]
    pub ask_price:         Option<f64>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(rename = "askSz")]
    pub ask_amt:           Option<f64>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(rename = "bidPx")]
    pub bid_price:         Option<f64>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(rename = "bidSz")]
    pub bid_amt:           Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "open24h")]
    pub open_price_24hr:   f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "high24h")]
    pub high_price_24h:    f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "low24h")]
    pub low_price_24h:     f64,
    /// 24h trading volume, with a unit of currency.
    /// If it is a derivatives contract, the value is the number of base
    /// currency. If it is SPOT/MARGIN, the value is the quantity in quote
    /// currency.
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "volCcy24h")]
    pub vol_currency_24hr: f64,
    /// 24h trading volume, with a unit of contract.
    /// If it is a derivatives contract, the value is the number of contracts.
    /// If it is SPOT/MARGIN, the value is the quantity in base currency.
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "vol24h")]
    pub vol_contract_24hr: f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "sodUtc0")]
    pub open_price_utc0:   f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "sodUtc8")]
    pub open_price_utc8:   f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "ts")]
    pub timestamp:         u64
}

impl OkexTicker {
    pub fn normalize(self) -> Option<NormalizedQuote> {
        if let (Some(ask_amount), Some(ask_price), Some(bid_amount), Some(bid_price)) = (self.ask_amt, self.ask_price, self.bid_amt, self.bid_price) {
            Some(NormalizedQuote {
                exchange: CexExchange::Okex,
                pair: self.pair.normalize(),
                time: DateTime::from_timestamp_millis(self.timestamp as i64).unwrap(),
                ask_amount,
                ask_price,
                bid_amount,
                bid_price,
                quote_id: None
            })
        } else {
            None
        }
    }
}

impl PartialEq<NormalizedQuote> for OkexTicker {
    fn eq(&self, other: &NormalizedQuote) -> bool {
        let equals = other.exchange == CexExchange::Okex
            && other.pair == self.pair.normalize()
            && other.time == DateTime::from_timestamp_millis(self.timestamp as i64).unwrap()
            && (other.bid_amount - self.bid_amt.unwrap_or_default()).abs() < EPSILON
            && (other.bid_price - self.bid_price.unwrap_or_default()).abs() < EPSILON
            && (other.ask_amount - self.ask_amt.unwrap_or_default()).abs() < EPSILON
            && (other.ask_price - self.ask_price.unwrap_or_default()).abs() < EPSILON
            && other.quote_id.is_none();

        if !equals {
            warn!(target: "cex-exchanges::okex", "okex ticker: {:?}", self);
            warn!(target: "cex-exchanges::okex", "normalized quote: {:?}", other);
        }

        equals
    }
}
