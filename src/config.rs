use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config<'a> {
    pub debug: bool,
    pub timeout: u32,
    pub multiclient_limit: u8,
    pub max_chars: u32,
    pub zalgo_tolerance: u8,
    #[serde(borrow)]
    pub general: GeneraConfig<'a>,
    #[serde(borrow)]
    pub masterserver: MasterServerConfig<'a>,
    pub wtce_floodguard: FloodGuardConfig,
    pub music_change_floodguard: FloodGuardConfig,
}

#[derive(Debug, Deserialize)]
pub struct GeneraConfig<'a> {
    pub hostname: &'a str,
    pub playerlimit: u8,
    pub port: u32,
    pub local: bool,
    pub modpass: &'a str,
    pub motd: &'a str,
    pub use_websockets: bool,
    pub websocket_port: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct MasterServerConfig<'a> {
    #[serde(rename = "use")]
    pub use_masterserver: bool,
    pub ip: &'a str,
    pub port: u16,
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct FloodGuardConfig {
    pub times_per_interval: u8,
    pub interval_length: u8,
    pub mute_length: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::current_dir;

    #[test]
    fn test_config_parsing() {
        let config_str = std::fs::read_to_string("./config/config.toml").unwrap();
        let config: Config = toml::from_str(&config_str).unwrap();

        assert_eq!(config.debug, false);
        assert_eq!(config.masterserver.name, "My server")
    }
}
