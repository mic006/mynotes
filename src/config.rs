use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Application configuration mapped from Rocket.toml app field
#[derive(Deserialize, Debug)]
pub struct AppConfig {
    /// Path to the directory containing markdown files.
    pub content_path: PathBuf,
    /// Map of usernames to passwords for basic authentication.
    pub users: HashMap<String, String>,
    /// Path to the HTML template file, relative to `content_path`.
    pub template_path: PathBuf,
    /// Content of the template file, loaded at startup.
    #[serde(skip)]
    pub template_content: String,
}

impl AppConfig {
    /// Reads the template file into memory.
    pub fn load_template(&mut self) -> std::io::Result<()> {
        self.template_content =
            std::fs::read_to_string(self.content_path.join(&self.template_path))?;
        Ok(())
    }
}
