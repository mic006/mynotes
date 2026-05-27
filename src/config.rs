use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

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
    template_content_cache: TemplateContentCache,
}

#[derive(Debug)]
struct TemplateContentCache {
    /// File path,
    path: PathBuf,
    /// last modification time when the file was read
    mtime: SystemTime,
    /// Content of the template file
    content: String,
}
impl Default for TemplateContentCache {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            mtime: SystemTime::UNIX_EPOCH,
            content: String::new(),
        }
    }
}
impl TemplateContentCache {
    fn get_content(&mut self) -> std::io::Result<&str> {
        let current_mtime = std::fs::metadata(&self.path)?.modified()?;
        if self.mtime != current_mtime {
            self.content = std::fs::read_to_string(&self.path)?;
            self.mtime = current_mtime;
        }
        Ok(&self.content)
    }
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
    pub fn get_html_template(&mut self) -> std::io::Result<&str> {
        if self.template_content_cache.path.as_os_str().is_empty() {
            self.template_content_cache.path = self.content_path.join(&self.template_path);
        }
        self.template_content_cache.get_content()
    }
}
