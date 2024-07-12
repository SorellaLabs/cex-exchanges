use serde::{Deserialize, Serialize};

use super::{BinanceAllInstruments, BinanceAllSymbols, BinanceInstrument, BinanceSymbol, BinanceTradeFees};
use crate::exchanges::normalized::rest_api::NormalizedRestApiDataTypes;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum BinanceRestApiResponse {
    Symbols(BinanceAllSymbols),
    Instruments(BinanceAllInstruments),
    TradeFees(BinanceTradeFees),
}

impl BinanceRestApiResponse {
    pub fn normalize(self) -> NormalizedRestApiDataTypes {
        match self {
            BinanceRestApiResponse::Symbols(v) => NormalizedRestApiDataTypes::AllCurrencies(v.normalize()),
            BinanceRestApiResponse::Instruments(v) => NormalizedRestApiDataTypes::AllInstruments(v.normalize()),
            BinanceRestApiResponse::TradeFees(v) => NormalizedRestApiDataTypes::TradeFees(v.normalize()),
        }
    }

    pub fn take_symbols(self) -> Option<Vec<BinanceSymbol>> {
        match self {
            BinanceRestApiResponse::Symbols(val) => Some(val.symbols),
            _ => None
        }
    }

    pub fn take_instruments(self, active_only: bool) -> Option<Vec<BinanceInstrument>> {
        let instruments = match self {
            BinanceRestApiResponse::Instruments(val) => val.instruments,
            _ => return None
        };

        if active_only {
            Some(
                instruments
                    .into_iter()
                    .filter(|instr| instr.status.to_uppercase() == "TRADING")
                    .collect::<Vec<_>>()
            )
        } else {
            Some(instruments)
        }
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BinanceRestApiResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match self {
            BinanceRestApiResponse::Symbols(vals) => vals == other,
            BinanceRestApiResponse::Instruments(vals) => vals == other,
            BinanceRestApiResponse::TradeFees(vals) => vals == other,
        }
    }
}
