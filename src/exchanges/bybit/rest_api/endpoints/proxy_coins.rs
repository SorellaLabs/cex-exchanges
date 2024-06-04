use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tracing::warn;

use crate::{
    binance::rest_api::{BinanceSymbol, BinanceSymbolPlatform},
    normalized::{
        rest_api::NormalizedRestApiDataTypes,
        types::{BlockchainCurrency, NormalizedCurrency}
    },
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BybitAllCoins {
    pub coins: Vec<BybitCoin>
}

impl BybitAllCoins {
    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        NormalizedCurrency::handle_unwrapped(self.coins.into_iter().map(BybitCoin::normalize).collect())
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BybitAllCoins {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllCurrencies(other_currs) => {
                let this_currencies = self
                    .coins
                    .iter()
                    .map(|sym| (&sym.name, &sym.symbol))
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

                self.coins.len() == others_currencies.len() + normalized_out
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
pub struct BybitCoin {
    pub symbol:   String,
    pub name:     String,
    pub platform: Option<BybitProxyCoinPlatform>
}

impl BybitCoin {
    fn parse_blockchain(&self) -> Option<BlockchainCurrency> {
        self.platform.as_ref().map(|pl| {
            let is_wrapped = self.name.to_lowercase().contains("wrapped") && self.symbol.to_lowercase().starts_with('w');
            BlockchainCurrency { blockchain: pl.name.parse().unwrap(), address: Some(pl.token_address.clone()), is_wrapped, wrapped_currency: None }
        })
    }

    pub fn normalize(self) -> NormalizedCurrency {
        let blockchains = self.parse_blockchain().map(|b| vec![b]).unwrap_or_default();
        NormalizedCurrency {
            exchange: CexExchange::Bybit,
            symbol: self.symbol,
            name: self.name,
            display_name: None,
            status: "binance proxy".to_string(),
            blockchains
        }
    }
}

impl From<BinanceSymbol> for BybitCoin {
    fn from(value: BinanceSymbol) -> Self {
        Self { symbol: value.symbol, name: value.name, platform: value.platform.map(Into::into) }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BybitProxyCoinPlatform {
    pub symbol:        String,
    pub name:          String,
    pub token_address: String
}

impl From<BinanceSymbolPlatform> for BybitProxyCoinPlatform {
    fn from(value: BinanceSymbolPlatform) -> Self {
        Self { symbol: value.symbol, name: value.name, token_address: value.token_address }
    }
}

impl PartialEq<NormalizedCurrency> for BybitCoin {
    fn eq(&self, other: &NormalizedCurrency) -> bool {
        let blockchains = self.parse_blockchain().map(|p| vec![p]).unwrap_or_default();
        let equals = other.exchange == CexExchange::Bybit
            && other.symbol == self.symbol
            && other.name == self.name
            && other.display_name.is_none()
            && other.status == *"binance proxy"
            && other
                .blockchains
                .iter()
                .filter(|&blk| blk.wrapped_currency.is_none())
                .cloned()
                .collect::<Vec<_>>()
                == blockchains;

        if !equals {
            warn!(target: "cex-exchanges::bybit", "bybit coin: {:?}", self);
            warn!(target: "cex-exchanges::bybit", "normalized currency: {:?}", other);
        }

        equals
    }
}
