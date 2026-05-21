use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Application configuration mapped from Rocket.toml app field
#[derive(Deserialize, Default, Debug)]
pub struct AppConfig {
    /// Path to the directory containing markdown files.
    pub content_path: PathBuf,
    /// Due actions configuration
    #[serde(default)]
    pub due_action: AppConfigDueAction,
    /// Map of usernames to passwords for basic authentication.
    pub users: HashMap<String, String>,
    /// Path to the HTML template file, relative to `content_path`.
    pub template_path: PathBuf,
    /// Content of the template file, loaded at startup.
    #[serde(skip)]
    pub template_content: String,
}

/// Due actions configuration
#[derive(Deserialize, Debug)]
pub struct AppConfigDueAction {
    /// Title for due actions in index page
    pub title: String,
    /// Ignore due actions when they are too far in the future
    pub ignore_future_days: i64,
    /// Warn for due actions in a near future
    pub warn_future_days: i64,
    /// Alert for due actions near or past the deadline
    pub alert_future_days: i64,
}
impl Default for AppConfigDueAction {
    fn default() -> Self {
        Self {
            title: "Due actions".to_string(),
            ignore_future_days: 60,
            warn_future_days: 30,
            alert_future_days: 0,
        }
    }
}

impl AppConfig {
    /// Reads the template file into memory.
    pub fn load_template(&mut self) -> std::io::Result<()> {
        self.template_content =
            std::fs::read_to_string(self.content_path.join(&self.template_path))?;
        Ok(())
    }
}
