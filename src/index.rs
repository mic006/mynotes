use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// A node in the directory tree for the index page.
#[derive(Default)]
pub struct Node {
    pub title: Option<String>,
    pub href: Option<String>,
    pub children: BTreeMap<String, Node>,
}

impl Node {
    /// Recursively render the tree as nested HTML lists.
    pub fn render(&self, html: &mut String) {
        if self.children.is_empty() {
            return;
        }
        html.push_str("<ul>");
        for (name, child) in &self.children {
            html.push_str("<li>");
            if let (Some(title), Some(href)) = (&child.title, &child.href) {
                let _ = write!(html, "<a href=\"{href}\">{title}</a>");
            } else {
                html.push_str(name);
            }
            child.render(html);
            html.push_str("</li>");
        }
        html.push_str("</ul>");
    }
}

/// Extract the title from a markdown file's first line or use its filename.
async fn get_title(path: &Path) -> String {
    if let Ok(content) = rocket::tokio::fs::read_to_string(path).await
        && let Some(line) = content.lines().next()
        && let Some(title) = line.trim().strip_prefix("# ")
    {
        return title.trim().to_string();
    }
    // fallback: use filename
    path.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Recursively walk the directory to build a tree of markdown files.
pub async fn walk(current_path: PathBuf, base_path: &Path, node: &mut Node) {
    if let Ok(mut read_dir) = rocket::tokio::fs::read_dir(&current_path).await {
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if let Ok(ft) = entry.file_type().await {
                if ft.is_dir() {
                    let mut child_node = Node::default();
                    Box::pin(walk(path, base_path, &mut child_node)).await;
                    if !child_node.children.is_empty() {
                        node.children.insert(name, child_node);
                    }
                } else if path.extension().is_some_and(|ext| ext == "md") {
                    let title = get_title(&path).await;
                    let rel_path = path
                        .strip_prefix(base_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/");
                    node.children.insert(
                        name,
                        Node {
                            title: Some(title),
                            href: Some(rel_path),
                            children: BTreeMap::new(),
                        },
                    );
                }
            }
        }
    }
}
