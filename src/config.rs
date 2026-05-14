use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Application configuration structure mapped from the YAML file.
#[derive(Deserialize)]
pub struct AppConfig {
    /// Authorized users; key is user name, value is user's password
    pub users: HashMap<String, String>,
    /// Path to root directory containing content (markdown + static files)
    pub content_path: PathBuf,
}
