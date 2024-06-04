use tracing::{Level, Subscriber};
use tracing_subscriber::{registry::LookupSpan, EnvFilter, Layer};

/// Builds a new tracing layer that writes to stdout.
/// The events are filtered by `level`
pub fn make_tracing_layer<S>(level: Level) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber,
    for<'a> S: LookupSpan<'a>
{
    let filter = EnvFilter::builder()
        .with_default_directive(format!("cex-exchanges={level}").parse().unwrap())
        .from_env_lossy();

    tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(true)
        .with_filter(filter)
        .boxed()
}
