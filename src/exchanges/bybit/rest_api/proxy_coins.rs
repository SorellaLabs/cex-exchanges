use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

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

                let mut others_currencies = other_currs.clone();
                let mut normalized_out = 0;

                others_currencies.retain(|curr| {
                    let vvv = !(curr
                        .blockchains
                        .iter()
                        .any(|blk| blk.wrapped_currency.is_some() && blk.is_wrapped)
                        && curr.blockchains.len() == 1);

                    if !vvv {
                        normalized_out += 1;
                        println!("vvv: {:?}", curr);
                    }

                    vvv
                });

                // let a0 = ;
                // let a1 = ;
                // let a2 = normalized_out;
                // println!("{}", a0);
                // println!("{}", a1);
                // println!("{}", a2);

                // let b = others_currencies
                //     .iter()
                //     .all(|curr| this_currencies.contains(&(&curr.name, &curr.symbol)));
                // println!("{}", b);

                self.coins.len() == others_currencies.len() + normalized_out
                    && others_currencies
                        .iter()
                        .all(|curr| this_currencies.contains(&(&curr.name, &curr.symbol)))
                // true
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
            let is_wrapped = if self.name.to_lowercase().contains("wrapped") && self.symbol.to_lowercase().starts_with("w") { true } else { false };
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
            status: format!("binance proxy"),
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
            && other.status == format!("binance proxy")
            && other
                .blockchains
                .iter()
                .cloned()
                .filter(|blk| blk.wrapped_currency.is_none())
                .collect::<Vec<_>>()
                == blockchains;

        if !equals {
            println!("\n\nSELF: {:?}\n", self);
            println!("NORMALIZED: {:?}\n\n", other);
        }

        equals
    }
}
