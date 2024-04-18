use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::{normalized::types::NormalizedQuote, okex::pairs::OkexTradingPair},
    CexExchange
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct OkexTickersMessage {
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
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "askPx")]
    pub ask_price:         f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "askSz")]
    pub ask_amt:           f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "bidPx")]
    pub bid_price:         f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "bidSz")]
    pub bid_amt:           f64,
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

impl OkexTickersMessage {
    pub fn normalize(self) -> NormalizedQuote {
        NormalizedQuote {
            exchange:   CexExchange::Okex,
            pair:       self.pair.normalize(),
            time:       DateTime::from_timestamp_millis(self.timestamp as i64).unwrap(),
            ask_amount: self.ask_amt,
            ask_price:  self.ask_price,
            bid_amount: self.bid_amt,
            bid_price:  self.bid_price,
            quote_id:   None
        }
    }
}

impl PartialEq<NormalizedQuote> for OkexTickersMessage {
    fn eq(&self, other: &NormalizedQuote) -> bool {
        let equals = other.exchange == CexExchange::Okex
            && other.pair == self.pair.normalize()
            && other.time == DateTime::from_timestamp_millis(self.timestamp as i64).unwrap()
            && other.bid_amount == self.bid_amt
            && other.bid_price == self.bid_price
            && other.ask_amount == self.ask_amt
            && other.ask_price == self.ask_price
            && other.quote_id == None;

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
