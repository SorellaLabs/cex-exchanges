#![allow(unused)]
use std::fmt::Debug;

use cex_exchanges::{
    clients::ws::{MutliWsStreamBuilder, WsStream},
    normalized::ws::NormalizedExchangeBuilder,
    Exchange
};
use futures::StreamExt;
use serde::Serialize;
use tracing::{error, info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub async fn stream_util<E: Exchange + Unpin + Debug + Send + 'static>(exchange: E, iterations: usize) {
    let mut stream = WsStream::new(exchange, None);
    stream.connect().await.unwrap();
    info!(target: "cex-exchanges::tests::ws", "connected stream");

    let mut i = 0;
    while let Some(val) = stream.next().await {
        if val.is_err() {
            error!(target: "cex-exchanges::tests::ws", "{:?}", val);
        }

        assert!(val.is_ok());

        let normalized = val.clone().normalize();
        assert_eq!(val, normalized);

        if i == iterations {
            break;
        }
        i += 1;
    }
}

pub async fn mutlistream_util<E: Exchange + Unpin + Debug + Send + 'static>(builder: MutliWsStreamBuilder<E>, iterations: usize) {
    let mut stream = builder.build_multistream_unconnected(None);
    info!(target: "cex-exchanges::tests::ws", "connected stream");

    let mut i = 0;
    while let Some(val) = stream.next().await {
        if val.is_err() {
            error!(target: "cex-exchanges::tests::ws", "{:?}", val);
        }

        info!(target: "cex-exchanges::tests::ws", "completed {i}/{iterations}");

        assert!(val.is_ok());

        let normalized = val.clone().normalize();
        assert_eq!(val, normalized);

        if i == iterations {
            break;
        }
        i += 1;
    }
}

pub async fn mutlithreaded_util<E: Exchange + Unpin + Debug + Send + 'static>(builder: MutliWsStreamBuilder<E>, iterations: usize) {
    let mut rx = builder.spawn_multithreaded(8, None, tokio::runtime::Handle::current());
    info!(target: "cex-exchanges::tests::ws", "connected stream");

    let mut i = 0;
    while let Some(val) = rx.recv().await {
        if val.is_err() {
            error!(target: "cex-exchanges::tests::ws", "{:?}", val);
        }

        info!(target: "cex-exchanges::tests::ws", "completed {i}/{iterations}");

        assert!(val.is_ok());

        let normalized = val.clone().normalize();
        assert_eq!(val, normalized);

        if i == iterations {
            break;
        }
        i += 1;
    }
}

pub async fn normalized_mutlithreaded_util(builder: NormalizedExchangeBuilder, iterations: usize) {
    let mut rx = builder
        .build_all_multithreaded(tokio::runtime::Handle::current(), 1, Some(10), Some(25))
        .unwrap()
        .unwrap();
    info!(target: "cex-exchanges::tests::ws", "connected stream"); //

    let mut i = 0;
    while let Some(val) = rx.next().await {
        if val.is_err() {
            error!(target: "cex-exchanges::tests::ws", "{:?}", val);
        }

        info!(target: "cex-exchanges::tests::ws", "completed {i}/{iterations}");

        assert!(val.is_ok());

        let normalized = val.clone().normalize();
        assert_eq!(val, normalized);

        if i == iterations {
            break;
        }
        i += 1;
    }
}

pub fn write_json<D>(a: D, path: &str)
where
    D: Serialize
{
    use std::io::Write;

    let mut f0 = std::fs::File::create(path).unwrap();

    writeln!(f0, "{}", serde_json::to_string(&a).unwrap()).unwrap();
}

pub fn init_test_tracing() {
    let data_filter = EnvFilter::builder()
        .with_default_directive(format!("cex-exchanges={}", Level::TRACE).parse().unwrap())
        .from_env_lossy();

    let data_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(true)
        .with_filter(data_filter)
        .boxed();

    let general_filter = EnvFilter::builder()
        .with_default_directive(Level::DEBUG.into())
        .from_env_lossy();

    let general_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(true)
        .with_filter(general_filter)
        .boxed();

    tracing_subscriber::registry()
        .with(vec![data_layer, general_layer])
        .init();
}
