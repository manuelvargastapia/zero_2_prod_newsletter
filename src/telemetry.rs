use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We're using `impl Subscriber` as return type toa void having to spell out
/// the actual type of type of the returned subscriber.
/// We need to explicitely call out that the returned subscriber is `Send` and
/// `Sync` to make it possible to pass it to [init_subscriber] later on.
pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    // The EnvFilter struct discards spans based on their log levels ansd their origins.
    // We're falling back to printing all logs at info-level or above if the RUST_LOG env var has
    // not been set
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    // The Layer trait helps with the composition of our processing pipeline for the spans.
    // We can combine multiple small layers to reach our goal.
    // JsonStorageLayer processes spans data and stores the associated metadata in a easy-to-comsume
    // JSON format for downstream layers.
    // BunyanFormattingLayer build on top of JsonStorageLayer and outputs log records in
    // bunyan-compoatible JSON format
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    // The Subscriber triat handles the Span's lifecycle.
    // Registry implements the Subscriber trait and takes care of collecting and store
    // spans metadata, recording relationships between spans, and tracking which spans are
    // active and which are closed
    Registry::default()
        // with() is a extension function provided by SubscriberExt
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all log's (tracing's log crate dependency) events to subscriber
    // (this is required because tracing show all the app-level logs but not
    // actix-web and other info)
    LogTracer::init().expect("Failed to set logger");
    // Specify what subscriber should be used to process spans
    set_global_default(subscriber).expect("Failed to set subscriber.");
}
