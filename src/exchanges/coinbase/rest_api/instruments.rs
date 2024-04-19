use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};

use super::CoinbaseCurrency;
use crate::{
    coinbase::CoinbaseTradingPair,
    exchanges::normalized::types::NormalizedInstrument,
    normalized::{rest_api::NormalizedRestApiDataTypes, types::NormalizedTradingType},
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct CoinbaseAllInstruments {
    pub instruments: Vec<CoinbaseCompleteInstrument>
}

impl CoinbaseAllInstruments {
    pub(crate) fn new(instruments: Vec<CoinbaseInstrument>, currencies: Vec<CoinbaseCurrency>) -> Self {
        let product_map = currencies
            .into_iter()
            .map(|p| (p.id.clone(), p))
            .collect::<HashMap<_, _>>();

        let complete = instruments
            .into_iter()
            .filter_map(|instr| {
                product_map
                    .get(&instr.base_currency)
                    .map(|prod| CoinbaseCompleteInstrument {
                        id: instr.id.clone(),
                        trading_type: NormalizedTradingType::Spot,
                        base_currency: instr.base_currency.clone(),
                        quote_currency: instr.quote_currency.clone(),
                        quote_increment: instr.quote_increment.clone(),
                        base_increment: instr.base_increment.clone(),
                        display_name: instr.display_name.clone(),
                        min_market_funds: instr.min_market_funds.clone(),
                        margin_enabled: instr.margin_enabled.clone(),
                        post_only: instr.post_only.clone(),
                        limit_only: instr.limit_only.clone(),
                        cancel_only: instr.cancel_only.clone(),
                        status: instr.status.clone(),
                        status_message: instr.status_message.clone(),
                        trading_disabled: instr.trading_disabled.clone(),
                        fx_stablecoin: instr.fx_stablecoin.clone(),
                        max_slippage_percentage: instr.max_slippage_percentage.clone(),
                        auction_mode: instr.auction_mode.clone(),
                        high_bid_limit_percentage: instr.high_bid_limit_percentage.clone(),
                        sort_order: prod.details.sort_order.unwrap_or(1000) as f64
                    })
            })
            .collect();

        Self { instruments: complete }
    }

    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.instruments
            .into_iter()
            .map(CoinbaseCompleteInstrument::normalize)
            .collect()
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for CoinbaseAllInstruments {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct CoinbaseCompleteInstrument {
    pub id: CoinbaseTradingPair,
    pub trading_type: NormalizedTradingType,
    pub base_currency: String,
    pub quote_currency: String,
    pub quote_increment: f64,
    pub base_increment: f64,
    pub display_name: String,
    pub min_market_funds: f64,
    pub margin_enabled: bool,
    pub post_only: bool,
    pub limit_only: bool,
    pub cancel_only: bool,
    pub status: String,
    pub status_message: Option<String>,
    pub trading_disabled: bool,
    pub fx_stablecoin: bool,
    pub max_slippage_percentage: f64,
    pub auction_mode: bool,
    pub high_bid_limit_percentage: Option<f64>,
    pub sort_order: f64
}

impl CoinbaseCompleteInstrument {
    pub fn normalize(self) -> NormalizedInstrument {
        NormalizedInstrument {
            exchange:              CexExchange::Okex,
            trading_pair:          self.id.normalize(),
            trading_type:          self.trading_type,
            base_asset_symbol:     self.base_currency,
            quote_asset_symbol:    self.quote_currency,
            active:                (&self.status == "online"),
            exchange_ranking:      self.sort_order.round() as i64 * -1, // reverse
            exchange_ranking_kind: "reverse sort order".to_string(),
            futures_expiry:        None
        }
    }
}

impl PartialEq<NormalizedInstrument> for CoinbaseCompleteInstrument {
    fn eq(&self, other: &NormalizedInstrument) -> bool {
        let equals = other.exchange == CexExchange::Coinbase
            && other.trading_pair == self.id.normalize()
            && other.trading_type == self.trading_type
            && other.base_asset_symbol == *self.base_currency
            && other.quote_asset_symbol == *self.quote_currency
            && other.active == (&self.status == "online")
            && other.exchange_ranking == self.sort_order.round() as i64 * -1;

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct CoinbaseInstrument {
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
