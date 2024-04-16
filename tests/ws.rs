mod utils;
use utils::*;

#[cfg(test)]
mod coinbase_tests {
    use cex_exchanges::{exchanges::coinbase::CoinbaseWsBuilder, types::coinbase::channels::CoinbaseChannel};
    use serial_test::serial;

    use crate::{mutlistream_util, stream_util};

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
    async fn test_match() {
        let builder =
            CoinbaseWsBuilder::default().add_channel(CoinbaseChannel::new_match_from_pair(vec!["ETH_USD".to_string(), "btc_usd".to_string()], '_'));
        coinbase_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_ticker() {
        let builder =
            CoinbaseWsBuilder::default().add_channel(CoinbaseChannel::new_ticker_from_pair(vec!["ETH_USD".to_string(), "btc_usd".to_string()], '_'));
        coinbase_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi() {
        let builder = CoinbaseWsBuilder::default()
            .add_channel(CoinbaseChannel::new_ticker_from_pair(vec!["ETH_USD".to_string(), "btc_usd".to_string()], '_'))
            .add_channel(CoinbaseChannel::new_match_from_pair(vec!["ETH_USD".to_string(), "btc_usd".to_string()], '_'))
            .add_channel(CoinbaseChannel::Status)
            .set_channels_per_stream(1)
            .build_many();

        mutlistream_util(builder, 50).await;
    }
}
