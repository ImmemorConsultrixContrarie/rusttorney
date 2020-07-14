#![allow(unused)]
use log::LevelFilter;
use rusttorney::{config::CONFIG, server::AOServer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut logger = pretty_env_logger::formatted_timed_builder();
    let config_str = std::fs::read_to_string("./config/config.toml")?;
    CONFIG.set(toml::from_str(&config_str)?);

    let level_filter;

    if CONFIG.get().unwrap().debug {
        if let Ok(log_filt) = std::env::var("RUST_LOG") {
            level_filter = log_filt.parse()?;
        } else {
            level_filter = LevelFilter::Debug
        }
    } else {
        level_filter = LevelFilter::Info
    }

    logger.filter_level(level_filter).init();

    AOServer::run().await
}
