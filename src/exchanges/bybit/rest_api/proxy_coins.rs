use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    binance::rest_api::{BinanceSymbol, BinanceSymbolPlatform},
    normalized::{
        rest_api::NormalizedRestApiDataTypes,
        types::{Blockchain, NormalizedCurrency}
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
        self.coins.into_iter().map(BybitCoin::normalize).collect()
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BybitAllCoins {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllCurrencies(other_currs) => {
                let mut this_currencies = self.coins.clone();
                this_currencies.sort_by(|a, b| a.symbol.partial_cmp(&b.symbol).unwrap());

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
pub struct BybitCoin {
    pub symbol:   String,
    pub name:     String,
    pub platform: Option<BybitProxyCoinPlatform>
}

impl BybitCoin {
    pub fn normalize(self) -> NormalizedCurrency {
        NormalizedCurrency {
            exchange:     CexExchange::Bybit,
            symbol:       self.symbol,
            name:         self.name,
            display_name: None,
            status:       format!("Binance Proxy"),
            blockchains:  self
                .platform
                .map(|v| vec![v.parse_blockchain_address()])
                .unwrap_or_default()
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

impl BybitProxyCoinPlatform {
    pub fn parse_blockchain_address(self) -> (Blockchain, Option<String>) {
        (self.name.parse().unwrap(), Some(self.token_address))
    }
}

impl From<BinanceSymbolPlatform> for BybitProxyCoinPlatform {
    fn from(value: BinanceSymbolPlatform) -> Self {
        Self { symbol: value.symbol, name: value.name, token_address: value.token_address }
    }
}

impl PartialEq<NormalizedCurrency> for BybitCoin {
    fn eq(&self, other: &NormalizedCurrency) -> bool {
        let equals = other.exchange == CexExchange::Binance
            && other.symbol == self.symbol
            && other.name == self.name
            && other.display_name.is_none()
            && other.status == format!("Binance Proxy")
            && other.blockchains
                == self
                    .platform
                    .as_ref()
                    .map(|v| vec![v.clone().parse_blockchain_address()])
                    .unwrap_or_default();

        if !equals {
            println!("\n\nSELF: {:?}\n", self);
            println!("NORMALIZED: {:?}\n\n", other);
        }

        equals
    }
}
