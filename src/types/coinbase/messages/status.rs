use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::types::coinbase::pairs::CoinbaseTradingPair;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseStatusMessage {
    pub products:   Vec<CoinbaseStatusProduct>,
    pub currencies: Vec<CoinbaseStatusCurrency>
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseStatusProduct {
    pub id: CoinbaseTradingPair,
    pub base_currency: String,
    pub quote_currency: String,
    #[serde_as(as = "DisplayFromStr")]
    pub base_increment: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub quote_increment: f64,
    pub display_name: String,
    pub status: String,
    pub margin_enabled: bool,
    pub status_message: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    pub min_market_funds: f64,
    pub post_only: bool,
    pub limit_only: bool,
    pub cancel_only: bool,
    pub auction_mode: bool,
    #[serde(rename = "type")]
    pub kind: String,
    pub fx_stablecoin: bool,
    #[serde_as(as = "DisplayFromStr")]
    pub max_slippage_percentage: f64
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseStatusCurrency {
    pub id:                 String,
    pub name:               String,
    pub display_name:       String,
    #[serde_as(as = "DisplayFromStr")]
    pub min_size:           f64,
    pub status:             String,
    pub funding_account_id: String,
    pub status_message:     Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    pub max_precision:      f64,
    pub convertible_to:     Vec<String>,
    pub details:            CoinbaseStatusCurrencyDetails,
    pub default_network:    String,
    pub supported_networks: Vec<CoinbaseSupportedNetwork>
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseSupportedNetwork {
    pub id: String,
    pub name: String,
    pub status: String,
    pub contract_address: String,
    pub crypto_address_link: String,
    pub crypto_transaction_link: String,
    pub min_withdrawal_amount: f64,
    pub max_withdrawal_amount: f64,
    pub network_confirmations: u64,
    pub processing_time_seconds: u64,
    pub destination_tag_regex: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseStatusCurrencyDetails {
    #[serde(rename = "type")]
    pub kind:                    String,
    pub symbol:                  String,
    pub network_confirmations:   u64,
    pub sort_order:              u64,
    pub crypto_address_link:     String,
    pub crypto_transaction_link: String,
    pub push_payment_methods:    Option<Vec<String>>,
    pub min_withdrawal_amount:   Option<f64>,
    pub max_withdrawal_amount:   Option<f64>
}
