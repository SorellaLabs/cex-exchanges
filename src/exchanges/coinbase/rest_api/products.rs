use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};

use crate::{
    coinbase::CoinbaseTradingPair,
    exchanges::normalized::types::NormalizedInstrument,
    normalized::{rest_api::NormalizedRestApiDataTypes, types::NormalizedTradingType},
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct CoinbaseAllProducts {
    pub instruments: Vec<CoinbaseProduct>
}

impl CoinbaseAllProducts {
    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.instruments
            .into_iter()
            .flat_map(CoinbaseProduct::normalize)
            .collect()
    }
}

impl<'de> Deserialize<'de> for CoinbaseAllProducts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let instruments = Vec::<CoinbaseProduct>::deserialize(deserializer)?;

        Ok(CoinbaseAllProducts { instruments })
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for CoinbaseAllProducts {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllInstruments(other_instrs) => {
                let this_instruments = self
                    .instruments
                    .iter()
                    .map(|instr| (instr.base_currency.clone(), instr.quote_currency.clone(), instr.id.normalize()))
                    .collect::<HashSet<_>>();

                let others_instruments = other_instrs
                    .iter()
                    .map(|instr| (instr.base_asset_symbol.clone(), instr.quote_asset_symbol.clone(), instr.trading_pair.clone()))
                    .collect::<HashSet<_>>();

                others_instruments
                    .into_iter()
                    .all(|instr| this_instruments.contains(&instr))
            }
            _ => false
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct CoinbaseProduct {
    pub id: CoinbaseTradingPair,
    pub base_currency: String,
    pub quote_currency: String,
    #[serde_as(as = "DisplayFromStr")]
    pub quote_increment: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub base_increment: f64,
    pub display_name: String,
    #[serde_as(as = "DisplayFromStr")]
    pub min_market_funds: f64,
    pub margin_enabled: bool,
    pub post_only: bool,
    pub limit_only: bool,
    pub cancel_only: bool,
    pub status: String,
    #[serde_as(as = "NoneAsEmptyString")]
    pub status_message: Option<String>,
    pub trading_disabled: bool,
    pub fx_stablecoin: bool,
    #[serde_as(as = "DisplayFromStr")]
    pub max_slippage_percentage: f64,
    pub auction_mode: bool,
    #[serde_as(as = "NoneAsEmptyString")]
    pub high_bid_limit_percentage: Option<f64>
}

impl CoinbaseProduct {
    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        let mut instruments = vec![NormalizedInstrument {
            exchange:           CexExchange::Coinbase,
            trading_pair:       self.id.normalize(),
            trading_type:       NormalizedTradingType::Spot,
            base_asset_symbol:  self.base_currency.clone(),
            quote_asset_symbol: self.quote_currency.clone(),
            active:             !self.trading_disabled,
            futures_expiry:     None
        }];

        if self.margin_enabled {
            instruments.push(NormalizedInstrument {
                exchange:           CexExchange::Coinbase,
                trading_pair:       self.id.normalize(),
                trading_type:       NormalizedTradingType::Margin,
                base_asset_symbol:  self.base_currency,
                quote_asset_symbol: self.quote_currency,
                active:             !self.trading_disabled,
                futures_expiry:     None
            });
        }

        instruments
    }
}

impl PartialEq<NormalizedInstrument> for CoinbaseProduct {
    fn eq(&self, other: &NormalizedInstrument) -> bool {
        let equals = other.exchange == CexExchange::Coinbase
            && other.trading_pair == self.id.normalize()
            && other.base_asset_symbol == *self.base_currency
            && other.quote_asset_symbol == *self.quote_currency
            && other.active != self.trading_disabled
            && (other.trading_type == NormalizedTradingType::Spot || (other.trading_type == NormalizedTradingType::Margin && self.margin_enabled));

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
