mod utils;
use utils::*;

#[cfg(feature = "us")]
#[cfg(test)]
mod coinbase_tests {
    use cex_exchanges::exchanges::{
        coinbase::ws::{CoinbaseWsBuilder, CoinbaseWsChannel},
        normalized::types::RawTradingPair
    };
    use serial_test::serial;

    use crate::{mutlistream_util, stream_util};

    async fn coinbase_util(builder: CoinbaseWsBuilder, iterations: usize) {
        stream_util(builder.build(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_status() {
        let builder = CoinbaseWsBuilder::default().add_channel(CoinbaseWsChannel::Status);
        coinbase_util(builder, 1).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_matches() {
        let builder = CoinbaseWsBuilder::default().add_channel(
            CoinbaseWsChannel::new_match(vec![RawTradingPair::new_raw("ETH_USD", '_'), RawTradingPair::new_no_delim("BTC-USD")]).unwrap()
        );
        coinbase_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_ticker() {
        let builder = CoinbaseWsBuilder::default().add_channel(
            CoinbaseWsChannel::new_ticker(vec![RawTradingPair::new_raw("ETH_USD", '_'), RawTradingPair::new_no_delim("BTC-USD")]).unwrap()
        );
        coinbase_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi() {
        let builder = CoinbaseWsBuilder::default()
            .add_channel(CoinbaseWsChannel::new_ticker(vec![RawTradingPair::new_raw("ETH_USD", '_')]).unwrap())
            .add_channel(
                CoinbaseWsChannel::new_match(vec![RawTradingPair::new_raw("ETH_USD", '_'), RawTradingPair::new_no_delim("BTC-USD")]).unwrap()
            )
            .add_channel(CoinbaseWsChannel::Status)
            .set_channels_per_stream(1)
            .build_many()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }
}

#[cfg(feature = "us")]
#[cfg(test)]
mod okex_tests {
    use cex_exchanges::{
        exchanges::{
            normalized::types::RawTradingPair,
            okex::ws::{OkexWsBuilder, OkexWsChannel}
        },
        CexExchange
    };
    use serial_test::serial;

    use crate::{mutlistream_util, stream_util};

    async fn okex_util(builder: OkexWsBuilder, iterations: usize) {
        stream_util(builder.build(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_trades() {
        let builder = OkexWsBuilder::new(CexExchange::Binance)
            .add_channel(OkexWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdt")]).unwrap());
        okex_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_book_ticker() {
        let builder = OkexWsBuilder::new(CexExchange::Binance).add_channel(
            OkexWsChannel::new_book_ticker(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        okex_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi() {
        let builder = OkexWsBuilder::new(CexExchange::Binance)
            .add_channel(
                OkexWsChannel::new_trade(vec![
                    RawTradingPair::new_raw("ETH_USDt", '_'),
                    RawTradingPair::new_base_quote("btc", "USDC", None),
                    RawTradingPair::new_raw("XLM/eUr", '/'),
                ])
                .unwrap()
            )
            .set_channels_per_stream(1)
            .build_many()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod binance_tests {
    use cex_exchanges::{
        binance::ws::BinanceWsChannelKind,
        exchanges::{
            binance::ws::{BinanceWsBuilder, BinanceWsChannel},
            normalized::types::RawTradingPair
        }
    };
    use serial_test::serial;

    use crate::{mutlistream_util, stream_util};

    async fn binance_util(builder: BinanceWsBuilder, iterations: usize) {
        stream_util(builder.build(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_trade() {
        let builder = BinanceWsBuilder::default().add_channel(
            BinanceWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        binance_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_book_ticker() {
        let builder = BinanceWsBuilder::default().add_channel(
            BinanceWsChannel::new_book_ticker(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        binance_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi() {
        let builder = BinanceWsBuilder::default()
            .add_channel(
                BinanceWsChannel::new_trade(vec![
                    RawTradingPair::new_raw("ETH_USDt", '_'),
                    RawTradingPair::new_no_delim("btcusdc"),
                    RawTradingPair::new_no_delim("SUSDETH"),
                ])
                .unwrap()
            )
            .set_channels_per_stream(1)
            .build_many()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_build_ranked_weighted_all_symbols() {
        let map = vec![(2, 3), (1, 10), (1, 30), (1, 50)];
        let channels = vec![BinanceWsChannelKind::Trade, BinanceWsChannelKind::BookTicker];

        let builder = BinanceWsBuilder::build_ranked_weighted_all_symbols(map, &channels)
            .await
            .unwrap();

        mutlistream_util(builder, 10000).await;
    }
}
