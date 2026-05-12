use serde::Deserialize;
use std::collections::HashMap;

/// Application configuration structure mapped from the YAML file.
#[derive(Deserialize)]
pub struct AppConfig {
    /// Authorized users; key is user name, value is user's password
    pub users: HashMap<String, String>,
}
