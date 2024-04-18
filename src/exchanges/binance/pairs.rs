use serde::{Deserialize, Serialize};

use crate::{exchanges::normalized::types::NormalizedTradingPair, CexExchange};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BinanceTradingPair(pub(crate) String);

impl BinanceTradingPair {
    pub fn new_checked(s: &str) -> eyre::Result<Self> {
        s.to_string().try_into()
    }

    pub fn is_valid(s: &str) -> bool {
        !s.contains('-') && !s.contains('_') && !s.contains('/')
    }

    pub fn normalize(&self) -> NormalizedTradingPair {
        NormalizedTradingPair::new_no_base_quote(CexExchange::Binance, &self.0)
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

impl TryFrom<NormalizedTradingPair> for BinanceTradingPair {
    type Error = eyre::Report;

    fn try_from(value: NormalizedTradingPair) -> Result<Self, Self::Error> {
        if let Some((base, quote)) = value.base_quote() {
            return Ok(BinanceTradingPair(format!("{}{}", base, quote)))
        }

        if let (Some(raw_pair), delim) = (value.pair(), value.delimiter()) {
            if let Ok(v) = Self::new_checked(&raw_pair) {
                return Ok(v)
            }

            if let Some(d) = delim {
                let mut split = raw_pair.split(d);
                return Ok(BinanceTradingPair(format!("{}{}", split.next().unwrap().to_uppercase(), split.next().unwrap().to_uppercase())));
            }

            let new_str = raw_pair.replace('_', "").replace('-', "").replace('/', "");
            if let Ok(this) = Self::new_checked(&new_str) {
                return Ok(this)
            }

            return Err(eyre::ErrReport::msg(format!("INVALID Binance trading pair '{raw_pair}'")))
        }

        return Err(eyre::ErrReport::msg(format!("INVALID Binance trading pair '{:?}'", value)))
    }
}

impl TryFrom<&str> for BinanceTradingPair {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if Self::is_valid(value) {
            Ok(BinanceTradingPair(value.to_uppercase()))
        } else {
            Err(eyre::ErrReport::msg(format!("INVALID Binance trading pair '{value}' contains a '-', '_', or '/'")))
        }
    }
}

impl TryFrom<String> for BinanceTradingPair {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}
