pub mod rfq_match;
pub mod status;

#[serde_with::serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum CoinbaseWsMessage {
    RfqMatch(rfq_match::CoinbaseRfqMatchesMessage),
    Status(status::CoinbaseStatusMessage),
    Error(String),
    Subscriptions(serde_json::Value),
}
