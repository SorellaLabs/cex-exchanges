mod utils;
use utils::*;

#[cfg(feature = "us")]
#[cfg(test)]
mod coinbase_tests {
    use cex_exchanges::{
        coinbase::ws::{
            channels::{CoinbaseWsChannel, CoinbaseWsChannelKind},
            CoinbaseWsBuilder
        },
        normalized::{
            types::{InstrumentFilter, RawTradingPair},
            ws::{NormalizedExchangeBuilder, NormalizedWsChannelKinds}
        },
        traits::{SpecificWsBuilder, SpecificWsChannel},
        CexExchange
    };
    use serial_test::serial;

    use super::*;

    async fn coinbase_util(builder: CoinbaseWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_matches() {
        init_test_tracing();
        let builder = CoinbaseWsBuilder::default().add_channel(
            CoinbaseWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USD", '_'), RawTradingPair::new_no_delim("BTC-USD")]).unwrap()
        );
        coinbase_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_ticker() {
        init_test_tracing();
        let builder = CoinbaseWsBuilder::default().add_channel(
            CoinbaseWsChannel::new_quote(vec![RawTradingPair::new_raw("ETH_USD", '_'), RawTradingPair::new_no_delim("BTC-USD")]).unwrap()
        );
        coinbase_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi_distributed() {
        init_test_tracing();
        let builder = CoinbaseWsBuilder::default()
            .add_channel(CoinbaseWsChannel::new_quote(vec![RawTradingPair::new_raw("ETH_USD", '_')]).unwrap())
            .add_channel(
                CoinbaseWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USD", '_'), RawTradingPair::new_no_delim("BTC-USD")]).unwrap()
            )
            .add_channel(CoinbaseWsChannel::Status)
            .build_many_distributed()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments() {
        init_test_tracing();
        let channels = vec![CoinbaseWsChannelKind::Matches, CoinbaseWsChannelKind::Ticker];

        let builder = CoinbaseWsBuilder::build_from_all_instruments(&channels, Some(10), None)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments_multithread() {
        init_test_tracing();
        let channels = vec![CoinbaseWsChannelKind::Matches, CoinbaseWsChannelKind::Ticker];

        let builder = CoinbaseWsBuilder::build_from_all_instruments(&channels, Some(10), None)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments_multithread_from_normalized() {
        init_test_tracing();
        let channels = vec![NormalizedWsChannelKinds::Trades, NormalizedWsChannelKinds::Quotes];

        let normalized_symbols = CexExchange::Coinbase
            .get_all_instruments(Some(InstrumentFilter::Active))
            .await
            .unwrap();
        let mut builder = NormalizedExchangeBuilder::new();
        builder.add_pairs_all_channels(
            CexExchange::Coinbase,
            &channels,
            &normalized_symbols
                .into_iter()
                .map(|instr| instr.trading_pair.into())
                .collect::<Vec<_>>()
        );

        normalized_mutlithreaded_util(builder, 1000).await;
    }
}

#[cfg(feature = "us")]
#[cfg(test)]
mod okex_tests {
    use cex_exchanges::{
        normalized::{
            types::{InstrumentFilter, RawTradingPair},
            ws::{NormalizedExchangeBuilder, NormalizedWsChannelKinds}
        },
        okex::ws::{
            channels::{OkexWsChannel, OkexWsChannelKind},
            OkexWsBuilder
        },
        traits::{SpecificWsBuilder, SpecificWsChannel},
        CexExchange
    };
    use serial_test::serial;

    use super::*;

    async fn okex_util(builder: OkexWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_trades() {
        init_test_tracing();
        let builder = OkexWsBuilder::new(None)
            .add_channel(OkexWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdt")]).unwrap());
        okex_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_book_ticker() {
        init_test_tracing();
        let builder = OkexWsBuilder::new(None)
            .add_channel(OkexWsChannel::new_quote(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap());
        okex_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi_distributed() {
        init_test_tracing();
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
    async fn test_multi_all_instruments() {
        init_test_tracing();
        let channels = vec![OkexWsChannelKind::TradesAll, OkexWsChannelKind::BookTicker];

        let builder = OkexWsBuilder::build_from_all_instruments(&channels, Some(10), None)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments_multithread() {
        init_test_tracing();
        let channels = vec![OkexWsChannelKind::TradesAll, OkexWsChannelKind::BookTicker];

        let builder = OkexWsBuilder::build_from_all_instruments(&channels, Some(10), None)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments_multithread_from_normalized() {
        init_test_tracing();
        let channels = vec![NormalizedWsChannelKinds::Trades, NormalizedWsChannelKinds::Quotes];

        let normalized_symbols = CexExchange::Okex
            .get_all_instruments(Some(InstrumentFilter::Active))
            .await
            .unwrap();
        let mut builder = NormalizedExchangeBuilder::new();
        builder.add_pairs_all_channels(
            CexExchange::Okex,
            &channels,
            &normalized_symbols
                .into_iter()
                .map(|instr| instr.trading_pair.into())
                .collect::<Vec<_>>()
        );

        normalized_mutlithreaded_util(builder, 1000).await;
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod binance_tests {
    use cex_exchanges::{
        binance::ws::{
            channels::{BinanceWsChannel, BinanceWsChannelKind},
            BinanceWsBuilder
        },
        normalized::types::RawTradingPair,
        traits::{SpecificWsBuilder, SpecificWsChannel}
    };
    use serial_test::serial;

    use super::*;

    async fn binance_util(builder: BinanceWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_trade() {
        init_test_tracing();
        let builder = BinanceWsBuilder::default().add_channel(
            BinanceWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        binance_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_book_ticker() {
        init_test_tracing();
        let builder = BinanceWsBuilder::default().add_channel(
            BinanceWsChannel::new_quote(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        binance_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi_distributed() {
        init_test_tracing();
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
    async fn test_multi_all_instruments() {
        init_test_tracing();
        let channels = vec![BinanceWsChannelKind::Trade, BinanceWsChannelKind::BookTicker];

        let builder = BinanceWsBuilder::build_from_all_instruments(&channels, None, None)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments_multithread() {
        init_test_tracing();
        let channels = vec![BinanceWsChannelKind::Trade, BinanceWsChannelKind::BookTicker];

        let builder = BinanceWsBuilder::build_from_all_instruments(&channels, None, None)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod kucoin_tests {
    use cex_exchanges::{
        kucoin::ws::{
            channels::{KucoinWsChannel, KucoinWsChannelKind},
            KucoinWsBuilder
        },
        normalized::types::RawTradingPair,
        traits::{SpecificWsBuilder, SpecificWsChannel}
    };
    use serial_test::serial;

    use super::*;

    async fn kucoin_util(builder: KucoinWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_match() {
        init_test_tracing();
        let builder = KucoinWsBuilder::default().add_channel(
            KucoinWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        kucoin_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_book_ticker() {
        init_test_tracing();
        let builder = KucoinWsBuilder::default().add_channel(
            KucoinWsChannel::new_quote(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        kucoin_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi_distributed() {
        init_test_tracing();
        let builder = KucoinWsBuilder::default()
            .add_channel(
                KucoinWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("btc-usdc")]).unwrap()
            )
            .add_channel(
                KucoinWsChannel::new_quote(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("btc-usdc")]).unwrap()
            )
            .build_many_distributed()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments() {
        init_test_tracing();
        let channels = vec![KucoinWsChannelKind::Match, KucoinWsChannelKind::Ticker];

        let builder = KucoinWsBuilder::build_from_all_instruments(&channels, None, None)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments_multithread() {
        init_test_tracing();
        let channels = vec![KucoinWsChannelKind::Match, KucoinWsChannelKind::Ticker];

        let builder = KucoinWsBuilder::build_from_all_instruments(&channels, None, None)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
    }
}

#[cfg(feature = "non-us")]
#[cfg(test)]
mod bybit_tests {
    use cex_exchanges::{
        bybit::ws::{
            channels::{BybitWsChannel, BybitWsChannelKind},
            BybitWsBuilder
        },
        normalized::types::RawTradingPair,
        traits::{SpecificWsBuilder, SpecificWsChannel}
    };
    use serial_test::serial;

    use super::*;

    async fn bybit_util(builder: BybitWsBuilder, iterations: usize) {
        stream_util(builder.build_single(), iterations).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_trade() {
        init_test_tracing();
        let builder = BybitWsBuilder::default().add_channel(
            BybitWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        bybit_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_ticker() {
        init_test_tracing();
        let builder = BybitWsBuilder::default().add_channel(
            BybitWsChannel::new_quote(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("BTC-USdc")]).unwrap()
        );
        bybit_util(builder, 5).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_multi_distributed() {
        init_test_tracing();
        let builder = BybitWsBuilder::default()
            .add_channel(BybitWsChannel::new_trade(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("btc-usdc")]).unwrap())
            .add_channel(BybitWsChannel::new_quote(vec![RawTradingPair::new_raw("ETH_USDt", '_'), RawTradingPair::new_no_delim("btc-usdc")]).unwrap())
            .build_many_distributed()
            .unwrap();

        mutlistream_util(builder, 50).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments() {
        init_test_tracing();
        let channels = vec![BybitWsChannelKind::Trade, BybitWsChannelKind::OrderbookL1];

        let builder = BybitWsBuilder::build_from_all_instruments(&channels, None, None)
            .await
            .unwrap();

        mutlistream_util(builder, 1000).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    #[serial]
    async fn test_multi_all_instruments_multithread() {
        init_test_tracing();
        let channels = vec![BybitWsChannelKind::Trade, BybitWsChannelKind::OrderbookL1];

        let builder = BybitWsBuilder::build_from_all_instruments(&channels, None, None)
            .await
            .unwrap();
        mutlithreaded_util(builder, 1000).await;
    }
}
