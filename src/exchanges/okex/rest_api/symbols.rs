use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::OkexCompleteInstrument;
use crate::{exchanges::normalized::types::NormalizedCurrency, normalized::rest_api::NormalizedRestApiDataTypes};

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct OkexAllSymbols {
    pub currencies: Vec<NormalizedCurrency>
}

impl OkexAllSymbols {
    pub(crate) fn new(mut currencies: Vec<NormalizedCurrency>, instruments: Vec<OkexCompleteInstrument>) -> Self {
        currencies.retain(|curr| {
            instruments.iter().any(|instr| {
                let base = if let Some(s) = &instr.instrument.base_currency { s.to_uppercase() == curr.symbol.to_uppercase() } else { false };

                let quote = if let Some(s) = &instr.instrument.quote_currency { s.to_uppercase() == curr.symbol.to_uppercase() } else { false };

                let contract = if let Some(s) = &instr.instrument.contract_currency { s.to_uppercase() == curr.symbol.to_uppercase() } else { false };

                let settlement =
                    if let Some(s) = &instr.instrument.settlement_currency { s.to_uppercase() == curr.symbol.to_uppercase() } else { false };

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
