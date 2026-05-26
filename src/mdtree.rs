//! Manage the tree of Markdown files

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::SystemTime;

use regex::Regex;
use time::Date;
use time::format_description::well_known::Iso8601;

/// TODO item pattern
///
/// Use github markdown todo item = checkbox in a list item
///
/// Examples:
/// - [x] text
///   - [ ] text
pub static RE_TODO_ITEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(\s*)-\s*\[([ x])\]\s*(.*)$").unwrap());

/// Due action pattern
///
/// Use a TODO item, with a date using ISO8601 format at the beginning of the text
/// Only unchecked (not done) items are matching
///
/// Example:
/// - [ ] 2026-06-06 some action
pub static RE_DUE_ACTION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(\s*)-\s*\[ \]\s*(\d{4}-\d{2}-\d{2}) (.*)$").unwrap());

/// Access to the markdown files tree
///
/// Content is cached inside the object, but is always fresh:
/// mtime is checked on any access and cache is updated when needed
pub struct MdTree {
    root: Dir,
    content_path: PathBuf,
}

impl MdTree {
    /// Create a new cache by walking the content directory.
    pub fn new(content_path: PathBuf) -> Self {
        let mut tree = Self {
            root: Dir::default(),
            content_path,
        };
        tree.refresh();
        tree
    }

    /// Refresh the cache: add new files, update changed ones, remove deleted ones.
    fn refresh(&mut self) {
        self.root.refresh(&self.content_path, &self.content_path);
    }

    /// Get access to one Markdown file
    pub fn get_md_file(&mut self, path: &str) -> Option<&MdFile> {
        self.refresh();
        let mut current_dir = &self.root;
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            return None;
        }

        for (i, part) in parts.iter().enumerate() {
            match current_dir.children.get(*part) {
                Some(Node::Dir(dir)) => {
                    current_dir = dir;
                }
                Some(Node::File(file)) => {
                    if i == parts.len() - 1 {
                        return Some(file);
                    }
                    return None;
                }
                None => return None,
            }
        }
        None
    }

    /// Returns an iterator over all Markdown files in the tree.
    pub fn md_files_iter(&mut self) -> impl Iterator<Item = &MdFile> {
        self.refresh();
        let mut stack = vec![self.root.children.values()];
        std::iter::from_fn(move || {
            while let Some(top_iter) = stack.last_mut() {
                match top_iter.next() {
                    Some(Node::File(file)) => return Some(file),
                    Some(Node::Dir(dir)) => stack.push(dir.children.values()),
                    None => {
                        stack.pop();
                    }
                }
            }
            None
        })
    }
}

/// A directory entry in the tree.
#[derive(Default)]
struct Dir {
    children: BTreeMap<String, Node>,
}

impl Dir {
    fn refresh(&mut self, current_path: &Path, base_path: &Path) {
        let mut seen = HashSet::new();

        if let Ok(read_dir) = std::fs::read_dir(current_path) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                seen.insert(name.clone());

                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        if !matches!(self.children.get(&name), Some(Node::Dir(_))) {
                            self.children
                                .insert(name.clone(), Node::Dir(Dir::default()));
                        }
                        if let Some(Node::Dir(dir)) = self.children.get_mut(&name) {
                            dir.refresh(&path, base_path);
                        }
                    } else if path.extension().is_some_and(|ext| ext == "md") {
                        let mtime = std::fs::metadata(&path)
                            .and_then(|m| m.modified())
                            .unwrap_or(SystemTime::UNIX_EPOCH);

                        let needs_update = match self.children.get(&name) {
                            Some(Node::File(f)) => f.mtime != mtime,
                            _ => true,
                        };

                        if needs_update {
                            let rel_path = path
                                .strip_prefix(base_path)
                                .unwrap_or(&path)
                                .to_string_lossy()
                                .replace('\\', "/");
                            if let Some(md_file) = MdFile::read(base_path, &rel_path) {
                                self.children.insert(name, Node::File(md_file));
                            }
                        }
                    }
                }
            }
        }

        // Cleanup deleted entries and empty directories
        self.children.retain(|name, node| {
            if !seen.contains(name) {
                return false;
            }
            if let Node::Dir(dir) = node {
                return !dir.children.is_empty();
            }
            true
        });
    }
}

/// A node in the directory tree for the index page.
enum Node {
    Dir(Dir),
    File(MdFile),
}

/// A markdown file entry in the tree.
pub struct MdFile {
    /// last modification time when the file was read
    mtime: SystemTime,
    /// relative path, used for href link
    pub href: String,
    /// Markdown title, including parent path (parent/title)
    pub title: String,
    /// Due actions in the file
    pub due_actions: Vec<DueAction>,
    /// Raw Markdown body, before any rendering
    raw_md_body: String,
}

impl MdFile {
    /// Read file
    pub fn read(base_path: &Path, rel_path: &str) -> Option<Self> {
        let path = base_path.join(rel_path);
        let content = std::fs::read_to_string(&path).ok()?;

        let (title, body) = Self::split_title_body(&content)?;
        let due_actions = Self::get_due_actions(body);

        let parent = Path::new(&rel_path).parent().unwrap_or(Path::new(""));
        let title = if parent.as_os_str().is_empty() {
            title.to_string()
        } else {
            format!("{}/{title}", parent.display())
        };

        Some(Self {
            mtime: std::fs::metadata(&path).ok()?.modified().ok()?,
            href: rel_path.to_string(),
            title,
            due_actions,
            raw_md_body: body.to_string(),
        })
    }

    fn split_title_body(content: &str) -> Option<(&str, &str)> {
        static RE_TITLE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?s)^# ([^\n]*)\n\n(.*)$").unwrap());
        let (_, [title, body]) = RE_TITLE.captures(content)?.extract();
        Some((title, body))
    }

    fn get_due_actions(content: &str) -> Vec<DueAction> {
        RE_DUE_ACTION
            .captures_iter(content)
            .map(|caps| {
                let (_, [_, date, action]) = caps.extract();
                DueAction {
                    date: Date::parse(date, &Iso8601::DATE).unwrap(),
                    action: action.to_string(),
                }
            })
            .collect()
    }
}

/// Action to perform on a due date.
#[derive(PartialEq, Debug)]
pub struct DueAction {
    pub date: Date,
    pub action: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_title_body_success() {
        let content = "# My Title\n\nThis is the body content.";
        let result = MdFile::split_title_body(content);
        assert!(result.is_some());
        let (title, body) = result.unwrap();
        assert_eq!(title, "My Title");
        assert_eq!(body, "This is the body content.");
    }

    #[test]
    fn test_split_title_body_no_header() {
        let content = "Just a body\n\nNo title here.";
        let result = MdFile::split_title_body(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_split_title_body_missing_double_newline() {
        let content = "# Title\nBody without double newline";
        let result = MdFile::split_title_body(content);
        assert!(result.is_none());
    }

    #[test]
    fn test_split_title_body_empty_body() {
        let content = "# Title\n\n";
        let (title, body) = MdFile::split_title_body(content).expect("Should parse");
        assert_eq!(title, "Title");
        assert_eq!(body, "");
    }

    #[test]
    fn test_split_title_body_multiline_body() {
        let content = "# Title\n\nLine 1\nLine 2\nLine 3";
        let (title, body) = MdFile::split_title_body(content).expect("Should parse");
        assert_eq!(title, "Title");
        assert_eq!(body, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_get_due_actions() {
        let content = r"
- [ ] 2026-01-01 First action
- [x] 2026-01-02 Completed action (ignore)
  - [ ] 2026-01-03 Indented action
Some other text without a task.
";
        let actions = MdFile::get_due_actions(content);

        let expected_actions = vec![
            DueAction {
                date: Date::parse("2026-01-01", &Iso8601::DATE).unwrap(),
                action: "First action".to_string(),
            },
            DueAction {
                date: Date::parse("2026-01-03", &Iso8601::DATE).unwrap(),
                action: "Indented action".to_string(),
            },
        ];
        assert_eq!(actions, expected_actions);
    }
}
