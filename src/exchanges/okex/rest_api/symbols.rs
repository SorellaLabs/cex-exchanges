use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::OkexInstrument;
use crate::{
    exchanges::normalized::types::NormalizedCurrency,
    normalized::{rest_api::NormalizedRestApiDataTypes, types::BlockchainCurrency},
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct OkexAllSymbols {
    pub currencies: Vec<OkexCurrency>
}

impl OkexAllSymbols {
    pub(crate) fn new(mut currencies: Vec<NormalizedCurrency>, instruments: Vec<OkexInstrument>) -> Self {
        currencies.retain_mut(|curr| {
            instruments.iter().any(|instr| {
                let base = if let Some(s) = &instr.base_currency { s.to_uppercase() == curr.symbol.to_uppercase() } else { false };

                let quote = if let Some(s) = &instr.quote_currency { s.to_uppercase() == curr.symbol.to_uppercase() } else { false };

                let contract = if let Some(s) = &instr.contract_currency { s.to_uppercase() == curr.symbol.to_uppercase() } else { false };

                let settlement = if let Some(s) = &instr.settlement_currency { s.to_uppercase() == curr.symbol.to_uppercase() } else { false };

                curr.exchange = CexExchange::Okex;

                base || quote || contract || settlement
            })
        });

        Self { currencies: currencies.into_iter().map(|c| c.into()).collect() }
    }

    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        self.currencies
            .into_iter()
            .map(OkexCurrency::normalize)
            .collect()
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for OkexAllSymbols {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllCurrencies(other_currs) => {
                let this_currencies = self
                    .currencies
                    .iter()
                    .map(|sym| (&sym.name, &sym.symbol))
                    .collect::<HashSet<_>>();

                let others_currencies = other_currs.clone();
                let mut normalized_out = 0;

                others_currencies.iter().for_each(|curr| {
                    if curr.blockchains.iter().any(|blk| {
                        if let Some(blk_curr) = blk.wrapped_currency.as_ref() {
                            blk.is_wrapped && blk_curr.name.to_lowercase().contains("wrapped") && blk_curr.symbol.to_lowercase().starts_with("w")
                        } else {
                            false
                        }
                    }) {
                        normalized_out += 1;
                    }
                });

                println!("A: {}", self.currencies.len());
                println!("B: {}", others_currencies.len());
                println!("C: {}", normalized_out);
                println!(
                    "D: {}",
                    others_currencies
                        .iter()
                        .all(|curr| this_currencies.contains(&(&curr.name, &curr.symbol)))
                );

                self.currencies.len() == others_currencies.len() + normalized_out
                    && others_currencies
                        .iter()
                        .all(|curr| this_currencies.contains(&(&curr.name, &curr.symbol)))
            }
            _ => false
        }
    }
}

/// akin to normalized currencies since we have to use proxies for non-apikey
/// users
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexCurrency {
    pub exchange:     CexExchange,
    pub symbol:       String,
    pub name:         String,
    pub display_name: Option<String>,
    pub status:       String,
    pub blockchains:  Vec<BlockchainCurrency>
}

impl OkexCurrency {
    pub fn normalize(self) -> NormalizedCurrency {
        NormalizedCurrency {
            exchange:     self.exchange,
            symbol:       self.symbol,
            name:         self.name,
            display_name: self.display_name,
            status:       self.status,
            blockchains:  self.blockchains
        }
    }
}

impl From<NormalizedCurrency> for OkexCurrency {
    fn from(value: NormalizedCurrency) -> Self {
        Self {
            exchange:     value.exchange,
            symbol:       value.symbol,
            name:         value.name,
            display_name: value.display_name,
            status:       value.status,
            blockchains:  value.blockchains
        }
    }
}

impl PartialEq<NormalizedCurrency> for OkexCurrency {
    fn eq(&self, other: &NormalizedCurrency) -> bool {
        let equals = other.exchange == CexExchange::Okex
            && other.symbol == self.symbol
            && other.name == self.name
            && other.display_name == self.display_name
            && other.status == self.status
            && other.blockchains == self.blockchains;

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
