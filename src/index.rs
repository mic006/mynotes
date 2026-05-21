use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use time::{Date, OffsetDateTime};

use crate::config::AppConfig;
use crate::markdown::{DueAction, MarkdownFile};

/// A node in the directory tree for the index page.
pub enum Node {
    Dir(Dir),
    File(MdFile),
}

/// A directory entry in the tree.
#[derive(Default)]
pub struct Dir {
    pub children: BTreeMap<String, Node>,
}

/// A markdown file entry in the tree.
pub struct MdFile {
    pub file: MarkdownFile,
    pub href: String,
}

struct DueActionItem<'a> {
    md_file: &'a MdFile,
    due_action: &'a DueAction,
    html: String,
}

impl Dir {
    /// Render the index page body
    pub fn render(&self, html: &mut String, cfg: &AppConfig) {
        if self.children.is_empty() {
            html.push_str("<h2>No content</h2>");
            return;
        }

        // collect all due actions
        let now = OffsetDateTime::now_utc().date();
        let mut due_actions = Vec::new();
        self.get_due_actions_recursive(&mut due_actions, &now, cfg);
        if !due_actions.is_empty() {
            // sort by ascending due date
            due_actions.sort_by_key(|due_action| due_action.due_action.date);

            html.push_str("<h2>Due actions</h2>");
            html.push_str("<ul>");
            for due_action in due_actions {
                html.push_str("<li>");
                html.push_str(&due_action.html);
                let _ = write!(
                    html,
                    " - <a href=\"{}\">{}</a>",
                    due_action.md_file.href, due_action.md_file.file.title
                );
                html.push_str("</li>");
            }
            html.push_str("</ul>");
        }

        html.push_str("<h2>Notes</h2>");
        self.render_index_recursive(html);
    }

    fn render_index_recursive(&self, html: &mut String) {
        html.push_str("<ul>");
        for (name, node) in &self.children {
            html.push_str("<li>");
            match node {
                Node::Dir(dir) => {
                    html.push_str(name);
                    dir.render_index_recursive(html);
                }
                Node::File(md_file) => {
                    let _ = write!(
                        html,
                        "<a href=\"{}\">{}</a>",
                        md_file.href, md_file.file.title
                    );
                }
            }
            html.push_str("</li>");
        }
        html.push_str("</ul>");
    }

    fn get_due_actions_recursive<'a>(
        &'a self,
        due_actions: &mut Vec<DueActionItem<'a>>,
        now: &Date,
        cfg: &AppConfig,
    ) {
        for node in self.children.values() {
            match node {
                Node::Dir(dir) => dir.get_due_actions_recursive(due_actions, now, cfg),
                Node::File(md_file) => {
                    for due_action in &md_file.file.due_actions {
                        if let Some(html) = due_action.render_in_index(now, &cfg.due_action) {
                            due_actions.push(DueActionItem {
                                md_file,
                                due_action,
                                html,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Recursively walk the directory to build a tree of markdown files.
pub fn walk(current_path: PathBuf, base_path: &Path, dir: &mut Dir, cfg: &AppConfig) {
    if let Ok(read_dir) = std::fs::read_dir(&current_path) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let mut child_dir = Dir::default();
                    walk(path, base_path, &mut child_dir, cfg);
                    if !child_dir.children.is_empty() {
                        dir.children.insert(name, Node::Dir(child_dir));
                    }
                } else if path.extension().is_some_and(|ext| ext == "md")
                    && let Some(md) = MarkdownFile::read(&path, false, cfg)
                {
                    let rel_path = path
                        .strip_prefix(base_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/");
                    dir.children.insert(
                        name,
                        Node::File(MdFile {
                            file: md,
                            href: rel_path,
                        }),
                    );
                }
            }
        }
    }
}
