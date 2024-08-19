use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
pub struct NormalizedTradeFee {
    pub symbol: String,
    pub maker_fee: f64,
    pub taker_fee: f64,
}