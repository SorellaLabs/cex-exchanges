use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DefaultOnError, DisplayFromStr};

use crate::{
    binance::BinanceTradingPair,
    exchanges::normalized::types::NormalizedInstrument,
    normalized::{rest_api::NormalizedRestApiDataTypes, types::NormalizedTradingType},
    CexExchange
};

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct BinanceAllInstruments {
    pub instruments: Vec<BinanceCompleteInstrument>
}
impl BinanceAllInstruments {
    pub fn new(instrument: Vec<BinanceInstrument>, trading_ticker: Vec<BinanceTradingDayTicker>) -> Self {
        let ticker_map = trading_ticker
            .into_iter()
            .map(|ticker| (ticker.symbol.clone(), ticker))
            .collect::<HashMap<_, _>>();

        let complete = instrument
            .into_iter()
            .filter_map(|instr| {
                ticker_map
                    .get(&instr.symbol)
                    .map(|s| BinanceCompleteInstrument {
                        symbol: instr.symbol,
                        status: instr.status,
                        base_asset: instr.base_asset,
                        base_asset_precision: instr.base_asset_precision,
                        quote_asset: instr.quote_asset,
                        quote_precision: instr.quote_precision,
                        quote_asset_precision: instr.quote_asset_precision,
                        order_types: instr.order_types,
                        iceberg_allowed: instr.iceberg_allowed,
                        oco_allowed: instr.oco_allowed,
                        quote_order_qty_market_allowed: instr.quote_order_qty_market_allowed,
                        allow_trailing_stop: instr.allow_trailing_stop,
                        cancel_replace_allowed: instr.cancel_replace_allowed,
                        is_spot_trading_allowed: instr.is_spot_trading_allowed,
                        is_margin_trading_allowed: instr.is_margin_trading_allowed,
                        permissions: instr.permissions,
                        default_self_trade_prevention_mode: instr.default_self_trade_prevention_mode,
                        allowed_self_trade_prevention_modes: instr.allowed_self_trade_prevention_modes,
                        trade_count: s.count
                    })
            })
            .collect();

        Self { instruments: complete }
    }

    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.instruments
            .into_iter()
            .flat_map(BinanceCompleteInstrument::normalize)
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub(crate) struct BinanceAllInstrumentsUtil {
    #[serde(rename = "symbols")]
    pub(crate) instruments: Vec<BinanceInstrument>
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BinanceCompleteInstrument {
    pub symbol: BinanceTradingPair,
    pub status: String,
    pub base_asset: String,
    pub base_asset_precision: u64,
    pub quote_asset: String,
    pub quote_precision: u64,
    pub quote_asset_precision: u64,
    pub order_types: Vec<String>,
    pub iceberg_allowed: bool,
    pub oco_allowed: bool,
    pub quote_order_qty_market_allowed: bool,
    pub allow_trailing_stop: bool,
    pub cancel_replace_allowed: bool,
    pub is_spot_trading_allowed: bool,
    pub is_margin_trading_allowed: bool,
    pub permissions: Vec<NormalizedTradingType>,
    pub default_self_trade_prevention_mode: String,
    pub allowed_self_trade_prevention_modes: Vec<String>,
    pub trade_count: u64
}

impl BinanceCompleteInstrument {
    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.permissions
            .into_iter()
            .filter_map(|perm| {
                if perm != NormalizedTradingType::Other {
                    Some(NormalizedInstrument {
                        exchange:              CexExchange::Binance,
                        trading_pair:          self.symbol.normalize(),
                        trading_type:          perm,
                        base_asset_symbol:     self.base_asset.clone(),
                        quote_asset_symbol:    self.quote_asset.clone(),
                        active:                (&self.status == "TRADING"),
                        exchange_ranking:      self.trade_count as i64,
                        exchange_ranking_kind: "trade count".to_string(),
                        futures_expiry:        None
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

impl PartialEq<NormalizedInstrument> for BinanceCompleteInstrument {
    fn eq(&self, other: &NormalizedInstrument) -> bool {
        let equals = other.exchange == CexExchange::Binance
            && other.trading_pair == self.symbol.normalize()
            && self.permissions.iter().any(|t| t == &other.trading_type)
            && other.base_asset_symbol == *self.base_asset
            && other.quote_asset_symbol == *self.quote_asset
            && other.active == (&self.status == "TRADING")
            && other.exchange_ranking == self.trade_count as i64
            && other.exchange_ranking_kind == "trade count".to_string()
            && other.futures_expiry == None;

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BinanceAllTradingDayTickers {
    pub symbols: Vec<BinanceTradingDayTicker>
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BinanceTradingDayTicker {
    pub symbol:               BinanceTradingPair,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "priceChange")]
    pub price_change:         f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "priceChangePercent")]
    pub price_change_percent: f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "weightedAvgPrice")]
    pub weighted_avg_price:   f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "openPrice")]
    pub open_price:           f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "highPrice")]
    pub high_price:           f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "lowPrice")]
    pub low_price:            f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "lastPrice")]
    pub last_price:           f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "volume")]
    pub base_volume:          f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "quoteVolume")]
    pub quote_volume:         f64,
    #[serde(rename = "openTime")]
    pub open_time:            u64,
    #[serde(rename = "closeTime")]
    pub close_time:           u64,
    #[serde_as(as = "DefaultOnError")]
    #[serde(rename = "firstId")]
    pub first_id:             Option<u64>,
    #[serde_as(as = "DefaultOnError")]
    #[serde(rename = "lastId")]
    pub last_id:              Option<u64>,
    pub count:                u64
}

impl BinanceTradingDayTicker {
    pub fn build_url_extension_from_symbols(instruments: &[BinanceInstrument]) -> Vec<String> {
        let all_symbols = instruments
            .iter()
            .map(|instr| instr.symbol.0.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        all_symbols
            .chunks(50)
            .map(|chk| format!("%5B%22{}%22%5D", chk.join("%22,%22")))
            .collect()
    }
}
