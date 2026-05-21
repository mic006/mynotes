use std::sync::LazyLock;

use pulldown_cmark::Options;
use regex::{Captures, Regex};
use time::{Date, OffsetDateTime, format_description::well_known::Iso8601};

use crate::config::{AppConfig, AppConfigDueAction};
use crate::markdown::DueAction;

/// Markdown options used to transform markdown to HTML.
/// See <https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Options.html>
pub fn get_markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_GFM
}

/// Process markdown body
///
/// - extract due actions + format due action date with CSS style
/// - format money values
pub fn user_process_markdown(
    body: &mut String,
    will_render_html: bool,
    cfg: &AppConfig,
) -> Vec<DueAction> {
    static RE_DUE_ACTION: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?m)Next: (\d{4}-\d{2}-\d{2}) (.*)$").unwrap());

    // extract due actions + format due action date with CSS style
    let now = OffsetDateTime::now_utc().date();
    let mut due_actions = Vec::new();
    let body_modified = RE_DUE_ACTION.replace_all(body, |caps: &Captures<'_>| {
        let (_, [date, action]) = caps.extract();
        let due_action = DueAction {
            date: Date::parse(date, &Iso8601::DATE).unwrap(),
            action: action.to_string(),
        };
        let style = due_action.get_css_style(&now, &cfg.due_action);
        due_actions.push(due_action);
        format!("Next: {} {action}", render_date(date, style))
    });

    // format money values
    if will_render_html {
        static RE_MONEY: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(-?\d+(?:\.\d{2})?)€").unwrap());

        let body_modified = RE_MONEY.replace_all(&body_modified, |caps: &Captures<'_>| {
            render_money(caps.get(1).unwrap().as_str())
        });
        *body = body_modified.to_string();
    }

    due_actions
}

// Add span around date, with extra classes if provided
pub fn render_date(d: &str, style: &str) -> String {
    format!(r#"<span class="mynotes-date {style}">{d}</span>"#)
}

// Add span around money
pub fn render_money(s: &str) -> String {
    format!(r#"<span class="mynotes-money">{s} €</span>"#)
}

impl DueAction {
    /// Whether this due action shall be rendered in the index page.
    pub fn render_in_index(&self, now: &Date, cfg: &AppConfigDueAction) -> Option<String> {
        let remaining_days = (self.date - *now).whole_days();
        if remaining_days >= cfg.ignore_future_days {
            // action is too far in the future, ignore it
            return None;
        }
        Some(format!(
            "{} {}",
            render_date(
                &self.date.format(&Iso8601::DATE).ok()?,
                self.get_css_style(now, cfg)
            ),
            self.action
        ))
    }

    /// Get CSS class for this due action.
    pub fn get_css_style(&self, now: &Date, cfg: &AppConfigDueAction) -> &'static str {
        let remaining_days = (self.date - *now).whole_days();
        if remaining_days >= cfg.warn_future_days {
            "mynotes-date-ok"
        } else if remaining_days >= cfg.alert_future_days {
            "mynotes-date-warn"
        } else {
            "mynotes-date-alert"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_process_markdown_extracts_actions() {
        let mut body = String::from(
            "Task List:\n\
            \n\
            - Next: 2024-05-20 Buy groceries\n\
            - Next: 2024-05-21 Call the bank",
        );

        let actions = user_process_markdown(&mut body, false, &AppConfig::default());

        assert_eq!(actions.len(), 2);
        assert_eq!(
            actions[0].date,
            Date::parse("2024-05-20", &Iso8601::DATE).unwrap()
        );
        assert_eq!(actions[0].action, "Buy groceries");
        assert_eq!(
            actions[1].date,
            Date::parse("2024-05-21", &Iso8601::DATE).unwrap()
        );
        assert_eq!(actions[1].action, "Call the bank");
    }

    #[test]
    fn test_user_process_markdown_no_matches() {
        let mut body = String::from("This is a simple note without any due actions.");
        let actions = user_process_markdown(&mut body, false, &AppConfig::default());
        assert!(actions.is_empty());
    }
}
