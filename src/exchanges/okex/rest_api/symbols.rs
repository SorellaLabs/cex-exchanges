use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;

use crate::{
    exchanges::normalized::types::NormalizedCurrency,
    normalized::{rest_api::NormalizedRestApiDataTypes, types::Blockchain},
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct OkexAllSymbolsResponse {
    pub currencies: Vec<OkexAllSymbolsProperties>
}

impl OkexAllSymbolsResponse {
    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        self.currencies
            .into_iter()
            .map(OkexAllSymbolsProperties::normalize)
            .collect()
    }
}

impl<'de> Deserialize<'de> for OkexAllSymbolsResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let val = Value::deserialize(deserializer)?;

        let data = val
            .get("data")
            .ok_or(eyre::ErrReport::msg("Could not find 'data' field in Okex symbols with addresses request".to_string()))
            .map_err(serde::de::Error::custom)?
            .get("body")
            .ok_or(eyre::ErrReport::msg("Could not find 'body' field in Okex symbols with addresses request".to_string()))
            .map_err(serde::de::Error::custom)?
            .get("data")
            .ok_or(eyre::ErrReport::msg("Could not find nested 'data' field in Okex symbols with addresses request".to_string()))
            .map_err(serde::de::Error::custom)?
            .as_array()
            .ok_or(eyre::ErrReport::msg("Could not convert nested 'data' field in Okex symbols with addresses request to array".to_string()))
            .map_err(serde::de::Error::custom)?;

        let currencies = data
            .iter()
            .map(|v| serde_json::from_value(v.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::de::Error::custom)?;

        Ok(OkexAllSymbolsResponse { currencies })
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for OkexAllSymbolsResponse {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllCurrencies(other_currs) => {
                let mut this_currencies = self.currencies.clone();
                this_currencies.sort_by(|a, b| a.symbol.partial_cmp(&b.symbol).unwrap());

                let mut others_currencies = other_currs.clone();
                others_currencies.sort_by(|a, b| a.symbol.partial_cmp(&b.symbol).unwrap());

                this_currencies == *others_currencies
            }
            _ => false
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexAllSymbolsProperties {
    pub symbol: String,
    pub circulating_supply: f64,
    pub last_updated: DateTime<Utc>,
    pub total_supply: f64,
    pub tvl_ratio: Option<f64>,
    pub cmc_rank: u64,
    pub self_reported_circulating_supply: Option<f64>,
    pub platform: Option<OkexAllSymbolsPlatform>,
    pub tags: Vec<String>,
    pub date_added: DateTime<Utc>,
    pub quote: OkexAllSymbolsQuote,
    pub num_market_pairs: u64,
    pub infinite_supply: bool,
    pub name: String,
    pub max_supply: Option<f64>,
    pub id: u64,
    pub self_reported_market_cap: Option<f64>,
    pub slug: String
}

impl OkexAllSymbolsProperties {
    pub fn normalize(self) -> NormalizedCurrency {
        NormalizedCurrency {
            exchange:     CexExchange::Okex,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexAllSymbolsQuote {
    #[serde(rename = "USD")]
    pub usd: OkexAllSymbolsQuoteUSD
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexAllSymbolsQuoteUSD {
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct OkexAllSymbolsPlatform {
    pub symbol:        String,
    pub name:          String,
    pub token_address: String,
    pub id:            u64,
    pub slug:          String
}

impl OkexAllSymbolsPlatform {
    pub fn parse_blockchain_address(self) -> (Blockchain, Option<String>) {
        (self.name.parse().unwrap(), Some(self.token_address))
    }
}

impl PartialEq<NormalizedCurrency> for OkexAllSymbolsProperties {
    fn eq(&self, other: &NormalizedCurrency) -> bool {
        let equals = other.exchange == CexExchange::Okex
            && other.symbol == self.symbol
            && other.name == self.name
            && other.display_name.is_none()
            && other.status == format!("last updated: {:?}", self.last_updated)
            && other.blockchains
                == self
                    .platform
                    .as_ref()
                    .map(|v| vec![v.clone().parse_blockchain_address()])
                    .unwrap_or_default();

        if !equals {
            println!("SELF: {:?}", self);
            println!("NORMALIZED: {:?}", other);
        }

        equals
    }
}
