use std::sync::LazyLock;

use pulldown_cmark::Options;
use regex::Regex;
use time::{Date, format_description::well_known::Iso8601};

use crate::markdown::DueAction;

/// Ignore due actions when they are too far in the future
const DUE_ACTION_IGNORE_FUTURE_DAYS: i64 = 60;
/// Warn for due actions in a near future
const DUE_ACTION_WARN_FUTURE_DAYS: i64 = 30;
/// Alert for due actions near or past the deadline
const DUE_ACTION_ALERT_FUTURE_DAYS: i64 = 0;

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
/// - extract due actions
/// - modify body for better rendering (if `will_render_html` is set)
pub fn user_process_markdown(body: &mut String, _will_render_html: bool) -> Vec<DueAction> {
    static RE_DUE_ACTION: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?m)Next: (\d{4}-\d{2}-\d{2}) (.*)$").unwrap());

    let mut due_actions = Vec::new();
    for (_, [date, action]) in RE_DUE_ACTION.captures_iter(body).map(|c| c.extract()) {
        due_actions.push(DueAction {
            date: Date::parse(date, &Iso8601::DATE).unwrap(),
            action: action.to_string(),
        });
    }
    due_actions
}

// Add span around date, with extra classes if provided
pub fn render_date(d: &str, classes: &str) -> String {
    format!(r#"<span class="mynotes-date {classes}">{d}</span>"#)
}

// Add span around money
pub fn render_money(s: &str) -> String {
    format!(r#"<span class="mynotes-money">{s}</span>"#)
}

impl DueAction {
    /// Whether this due action shall be rendered in the index page.
    pub fn render_in_index(&self, now: &Date) -> Option<String> {
        let remaining_days = (self.date - *now).whole_days();
        if remaining_days >= DUE_ACTION_IGNORE_FUTURE_DAYS {
            // action is too far in the future, ignore it
            return None;
        }
        Some(format!(
            "{} {}",
            render_date(
                &self.date.format(&Iso8601::DATE).ok()?,
                self.get_css_class(now)
            ),
            self.action
        ))
    }

    /// Get CSS class for this due action.
    pub fn get_css_class(&self, now: &Date) -> &'static str {
        let remaining_days = (self.date - *now).whole_days();
        if remaining_days >= DUE_ACTION_WARN_FUTURE_DAYS {
            "mynotes-date-ok"
        } else if remaining_days >= DUE_ACTION_ALERT_FUTURE_DAYS {
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

        let actions = user_process_markdown(&mut body, false);

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
        let actions = user_process_markdown(&mut body, false);
        assert!(actions.is_empty());
    }
}
