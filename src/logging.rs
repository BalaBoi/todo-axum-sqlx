use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, registry, util::SubscriberInitExt,
};

pub fn init_tracing_subscriber() {
    registry()
        .with(fmt::layer())
        .with(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .init();
}
