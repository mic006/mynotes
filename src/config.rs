mod field_duration;

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use time::{Date, PrimitiveDateTime};

pub use field_duration::CfgDuration;

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
    pub mail: Option<AppConfigMailAlert>,
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
    /// Determine due date category based on config
    pub fn get_category(&self, now: &Date, due_date: &Date) -> DueActionCategory {
        let remaining_days = (*due_date - *now).whole_days();
        if remaining_days >= self.ignore_future_days {
            DueActionCategory::FarFuture
        } else if remaining_days >= self.warn_future_days {
            DueActionCategory::Normal
        } else if remaining_days >= self.alert_future_days {
            DueActionCategory::Warn
        } else {
            DueActionCategory::Alert
        }
    }
}

/// Category for due actions
#[derive(PartialEq, Debug)]
pub enum DueActionCategory {
    /// Due action is in a far future, may be ignored for now
    FarFuture,
    /// Due action to be performed when available
    Normal,
    /// Due action in a near future
    Warn,
    /// Due action to be done ASAP (deadline is short or passed)
    Alert,
}
impl DueActionCategory {
    /// Get CSS class for this due date category
    pub fn get_css_style(&self) -> &'static str {
        match self {
            Self::FarFuture | Self::Normal => "mynotes-date-ok",
            Self::Warn => "mynotes-date-warn",
            Self::Alert => "mynotes-date-alert",
        }
    }
}

/// Configuration to send email for due actions
#[derive(Deserialize, Debug)]
pub struct AppConfigMailAlert {
    /// Mail title
    pub mail_title: String,
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
    /// Period to send mail when there are normal due actions (`warn_future_days` < due action <= `ignore_future_days`)
    pub mail_period_normal: CfgDuration,
    /// Period to send mail when there are warning due actions (`alert_future_days` < due action <= `warn_future_days`)
    pub mail_period_warn: CfgDuration,
    /// Period to send mail when there are alert due actions (due action < `alert_future_days`)
    pub mail_period_alert: CfgDuration,
    /// Local time to send mail
    /// Fields below the period will be used to send the mail
    /// Ex: for a 1-day or 2-day period, mail will be sent at the given local time (HH:MM:SS)
    ///     for a 1-week or 2-week period, mail will be sent at the given local time (HH:MM:SS) + same week day
    pub wall_clock_send_mail: PrimitiveDateTime,
}

#[cfg(test)]
impl Default for AppConfigMailAlert {
    fn default() -> Self {
        Self {
            mail_title: String::new(),
            smtp_addr: String::new(),
            smtp_port: 0,
            smtp_user: String::new(),
            smtp_password: String::new(),
            sender_email: String::new(),
            mail_period_normal: CfgDuration::default(),
            mail_period_warn: CfgDuration::default(),
            mail_period_alert: CfgDuration::default(),
            wall_clock_send_mail: PrimitiveDateTime::MIN,
        }
    }
}
