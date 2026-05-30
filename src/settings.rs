use std::sync::LazyLock;

use pulldown_cmark::Options;
use regex::{Captures, Regex};

/// Markdown options used to transform markdown to HTML.
/// See <https://docs.rs/pulldown-cmark/latest/pulldown_cmark/struct.Options.html>
pub fn get_markdown_options() -> Options {
    Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_GFM
}
/// Process markdown body
///
/// - format money values
pub fn user_process_markdown(body: &mut String) {
    {
        static RE_MONEY: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(-?\d+(?:\.\d{2})?)€").unwrap());

        let body_modified = RE_MONEY.replace_all(body, |caps: &Captures<'_>| {
            render_money(caps.get(1).unwrap().as_str())
        });
        *body = body_modified.to_string();
    }
}

// Add span around money
pub fn render_money(s: &str) -> String {
    format!(r#"<span class="mynotes-money">{s} €</span>"#)
}
