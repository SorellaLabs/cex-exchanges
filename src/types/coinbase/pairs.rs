use serde::Serialize;

use crate::types::normalized::pairs::NormalizedTradingPair;

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
}

impl TryFrom<String> for CoinbaseTradingPair {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::is_valid(&value) {
            Ok(CoinbaseTradingPair(value))
        } else {
            Err(eyre::ErrReport::msg(format!(
                "trading pair '{value}' does not contain a '-'"
            )))
        }
    }
}

impl TryFrom<&str> for CoinbaseTradingPair {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if Self::is_valid(value) {
            Ok(CoinbaseTradingPair(value.to_string()))
        } else {
            Err(eyre::ErrReport::msg(format!(
                "trading pair '{value}' does not contain a '-'"
            )))
        }
    }
}

impl Serialize for CoinbaseTradingPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl From<NormalizedTradingPair> for CoinbaseTradingPair {
    fn from(value: NormalizedTradingPair) -> Self {
        CoinbaseTradingPair(format!(
            "{}-{}",
            value.base.to_uppercase(),
            value.quote.to_uppercase()
        ))
    }
}
