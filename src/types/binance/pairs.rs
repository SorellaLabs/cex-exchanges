use serde::{Deserialize, Serialize};

use crate::{exchanges::CexExchange, types::normalized::pairs::NormalizedTradingPair};

#[derive(Debug, Clone)]
pub struct BinanceTradingPair(pub(crate) String);

impl BinanceTradingPair {
    pub fn new_checked(s: &str) -> eyre::Result<Self> {
        s.to_string().try_into()
    }

    pub(crate) fn new_unchecked(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn is_valid(s: &str) -> bool {
        !s.contains('-') && !s.contains('_') && !s.contains('/')
    }

    pub fn normalize(&self) -> NormalizedTradingPair {
        let mut split = self.0.split('-');
        NormalizedTradingPair::new(CexExchange::Binance, split.next().unwrap(), split.next().unwrap(), Some('-'))
    }
}

impl TryFrom<String> for BinanceTradingPair {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::is_valid(&value) {
            Ok(BinanceTradingPair(value))
        } else {
            Err(eyre::ErrReport::msg(format!("trading pair '{value}' does not contain a '-'")))
        }
    }
}

impl TryFrom<&str> for BinanceTradingPair {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if Self::is_valid(value) {
            Ok(BinanceTradingPair(value.to_string()))
        } else {
            Err(eyre::ErrReport::msg(format!("trading pair '{value}' does not contain a '-'")))
        }
    }
}

impl Serialize for BinanceTradingPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BinanceTradingPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        Ok(BinanceTradingPair(s))
    }
}

impl From<NormalizedTradingPair> for BinanceTradingPair {
    fn from(value: NormalizedTradingPair) -> Self {
        BinanceTradingPair(format!("{}{}", value.base.to_uppercase(), value.quote.to_uppercase()))
    }
}
