use serde::{Deserialize, Serialize};

use crate::{exchanges::normalized::types::NormalizedTradingPair, CexExchange};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Hash)]
pub struct KucoinTradingPair(pub(crate) String);

impl KucoinTradingPair {
    pub fn new_checked(s: &str) -> eyre::Result<Self> {
        s.to_string().try_into()
    }

    pub fn is_valid(s: &str) -> bool {
        s.contains('-') && !s.contains('_') && !s.contains('/')
    }

    pub fn normalize(&self) -> NormalizedTradingPair {
        let mut split = self.0.split('-');
        NormalizedTradingPair::new_base_quote(CexExchange::Kucoin, split.next().unwrap(), split.next().unwrap(), Some('-'), None)
    }
}

impl Serialize for KucoinTradingPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for KucoinTradingPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        Ok(KucoinTradingPair(s))
    }
}

impl TryFrom<NormalizedTradingPair> for KucoinTradingPair {
    type Error = eyre::Report;

    fn try_from(value: NormalizedTradingPair) -> Result<Self, Self::Error> {
        if let Some((base, quote)) = value.base_quote() {
            return Ok(KucoinTradingPair(format!("{}-{}", base, quote)))
        }

        if let (Some(raw_pair), delim) = (value.pair(), value.delimiter()) {
            if let Ok(v) = Self::new_checked(&raw_pair) {
                return Ok(v)
            }

            if let Some(d) = delim {
                let mut split = raw_pair.split(d);
                return Ok(KucoinTradingPair(format!("{}-{}", split.next().unwrap().to_uppercase(), split.next().unwrap().to_uppercase())));
            }

            let new_str = raw_pair.replace('_', "-").replace('/', "-");
            if let Ok(this) = Self::new_checked(&new_str) {
                return Ok(this)
            }

            return Err(eyre::ErrReport::msg(format!("INVALID Kucoin trading pair '{raw_pair}' contains no '-'")))
        }

        Err(eyre::ErrReport::msg(format!("INVALID Kucoin trading pair: '{:?}'", value)))
    }
}

impl TryFrom<String> for KucoinTradingPair {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::is_valid(&value) {
            Ok(KucoinTradingPair(value))
        } else {
            Err(eyre::ErrReport::msg(format!("INVALID Kucoin trading pair '{value}' contains either: 1) '_', and/or '/' - OR -  2) no '-'")))
        }
    }
}

impl TryFrom<&str> for KucoinTradingPair {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if Self::is_valid(value) {
            Ok(KucoinTradingPair(value.to_uppercase()))
        } else {
            Err(eyre::ErrReport::msg(format!("INVALID Kucoin trading pair '{value}' contains either: 1) '_', and/or '/' - OR -  2) no '-'")))
        }
    }
}
