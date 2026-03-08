use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logger() {
    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    let filter_layer = filter::LevelFilter::from_level(Level::TRACE); //maybe env filter

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
}
