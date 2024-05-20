use std::fmt::Debug;

use cex_exchanges::{
    clients::ws::{MutliWsStreamBuilder, WsStream},
    exchanges::Exchange
};
use futures::StreamExt;
use serde::Serialize;

pub async fn stream_util<E: Exchange + Unpin + Debug + Send + 'static>(exchange: E, iterations: usize) {
    let mut stream = WsStream::new(exchange, None);
    stream.connect().await.unwrap();

    let mut i = 0;
    while let Some(val) = stream.next().await {
        if val.is_err() {
            println!("ERROR: {:?}", val);
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
    println!("CONNECTED STREAM");

    let mut i = 0;
    while let Some(val) = stream.next().await {
        if val.is_err() {
            println!("ERROR: {:?}", val);
        }

        println!("VAL: {i}/{iterations}");

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
    let mut i = 0;
    while let Some(val) = rx.recv().await {
        if val.is_err() {
            println!("ERROR: {:?}", val);
        }

        println!("VAL: {i}/{iterations}");

        assert!(val.is_ok());

        let normalized = val.clone().normalize();
        assert_eq!(val, normalized);

        if i == iterations {
            break;
        }
        i += 1;
    }
}

#[allow(unused)]
pub fn write_json<D>(a: D, path: &str)
where
    D: Serialize
{
    use std::io::Write;

    let mut f0 = std::fs::File::create(path).unwrap();

    writeln!(f0, "{}", serde_json::to_string(&a).unwrap()).unwrap();
}
