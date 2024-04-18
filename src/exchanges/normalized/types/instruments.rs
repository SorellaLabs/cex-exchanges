use std::fmt::Display;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use super::NormalizedTradingPair;
use crate::exchanges::CexExchange;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedInstrument {
    pub exchange:           CexExchange,
    pub trading_pair:       NormalizedTradingPair,
    pub trading_type:       NormalizedTradingType,
    pub base_asset_symbol:  String,
    pub quote_asset_symbol: String,
    pub active:             bool,
    /// THIS IS AN ESTIMATE:
    /// calculated by 24hr avg price * 24 hours volume
    /// should only be used when estimate ranking exchanges by 24hr volume (in
    /// usdc)
    pub avg_vol_24hr_usdc:  f64
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, EnumIter)]
pub enum NormalizedTradingType {
    Spot,
    Perpetual,
    Margin,
    Future,
    Option
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

impl TryFrom<&str> for NormalizedTradingType {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let s = value.to_lowercase();

        match s.as_str() {
            "spot" => Ok(NormalizedTradingType::Spot),
            "perpetual" | "perp" | "swap" => Ok(NormalizedTradingType::Perpetual),
            "futures" => Ok(NormalizedTradingType::Future),
            "margin" => Ok(NormalizedTradingType::Margin),
            "option" => Ok(NormalizedTradingType::Option),
            _ => Err(eyre::ErrReport::msg(format!("'{value}' is not a valid trading type")))
        }
    }
}

impl TryFrom<String> for NormalizedTradingType {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl Display for NormalizedTradingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = format!("{:?}", self).to_uppercase();
        write!(f, "{}", s)
    }
}
