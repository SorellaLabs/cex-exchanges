use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{exchanges::normalized::types::NormalizedTradingPair, CexExchange};

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct OkexTradingPair(pub(crate) String);

impl OkexTradingPair {
    pub fn new_checked(s: &str) -> eyre::Result<Self> {
        s.to_string().try_into()
    }

    pub fn is_valid(s: &str) -> bool {
        s.contains('-') && !s.contains('_') && !s.contains('/')
    }

    pub fn normalize(&self) -> NormalizedTradingPair {
        let mut split = self.0.split('-');
        let (base, quote) = (split.next().unwrap(), split.next().unwrap());

        let extra_data = split.collect::<Vec<_>>();
        let ed = if !extra_data.is_empty() { Some(extra_data.join("-")) } else { None };

        NormalizedTradingPair::new_base_quote(CexExchange::Okex, base, quote, Some('-'), ed)
    }
}

impl Display for OkexTradingPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for OkexTradingPair {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for OkexTradingPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;

        Ok(OkexTradingPair(s))
    }
}

impl TryFrom<NormalizedTradingPair> for OkexTradingPair {
    type Error = eyre::Report;

    fn try_from(value: NormalizedTradingPair) -> Result<Self, Self::Error> {
        if let Some((base, quote)) = value.base_quote() {
            if value.pair().is_none() {
                if let Ok(this) = Self::new_checked(&format!("{base}-{quote}")) {
                    return Ok(this)
                }
            } else if format!("{base}-{quote}").len() == value.pair().as_ref().unwrap().len() {
                if let Ok(this) = Self::new_checked(&format!("{base}-{quote}")) {
                    return Ok(this)
                }
            } else if value.delimiter().is_none() {
                let mut temp = value.pair().clone().unwrap();
                temp = temp.replace(&format!("{base}{quote}"), "");

                let new = if !temp.is_empty() { format!("{base}-{quote}-{temp}") } else { format!("{base}-{quote}") };
                if let Ok(this) = Self::new_checked(&new) {
                    return Ok(this)
                }
            }
        }

        if let (Some(raw_pair), delim) = (value.pair(), value.delimiter()) {
            if let Ok(v) = OkexTradingPair::new_checked(&raw_pair) {
                return Ok(v)
            }

            if let Some(d) = delim {
                let mut split = raw_pair.split(d);
                let (base, quote) = (split.next().unwrap().to_uppercase(), split.next().unwrap().to_uppercase());

                let extra_data = split.collect::<Vec<_>>();

                let attempt_pair = if !extra_data.is_empty() {
                    let ed = extra_data.join("-");
                    OkexTradingPair::new_checked(&format!("{base}-{quote}-{ed}"))
                } else {
                    OkexTradingPair::new_checked(&format!("{base}-{quote}"))
                };

                if let Ok(v) = attempt_pair {
                    return Ok(v)
                }
            }

            let new_str = raw_pair.replace('_', "-").replace('/', "-");
            if let Ok(this) = Self::new_checked(&new_str) {
                return Ok(this)
            }

            return Err(eyre::ErrReport::msg(format!("INVALID Okex trading pair: '{raw_pair}' contains no '-'")))
        }

        Err(eyre::ErrReport::msg(format!("INVALID Okex trading pair: '{:?}'", value)))
    }
}

impl TryFrom<String> for OkexTradingPair {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::is_valid(&value) {
            Ok(OkexTradingPair(value))
        } else {
            Err(eyre::ErrReport::msg(format!("INVALID Okex trading pair '{value}' contains either: 1) '_', and/or '/' - OR -  2) no '-'")))
        }
    }
}

impl TryFrom<&str> for OkexTradingPair {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if Self::is_valid(value) {
            Ok(OkexTradingPair(value.to_uppercase()))
        } else {
            Err(eyre::ErrReport::msg(format!("INVALID Okex trading pair '{value}' contains either: 1) '_', and/or '/' - OR -  2) no '-'")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_normalized_pair() {
        let pair = NormalizedTradingPair::new_base_quote(CexExchange::Okex, "ETH", "USDC", Some('_'), None);
        let calculated_okex_pair: OkexTradingPair = pair.try_into().unwrap();
        let okex_pair = OkexTradingPair("ETH-USDC".to_string());
        assert_eq!(okex_pair, calculated_okex_pair);

        let pair = NormalizedTradingPair::new_base_quote(CexExchange::Okex, "ETH", "USDC", Some('-'), None);
        let calculated_okex_pair: OkexTradingPair = pair.try_into().unwrap();
        let okex_pair = OkexTradingPair("ETH-USDC".to_string());
        assert_eq!(okex_pair, calculated_okex_pair);

        let pair = NormalizedTradingPair::new_base_quote(CexExchange::Okex, "EtH", "usdC", None, None);
        let calculated_okex_pair: OkexTradingPair = pair.try_into().unwrap();
        let okex_pair = OkexTradingPair("ETH-USDC".to_string());
        assert_eq!(okex_pair, calculated_okex_pair);

        let pair = NormalizedTradingPair::new_base_quote(CexExchange::Okex, "EtH", "usdC", None, Some("yesssssss".to_string()));
        let calculated_okex_pair: OkexTradingPair = pair.try_into().unwrap();
        let okex_pair = OkexTradingPair("ETH-USDC-YESSSSSSS".to_string());
        assert_eq!(okex_pair, calculated_okex_pair);

        let pair = NormalizedTradingPair::new_base_quote(CexExchange::Okex, "EtH", "usdC", Some('/'), Some("123-1234as-fd".to_string()));
        let calculated_okex_pair: OkexTradingPair = pair.try_into().unwrap();
        let okex_pair = OkexTradingPair("ETH-USDC-123-1234AS-FD".to_string());
        assert_eq!(okex_pair, calculated_okex_pair);
    }
}
