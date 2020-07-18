#![allow(unused)]
use env_logger::Env;
use log::LevelFilter;
use rusttorney::{config::Config, server::AOServer};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter: &str;

    let config_path = PathBuf::from("./config/config.toml");
    let config_string = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_string)?;

    if config.debug {
        filter = "debug"
    } else {
        filter = "info"
    }

    env_logger::from_env(Env::default().default_filter_or(filter)).init();

    AOServer::new(config)?.run().await
}
