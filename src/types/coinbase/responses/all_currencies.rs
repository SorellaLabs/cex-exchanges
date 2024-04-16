use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    exchanges::normalized::CexExchange,
    types::{blockchain::Blockchain, normalized::currencies::NormalizedCurrency}
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseAllCurrenciesResponse {
    pub currencies: Vec<CoinbaseAllCurrenciesProperties>
}

impl CoinbaseAllCurrenciesResponse {
    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        self.currencies
            .into_iter()
            .map(CoinbaseAllCurrenciesProperties::normalize)
            .collect()
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseAllCurrenciesProperties {
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
    pub details:            CoinbaseAllCurrenciesDetails,
    pub default_network:    String,
    pub supported_networks: Vec<CoinbaseSupportedNetwork>
}

impl CoinbaseAllCurrenciesProperties {
    pub fn normalize(self) -> NormalizedCurrency {
        NormalizedCurrency {
            exchange:      CexExchange::Coinbase,
            symbol:        self.id,
            name:          self.name,
            display_name:  self.display_name,
            min_size:      self.min_size,
            max_precision: self.max_precision,
            status:        self.status,
            is_fiat:       &self.details.kind.to_lowercase() == "fiat",
            blockchains:   self
                .supported_networks
                .into_iter()
                .map(|n| n.parse_blockchain_address())
                .collect()
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseSupportedNetwork {
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

impl CoinbaseSupportedNetwork {
    pub fn parse_blockchain_address(self) -> (Blockchain, Option<String>) {
        if self.contract_address == Some("".to_string()) {
            (self.name.parse().unwrap(), None)
        } else {
            (self.name.parse().unwrap(), self.contract_address)
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseAllCurrenciesDetails {
    #[serde(rename = "type")]
    pub kind:                    String,
    pub display_name:            Option<String>,
    pub symbol:                  Option<String>,
    pub network_confirmations:   Option<u64>,
    pub sort_order:              Option<u64>,
    pub crypto_address_link:     Option<String>,
    pub crypto_transaction_link: Option<String>,
    pub group_types:             Vec<String>,
    pub push_payment_methods:    Option<Vec<String>>,
    pub min_withdrawal_amount:   Option<f64>,
    pub max_withdrawal_amount:   Option<f64>,
    pub processing_time_seconds: Option<f64>
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for CoinbaseAllCurrenciesResponse {
    fn equals_normalized(self) -> bool {
        self.currencies.into_iter().all(|c| c.equals_normalized())
    }
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for CoinbaseAllCurrenciesProperties {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        let copy = self.clone();

        let equals = normalized.exchange == CexExchange::Coinbase
            && normalized.symbol == self.id
            && normalized.name == self.name
            && normalized.display_name == self.display_name
            && normalized.min_size == self.min_size
            && normalized.max_precision == self.max_precision
            && normalized.status == self.status
            && normalized.is_fiat == (&self.details.kind.to_lowercase() == "fiat")
            && normalized.blockchains
                == self
                    .supported_networks
                    .into_iter()
                    .map(|n| n.parse_blockchain_address())
                    .collect::<Vec<_>>();

        if !equals {
            println!("SELF: {:?}", copy);
            println!("NORMALIZED: {:?}", normalized);
        }

        equals
    }
}
