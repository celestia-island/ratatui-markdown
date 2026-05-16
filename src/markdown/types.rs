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
        children: Vec<Self>,
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
            children: vec![Self::Paragraph(vec![text.into()])],
            header_override: None,
            footer_override: None,
        }
    }

    pub fn blockquote(level: u8, children: Vec<Self>) -> Self {
        Self::Blockquote {
            level,
            children,
            header_override: None,
            footer_override: None,
        }
    }

    pub fn blockquote_with_overrides(
        level: u8,
        children: Vec<Self>,
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
            Self::Heading1(_)
            | Self::Heading2(_)
            | Self::Heading3(_)
            | Self::InlineCode(_)
            | Self::HorizontalRule
            | Self::BlankLine => 1,
            Self::Paragraph(lines) => lines.len().max(1),
            Self::CodeBlock { code, .. } => code.lines().count().max(1) + 2,
            Self::ListItem(_, _) | Self::TaskItem { .. } => 1,
            Self::Blockquote {
                children,
                header_override,
                footer_override,
                ..
            } => {
                let base = children
                    .iter()
                    .map(|c| c.line_count())
                    .sum::<usize>()
                    .max(1);
                let extra = header_override.as_ref().map_or(0, |_| 1)
                    + footer_override.as_ref().map_or(0, |_| 1);
                base + extra
            }
            Self::Table { rows, .. } => {
                let header_lines = 2;
                let row_lines = rows.len() * 2 + 1;
                header_lines + row_lines
            }
            Self::Image { .. } => 1,
        }
    }
}

#[derive(Debug)]
pub(crate) enum TextToken {
    Word(String),
    Space,
    Newline,
}
