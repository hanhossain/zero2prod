use tracing::Subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

#[must_use]
pub fn get_subscriber<Sink>(env_filter: impl AsRef<str>, sink: Sink) -> impl Subscriber
where
    Sink: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(sink).pretty();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
}
