#![allow(unused)]
use log::LevelFilter;
use rusttorney::{server::AOServer};
use env_logger::Env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    AOServer::new()?.run().await
}
