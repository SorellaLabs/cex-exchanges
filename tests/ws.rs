mod utils;
use utils::*;

#[cfg(feature = "us")]
#[cfg(test)]
mod coinbase_tests {
    use cex_exchanges::{
        coinbase::ws::CoinbaseWsChannelKind,
        exchanges::{
            coinbase::ws::{CoinbaseWsBuilder, CoinbaseWsChannel},
            normalized::types::RawTradingPair
        }
    };
    use serial_test::serial;

    use super::*;

    async fn coinbase_util(builder: CoinbaseWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
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
    async fn test_multi_distributed() {
        let builder = CoinbaseWsBuilder::default()
            .add_channel(CoinbaseWsChannel::new_ticker(vec![RawTradingPair::new_raw("ETH_USD", '_')]).unwrap())
            .add_channel(
                CoinbaseWsChannel::new_match(vec![RawTradingPair::new_raw("ETH_USD", '_'), RawTradingPair::new_no_delim("BTC-USD")]).unwrap()
            )
            .add_channel(CoinbaseWsChannel::Status)
            .build_many_distributed()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted() {
        let channels = vec![CoinbaseWsChannelKind::Matches, CoinbaseWsChannelKind::Ticker];

        let builder = CoinbaseWsBuilder::build_from_all_instruments(&channels)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted_multithread() {
        let channels = vec![CoinbaseWsChannelKind::Matches, CoinbaseWsChannelKind::Ticker];

        let builder = CoinbaseWsBuilder::build_from_all_instruments(&channels)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
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
        okex::ws::OkexWsChannelKind
    };
    use serial_test::serial;

    use super::*;

    async fn okex_util(builder: OkexWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_trades() {
        let builder = OkexWsBuilder::new(None)
            .add_channel(OkexWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdt")]).unwrap());
        okex_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_book_ticker() {
        let builder = OkexWsBuilder::new(None).add_channel(
            OkexWsChannel::new_book_ticker(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        okex_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi_distributed() {
        let builder = OkexWsBuilder::new(None)
            .add_channel(
                OkexWsChannel::new_trade(vec![
                    RawTradingPair::new_raw("ETH_USDt", '_'),
                    RawTradingPair::new_base_quote("btc", "USDC", None),
                    RawTradingPair::new_raw("XLM/eUr", '/'),
                ])
                .unwrap()
            )
            .build_many_distributed()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted() {
        let channels = vec![OkexWsChannelKind::TradesAll, OkexWsChannelKind::BookTicker];

        let builder = OkexWsBuilder::build_from_all_instruments(&channels, None)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted_multithread() {
        let channels = vec![OkexWsChannelKind::TradesAll, OkexWsChannelKind::BookTicker];

        let builder = OkexWsBuilder::build_from_all_instruments(&channels, None)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
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

    use super::*;

    async fn binance_util(builder: BinanceWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
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
    async fn test_multi_distributed() {
        let builder = BinanceWsBuilder::default()
            .add_channel(
                BinanceWsChannel::new_trade(vec![
                    RawTradingPair::new_raw("ETH_USDt", '_'),
                    RawTradingPair::new_no_delim("btcusdc"),
                    RawTradingPair::new_no_delim("SUSDETH"),
                ])
                .unwrap()
            )
            .build_many_distributed()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted() {
        let channels = vec![BinanceWsChannelKind::Trade, BinanceWsChannelKind::BookTicker];

        let builder = BinanceWsBuilder::build_from_all_instruments(&channels)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted_multithread() {
        let channels = vec![BinanceWsChannelKind::Trade, BinanceWsChannelKind::BookTicker];

        let builder = BinanceWsBuilder::build_from_all_instruments(&channels)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod kucoin_tests {
    use cex_exchanges::{
        exchanges::{
            kucoin::ws::{KucoinWsBuilder, KucoinWsChannel},
            normalized::types::RawTradingPair
        },
        kucoin::ws::KucoinWsChannelKind
    };
    use serial_test::serial;

    use super::*;

    async fn kucoin_util(builder: KucoinWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_match() {
        let builder = KucoinWsBuilder::default().add_channel(
            KucoinWsChannel::new_match(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        kucoin_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_book_ticker() {
        let builder = KucoinWsBuilder::default().add_channel(
            KucoinWsChannel::new_ticker(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        kucoin_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi_distributed() {
        let builder = KucoinWsBuilder::default()
            .add_channel(
                KucoinWsChannel::new_match(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("btc-usdc")]).unwrap()
            )
            .add_channel(
                KucoinWsChannel::new_ticker(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("btc-usdc")]).unwrap()
            )
            .build_many_distributed()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted() {
        let channels = vec![KucoinWsChannelKind::Match, KucoinWsChannelKind::Ticker];

        let builder = KucoinWsBuilder::build_from_all_instruments(&channels)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted_multithread() {
        let channels = vec![KucoinWsChannelKind::Match, KucoinWsChannelKind::Ticker];

        let builder = KucoinWsBuilder::build_from_all_instruments(&channels)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod bybit_tests {
    use cex_exchanges::{
        bybit::ws::BybitWsChannelKind,
        exchanges::{
            bybit::ws::{BybitWsBuilder, BybitWsChannel},
            normalized::types::RawTradingPair
        }
    };
    use serial_test::serial;

    use super::*;

    async fn bybit_util(builder: BybitWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_trade() {
        let builder = BybitWsBuilder::default().add_channel(
            BybitWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        bybit_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_ticker() {
        let builder = BybitWsBuilder::default().add_channel(
            BybitWsChannel::new_ticker(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        bybit_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi_distributed() {
        let builder = BybitWsBuilder::default()
            .add_channel(BybitWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("btc-usdc")]).unwrap())
            .add_channel(
                BybitWsChannel::new_ticker(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("btc-usdc")]).unwrap()
            )
            .build_many_distributed()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted() {
        let channels = vec![BybitWsChannelKind::Trade, BybitWsChannelKind::OrderbookL1];

        let builder = BybitWsBuilder::build_from_all_instruments(&channels)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_weighted_multithread() {
        let channels = vec![BybitWsChannelKind::Trade, BybitWsChannelKind::OrderbookL1];

        let builder = BybitWsBuilder::build_from_all_instruments(&channels)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
    }
}
