use once_cell::sync::OnceCell;
use serde::Deserialize;

pub static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Deserialize)]
pub struct Config {
    pub debug: bool,
    pub timeout: u32,
    pub multiclient_limit: u8,
    pub max_chars: u32,
    pub zalgo_tolerance: u8,
    pub general: GeneralConfig,
    pub masterserver: MasterServerConfig,
    pub wtce_floodguard: FloodGuardConfig,
    pub music_change_floodguard: FloodGuardConfig,
}

#[derive(Deserialize)]
pub struct GeneralConfig {
    pub hostname: String,
    pub playerlimit: u8,
    pub port: u32,
    pub local: bool,
    pub modpass: String,
    pub motd: String,
    pub use_websockets: bool,
    pub websocket_port: u32,
}

#[derive(Deserialize)]
pub struct MasterServerConfig {
    #[serde(rename = "use")]
    pub use_masterserver: bool,
    pub ip: String,
    pub port: u32,
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct FloodGuardConfig {
    pub times_per_interval: u8,
    pub interval_length: u8,
    pub mute_length: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let config_str =
            std::fs::read_to_string("./config/config.toml").unwrap();
        let config: Config = toml::from_str(&config_str).unwrap();

        assert_eq!(config.debug, false);
        assert_eq!(config.masterserver.name, "My server")
    }
}
