mod utils;
use utils::stream_util;

#[cfg(test)]
mod coinbase_tests {
    use cex_exchanges::{exchanges::coinbase::CoinbaseWsBuilder, types::coinbase::channels::CoinbaseChannel};
    use serial_test::serial;

    use crate::stream_util;

    async fn coinbase_util(builder: CoinbaseWsBuilder, iterations: usize) {
        stream_util(builder.build(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_status() {
        let builder = CoinbaseWsBuilder::default().add_channel(CoinbaseChannel::Status);
        coinbase_util(builder, 1).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_rfq_matches() {
        let builder = CoinbaseWsBuilder::default()
            .add_channel(CoinbaseChannel::new_rfq_matches_from_raw(vec!["ETH_USD".to_string(), "btc_usd".to_string()], '_'));
        coinbase_util(builder, 5).await;
    }
}
