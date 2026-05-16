#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownBlock {
    Heading1(String),
    Heading2(String),
    Heading3(String),
    Paragraph(Vec<String>),
    CodeBlock {
        lang: String,
        code: String,
        header_override: Option<String>,
        footer_override: Option<String>,
        prefix_override: Option<String>,
    },
    InlineCode(String),
    ListItem(String, u8),
    TaskItem {
        text: String,
        indent: u8,
        checked: bool,
    },
    Blockquote {
        level: u8,
        children: Vec<MarkdownBlock>,
        header_override: Option<String>,
        footer_override: Option<String>,
    },
    HorizontalRule,
    BlankLine,
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Image {
        alt: String,
        path: String,
    },
}

impl MarkdownBlock {
    pub fn code_block(lang: impl Into<String>, code: impl Into<String>) -> Self {
        Self::CodeBlock {
            lang: lang.into(),
            code: code.into(),
            header_override: None,
            footer_override: None,
            prefix_override: None,
        }
    }

    pub fn blockquote_text(text: impl Into<String>) -> Self {
        Self::Blockquote {
            level: 1,
            children: vec![MarkdownBlock::Paragraph(vec![text.into()])],
            header_override: None,
            footer_override: None,
        }
    }

    pub fn blockquote(level: u8, children: Vec<MarkdownBlock>) -> Self {
        Self::Blockquote {
            level,
            children,
            header_override: None,
            footer_override: None,
        }
    }

    pub fn blockquote_with_overrides(
        level: u8,
        children: Vec<MarkdownBlock>,
        header_override: Option<String>,
        footer_override: Option<String>,
    ) -> Self {
        Self::Blockquote {
            level,
            children,
            header_override,
            footer_override,
        }
    }

    pub fn line_count(&self) -> usize {
        match self {
            MarkdownBlock::Heading1(_)
            | MarkdownBlock::Heading2(_)
            | MarkdownBlock::Heading3(_)
            | MarkdownBlock::InlineCode(_)
            | MarkdownBlock::HorizontalRule
            | MarkdownBlock::BlankLine => 1,
            MarkdownBlock::Paragraph(lines) => lines.len().max(1),
            MarkdownBlock::CodeBlock { code, .. } => code.lines().count().max(1) + 2,
            MarkdownBlock::ListItem(_, _) | MarkdownBlock::TaskItem { .. } => 1,
            MarkdownBlock::Blockquote { children, header_override, footer_override, .. } => {
                let base = children
                    .iter()
                    .map(|c| c.line_count())
                    .sum::<usize>()
                    .max(1);
                let extra = header_override.as_ref().map_or(0, |_| 1)
                    + footer_override.as_ref().map_or(0, |_| 1);
                base + extra
            }
            MarkdownBlock::Table { rows, .. } => {
                let header_lines = 2;
                let row_lines = rows.len() * 2 + 1;
                header_lines + row_lines
            }
            MarkdownBlock::Image { .. } => 1,
        }
    }
}

#[derive(Debug)]
pub(crate) enum TextToken {
    Word(String),
    Space,
    Newline,
}
