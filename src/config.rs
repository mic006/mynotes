use serde::Deserialize;
use std::collections::HashMap;

const CONFIG_FILE: &str = "config.yaml";

/// Application configuration structure mapped from the YAML file.
#[derive(Deserialize)]
pub struct AppConfig {
    /// TCP port of the web server
    pub port: u16,
    /// authorized users; key is user name, value is user's password
    pub users: HashMap<String, String>,
}

impl AppConfig {
    /// Loads the application configuration
    pub fn load() -> Self {
        let config_str = std::fs::read_to_string(CONFIG_FILE)
            .unwrap_or_else(|_| panic!("Could not read the configuration file {CONFIG_FILE}"));
        serde_yaml::from_str(&config_str).expect("Failed to parse YAML")
    }
}
