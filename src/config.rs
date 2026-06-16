use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use time::Date;

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
    /// Path to persistency file, storing application state, in TOML format.
    pub persist_path: PathBuf,
    /// Configuration to send email for due actions.
    pub mail: Option<AppConfigMail>,
}

/// Due actions configuration
#[derive(Deserialize, Debug)]
pub struct AppConfigDueAction {
    /// Title for due actions in index page
    pub title: String,
    /// String displayed when there is no due actions
    pub empty_str: String,
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
            empty_str: "None".to_string(),
            ignore_future_days: 60,
            warn_future_days: 30,
            alert_future_days: 0,
        }
    }
}
impl AppConfigDueAction {
    /// Get CSS class for a due date.
    pub fn get_css_style(&self, now: &Date, due_date: &Date) -> &'static str {
        let remaining_days = (*due_date - *now).whole_days();
        if remaining_days >= self.warn_future_days {
            "mynotes-date-ok"
        } else if remaining_days >= self.alert_future_days {
            "mynotes-date-warn"
        } else {
            "mynotes-date-alert"
        }
    }
}

/// Configuration to send email for due actions
#[derive(Deserialize, Debug)]
pub struct AppConfigMail {
    /// Mail title
    pub title: String,
    /// SMTP server address
    pub smtp_addr: String,
    /// SMTP server port
    pub smtp_port: u16,
    /// SMTP username
    pub smtp_user: String,
    /// SMTP password
    pub smtp_password: String,
    /// Sender email
    pub sender_email: String,
}
