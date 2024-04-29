use std::{fmt::Display, str::FromStr};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use super::NormalizedTradingPair;
use crate::exchanges::CexExchange;

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct NormalizedInstrument {
    pub exchange:           CexExchange,
    pub trading_pair:       NormalizedTradingPair,
    pub trading_type:       NormalizedTradingType,
    pub base_asset_symbol:  String,
    pub quote_asset_symbol: String,
    pub active:             bool,
    pub futures_expiry:     Option<NaiveDate>
}

#[derive(Debug, Default, Clone, Copy, Serialize, PartialEq, Eq, Hash, EnumIter, PartialOrd, Ord)]
pub enum NormalizedTradingType {
    Spot,
    Perpetual,
    Margin,
    Futures,
    Rfq,
    Option,
    #[default]
    Other
}

impl NormalizedTradingType {
    pub fn fmt_okex(&self) -> Option<&str> {
        match self {
            NormalizedTradingType::Spot => Some("SPOT"),
            NormalizedTradingType::Perpetual => Some("SWAP"),
            NormalizedTradingType::Margin => Some("MARGIN"),
            NormalizedTradingType::Futures => Some("FUTURES"),
            NormalizedTradingType::Rfq => None,
            NormalizedTradingType::Option => None,
            NormalizedTradingType::Other => None
        }
    }
}

impl<'de> Deserialize<'de> for NormalizedTradingType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        s.try_into().map_err(serde::de::Error::custom)
    }
}

impl FromStr for NormalizedTradingType {
    type Err = eyre::Report;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let s = value.to_lowercase();

        match s.as_str() {
            "spot" => Ok(NormalizedTradingType::Spot),
            "perpetual" | "perp" | "swap" | "linear" | "inverse" => Ok(NormalizedTradingType::Perpetual),
            "futures" => Ok(NormalizedTradingType::Futures),
            "margin" => Ok(NormalizedTradingType::Margin),
            "option" => Ok(NormalizedTradingType::Option),
            _ => Err(eyre::ErrReport::msg(format!("'{value}' is not a valid trading type")))
        }
    }
}

impl TryFrom<String> for NormalizedTradingType {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().parse()
    }
}

impl Display for NormalizedTradingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{:?}", self).to_uppercase();
        write!(f, "{}", s)
    }
}
