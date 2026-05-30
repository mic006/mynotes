use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::SystemTime;

use pulldown_cmark::Parser;
use time::Date;
use time::format_description::well_known::Iso8601;

use crate::config::AppConfig;
use crate::mdtree::{CheckboxTask, MdFile, MdTree};
use crate::settings;

/// Cache for HTML template file
pub struct HtmlTemplate {
    /// File path,
    path: PathBuf,
    /// last modification time when the file was read
    mtime: SystemTime,
    /// Content of the template file
    content: String,
}
impl HtmlTemplate {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            mtime: SystemTime::UNIX_EPOCH,
            content: String::new(),
        }
    }

    /// Get template content
    pub fn get_content(&mut self) -> std::io::Result<&str> {
        let current_mtime = std::fs::metadata(&self.path)?.modified()?;
        if self.mtime != current_mtime {
            self.content = std::fs::read_to_string(&self.path)?;
            self.mtime = current_mtime;
        }
        Ok(&self.content)
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
    let _ = write!(html, "<h2>{}</h2>", config.due_action.title);
    if due_actions.is_empty() {
        html.push_str("None");
    } else {
        due_actions.sort_by_key(|(_, due_action)| due_action.date);
        html.push_str("<ul>");
        for (md_file, due_action) in due_actions {
            let style = config.due_action.get_css_style(now, &due_action.date);
            let _ = write!(
                html,
                r#"<li><label><input type="checkbox" data-url="/{}" data-label="{}"> <span class="mynotes-date {style}">{}</span> {} - <a href="{}">{}</a></label></li>"#,
                md_file.href,
                due_action.action,
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

/// Get HTML body for Markdown page
pub fn get_body_md(md_file: &MdFile, config: &AppConfig, now: &Date) -> String {
    let mut md_content = md_file.raw_md_body.clone();

    // patches before MD to HTML transformation
    patch_md_checkbox_tasks(&mut md_content, &md_file.href, config, now);
    settings::user_process_markdown(&mut md_content);

    // MD to HTML
    let parser = Parser::new_ext(&md_content, settings::get_markdown_options());
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);

    html
}

/** MD patch: handle checkbox tasks
 *
 * pulldown-cmark can render TODO items, but there is no flexibility.
 *
 * Wanted behavior:
 * - put input inside a label element to be cleaner
 * - add data-* attributes to allow user check and server update
 * - highlight due date if present
 */
fn patch_md_checkbox_tasks(
    md_content: &mut String,
    rel_path: &str,
    config: &AppConfig,
    now: &Date,
) {
    *md_content = CheckboxTask::replace(md_content, |ct|{
        let opt_date_str=  ct.parse_date().map(|date| {
            let style = config.due_action.get_css_style(now, &date);
            format!(r#" <span class="mynotes-date {style}">{}</span>"#, ct.date.unwrap())
        }
        );
        Some(format!(
            r#"{}- <label><input type="checkbox"{} data-url="/{rel_path}" data-label="{}">{} {}</label>"#,
            ct.indent,
            if ct.checked { " checked" } else { "" },
            ct.text,
            opt_date_str.as_ref().map_or("", |s| s),
            ct.text
        ))}
    ).to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_md_checkbox_tasks() {
        let mut content = String::from(
            r"- [ ] Unchecked
- [x] Checked
  - [ ] Indented
- [ ] 2026-05-30 Today
- [x] 2027-06-06 Far future
Not a todo item",
        );
        let rel_path = "test.md";
        let config = AppConfig::default();
        let now = Date::parse("2026-05-30", &Iso8601::DATE).unwrap();
        patch_md_checkbox_tasks(&mut content, rel_path, &config, &now);

        let expected = r#"- <label><input type="checkbox" data-url="/test.md" data-label="Unchecked"> Unchecked</label>
- <label><input type="checkbox" checked data-url="/test.md" data-label="Checked"> Checked</label>
  - <label><input type="checkbox" data-url="/test.md" data-label="Indented"> Indented</label>
- <label><input type="checkbox" data-url="/test.md" data-label="Today"> <span class="mynotes-date mynotes-date-warn">2026-05-30</span> Today</label>
- <label><input type="checkbox" checked data-url="/test.md" data-label="Far future"> <span class="mynotes-date mynotes-date-ok">2027-06-06</span> Far future</label>
Not a todo item"#;
        assert_eq!(content, expected);
    }
}
