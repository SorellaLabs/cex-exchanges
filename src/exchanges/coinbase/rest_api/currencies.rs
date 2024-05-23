use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    normalized::{
        rest_api::NormalizedRestApiDataTypes,
        types::{Blockchain, BlockchainCurrency, NormalizedCurrency}
    },
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct CoinbaseAllCurrencies {
    pub currencies: Vec<CoinbaseCurrency>
}

impl CoinbaseAllCurrencies {
    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        self.currencies
            .into_iter()
            .map(CoinbaseCurrency::normalize)
            .collect()
    }
}

impl<'de> Deserialize<'de> for CoinbaseAllCurrencies {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let currencies = Vec::<CoinbaseCurrency>::deserialize(deserializer)?;

        Ok(CoinbaseAllCurrencies { currencies })
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for CoinbaseAllCurrencies {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllCurrencies(other_currs) => {
                let this_currencies = self
                    .currencies
                    .iter()
                    .map(|sym| (&sym.name, &sym.id))
                    .collect::<HashSet<_>>();

                let others_currencies = other_currs.clone();
                let mut normalized_out = 0;

                others_currencies.iter().for_each(|curr| {
                    curr.blockchains.iter().for_each(|blk| {
                        if blk.wrapped_currency.is_some() && blk.is_wrapped && curr.blockchains.len() == 1 {
                            normalized_out += 1;
                        }
                    })
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
pub struct CoinbaseCurrency {
    pub id:                 String,
    pub name:               String,
    #[serde_as(as = "DisplayFromStr")]
    pub min_size:           f64,
    pub status:             String,
    pub message:            String,
    #[serde_as(as = "DisplayFromStr")]
    pub max_precision:      f64,
    pub convertible_to:     Vec<String>,
    pub display_name:       Option<String>,
    pub details:            CoinbaseCurrencyDetails,
    pub default_network:    String,
    pub supported_networks: Vec<CoinbaseCurrencySupportedNetwork>
}

impl CoinbaseCurrency {
    fn parse_blockchains(&self) -> Vec<BlockchainCurrency> {
        let is_wrapped = if self.name.to_lowercase().contains("wrapped") && self.id.to_lowercase().starts_with("w") { true } else { false };

        self.supported_networks
            .iter()
            .map(|net| {
                if net.contract_address == Some("".to_string()) {
                    BlockchainCurrency { blockchain: net.name.parse().unwrap(), address: None, is_wrapped, wrapped_currency: None }
                } else {
                    BlockchainCurrency {
                        blockchain: net.name.parse().unwrap(),
                        address: net.contract_address.clone(),
                        is_wrapped,
                        wrapped_currency: None
                    }
                }
            })
            .collect()
    }

    pub fn normalize(self) -> NormalizedCurrency {
        let blockchains = self.parse_blockchains();
        NormalizedCurrency {
            exchange: CexExchange::Coinbase,
            symbol: self.id,
            name: self.name,
            display_name: self.display_name,
            status: self.status,
            blockchains
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct CoinbaseCurrencySupportedNetwork {
    pub id: String,
    pub name: String,
    pub status: String,
    pub contract_address: Option<String>,
    pub crypto_address_link: Option<String>,
    pub crypto_transaction_link: Option<String>,
    pub min_withdrawal_amount: Option<f64>,
    pub max_withdrawal_amount: Option<f64>,
    pub network_confirmations: Option<u64>,
    pub processing_time_seconds: Option<f64>
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct CoinbaseCurrencyDetails {
    #[serde(rename = "type")]
    pub kind:                    String,
    pub display_name:            Option<String>,
    pub symbol:                  Option<String>,
    pub network_confirmations:   Option<u64>,
    pub sort_order:              Option<i64>,
    pub crypto_address_link:     Option<String>,
    pub crypto_transaction_link: Option<String>,
    pub group_types:             Vec<String>,
    pub push_payment_methods:    Option<Vec<String>>,
    pub min_withdrawal_amount:   Option<f64>,
    pub max_withdrawal_amount:   Option<f64>,
    pub processing_time_seconds: Option<f64>
}

impl PartialEq<NormalizedCurrency> for CoinbaseCurrency {
    fn eq(&self, other: &NormalizedCurrency) -> bool {
        let blockchains = self.parse_blockchains();
        let equals = other.exchange == CexExchange::Coinbase
            && other.symbol == self.id
            && other.name == self.name
            && other.display_name == self.display_name
            && other.status == self.status
            && other
                .blockchains
                .iter()
                .cloned()
                .filter(|blk| blk.wrapped_currency.is_none())
                .collect::<Vec<_>>()
                == blockchains;

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
