use serde::{Deserialize, Serialize};

use crate::normalized::{rest_api::NormalizedRestApiDataTypes, types::NormalizedTradeFee};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BinanceTradeFee {
    pub symbol: String,
    #[serde(rename = "makerCommission")]
    pub maker_fee: f64,
    #[serde(rename = "takerCommission")]
    pub taker_fee: f64,
}

impl BinanceTradeFee {
    pub fn normalize(self) -> NormalizedTradeFee {
        NormalizedTradeFee {
            symbol: self.symbol,
            maker_fee: self.maker_fee,
            taker_fee: self.taker_fee,
        }
    }
}

impl PartialEq<NormalizedTradeFee> for BinanceTradeFee {
    fn eq(&self, other: &NormalizedTradeFee) -> bool {
        self.symbol == other.symbol && self.maker_fee == other.maker_fee && self.taker_fee == other.taker_fee
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BinanceTradeFees(pub Vec<BinanceTradeFee>);

impl BinanceTradeFees {
    pub fn normalize(self) -> Vec<NormalizedTradeFee> {
        self.0
            .into_iter()
            .map(BinanceTradeFee::normalize)
            .collect()
    }
}

impl PartialEq<NormalizedRestApiDataTypes> for BinanceTradeFees {
    fn eq(&self, other: &NormalizedRestApiDataTypes) -> bool {
        match other {
            NormalizedRestApiDataTypes::TradeFees(trade_fees) => {
                let mut this_trade_fees = self
                    .0
                    .clone();
                this_trade_fees.sort_by_key(|tf| tf.symbol.clone());

                let mut other_trade_fees = trade_fees.clone();
                other_trade_fees.sort_by_key(|tf| tf.symbol.clone());

                this_trade_fees
                    .into_iter()
                    .zip(other_trade_fees.into_iter())
                    .all(|(lhs, rhs)| lhs == rhs)
            }
            _ => false
        }
    }
}
