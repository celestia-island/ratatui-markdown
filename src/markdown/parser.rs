use super::{types::MarkdownBlock, MarkdownRenderer};

const MD_FENCE: &str = "```";
const MD_HRULE_DASH: &str = "---";
const MD_HRULE_STAR: &str = "***";
const MD_HRULE_UNDERSCORE: &str = "___";
const MD_H3: &str = "### ";
const MD_H2: &str = "## ";
const MD_H1: &str = "# ";
const MD_LIST_DASH: &str = "- ";
const MD_LIST_STAR: &str = "* ";
const MD_LIST_PLUS: &str = "+ ";

impl MarkdownRenderer {
    pub fn parse(&self, markdown: &str) -> Vec<MarkdownBlock> {
        let mut blocks = Vec::new();
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_content = String::new();
        let mut paragraph_lines: Vec<String> = Vec::new();
        let mut table_buffer: Vec<String> = Vec::new();

        for line in markdown.lines() {
            if in_code_block {
                if line.trim().starts_with(MD_FENCE) {
                    in_code_block = false;
                    blocks.push(MarkdownBlock::CodeBlock(
                        code_lang.clone(),
                        code_content.trim_end().to_string(),
                    ));
                    code_lang.clear();
                    code_content.clear();
                } else {
                    code_content.push_str(line);
                    code_content.push('\n');
                }
                continue;
            }

            if line.trim().starts_with(MD_FENCE) {
                Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
                in_code_block = true;
                code_lang = line.trim().chars().skip(3).collect::<String>();
                continue;
            }

            let trimmed = line.trim();

            if trimmed.is_empty() {
                Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
                if !paragraph_lines.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(paragraph_lines.clone()));
                    paragraph_lines.clear();
                }
                blocks.push(MarkdownBlock::BlankLine);
                continue;
            }

            if trimmed.starts_with(MD_HRULE_DASH)
                || trimmed.starts_with(MD_HRULE_STAR)
                || trimmed.starts_with(MD_HRULE_UNDERSCORE)
            {
                Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
                if !paragraph_lines.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(paragraph_lines.clone()));
                    paragraph_lines.clear();
                }
                blocks.push(MarkdownBlock::HorizontalRule);
                continue;
            }

            if line.starts_with(MD_H3) {
                Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
                if !paragraph_lines.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(paragraph_lines.clone()));
                    paragraph_lines.clear();
                }
                let text = trimmed.chars().skip(4).collect::<String>();
                blocks.push(MarkdownBlock::Heading3(text));
                continue;
            }

            if line.starts_with(MD_H2) {
                Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
                if !paragraph_lines.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(paragraph_lines.clone()));
                    paragraph_lines.clear();
                }
                let text = trimmed.chars().skip(3).collect::<String>();
                blocks.push(MarkdownBlock::Heading2(text));
                continue;
            }

            if line.starts_with(MD_H1) {
                Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
                if !paragraph_lines.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(paragraph_lines.clone()));
                    paragraph_lines.clear();
                }
                let text = trimmed.chars().skip(2).collect::<String>();
                blocks.push(MarkdownBlock::Heading1(text));
                continue;
            }

            if trimmed.starts_with('>') {
                Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
                if !paragraph_lines.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(paragraph_lines.clone()));
                    paragraph_lines.clear();
                }
                let content = trimmed
                    .strip_prefix('>')
                    .unwrap_or(trimmed)
                    .trim_start()
                    .to_string();
                blocks.push(MarkdownBlock::Blockquote(content));
                continue;
            }

            let list_indent = Self::count_list_indent(line);
            if trimmed.starts_with(MD_LIST_DASH)
                || trimmed.starts_with(MD_LIST_STAR)
                || trimmed.starts_with(MD_LIST_PLUS)
            {
                Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
                if !paragraph_lines.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(paragraph_lines.clone()));
                    paragraph_lines.clear();
                }
                let content = trimmed.chars().skip(2).collect::<String>();
                blocks.push(MarkdownBlock::ListItem(content, list_indent));
                continue;
            }

            if let Some(pos) = trimmed.find(". ") {
                let prefix = &trimmed[..pos];
                if pos > 0 && pos < 5 && prefix.parse::<u32>().is_ok() {
                    Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
                    if !paragraph_lines.is_empty() {
                        blocks.push(MarkdownBlock::Paragraph(paragraph_lines.clone()));
                        paragraph_lines.clear();
                    }
                    let content = trimmed[pos + 2..].to_string();
                    blocks.push(MarkdownBlock::ListItem(content, list_indent));
                    continue;
                }
            }

            if Self::is_table_line(trimmed) {
                if !paragraph_lines.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(paragraph_lines.clone()));
                    paragraph_lines.clear();
                }
                table_buffer.push(trimmed.to_string());
                continue;
            }

            Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
            paragraph_lines.push(trimmed.to_string());
        }

        Self::flush_table(&mut table_buffer, &mut blocks, &mut paragraph_lines);
        if !paragraph_lines.is_empty() {
            blocks.push(MarkdownBlock::Paragraph(paragraph_lines));
        }

        if in_code_block {
            blocks.push(MarkdownBlock::CodeBlock(
                code_lang,
                code_content.trim_end().to_string(),
            ));
        }

        blocks
    }

    fn is_table_line(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.contains('|') {
            return false;
        }
        if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 1 {
            return true;
        }
        let pipe_count = trimmed.chars().filter(|&c| c == '|').count();
        if pipe_count >= 2 {
            let non_sep = trimmed
                .chars()
                .filter(|c| *c != '|' && *c != '-' && *c != ':' && *c != ' ')
                .count();
            if non_sep > 0 {
                return true;
            }
            let sep_chars: Vec<char> = trimmed.chars().filter(|c| *c != '|' && *c != ' ').collect();
            if !sep_chars.is_empty() && sep_chars.iter().all(|c| *c == '-' || *c == ':') {
                return true;
            }
        }
        false
    }

    fn flush_table(
        table_buffer: &mut Vec<String>,
        blocks: &mut Vec<MarkdownBlock>,
        paragraph_lines: &mut Vec<String>,
    ) {
        if table_buffer.is_empty() {
            return;
        }
        if table_buffer.len() < 2 {
            for line in table_buffer.drain(..) {
                paragraph_lines.push(line);
            }
            return;
        }
        let separator_idx = table_buffer.iter().position(|l| {
            l.chars()
                .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
        });
        if separator_idx.is_none() || separator_idx == Some(0) {
            for line in table_buffer.drain(..) {
                paragraph_lines.push(line);
            }
            return;
        }
        let sep_pos = separator_idx.unwrap_or(0);
        let headers: Vec<String> = if sep_pos > 0 {
            Self::split_table_row(&table_buffer[sep_pos - 1])
        } else {
            vec![]
        };
        let sep = sep_pos + 1;
        let rows: Vec<Vec<String>> = if sep < table_buffer.len() {
            table_buffer[sep..]
                .iter()
                .filter(|l| {
                    !l.chars()
                        .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
                })
                .map(|l| Self::split_table_row(l))
                .collect()
        } else {
            vec![]
        };
        blocks.push(MarkdownBlock::Table { headers, rows });
        table_buffer.clear();
    }

    fn split_table_row(line: &str) -> Vec<String> {
        let trimmed = line.trim();
        let inner = if trimmed.starts_with('|') && trimmed.ends_with('|') && trimmed.len() > 1 {
            &trimmed[1..trimmed.len() - 1]
        } else if let Some(rest) = trimmed.strip_prefix('|') {
            rest
        } else if trimmed.ends_with('|') && trimmed.len() > 1 {
            &trimmed[..trimmed.len() - 1]
        } else {
            trimmed
        };
        inner
            .split('|')
            .map(|cell| cell.trim().to_string())
            .collect()
    }

    fn count_list_indent(line: &str) -> u8 {
        let spaces = line.chars().take_while(|&c| c == ' ').count();
        (spaces / 2).min(255) as u8
    }
}
