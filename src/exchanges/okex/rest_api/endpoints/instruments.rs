use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};

use crate::{
    exchanges::normalized::types::NormalizedInstrument,
    normalized::{rest_api::NormalizedRestApiDataTypes, types::NormalizedTradingType},
    okex::OkexTradingPair,
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexAllInstruments {
    #[serde(rename = "data")]
    pub instruments: Vec<OkexInstrument>
}

impl OkexAllInstruments {
    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.instruments
            .into_iter()
            .map(OkexInstrument::normalize)
            .collect()
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for OkexAllInstruments {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllInstruments(other_instrs) => {
                let mut this_instruments = self.instruments.clone();
                this_instruments.sort_by(|a, b| {
                    (
                        &a.base_currency
                            .as_ref()
                            .unwrap_or_else(|| a.contract_currency.as_ref().unwrap()),
                        &a.quote_currency
                            .as_ref()
                            .unwrap_or_else(|| a.settlement_currency.as_ref().unwrap())
                    )
                        .partial_cmp(&(
                            &b.base_currency
                                .as_ref()
                                .unwrap_or_else(|| b.contract_currency.as_ref().unwrap()),
                            &b.quote_currency
                                .as_ref()
                                .unwrap_or_else(|| b.settlement_currency.as_ref().unwrap())
                        ))
                        .unwrap()
                });

                let mut others_instruments = other_instrs.clone();
                others_instruments.sort_by(|a, b| {
                    (&a.base_asset_symbol, &a.quote_asset_symbol)
                        .partial_cmp(&(&b.base_asset_symbol, &b.quote_asset_symbol))
                        .unwrap()
                });

                this_instruments == *others_instruments
            }
            _ => false
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexInstrument {
    pub alias:               String,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "baseCcy")]
    pub base_currency:       Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "quoteCcy")]
    pub quote_currency:      Option<String>,
    #[serde(rename = "instType")]
    pub instrument_type:     NormalizedTradingType,
    #[serde(rename = "instId")]
    pub instrument:          OkexTradingPair,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "uly")]
    pub underlying:          Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "instFamily")]
    pub instrument_family:   Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "settleCcy")]
    pub settlement_currency: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "ctVal")]
    pub contract_value:      Option<f64>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "ctMult")]
    pub contract_multiplier: Option<u64>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "ctValCcy")]
    pub contract_currency:   Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "optType")]
    pub option_type:         Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "stk")]
    pub strike_price:        Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "listTime")]
    pub listing_time:        u64,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "expTime")]
    pub expiry_time:         Option<u64>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "lever")]
    pub leverage:            Option<u64>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "tickSz")]
    pub tick_size:           Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "lotSz")]
    pub lot_size:            f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "minSz")]
    pub minimum_size:        f64,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "ctType")]
    pub contract_type:       Option<String>,
    #[serde(rename = "state")]
    pub state:               String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxLmtSz")]
    pub max_limit_size:      f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxMktSz")]
    pub max_market_size:     f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxLmtAmt")]
    pub max_limit_amount:    f64,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "maxMktAmt")]
    pub max_market_amount:   Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxTwapSz")]
    pub max_twap_size:       f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxIcebergSz")]
    pub max_iceberg_size:    f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxTriggerSz")]
    pub max_trigger_size:    f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxStopSz")]
    pub max_stop_size:       f64
}

impl OkexInstrument {
    pub fn normalize(self) -> NormalizedInstrument {
        NormalizedInstrument {
            exchange:           CexExchange::Okex,
            trading_pair:       self.instrument.normalize(),
            trading_type:       self.instrument_type,
            base_asset_symbol:  self
                .base_currency
                .unwrap_or_else(|| self.contract_currency.unwrap()),
            quote_asset_symbol: self
                .quote_currency
                .unwrap_or_else(|| self.settlement_currency.unwrap()),
            active:             &self.state == "live",
            futures_expiry:     self
                .expiry_time
                .map(|t| Utc.timestamp_millis_opt(t as i64).unwrap().date_naive())
        }
    }
}

impl PartialEq<NormalizedInstrument> for OkexInstrument {
    fn eq(&self, other: &NormalizedInstrument) -> bool {
        let equals = other.exchange == CexExchange::Okex
            && other.trading_pair == self.instrument.normalize()
            && other.trading_type == self.instrument_type
            && other.base_asset_symbol
                == *self
                    .base_currency
                    .as_ref()
                    .unwrap_or_else(|| self.contract_currency.as_ref().unwrap())
            && other.quote_asset_symbol
                == *self
                    .quote_currency
                    .as_ref()
                    .unwrap_or_else(|| self.settlement_currency.as_ref().unwrap())
            && other.active == (&self.state == "live");

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
