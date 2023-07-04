use once_cell::sync::OnceCell;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{
    fmt::MakeWriter,
    layer::{Layered, SubscriberExt},
    registry::LookupSpan,
    EnvFilter, Registry,
};

/// Keeps guard for non-blocking file writer.
static FILE_WRITER_GUARD: OnceCell<WorkerGuard> = OnceCell::new();

/// Create basic tracing [Subscriber] with [EnvFilter] and bunyan formatted JSON output to provided sink.
pub fn create_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> Layered<BunyanFormattingLayer<Sink>, Layered<JsonStorageLayer, Layered<EnvFilter, Registry>>>
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let base_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(base_layer)
}

/// Add file sink to tracing [Subscriber].
pub fn add_file_sink<T>(registry: T, name: String) -> Layered<BunyanFormattingLayer<NonBlocking>, T>
where
    T: Subscriber + Send + Sync,
    T: for<'a> LookupSpan<'a>,
{
    let file_appender = tracing_appender::rolling::daily("./logs", "{{project-name}}.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    FILE_WRITER_GUARD
        .set(_guard)
        .expect("Failed to attach FILE_WRITER_GUARD");
    let file_layer = BunyanFormattingLayer::new(name, file_writer);

    registry.with(file_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    set_global_default(subscriber).expect("Failed to set subscriber");
}
