use serde::{Deserialize, Serialize};

use crate::{exchanges::normalized::CexExchange, types::normalized::pairs::NormalizedTradingPair};

#[derive(Debug, Clone)]
pub struct CoinbaseTradingPair(pub(crate) String);

impl CoinbaseTradingPair {
    pub fn new_checked(s: &str) -> eyre::Result<Self> {
        s.to_string().try_into()
    }

    pub(crate) fn new_unchecked(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn is_valid(s: &str) -> bool {
        s.contains('-')
    }

    pub fn normalize(&self) -> NormalizedTradingPair {
        let mut split = self.0.split('-');
        NormalizedTradingPair::new(CexExchange::Coinbase, split.next().unwrap(), split.next().unwrap(), Some('-'))
    }
}

impl TryFrom<String> for CoinbaseTradingPair {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::is_valid(&value) {
            Ok(CoinbaseTradingPair(value))
        } else {
            Err(eyre::ErrReport::msg(format!("trading pair '{value}' does not contain a '-'")))
        }
    }
}

impl TryFrom<&str> for CoinbaseTradingPair {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if Self::is_valid(value) {
            Ok(CoinbaseTradingPair(value.to_string()))
        } else {
            Err(eyre::ErrReport::msg(format!("trading pair '{value}' does not contain a '-'")))
        }
    }
}

impl Serialize for CoinbaseTradingPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CoinbaseTradingPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        Ok(CoinbaseTradingPair(s))
    }
}

impl From<NormalizedTradingPair> for CoinbaseTradingPair {
    fn from(value: NormalizedTradingPair) -> Self {
        CoinbaseTradingPair(format!("{}-{}", value.base.to_uppercase(), value.quote.to_uppercase()))
    }
}
