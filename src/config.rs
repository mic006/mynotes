use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Application configuration structure mapped from the YAML file.
#[derive(Deserialize)]
pub struct AppConfig {
    /// Authorized users; key is user name, value is user's password
    pub users: HashMap<String, String>,
    /// Markdown content folder
    pub markdown_folder: PathBuf,
}
