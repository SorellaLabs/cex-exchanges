use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum NormalizedRestApiRequest {
    AllCurrencies,
    AllInstruments
}
