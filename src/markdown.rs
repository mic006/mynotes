use std::sync::LazyLock;

use pulldown_cmark::Parser;
use regex::Regex;
use time::Date;

use crate::config::AppConfig;
use crate::settings;

/// Action to perform on a due date.
pub struct DueAction {
    pub date: Date,
    pub action: String,
}

/// Content of a markdown file
pub struct MarkdownFile {
    pub title: String,
    pub due_actions: Vec<DueAction>,
    pub html: Option<String>,
}

impl MarkdownFile {
    pub fn read(rel_path: &str, with_html: bool, cfg: &AppConfig) -> Option<Self> {
        let path = cfg.content_path.join(rel_path);
        let content = std::fs::read_to_string(path).ok()?;

        let (title, body) = Self::extract_title(&content)?;
        let mut body = body.to_string();
        let due_actions = settings::user_process_markdown(&mut body, with_html, cfg);

        let html = if with_html {
            Self::pre_render_md_patch(&mut body, rel_path);
            let parser = Parser::new_ext(&body, settings::get_markdown_options());
            let mut html = String::new();
            pulldown_cmark::html::push_html(&mut html, parser);
            Some(html)
        } else {
            None
        };
        Some(MarkdownFile {
            title: title.to_string(),
            due_actions,
            html,
        })
    }

    fn extract_title(content: &str) -> Option<(&str, &str)> {
        static RE_TITLE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?s)^# ([^\n]*)\n\n(.*)$").unwrap());
        let (_, [title, body]) = RE_TITLE.captures(content)?.extract();
        Some((title, body))
    }

    /// Patch Markdown content before rendering to HTML
    fn pre_render_md_patch(md_content: &mut String, rel_path: &str) {
        Self::patch_md_todo_items(md_content, rel_path);
    }

    /** MD patch: handle TODO items
     *
     * pulldown-cmark can render TODO items, but there is no flexibility.
     *
     * Wanted behavior:
     * - put input inside a label element to be cleaner
     * - add data-* attributes to allow user check and server update
     */
    fn patch_md_todo_items(md_content: &mut String, rel_path: &str) {
        static RE_TODO_ITEM: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?m)^(\s*)-\s*\[([ x])\]\s*(.*)$").unwrap());

        *md_content = RE_TODO_ITEM
            .replace_all(md_content, |caps: &regex::Captures<'_>| {
                let (_, [indent, checked, text]) = caps.extract();
                let checked = checked == "x";
                format!(
                    r#"{indent}- <label><input type="checkbox"{} data-path="{rel_path}" data-label="{text}"> {text}</label>"#,
                    if checked { " checked" } else { "" }
                )
            })
            .to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_success() {
        let content = "# My Title\n\nThis is the body content.";
        let result = MarkdownFile::extract_title(content);
        assert!(result.is_some());
        let (title, body) = result.unwrap();
        assert_eq!(title, "My Title");
        assert_eq!(body, "This is the body content.");
    }

    #[test]
    fn test_extract_title_no_header() {
        let content = "Just a body\n\nNo title here.";
        let result = MarkdownFile::extract_title(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_title_missing_double_newline() {
        let content = "# Title\nBody without double newline";
        let result = MarkdownFile::extract_title(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_title_empty_body() {
        let content = "# Title\n\n";
        let (title, body) = MarkdownFile::extract_title(content).expect("Should parse");
        assert_eq!(title, "Title");
        assert_eq!(body, "");
    }

    #[test]
    fn test_extract_title_multiline_body() {
        let content = "# Title\n\nLine 1\nLine 2\nLine 3";
        let (title, body) = MarkdownFile::extract_title(content).expect("Should parse");
        assert_eq!(title, "Title");
        assert_eq!(body, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_patch_md_todo_items() {
        let mut content = String::from(
            "- [ ] Unchecked\n\
             - [x] Checked\n\
               - [ ] Indented\n\
             Not a todo item",
        );
        let rel_path = "test.md";
        MarkdownFile::patch_md_todo_items(&mut content, rel_path);

        let expected = "- <label><input type=\"checkbox\" data-path=\"test.md\" data-label=\"Unchecked\"> Unchecked</label>\n\
                        - <label><input type=\"checkbox\" checked data-path=\"test.md\" data-label=\"Checked\"> Checked</label>\n\
                          - <label><input type=\"checkbox\" data-path=\"test.md\" data-label=\"Indented\"> Indented</label>\n\
                        Not a todo item";
        assert_eq!(content, expected);
    }
}
