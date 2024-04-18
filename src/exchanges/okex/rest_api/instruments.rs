use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, NoneAsEmptyString};

use crate::{
    exchanges::normalized::types::NormalizedInstrument,
    normalized::types::{NormalizedTradingPair, NormalizedTradingType},
    okex::{ws::OkexTickersMessage, OkexTradingPair},
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
                let quote_currency = instr.quote_currency.clone();
                let day_vol_base_amt = tickers.get(&pair).map(|v| v.vol_contract_24hr);
                let day_avg_price_usd = tickers
                    .get(&OkexTradingPair(format!("{quote_currency}-USDC")).normalize())
                    .map(|v| (v.high_price_24h + v.low_price_24h) / 2.0);

                if let (Some(vol), Some(price)) = (day_vol_base_amt, day_avg_price_usd) {
                    Some(OkexCompleteInstrument { instrument: instr, avg_vol_24hr_usdc: vol * price })
                } else {
                    None
                }
            })
            .collect();

        OkexCompleteAllInstruments { instruments: completed_instruments }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexCompleteInstrument {
    pub instrument:        OkexInstrument,
    pub avg_vol_24hr_usdc: f64
}

impl OkexCompleteInstrument {
    pub fn normalize(self) -> NormalizedInstrument {
        NormalizedInstrument {
            exchange:           CexExchange::Okex,
            trading_pair:       self.instrument.instrument.normalize(),
            trading_type:       self.instrument.instrument_type,
            base_asset_symbol:  self.instrument.base_currency,
            quote_asset_symbol: self.instrument.quote_currency,
            active:             &self.instrument.state == "live",
            avg_vol_24hr_usdc:  self.avg_vol_24hr_usdc
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexInstrument {
    pub alias:                   String,
    #[serde(rename = "baseCcy")]
    pub base_currency:           String,
    #[serde(rename = "quoteCcy")]
    pub quote_currency:          String,
    #[serde(rename = "instType")]
    pub instrument_type:         NormalizedTradingType,
    #[serde(rename = "instId")]
    pub instrument:              OkexTradingPair,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "uly")]
    pub underlying:              Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "instFamily")]
    pub instrument_family:       Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "settleCcy")]
    pub settlement_currency:     Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "ctVal")]
    pub contract_value:          Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "ctMult")]
    pub contract_multiplier:     Option<u64>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "ctValCcy")]
    pub contract_value_currency: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "optType")]
    pub option_type:             Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "stk")]
    pub strike_price:            Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "listTime")]
    pub listing_time:            u64,
    #[serde(rename = "expTime")]
    pub expiry_time:             Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "lever")]
    pub leverage:                u64,
    #[serde_as(as = "NoneAsEmptyString")]
    #[serde(rename = "tickSz")]
    pub tick_size:               Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "lotSz")]
    pub lot_size:                f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "minSz")]
    pub minimum_size:            f64,
    #[serde(rename = "ctType")]
    pub contract_type:           Option<String>,
    #[serde(rename = "state")]
    pub state:                   String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxLmtSz")]
    pub max_limit_size:          f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxMktSz")]
    pub max_market_size:         f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxLmtAmt")]
    pub max_limit_amount:        f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxMktAmt")]
    pub max_market_amount:       f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxTwapSz")]
    pub max_twap_size:           f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxIcebergSz")]
    pub max_iceberg_size:        f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxTriggerSz")]
    pub max_trigger_size:        f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "maxStopSz")]
    pub max_stop_size:           f64
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexAllTickersResponse {
    #[serde(rename = "data")]
    pub tickers: Vec<OkexTickersMessage>
}

impl OkexAllTickersResponse {
    fn into_pair_map(self) -> HashMap<NormalizedTradingPair, OkexTickersMessage> {
        self.tickers
            .into_iter()
            .map(|ticker| (ticker.pair.clone().normalize(), ticker))
            .collect()
    }
}

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for OkexCompleteAllInstruments {
    fn equals_normalized(self) -> bool {
        self.instruments.into_iter().all(|c| c.equals_normalized())
    }
}

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for OkexCompleteInstrument {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        let copy = self.clone();

        let equals = normalized.exchange == CexExchange::Okex
            && normalized.trading_pair == self.instrument.instrument.normalize()
            && normalized.trading_type == self.instrument.instrument_type
            && normalized.base_asset_symbol == self.instrument.base_currency
            && normalized.quote_asset_symbol == self.instrument.quote_currency
            && normalized.active == (&self.instrument.state == "live")
            && normalized.avg_vol_24hr_usdc == self.avg_vol_24hr_usdc;

        if !equals {
            println!("SELF: {:?}", copy);
            println!("NORMALIZED: {:?}", normalized);
        }

        equals
    }
}
