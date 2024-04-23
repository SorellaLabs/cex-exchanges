use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull, DisplayFromStr, NoneAsEmptyString};

use crate::{
    normalized::{
        rest_api::NormalizedRestApiDataTypes,
        types::{Blockchain, NormalizedCurrency}
    },
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BybitAllCurrencies {
    #[serde(rename = "data")]
    pub currencies: Vec<BybitCurrency>
}

impl BybitAllCurrencies {
    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        self.currencies
            .into_iter()
            .map(BybitCurrency::normalize)
            .collect()
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BybitAllCurrencies {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllCurrencies(other_currs) => {
                let mut this_currencies = self.currencies.clone();
                this_currencies.sort_by(|a, b| a.currency.partial_cmp(&b.currency).unwrap());

                let mut others_currencies = other_currs.clone();
                others_currencies.sort_by(|a, b| a.symbol.partial_cmp(&b.symbol).unwrap());

                this_currencies == others_currencies
            }
            _ => false
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BybitCurrency {
    pub currency:          String,
    pub name:              String,
    #[serde(rename = "fullName")]
    pub full_name:         String,
    pub precision:         u64,
    pub confirms:          Option<u64>,
    #[serde(rename = "contractAddress")]
    pub contract_address:  Option<String>,
    #[serde(rename = "isMarginEnabled")]
    pub is_margin_enabled: bool,
    #[serde(rename = "isDebitEnabled")]
    pub is_debit_enabled:  bool,
    #[serde_as(as = "DefaultOnNull")]
    pub chains:            Vec<BybitCurrencyChain>
}

impl BybitCurrency {
    pub fn normalize(self) -> NormalizedCurrency {
        NormalizedCurrency {
            exchange:     CexExchange::Bybit,
            symbol:       self.currency,
            name:         self.full_name,
            display_name: None,
            status:       "".to_string(),
            blockchains:  self
                .chains
                .into_iter()
                .map(|c| c.parse_blockchain_address())
                .collect()
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BybitCurrencyChain {
    #[serde(rename = "chainName")]
    pub chain_name:          String,
    #[serde(rename = "withdrawalMinFee")]
    #[serde_as(as = "DisplayFromStr")]
    pub withdrawal_min_fee:  f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "withdrawalMinSize")]
    pub withdrawal_min_size: f64,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "withdrawFeeRate")]
    pub withdraw_fee_rate:   Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "depositMinSize")]
    pub deposit_min_size:    Option<f64>,
    #[serde(rename = "isWithdrawEnabled")]
    pub is_withdraw_enabled: bool,
    #[serde(rename = "isDepositEnabled")]
    pub is_deposit_enabled:  bool,
    #[serde(rename = "preConfirms")]
    pub pre_confirms:        u64,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "contractAddress")]
    pub contract_address:    Option<String>,
    #[serde(rename = "chainId")]
    pub chain_id:            String,
    pub confirms:            Option<u64>
}

impl BybitCurrencyChain {
    pub fn parse_blockchain_address(self) -> (Blockchain, Option<String>) {
        (self.chain_name.parse().unwrap(), self.contract_address)
    }
}

impl PartialEq<NormalizedCurrency> for BybitCurrency {
    fn eq(&self, other: &NormalizedCurrency) -> bool {
        let equals = other.exchange == CexExchange::Bybit
            && other.symbol == self.currency
            && other.name == self.full_name
            && other.display_name.is_none()
            && other.status == "".to_string()
            && self.chains.iter().all(|c| {
                other
                    .blockchains
                    .contains(&c.clone().parse_blockchain_address())
            });

        if !equals {
            println!("\n\nSELF: {:?}\n", self);
            println!("NORMALIZED: {:?}\n\n", other);
        }

        equals
    }
}
