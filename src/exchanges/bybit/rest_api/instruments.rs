use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};

use crate::{
    bybit::{BybitTradingPair, BybitTradingType},
    exchanges::normalized::types::NormalizedInstrument,
    normalized::{rest_api::NormalizedRestApiDataTypes, types::NormalizedTradingType},
    CexExchange
};

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct BybitAllIntruments {
    pub instruments: Vec<BybitIntrument>
}
impl BybitAllIntruments {
    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.instruments
            .into_iter()
            .map(|e| e.normalize())
            .collect()
    }
}

impl<'de> Deserialize<'de> for BybitAllIntruments {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let val = Value::deserialize(deserializer)?;

        let result = val
            .get("result")
            .ok_or(eyre::ErrReport::msg(format!("could not get field 'result' for BybitAllIntruments in {:?}", val)))
            .map_err(serde::de::Error::custom)?;

        let util: BybitAllIntrumentsUtil = serde_json::from_value(result.clone()).map_err(serde::de::Error::custom)?;

        Ok(util.into())
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BybitAllIntruments {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllInstruments(other_instrs) => {
                let this_symbols = self
                    .instruments
                    .iter()
                    .map(|instr| (instr.inner.base_currency.clone(), instr.inner.quote_currency.clone(), instr.inner.symbol.normalize()))
                    .collect::<HashSet<_>>();

                let others_symbols = other_instrs
                    .iter()
                    .map(|instr| (instr.base_asset_symbol.clone(), instr.quote_asset_symbol.clone(), instr.trading_pair.clone()))
                    .collect::<HashSet<_>>();

                others_symbols
                    .into_iter()
                    .all(|instr| this_symbols.contains(&instr))
            }
            _ => false
        }
    }
}

impl From<BybitAllIntrumentsUtil> for BybitAllIntruments {
    fn from(value: BybitAllIntrumentsUtil) -> Self {
        let instruments = value
            .list
            .into_iter()
            .map(|instr| BybitIntrument::from_inner_with_tt(instr, value.category))
            .collect();

        BybitAllIntruments { instruments }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct BybitAllIntrumentsUtil {
    category: BybitTradingType,
    list:     Vec<BybitIntrumentInner>
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BybitIntrument {
    pub trading_type: BybitTradingType,
    pub inner:        BybitIntrumentInner
}

impl BybitIntrument {
    pub fn normalize(self) -> NormalizedInstrument {
        self.inner.normalize(self.trading_type.into())
    }

    fn from_inner_with_tt(inner: BybitIntrumentInner, tt: BybitTradingType) -> Self {
        Self { trading_type: tt, inner }
    }
}

impl PartialEq<NormalizedInstrument> for BybitIntrument {
    fn eq(&self, other: &NormalizedInstrument) -> bool {
        &self.inner == other
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BybitIntrumentInner {
    pub symbol:               BybitTradingPair,
    #[serde(rename = "baseCoin")]
    pub base_currency:        String,
    #[serde(rename = "quoteCoin")]
    pub quote_currency:       String,
    pub status:               String,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub innovation:           Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "launchTime")]
    pub launch_time:          Option<u64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "deliveryTime")]
    pub delivery_time:        Option<u64>,
    #[serde_as(as = "Option<NoneAsEmptyString>")]
    #[serde(rename = "deliveryFeeRate")]
    pub delivery_fee_rate:    Option<Option<f64>>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "priceScale")]
    pub price_scale:          Option<u64>,
    #[serde(rename = "unifiedMarginTrade")]
    pub unified_margin_trade: Option<bool>,
    #[serde(rename = "fundingInterval")]
    pub funding_interval:     Option<u64>,
    #[serde(rename = "settleCoin")]
    pub settle_coin:          Option<String>,
    #[serde(rename = "copyTrading")]
    pub copy_trading:         Option<String>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "upperFundingRate")]
    pub upper_funding_rate:   Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "lowerFundingRate")]
    pub lower_funding_rate:   Option<f64>,
    #[serde(rename = "marginTrading")]
    pub margin_trading:       Option<String>,
    #[serde(rename = "optionsType")]
    pub options_type:         Option<String>,
    #[serde(rename = "lotSizeFilter")]
    pub lot_size_filter:      BybitInstrumentLotSizeFilter,
    #[serde(rename = "leverageFilter")]
    pub leverage_filter:      Option<BybitInstrumentLeverageFilter>,
    #[serde(rename = "priceFilter")]
    pub price_filter:         BybitInstrumentPriceFilter,
    #[serde(rename = "riskParameters")]
    pub risk_parameters:      Option<BybitInstrumentRiskParameters>
}

impl BybitIntrumentInner {
    pub fn normalize(self, trading_type: NormalizedTradingType) -> NormalizedInstrument {
        let futures_expiry = if matches!(trading_type, NormalizedTradingType::Option) {
            let ds = self.symbol.0.clone();
            let date_str = ds.split('-').nth(1).unwrap();
            let date_chars = date_str.chars();
            let mut day = "".to_string();
            let mut month = "".to_string();
            let mut year = "".to_string();
            for ch in date_chars {
                if ch.is_numeric() {
                    if month.is_empty() {
                        day.push(ch);
                    } else {
                        year.push(ch);
                    }
                }

                if ch.is_alphabetic() {
                    month.push(ch)
                }
            }

            let month_digit = parse_month_digit(month);
            let date = NaiveDate::from_ymd_opt(year.parse().unwrap(), month_digit, day.parse().unwrap()).unwrap();
            Some(date)
        } else {
            None
        };
        NormalizedInstrument {
            exchange: CexExchange::Bybit,
            trading_pair: self
                .symbol
                .normalize_with(&self.base_currency, &self.quote_currency),
            trading_type,
            base_asset_symbol: self.base_currency.clone(),
            quote_asset_symbol: self.quote_currency.clone(),
            active: &self.status == "Trading",
            futures_expiry
        }
    }
}

impl PartialEq<NormalizedInstrument> for BybitIntrumentInner {
    fn eq(&self, other: &NormalizedInstrument) -> bool {
        let equals = other.exchange == CexExchange::Bybit
            && other.trading_pair == self.symbol.normalize()
            && other.trading_type == NormalizedTradingType::Spot
            && other.base_asset_symbol == *self.base_currency
            && other.quote_asset_symbol == *self.quote_currency
            && other.active == (&self.status == "Trading")
            && other.futures_expiry.is_none();

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BybitInstrumentLotSizeFilter {
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "basePrecision")]
    pub base_precision:             Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "maxOrderQty")]
    pub max_order_amount:           Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "minOrderQty")]
    pub min_order_amount:           Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "qtyStep")]
    pub amount_step:                Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "postOnlyMaxOrderQty")]
    pub post_only_max_order_amount: Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "maxMktOrderQty")]
    pub max_market_order_amount:    Option<f64>,
    #[serde_as(as = "Option<NoneAsEmptyString>")]
    #[serde(rename = "minNotionalValue")]
    pub min_notional_value:         Option<Option<f64>>
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BybitInstrumentPriceFilter {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "tickSize")]
    pub tick_size: f64,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "maxPrice")]
    pub max_price: Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "minPrice")]
    pub min_price: Option<f64>
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BybitInstrumentRiskParameters {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "limitParameter")]
    pub limit_parameter:  f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "marketParameter")]
    pub market_parameter: f64
}

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct BybitInstrumentLeverageFilter {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "minLeverage")]
    pub min_leverage: f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxLeverage")]
    pub max_leverage: f64,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "tickSize")]
    pub tick_size:    Option<f64>
}

fn parse_month_digit(month: String) -> u32 {
    match month.as_str() {
        "JAN" => 1,
        "FEB" => 2,
        "MAR" => 3,
        "APR" => 4,
        "MAY" => 5,
        "JUN" => 6,
        "JUL" => 7,
        "AUG" => 8,
        "SEP" => 9,
        "OCT" => 10,
        "NOV" => 11,
        "DEC" => 12,
        _ => unreachable!("invalid month")
    }
}
