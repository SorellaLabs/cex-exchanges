use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::OkexInstrument;
use crate::{exchanges::normalized::types::NormalizedCurrency, normalized::rest_api::NormalizedRestApiDataTypes, CexExchange};

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct OkexAllSymbols {
    pub currencies: Vec<NormalizedCurrency>
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

        Self { currencies }
    }

    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        self.currencies
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for OkexAllSymbols {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllCurrencies(other_currs) => *other_currs == self.currencies,
            _ => false
        }
    }
}
