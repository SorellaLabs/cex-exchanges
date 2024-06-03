use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};

use crate::{
    normalized::{
        rest_api::NormalizedRestApiDataTypes,
        types::{Blockchain, NormalizedCurrency}
    },
    CexExchange
};

#[serde_as]
#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct BybitAllCoins {
    pub coins: Vec<BybitCoin>
}

impl BybitAllCoins {
    pub fn normalize(self) -> Vec<NormalizedCurrency> {
        self.coins.into_iter().map(BybitCoin::normalize).collect()
    }
}

impl<'de> Deserialize<'de> for BybitAllCoins {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>
    {
        let val = Value::deserialize(deserializer)?;

        let result = val
            .get("result")
            .ok_or(eyre::ErrReport::msg(format!("could not get field 'result' for BybitAllCoins in {:?}", val)))
            .map_err(serde::de::Error::custom)?;

        let rows = result
            .get("rows")
            .ok_or(eyre::ErrReport::msg(format!("could not get field 'coins' for BybitAllCoins in {:?}", val)))
            .map_err(serde::de::Error::custom)?;

        let coins = serde_json::from_value(rows.clone()).map_err(serde::de::Error::custom)?;

        Ok(BybitAllCoins { coins })
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BybitAllCoins {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::AllCurrencies(other_currs) => {
                let mut this_currencies = self.coins.clone();
                this_currencies.sort_by(|a, b| a.coin.partial_cmp(&b.coin).unwrap());

                let mut others_currencies = other_currs.clone();
                others_currencies.sort_by(|a, b| a.symbol.partial_cmp(&b.symbol).unwrap());

                this_currencies == others_currencies
            }
            _ => false
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BybitCoin {
    pub name:          String,
    pub coin:          String,
    #[serde(rename = "remainAmount")]
    pub remain_amount: u64,
    pub chains:        Vec<BybitCoinChain>
}

impl BybitCoin {
    pub fn normalize(self) -> NormalizedCurrency {
        NormalizedCurrency {
            exchange:     CexExchange::Bybit,
            symbol:       self.coin,
            name:         self.name,
            display_name: None,
            status:       self.remain_amount.to_string(),
            blockchains:  self
                .chains
                .into_iter()
                .map(|c| c.parse_blockchain_address())
                .collect()
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct BybitCoinChain {
    #[serde(rename = "chainType")]
    pub chain_type:              String,
    pub confirmation:            u64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "withdrawFee")]
    pub withdraw_fee:            f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "depositMin")]
    pub deposit_min:             f64,
    #[serde(rename = "withdrawMin")]
    pub withdraw_min:            f64,
    pub chain:                   String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "chainDeposit")]
    pub chain_deposit:           f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "chainWithdraw")]
    pub chain_withdraw:          f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "minAccuracy")]
    pub min_accuracy:            f64,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "withdrawPercentageFee")]
    pub withdraw_percentage_fee: f64
}

impl BybitCoinChain {
    pub fn parse_blockchain_address(self) -> (Blockchain, Option<String>) {
        (self.chain.parse().unwrap(), None)
    }
}

impl PartialEq<NormalizedCurrency> for BybitCoin {
    fn eq(&self, other: &NormalizedCurrency) -> bool {
        let equals = other.exchange == CexExchange::Bybit
            && other.symbol == self.coin
            && other.name == self.name
            && other.display_name.is_none()
            && other.status == self.remain_amount.to_string()
            && self.chains.iter().all(|c| {
                other
                    .blockchains
                    .contains(&c.clone().parse_blockchain_address())
            });

        if !equals {
            println!("\n\nSELF: {:?}\n", self);
            println!("NORMALIZED: {:?}\n\n", other);
        }

        equals
    }
}
