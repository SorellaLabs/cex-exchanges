use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::CexExchange;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NormalizedTradingPair {
    exchange:   CexExchange,
    pair:       Option<String>,
    base_quote: Option<(String, String)>,
    delimiter:  Option<char>
}

impl NormalizedTradingPair {
    /// extra data would be like '240201' in 'BTC-USD-240201'
    pub(crate) fn new_base_quote(exchange: CexExchange, base: &str, quote: &str, delimiter: Option<char>, extra_data: Option<String>) -> Self {
        let extra_data = extra_data.map(|d| d.to_uppercase());
        let pair = if let Some(del) = delimiter {
            if let Some(ed) = extra_data {
                format!("{}{del}{}{del}{ed}", base.to_uppercase(), quote.to_uppercase())
            } else {
                format!("{}{del}{}", base.to_uppercase(), quote.to_uppercase())
            }
        } else if let Some(ed) = extra_data {
            format!("{}{}{ed}", base.to_uppercase(), quote.to_uppercase())
        } else {
            format!("{}{}", base.to_uppercase(), quote.to_uppercase())
        };

        Self { pair: Some(pair), base_quote: Some((base.to_uppercase(), quote.to_uppercase())), exchange, delimiter }
    }

    pub(crate) fn new_no_base_quote(exchange: CexExchange, pair: &str) -> Self {
        Self { pair: Some(pair.to_uppercase()), base_quote: None, exchange, delimiter: None }
    }

    pub fn exchange(&self) -> CexExchange {
        self.exchange
    }

    pub fn pair(&self) -> &Option<String> {
        &self.pair
    }

    pub fn base_quote(&self) -> &Option<(String, String)> {
        &self.base_quote
    }

    pub fn base(&self) -> Option<&String> {
        self.base_quote.as_ref().map(|(b, _)| b)
    }

    pub fn quote(&self) -> Option<&String> {
        self.base_quote.as_ref().map(|(_, q)| q)
    }

    pub fn delimiter(&self) -> Option<char> {
        self.delimiter
    }

    pub fn make_pair(&self) -> String {
        if let Some(pair) = self.pair().clone() {
            return pair
        }

        if let Some((base, quote)) = self.base_quote().clone() {
            if let Some(del) = self.delimiter() {
                return format!("{}{del}{}", base.to_uppercase(), quote.to_uppercase())
            } else {
                return format!("{}{}", base.to_uppercase(), quote.to_uppercase())
            }
        }

        unreachable!()
    }

    pub fn extra_data(&self) -> Option<String> {
        if let (Some(pair), Some((base, quote))) = (self.pair(), self.base_quote()) {
            let mut ed = pair.clone();
            if let Some(del) = self.delimiter() {
                ed = ed.replace(&format!("{base}{del}{quote}"), "");
                if ed.is_empty() {
                    None
                } else {
                    ed = ed[1..].to_string();
                    Some(ed)
                }
            } else {
                ed = ed.replace(&format!("{base}{quote}"), "");
                if ed.is_empty() {
                    None
                } else {
                    Some(ed)
                }
            }
        } else {
            None
        }
    }
}

impl Display for NormalizedTradingPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pair = self.make_pair();

        writeln!(f, "{}", pair)
    }
}

#[derive(Debug, Clone)]
pub enum RawTradingPair {
    /// (base, quote)
    /// ex: (ETH, USDC)
    /// ex: (ETH, USDC, 241202)
    Split { base: String, quote: String, extra_data: Option<String> },
    /// (base + quote, delimiter)
    /// ex: (ETH_USDC, _)
    RawDelim { pair: String, delimiter: char },
    /// raw trading pair w/o delimiter
    /// ex: ETHUSDC
    RawNoDelim { pair: String }
}

impl RawTradingPair {
    pub fn new_base_quote(base: &str, quote: &str, extra_data: Option<String>) -> Self {
        RawTradingPair::Split { base: base.to_uppercase(), quote: quote.to_uppercase(), extra_data: extra_data.map(|d| d.to_uppercase()) }
    }

    pub fn new_raw(pair: &str, delimiter: char) -> Self {
        if delimiter == '\0' {
            panic!("delimiter for coinbase cannot be empty/null - use 'new_no_delim' instead")
        }
        RawTradingPair::RawDelim { pair: pair.to_uppercase(), delimiter }
    }

    pub fn new_no_delim(pair: &str) -> Self {
        RawTradingPair::RawNoDelim { pair: pair.to_uppercase() }
    }

    pub fn get_normalized_pair(&self, exchange: CexExchange) -> NormalizedTradingPair {
        let this = self.clone();
        match this {
            RawTradingPair::Split { base, quote, extra_data } => NormalizedTradingPair::new_base_quote(exchange, &base, &quote, None, extra_data),
            RawTradingPair::RawDelim { pair, delimiter } => {
                let mut split = pair.split(delimiter);
                let (base, quote) = (split.next().unwrap(), split.next().unwrap());
                let extra_data = split.collect::<Vec<_>>();
                let ed = if extra_data.is_empty() { None } else { Some(extra_data.join(&delimiter.to_string())) };
                NormalizedTradingPair::new_base_quote(exchange, base, quote, Some(delimiter), ed)
            }
            RawTradingPair::RawNoDelim { pair } => NormalizedTradingPair::new_no_base_quote(exchange, &pair)
        }
    }
}

impl From<NormalizedTradingPair> for RawTradingPair {
    fn from(value: NormalizedTradingPair) -> Self {
        if let Some((base, quote)) = value.base_quote.clone() {
            return Self::Split { base, quote, extra_data: value.extra_data() }
        }

        if let Some(pair) = value.pair().clone() {
            if let Some(delimiter) = value.delimiter() {
                return Self::RawDelim { pair, delimiter }
            } else {
                return Self::RawNoDelim { pair }
            }
        }

        unreachable!("NormalizedTradingPair must be convertable")
    }
}
