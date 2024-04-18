use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    coinbase::CoinbaseTradingPair, exchanges::normalized::types::NormalizedInstrument, normalized::types::NormalizedTradingType, CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct CoinbaseAllInstrumentsResponse {
    pub instruments: Vec<CoinbaseInstrument>
}

impl CoinbaseAllInstrumentsResponse {
    pub fn normalize(self) -> Vec<NormalizedInstrument> {
        self.instruments
            .into_iter()
            .map(CoinbaseInstrument::normalize)
            .collect()
    }
}

impl<'de> Deserialize<'de> for CoinbaseAllInstrumentsResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let instruments = Vec::<CoinbaseInstrument>::deserialize(deserializer)?;

        Ok(CoinbaseAllInstrumentsResponse { instruments })
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseInstrument {
    #[serde_as(as = "DisplayFromStr")]
    pub instrument_id:          u64,
    pub instrument_uuid:        String,
    pub symbol:                 CoinbaseTradingPair,
    #[serde(rename = "type")]
    pub instrument_type:        NormalizedTradingType,
    pub mode:                   String,
    #[serde_as(as = "DisplayFromStr")]
    pub base_asset_id:          u64,
    pub base_asset_uuid:        String,
    pub base_asset_name:        String,
    #[serde_as(as = "DisplayFromStr")]
    pub quote_asset_id:         u64,
    pub quote_asset_uuid:       String,
    pub quote_asset_name:       String,
    #[serde_as(as = "DisplayFromStr")]
    pub base_increment:         f64,
    #[serde_as(as = "DisplayFromStr")]
    pub quote_increment:        f64,
    pub price_band_percent:     f64,
    pub market_order_percent:   f64,
    #[serde_as(as = "DisplayFromStr")]
    pub qty_24hr:               f64,
    #[serde_as(as = "DisplayFromStr")]
    pub notional_24hr:          f64,
    #[serde_as(as = "DisplayFromStr")]
    pub avg_daily_qty:          f64,
    #[serde_as(as = "DisplayFromStr")]
    pub avg_daily_notional:     f64,
    #[serde_as(as = "DisplayFromStr")]
    pub previous_day_qty:       f64,
    #[serde_as(as = "DisplayFromStr")]
    pub open_interest:          f64,
    #[serde_as(as = "DisplayFromStr")]
    pub position_limit_qty:     f64,
    pub position_limit_adq_pct: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub replacement_cost:       f64,
    pub base_imf:               f64,
    pub default_imf:            Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    pub min_notional_value:     f64,
    #[serde_as(as = "DisplayFromStr")]
    pub funding_interval:       f64,
    #[serde_as(as = "DisplayFromStr")]
    pub trading_state:          String,
    pub quote:                  CoinbaseInstrumentQuote
}

impl CoinbaseInstrument {
    pub fn normalize(self) -> NormalizedInstrument {
        NormalizedInstrument {
            exchange:           CexExchange::Coinbase,
            trading_pair:       self.symbol.normalize(),
            trading_type:       self.instrument_type,
            base_asset_symbol:  self.base_asset_name,
            quote_asset_symbol: self.quote_asset_name,
            active:             &self.trading_state == "TRADING",
            avg_vol_24hr_usdc:  self.notional_24hr
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinbaseInstrumentQuote {
    #[serde_as(as = "DisplayFromStr")]
    pub best_bid_price:    f64,
    #[serde_as(as = "DisplayFromStr")]
    pub best_bid_size:     f64,
    #[serde_as(as = "DisplayFromStr")]
    pub best_ask_price:    f64,
    #[serde_as(as = "DisplayFromStr")]
    pub best_ask_size:     f64,
    #[serde_as(as = "DisplayFromStr")]
    pub trade_price:       f64,
    #[serde_as(as = "DisplayFromStr")]
    pub trade_qty:         f64,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub index_price:       Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    pub mark_price:        f64,
    #[serde_as(as = "DisplayFromStr")]
    pub settlement_price:  f64,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub limit_up:          Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub limit_down:        Option<f64>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub predicted_funding: Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp:         DateTime<Utc>
}

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for CoinbaseAllInstrumentsResponse {
    fn equals_normalized(self) -> bool {
        self.instruments.into_iter().all(|c| c.equals_normalized())
    }
}

#[cfg(feature = "test-utils")]
impl crate::exchanges::test_utils::NormalizedEquals for CoinbaseInstrument {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        let copy = self.clone();

        let equals = normalized.exchange == CexExchange::Coinbase
            && normalized.trading_pair == self.symbol.normalize()
            && normalized.trading_type == self.instrument_type
            && normalized.base_asset_symbol == self.base_asset_name
            && normalized.quote_asset_symbol == self.quote_asset_name
            && normalized.active == (&self.trading_state == "TRADING")
            && normalized.avg_vol_24hr_usdc == self.notional_24hr;

        if !equals {
            println!("SELF: {:?}", copy);
            println!("NORMALIZED: {:?}", normalized);
        }

        equals
    }
}
