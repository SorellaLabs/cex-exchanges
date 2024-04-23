use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DefaultOnError};

use crate::{
    binance::BinanceTradingPair,
    exchanges::normalized::types::NormalizedInstrument,
    normalized::{rest_api::NormalizedRestApiDataTypes, types::NormalizedTradingType},
    CexExchange
};

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct BinanceAllInstruments {
    #[serde(rename = "symbols")]
    pub instruments: Vec<BinanceInstrument>
}
impl BinanceAllInstruments {
    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.instruments
            .into_iter()
            .flat_map(BinanceInstrument::normalize)
            .collect()
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BinanceAllInstruments {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllInstruments(other_instrs) => {
                let this_instruments = self
                    .instruments
                    .iter()
                    .map(|instr| (instr.base_asset.clone(), instr.quote_asset.clone(), instr.symbol.normalize()))
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

impl<'de> Deserialize<'de> for BinanceAllInstruments {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let val = Value::deserialize(deserializer)?;

        let instruments_value = val
            .get("symbols")
            .ok_or(eyre::ErrReport::msg("could not find 'symbols' field in binance instruments response".to_string()))
            .map_err(serde::de::Error::custom)?;

        let instruments = serde_json::from_value(instruments_value.clone()).map_err(serde::de::Error::custom)?;

        Ok(BinanceAllInstruments { instruments })
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BinanceInstrument {
    pub symbol: BinanceTradingPair,
    pub status: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
    #[serde(rename = "baseAssetPrecision")]
    pub base_asset_precision: u64,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
    #[serde(rename = "quotePrecision")]
    pub quote_precision: u64,
    #[serde(rename = "quoteAssetPrecision")]
    pub quote_asset_precision: u64,
    #[serde(rename = "orderTypes")]
    pub order_types: Vec<String>,
    #[serde(rename = "icebergAllowed")]
    pub iceberg_allowed: bool,
    #[serde(rename = "ocoAllowed")]
    pub oco_allowed: bool,
    #[serde(rename = "quoteOrderQtyMarketAllowed")]
    pub quote_order_qty_market_allowed: bool,
    #[serde(rename = "allowTrailingStop")]
    pub allow_trailing_stop: bool,
    #[serde(rename = "cancelReplaceAllowed")]
    pub cancel_replace_allowed: bool,
    #[serde(rename = "isSpotTradingAllowed")]
    pub is_spot_trading_allowed: bool,
    #[serde(rename = "isMarginTradingAllowed")]
    pub is_margin_trading_allowed: bool,
    #[serde_as(deserialize_as = "Vec<DefaultOnError>")]
    pub permissions: Vec<NormalizedTradingType>,
    #[serde(rename = "defaultSelfTradePreventionMode")]
    pub default_self_trade_prevention_mode: String,
    #[serde(rename = "allowedSelfTradePreventionModes")]
    pub allowed_self_trade_prevention_modes: Vec<String>
}

impl BinanceInstrument {
    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.permissions
            .into_iter()
            .filter_map(|perm| {
                if perm != NormalizedTradingType::Other {
                    Some(NormalizedInstrument {
                        exchange:           CexExchange::Binance,
                        trading_pair:       self.symbol.normalize(),
                        trading_type:       perm,
                        base_asset_symbol:  self.base_asset.clone(),
                        quote_asset_symbol: self.quote_asset.clone(),
                        active:             (&self.status == "TRADING"),
                        futures_expiry:     None
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

impl PartialEq<NormalizedInstrument> for BinanceInstrument {
    fn eq(&self, other: &NormalizedInstrument) -> bool {
        let equals = other.exchange == CexExchange::Binance
            && other.trading_pair == self.symbol.normalize()
            && self.permissions.iter().any(|t| t == &other.trading_type)
            && other.base_asset_symbol == *self.base_asset
            && other.quote_asset_symbol == *self.quote_asset
            && other.active == (&self.status == "TRADING")
            && other.futures_expiry == None;

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
