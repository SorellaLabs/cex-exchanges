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
    pub(crate) fn new_base_quote(exchange: CexExchange, base: &str, quote: &str, delimiter: Option<char>) -> Self {
        let pair = if let Some(del) = delimiter {
            format!("{}{del}{}", base.to_uppercase(), quote.to_uppercase())
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
}

#[derive(Debug, Clone)]
pub enum RawTradingPair {
    /// (base, quote)
    /// ex: (ETH, USDC)
    Split { base: String, quote: String },
    /// (base + quote, delimiter)
    /// ex: (ETH_USDC, _)
    Raw { pair: String, delimiter: char },
    /// raw trading pair w/o delimiter
    /// ex: ETHUSDC
    RawNoDelim { pair: String }
}

impl RawTradingPair {
    pub fn new_base_quote(base: &str, quote: &str) -> Self {
        RawTradingPair::Split { base: base.to_uppercase(), quote: quote.to_uppercase() }
    }

    pub fn new_raw(pair: &str, delimiter: char) -> Self {
        if delimiter == '\0' {
            panic!("delimiter for coinbase cannot be empty/null - use 'new_no_delim' instead")
        }
        RawTradingPair::Raw { pair: pair.to_uppercase(), delimiter }
    }

    pub fn new_no_delim(pair: &str) -> Self {
        RawTradingPair::RawNoDelim { pair: pair.to_uppercase() }
    }

    pub fn get_normalized_pair(&self, exchange: CexExchange) -> NormalizedTradingPair {
        let this = self.clone();
        match this {
            RawTradingPair::Split { base, quote } => NormalizedTradingPair::new_base_quote(exchange, &base, &quote, None),
            RawTradingPair::Raw { pair, delimiter } => {
                let mut split = pair.split(delimiter);
                NormalizedTradingPair::new_base_quote(exchange, split.next().unwrap(), split.next().unwrap(), Some(delimiter))
            }
            RawTradingPair::RawNoDelim { pair } => NormalizedTradingPair::new_no_base_quote(exchange, &pair)
        }
    }
}
