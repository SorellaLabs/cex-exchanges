use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull, DisplayFromStr, NoneAsEmptyString};
use tracing::warn;

use crate::{
    normalized::{
        rest_api::NormalizedRestApiDataTypes,
        types::{BlockchainCurrency, NormalizedCurrency}
    },
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct KucoinAllCurrencies {
    #[serde(rename = "data")]
    pub currencies: Vec<KucoinCurrency>
}

impl KucoinAllCurrencies {
    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        NormalizedCurrency::handle_unwrapped(
            self.currencies
                .into_iter()
                .map(KucoinCurrency::normalize)
                .collect::<Vec<_>>()
        )
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for KucoinAllCurrencies {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllCurrencies(other_currs) => {
                let this_currencies = self
                    .currencies
                    .iter()
                    .map(|sym| (&sym.full_name, &sym.currency))
                    .collect::<HashSet<_>>();

                let others_currencies = other_currs.clone();
                let mut normalized_out = 0;

                others_currencies.iter().for_each(|curr| {
                    if curr.blockchains.iter().any(|blk| {
                        if let Some(blk_curr) = blk.wrapped_currency.as_ref() {
                            blk.is_wrapped && blk_curr.name.to_lowercase().contains("wrapped") && blk_curr.symbol.to_lowercase().starts_with('w')
                        } else {
                            false
                        }
                    }) {
                        normalized_out += 1;
                    }
                });

                self.currencies.len() == others_currencies.len() + normalized_out
                    && others_currencies
                        .iter()
                        .all(|curr| this_currencies.contains(&(&curr.name, &curr.symbol)))
            }
            _ => false
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct KucoinCurrency {
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
    pub chains:            Vec<KucoinCurrencyChain>
}

impl KucoinCurrency {
    pub fn normalize(self) -> NormalizedCurrency {
        let is_wrapped = self.full_name.to_lowercase().contains("wrapped") && self.currency.to_lowercase().starts_with('w');
        NormalizedCurrency {
            exchange:     CexExchange::Kucoin,
            symbol:       self.currency,
            name:         self.full_name,
            display_name: None,
            status:       "".to_string(),
            blockchains:  self
                .chains
                .into_iter()
                .map(|c| {
                    let mut bc: BlockchainCurrency = c.into();
                    bc.wrapped(is_wrapped);
                    bc
                })
                .collect()
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct KucoinCurrencyChain {
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

impl From<KucoinCurrencyChain> for BlockchainCurrency {
    fn from(val: KucoinCurrencyChain) -> Self {
        BlockchainCurrency {
            blockchain:       val.chain_name.parse().unwrap(),
            address:          val.contract_address,
            is_wrapped:       false,
            wrapped_currency: None
        }
    }
}

impl PartialEq<NormalizedCurrency> for KucoinCurrency {
    fn eq(&self, other: &NormalizedCurrency) -> bool {
        let equals = other.exchange == CexExchange::Kucoin
            && other.symbol == self.currency
            && other.name == self.full_name
            && other.display_name.is_none()
            && other.status == *""
            && self
                .chains
                .iter()
                .all(|c| other.blockchains.contains(&c.clone().into()));

        if !equals {
            warn!(target: "cex-exchanges::kucoin", "kucoin currency: {:?}", self);
            warn!(target: "cex-exchanges::kucoin", "normalized currency: {:?}", other);
        }

        equals
    }
}
