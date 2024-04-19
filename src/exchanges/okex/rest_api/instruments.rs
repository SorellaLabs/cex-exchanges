use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};

use crate::{
    exchanges::normalized::types::NormalizedInstrument,
    normalized::{
        rest_api::NormalizedRestApiDataTypes,
        types::{NormalizedTradingPair, NormalizedTradingType}
    },
    okex::OkexTradingPair,
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexCompleteAllInstruments {
    pub instruments: Vec<OkexCompleteInstrument>
}

impl OkexCompleteAllInstruments {
    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.instruments
            .into_iter()
            .map(OkexCompleteInstrument::normalize)
            .collect()
    }
}

impl From<(OkexAllTickersResponse, OkexAllInstrumentsResponse)> for OkexCompleteAllInstruments {
    fn from(value: (OkexAllTickersResponse, OkexAllInstrumentsResponse)) -> Self {
        let (tickers, instruments) = (value.0.into_pair_map(), value.1.into_pair_map());

        let completed_instruments = instruments
            .into_iter()
            .filter_map(|(pair, instr)| {
                let mut get_currency = instr.instrument.clone();

                if instr.instrument_type == NormalizedTradingType::Perpetual || instr.instrument_type == NormalizedTradingType::Futures {
                    let (settlement, contract) = (instr.settlement_currency.as_ref().unwrap(), instr.contract_currency.as_ref().unwrap());
                    get_currency.0 = get_currency.0.replace(settlement, "USD");
                    get_currency.0 = get_currency.0.replace(contract, settlement);
                } else {
                    let (base, quote) = (instr.base_currency.as_ref().unwrap(), instr.quote_currency.as_ref().unwrap());
                    get_currency.0 = get_currency.0.replace(quote, "USD");
                    get_currency.0 = get_currency.0.replace(base, quote);
                }

                let pair_ticker = tickers.get(&pair);

                let usd_price = if pair.quote() != Some(&"USD".to_string()) && pair.quote() != Some(&"USDC".to_string()) {
                    tickers
                        .get(&get_currency.normalize())
                        .map(|v| if let (Some(hp), Some(lp)) = (v.high_price_24h, v.low_price_24h) { Some((hp + lp) / 2.0) } else { None })
                        .flatten()
                } else {
                    None
                };

                if let Some(ticker) = pair_ticker {
                    if let (Some(hp), Some(lp)) = (ticker.high_price_24h, ticker.low_price_24h) {
                        let rank = if let Some(p) = usd_price {
                            ticker.vol_contract_24hr * (1.0 / (p * ((hp + lp) / 2.0)))
                        } else {
                            ticker.vol_currency_24hr * ((hp + lp) / 2.0)
                        };
                        Some(OkexCompleteInstrument { instrument: instr, reverse_usd_vol_24hr: rank.round() as i64 * -1 })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        OkexCompleteAllInstruments { instruments: completed_instruments }
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for OkexCompleteAllInstruments {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllInstruments(other_instrs) => {
                let mut this_instruments = self.instruments.clone();
                this_instruments.sort_by(|a, b| {
                    (
                        &a.instrument
                            .base_currency
                            .as_ref()
                            .unwrap_or_else(|| a.instrument.contract_currency.as_ref().unwrap()),
                        &a.instrument
                            .quote_currency
                            .as_ref()
                            .unwrap_or_else(|| a.instrument.settlement_currency.as_ref().unwrap())
                    )
                        .partial_cmp(&(
                            &b.instrument
                                .base_currency
                                .as_ref()
                                .unwrap_or_else(|| b.instrument.contract_currency.as_ref().unwrap()),
                            &b.instrument
                                .quote_currency
                                .as_ref()
                                .unwrap_or_else(|| b.instrument.settlement_currency.as_ref().unwrap())
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
pub struct OkexCompleteInstrument {
    pub instrument:           OkexInstrument,
    pub reverse_usd_vol_24hr: i64
}

impl OkexCompleteInstrument {
    pub fn normalize(self) -> NormalizedInstrument {
        NormalizedInstrument {
            exchange:              CexExchange::Okex,
            trading_pair:          self.instrument.instrument.normalize(),
            trading_type:          self.instrument.instrument_type,
            base_asset_symbol:     self
                .instrument
                .base_currency
                .unwrap_or_else(|| self.instrument.contract_currency.unwrap()),
            quote_asset_symbol:    self
                .instrument
                .quote_currency
                .unwrap_or_else(|| self.instrument.settlement_currency.unwrap()),
            active:                &self.instrument.state == "live",
            exchange_ranking:      self.reverse_usd_vol_24hr,
            exchange_ranking_kind: "24vol (base currency) * avg 24hr price (usdc)".to_string(),
            futures_expiry:        self
                .instrument
                .expiry_time
                .clone()
                .map(|t| Utc.timestamp_millis_opt(t as i64).unwrap().date_naive())
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexAllInstrumentsResponse {
    #[serde(rename = "data")]
    pub instruments: Vec<OkexInstrument>
}

impl OkexAllInstrumentsResponse {
    fn into_pair_map(self) -> HashMap<NormalizedTradingPair, OkexInstrument> {
        self.instruments
            .into_iter()
            .map(|instr| (instr.instrument.normalize(), instr))
            .collect()
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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexAllTickersResponse {
    #[serde(rename = "data")]
    pub tickers: Vec<OkexTickersRestApi>
}

impl OkexAllTickersResponse {
    fn into_pair_map(self) -> HashMap<NormalizedTradingPair, OkexTickersRestApi> {
        self.tickers
            .into_iter()
            .map(|ticker| (ticker.pair.clone().normalize(), ticker))
            .collect()
    }
}

impl PartialEq<NormalizedInstrument> for OkexCompleteInstrument {
    fn eq(&self, other: &NormalizedInstrument) -> bool {
        let equals = other.exchange == CexExchange::Okex
            && other.trading_pair == self.instrument.instrument.normalize()
            && other.trading_type == self.instrument.instrument_type
            && other.base_asset_symbol
                == *self
                    .instrument
                    .base_currency
                    .as_ref()
                    .unwrap_or_else(|| &self.instrument.contract_currency.as_ref().unwrap())
            && other.quote_asset_symbol
                == *self
                    .instrument
                    .quote_currency
                    .as_ref()
                    .unwrap_or_else(|| &self.instrument.settlement_currency.as_ref().unwrap())
            && other.active == (&self.instrument.state == "live")
            && other.exchange_ranking == self.reverse_usd_vol_24hr;

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct OkexTickersRestApi {
    /// SWAP, PERP, OPTION, ..
    #[serde(rename = "instType")]
    pub pair_type:         String,
    #[serde(rename = "instId")]
    pub pair:              OkexTradingPair,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "last")]
    pub last_price:        Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "lastSz")]
    pub last_size:         f64,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "askPx")]
    pub ask_price:         Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "askSz")]
    pub ask_amt:           f64,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "bidPx")]
    pub bid_price:         Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "bidSz")]
    pub bid_amt:           f64,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "open24h")]
    pub open_price_24hr:   Option<f64>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "high24h")]
    pub high_price_24h:    Option<f64>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "low24h")]
    pub low_price_24h:     Option<f64>,
    /// 24h trading volume, with a unit of currency.
    /// If it is a derivatives contract, the value is the number of base
    /// currency. If it is SPOT/MARGIN, the value is the quantity in quote
    /// currency.
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "volCcy24h")]
    pub vol_currency_24hr: f64,
    /// 24h trading volume, with a unit of contract.
    /// If it is a derivatives contract, the value is the number of contracts.
    /// If it is SPOT/MARGIN, the value is the quantity in base currency.
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "vol24h")]
    pub vol_contract_24hr: f64,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "sodUtc0")]
    pub open_price_utc0:   Option<f64>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "sodUtc8")]
    pub open_price_utc8:   Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "ts")]
    pub timestamp:         u64
}
