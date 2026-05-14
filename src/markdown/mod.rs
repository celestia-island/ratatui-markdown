mod inline;
mod parser;
mod render;
#[cfg(test)]
mod render_tests;
#[cfg(test)]
mod tests;
mod text;
mod types;

pub use inline::parse_inline_formatting;
pub use types::MarkdownBlock;

pub struct MarkdownRenderer {
    pub(crate) max_width: usize,
}

impl MarkdownRenderer {
    pub fn new(max_width: usize) -> Self {
        Self { max_width }
    }
}
