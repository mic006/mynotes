use std::fmt::Write as _;

use time::Date;
use time::format_description::well_known::Iso8601;

use crate::config::{AppConfig, AppConfigDueAction};
use crate::mdtree::{DueAction, MdTree};

impl DueAction {
    /// Get CSS class for this due action.
    pub fn get_css_style(&self, now: &Date, config: &AppConfigDueAction) -> &'static str {
        let remaining_days = (self.date - *now).whole_days();
        if remaining_days >= config.warn_future_days {
            "mynotes-date-ok"
        } else if remaining_days >= config.alert_future_days {
            "mynotes-date-warn"
        } else {
            "mynotes-date-alert"
        }
    }
}
/// Get HTML body for index page
pub fn get_body_index(md_tree: &mut MdTree, config: &AppConfig, now: &Date) -> String {
    let md_files = md_tree.md_files_iter().collect::<Vec<_>>();

    if md_files.is_empty() {
        return "<h2>No content</h2>".to_string();
    }

    // get due actions, ignoring those are too far in the future, and sort them by due date
    let mut due_actions = md_files
        .iter()
        .flat_map(|md_file| {
            md_file
                .due_actions
                .iter()
                .map(move |due_action| (md_file, due_action))
        })
        .filter(|(_, due_action)| {
            let remaining_days = (due_action.date - *now).whole_days();
            remaining_days < config.due_action.ignore_future_days
        })
        .collect::<Vec<_>>();

    let mut html = String::new();

    // render due actions
    html.push_str("<h2>Due actions</h2>");
    if due_actions.is_empty() {
        html.push_str("None");
    } else {
        due_actions.sort_by_key(|(_, due_action)| due_action.date);
        html.push_str("<ul>");
        for (md_file, due_action) in due_actions {
            let style = due_action.get_css_style(now, &config.due_action);
            let _ = write!(
                html,
                r#"<li><span class="mynotes-date {style}">{}</span> {} - <a href="{}">{}</a></li>"#,
                due_action.date.format(&Iso8601::DATE).unwrap(),
                due_action.action,
                md_file.href,
                md_file.title
            );
        }
        html.push_str("</ul>");
    }

    // render notes index
    html.push_str("<h2>Notes</h2>");
    html.push_str("<ul>");
    for md_file in md_files {
        let _ = write!(
            html,
            r#"<li><a href="{}">{}</a></li>"#,
            md_file.href, md_file.title
        );
    }
    html.push_str("</ul>");

    html
}
