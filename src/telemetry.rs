use tokio::task::JoinHandle;
use tracing::Subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;

#[must_use]
pub fn get_subscriber<Sink>(env_filter: impl AsRef<str>, sink: Sink) -> impl Subscriber
where
    Sink: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(sink)
        .with_span_events(FmtSpan::CLOSE);
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
