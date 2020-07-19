#![allow(unused)]
use env_logger::Env;
use log::LevelFilter;
use rusttorney::{config::Config, server::AOServer};
use sqlx::sqlite::SqlitePool;
use std::{
    path::PathBuf,
    sync::Mutex
};

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

    let pool = Mutex::new(
        SqlitePool::builder()
            .max_size(8)
            .build("sqlite://database.sql")
            .await
            .unwrap()
        );

    AOServer::new(&config, &pool)?.run().await
}
