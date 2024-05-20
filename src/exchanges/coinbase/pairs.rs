use serde::{Deserialize, Serialize};

use crate::{exchanges::normalized::types::NormalizedTradingPair, CexExchange};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Hash)]
pub struct CoinbaseTradingPair(pub(crate) String);

impl CoinbaseTradingPair {
    pub fn new_checked(s: &str) -> eyre::Result<Self> {
        s.to_string().try_into()
    }

    pub fn is_valid(s: &str) -> bool {
        s.contains('-') && !s.contains('_') && !s.contains('/') && s.len() > 4
    }

    pub fn normalize(&self) -> NormalizedTradingPair {
        let mut split = self.0.split('-');
        NormalizedTradingPair::new_base_quote(CexExchange::Coinbase, split.next().unwrap(), split.next().unwrap(), Some('-'), None)
    }

    pub fn parse_for_bad_pair(value: &str) -> Option<Self> {
        let value = value.replace("'", " ").replace("\"", " ");
        value.split(" ").find_map(|v| Self::try_from(v).ok())
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

impl TryFrom<NormalizedTradingPair> for CoinbaseTradingPair {
    type Error = eyre::Report;

    fn try_from(value: NormalizedTradingPair) -> Result<Self, Self::Error> {
        if let Some((base, quote)) = value.base_quote() {
            return Ok(CoinbaseTradingPair(format!("{}-{}", base, quote)))
        }

        if let (Some(raw_pair), delim) = (value.pair(), value.delimiter()) {
            if let Ok(v) = Self::new_checked(raw_pair) {
                return Ok(v)
            }

            if let Some(d) = delim {
                let mut split = raw_pair.split(d);
                return Ok(CoinbaseTradingPair(format!("{}-{}", split.next().unwrap().to_uppercase(), split.next().unwrap().to_uppercase())));
            }

            let new_str = raw_pair.replace(['_', '/'], "-");
            if let Ok(this) = Self::new_checked(&new_str) {
                return Ok(this)
            }

            return Err(eyre::ErrReport::msg(format!("INVALID Coinbase trading pair '{raw_pair}' contains no '-'")))
        }

        Err(eyre::ErrReport::msg(format!("INVALID Coinbase trading pair: '{:?}'", value)))
    }
}

impl TryFrom<String> for CoinbaseTradingPair {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::is_valid(&value) {
            Ok(CoinbaseTradingPair(value))
        } else {
            Err(eyre::ErrReport::msg(format!("INVALID Coinbase trading pair '{value}' contains either: 1) '_', and/or '/' - OR -  2) no '-'")))
        }
    }
}

impl TryFrom<&str> for CoinbaseTradingPair {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if Self::is_valid(value) {
            Ok(CoinbaseTradingPair(value.to_uppercase()))
        } else {
            Err(eyre::ErrReport::msg(format!("INVALID Coinbase trading pair '{value}' contains either: 1) '_', and/or '/' - OR -  2) no '-'")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_for_bad_pair() {
        let test_str = r#"failed to deserialize the message: missing field `error` - {"type":"error","message":"Failed to subscribe","reason":"LOOM-USDC is delisted"}"#;

        let calculated = CoinbaseTradingPair::parse_for_bad_pair(&test_str);

        assert_eq!(calculated, Some(CoinbaseTradingPair("LOOM-USDC".to_string())))
    }
}
