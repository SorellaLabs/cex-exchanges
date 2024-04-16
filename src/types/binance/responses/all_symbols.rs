use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;

use crate::{
    exchanges::CexExchange,
    types::{blockchain::Blockchain, normalized::currencies::NormalizedCurrency}
};

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct BinanceAllSymbolsResponse {
    pub currencies: Vec<BinanceAllSymbolsProperties>
}

impl BinanceAllSymbolsResponse {
    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        self.currencies
            .into_iter()
            .map(BinanceAllSymbolsProperties::normalize)
            .collect()
    }
}

impl<'de> Deserialize<'de> for BinanceAllSymbolsResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let val = Value::deserialize(deserializer)?;

        let data = val
            .get("data")
            .ok_or(eyre::ErrReport::msg("Could not find 'data' field in Binance symbols with addresses request".to_string()))
            .map_err(serde::de::Error::custom)?
            .get("body")
            .ok_or(eyre::ErrReport::msg("Could not find 'body' field in Binance symbols with addresses request".to_string()))
            .map_err(serde::de::Error::custom)?
            .get("data")
            .ok_or(eyre::ErrReport::msg("Could not find nested 'data' field in Binance symbols with addresses request".to_string()))
            .map_err(serde::de::Error::custom)?
            .as_array()
            .ok_or(eyre::ErrReport::msg("Could not convert nested 'data' field in Binance symbols with addresses request to array".to_string()))
            .map_err(serde::de::Error::custom)?;

        let currencies = data
            .iter()
            .map(|v| serde_json::from_value(v.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::de::Error::custom)?;

        Ok(BinanceAllSymbolsResponse { currencies })
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceAllSymbolsProperties {
    pub symbol: String,
    pub circulating_supply: f64,
    pub last_updated: DateTime<Utc>,
    pub total_supply: f64,
    pub tvl_ratio: Option<f64>,
    pub cmc_rank: u64,
    pub self_reported_circulating_supply: Option<f64>,
    pub platform: Option<BinanceAllSymbolsPlatform>,
    pub tags: Vec<String>,
    pub date_added: DateTime<Utc>,
    pub quote: BinanceAllSymbolsQuote,
    pub num_market_pairs: u64,
    pub infinite_supply: bool,
    pub name: String,
    pub max_supply: Option<f64>,
    pub id: u64,
    pub self_reported_market_cap: Option<f64>,
    pub slug: String
}

impl BinanceAllSymbolsProperties {
    pub fn normalize(self) -> NormalizedCurrency {
        NormalizedCurrency {
            exchange:     CexExchange::Binance,
            symbol:       self.symbol,
            name:         self.name,
            display_name: None,
            status:       format!("last updated: {:?}", self.last_updated),
            blockchains:  self
                .platform
                .map(|v| vec![v.parse_blockchain_address()])
                .unwrap_or_default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceAllSymbolsQuote {
    #[serde(rename = "USD")]
    pub usd: BinanceAllSymbolsQuoteUSD
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceAllSymbolsQuoteUSD {
    pub fully_diluted_market_cap: f64,
    pub last_updated: DateTime<Utc>,
    pub market_cap_dominance: f64,
    pub tvl: Option<f64>,
    pub percent_change_30d: f64,
    pub percent_change_1h: f64,
    pub percent_change_24h: f64,
    pub market_cap: f64,
    pub volume_change_24h: f64,
    pub price: f64,
    pub percent_change_60d: f64,
    pub volume_24h: f64,
    pub percent_change_90d: f64,
    pub percent_change_7d: f64
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceAllSymbolsPlatform {
    pub symbol:        String,
    pub name:          String,
    pub token_address: String,
    pub id:            u64,
    pub slug:          String
}

impl BinanceAllSymbolsPlatform {
    pub fn parse_blockchain_address(self) -> (Blockchain, Option<String>) {
        (self.name.parse().unwrap(), Some(self.token_address))
    }
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for BinanceAllSymbolsResponse {
    fn equals_normalized(self) -> bool {
        self.currencies.into_iter().all(|c| c.equals_normalized())
    }
}

#[cfg(feature = "test-utils")]
impl crate::types::test_utils::NormalizedEquals for BinanceAllSymbolsProperties {
    fn equals_normalized(self) -> bool {
        let normalized = self.clone().normalize();
        let copy = self.clone();

        let equals = normalized.exchange == CexExchange::Binance
            && normalized.symbol == self.symbol
            && normalized.name == self.name
            && normalized.display_name == None
            && normalized.status == format!("last updated: {:?}", self.last_updated)
            && normalized.blockchains
                == self
                    .platform
                    .map(|v| vec![v.parse_blockchain_address()])
                    .unwrap_or_default();

        if !equals {
            println!("SELF: {:?}", copy);
            println!("NORMALIZED: {:?}", normalized);
        }

        equals
    }
}
