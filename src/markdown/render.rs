use ratatui::{
    prelude::Stylize,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

#[cfg(feature = "image")]
use super::image::{ImageResolver, MarkdownRenderOutput};
use super::{
    inline::parse_inline_formatting,
    types::{MarkdownBlock, TextToken},
    MarkdownRenderer,
};
use crate::{
    constants::list_prefix::{
        BOTTOM_MID, CORNER_BL, CORNER_BR, CORNER_TL, CORNER_TR, CROSS_MID, HLINE, MID_LEFT,
        MID_RIGHT, ROUNDED_BL, ROUNDED_TL, TOP_MID, VLINE,
    },
    theme::RichTextTheme,
};

const LANG_MERMAID: &str = "mermaid";

fn find_parent_pos(items: &[(usize, u8)], pos: usize) -> Option<usize> {
    let indent = items[pos].1;
    if indent == 0 {
        return None;
    }
    let target = indent - 1;
    for j in (0..pos).rev() {
        if items[j].1 == target {
            return Some(j);
        } else if items[j].1 < target {
            break;
        }
    }
    None
}

fn has_sibling_after(items: &[(usize, u8)], pos: usize) -> bool {
    let indent = items[pos].1;
    let parent = find_parent_pos(items, pos);
    for j in (pos + 1)..items.len() {
        if items[j].1 == indent && find_parent_pos(items, j) == parent {
            return true;
        } else if items[j].1 < indent {
            break;
        }
    }
    false
}

fn default_image_fallback(alt: &str, path: &str) -> Line<'static> {
    let label = if alt.is_empty() {
        path.to_string()
    } else {
        alt.to_string()
    };
    Line::from(Span::styled(
        format!("[image: {label}]"),
        Style::default().italic().fg(Color::Gray),
    ))
}

#[cfg(feature = "image")]
struct ImageRenderContext<'a, I: ImageResolver> {
    lines: &'a mut Vec<Line<'static>>,
    placements: &'a mut Vec<super::image::ImagePlacement>,
    resolved_images: &'a [super::image::ResolvedImage],
    next_image_idx: &'a mut usize,
    resolver: &'a mut I,
    max_image_width: u16,
    max_image_height: u16,
}

impl MarkdownRenderer {
    pub fn render(
        &self,
        blocks: &[MarkdownBlock],
        theme: &impl RichTextTheme,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        for (block_idx, block) in blocks.iter().enumerate() {
            self.render_block(block, block_idx, theme, blocks, &mut lines);
        }

        lines
    }

    #[cfg(feature = "image")]
    pub fn render_full<I: ImageResolver>(
        &self,
        blocks: &[MarkdownBlock],
        theme: &impl RichTextTheme,
        resolved_images: &[super::image::ResolvedImage],
        resolver: &mut I,
        max_image_width: u16,
        max_image_height: u16,
    ) -> MarkdownRenderOutput {
        let mut output = MarkdownRenderOutput::new();
        let mut next_image_idx = 0;

        let mut ctx = ImageRenderContext {
            lines: &mut output.lines,
            placements: &mut output.images,
            resolved_images,
            next_image_idx: &mut next_image_idx,
            resolver,
            max_image_width,
            max_image_height,
        };

        for (block_idx, block) in blocks.iter().enumerate() {
            self.render_block_with_images(block, block_idx, theme, blocks, &mut ctx);
        }

        output
    }

    #[cfg(feature = "image")]
    fn render_block_with_images<I: ImageResolver>(
        &self,
        block: &MarkdownBlock,
        _block_idx: usize,
        theme: &impl RichTextTheme,
        _blocks: &[MarkdownBlock],
        ctx: &mut ImageRenderContext<I>,
    ) {
        match block {
            MarkdownBlock::Image { alt, path } => {
                if *ctx.next_image_idx < ctx.resolved_images.len()
                    && ctx.resolved_images[*ctx.next_image_idx].path == *path
                {
                    let ref_img = &ctx.resolved_images[*ctx.next_image_idx].image;
                    let (w_cells, h_cells) = ctx.resolver.cell_dimensions(
                        ref_img,
                        ctx.max_image_width,
                        ctx.max_image_height,
                    );
                    if w_cells > 0 && h_cells > 0 {
                        let row = ctx.lines.len();
                        for _ in 0..h_cells {
                            ctx.lines.push(Line::raw(""));
                        }
                        ctx.placements.push(super::image::ImagePlacement {
                            row,
                            col: 0,
                            width_cells: w_cells,
                            height_cells: h_cells,
                            image: ref_img.clone(),
                            crop: None,
                        });
                    } else {
                        ctx.lines.push(default_image_fallback(alt, path));
                    }
                    *ctx.next_image_idx += 1;
                } else {
                    let hooks = self.hooks.as_deref();
                    if let Some(h) = hooks {
                        if let Some(custom) = h.image_fallback(alt, path) {
                            ctx.lines.extend(custom);
                            return;
                        }
                    }
                    ctx.lines.push(Line::from(ctx.resolver.fallback(path, alt)));
                }
            }
            MarkdownBlock::CodeBlock { lang, code, .. } if lang == LANG_MERMAID => {
                let hooks = self.hooks.as_deref();
                if let Some(h) = hooks {
                    if let Some(img) = h.render_mermaid_image(code) {
                        let prefix_w: usize = 2;
                        let (w_cells, h_cells) = ctx.resolver.cell_dimensions(
                            &img,
                            ctx.max_image_width.saturating_sub(prefix_w as u16),
                            ctx.max_image_height,
                        );
                        if w_cells > 0 && h_cells > 0 {
                            let border_style = Style::default().fg(theme.get_muted_text_color());
                            ctx.lines.push(Line::from(Span::styled(
                                format!("{ROUNDED_TL}{HLINE} mermaid"),
                                border_style,
                            )));
                            let prefix = format!("{VLINE} ");
                            let row = ctx.lines.len();
                            for _ in 0..h_cells {
                                ctx.lines
                                    .push(Line::from(Span::styled(prefix.clone(), border_style)));
                            }
                            ctx.placements.push(super::image::ImagePlacement {
                                row,
                                col: prefix_w,
                                width_cells: w_cells,
                                height_cells: h_cells,
                                image: img,
                                crop: None,
                            });
                            ctx.lines.push(Line::from(Span::styled(
                                format!("{ROUNDED_BL}{HLINE}"),
                                border_style,
                            )));
                            return;
                        }
                    }
                }
                self.render_block(block, _block_idx, theme, _blocks, ctx.lines);
            }
            _ => self.render_block(block, _block_idx, theme, _blocks, ctx.lines),
        }
    }

    fn render_block(
        &self,
        block: &MarkdownBlock,
        block_idx: usize,
        theme: &impl RichTextTheme,
        blocks: &[MarkdownBlock],
        lines: &mut Vec<Line<'static>>,
    ) {
        let hooks = self.hooks.as_deref();

        match block {
            MarkdownBlock::Heading1(text) => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.heading1(text) {
                        lines.push(custom);
                        return;
                    }
                }
                let parsed = parse_inline_formatting(text, theme);
                let style = Style::default()
                    .fg(theme.get_primary_color())
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
                if parsed.is_empty() {
                    lines.push(Line::from(Span::styled(text.clone(), style)));
                } else {
                    let styled: Vec<Span<'static>> = parsed
                        .into_iter()
                        .map(|mut s| {
                            s.style = style.patch(s.style);
                            s
                        })
                        .collect();
                    lines.push(Line::from(styled));
                }
            }
            MarkdownBlock::Heading2(text) => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.heading2(text) {
                        lines.push(custom);
                        return;
                    }
                }
                let parsed = parse_inline_formatting(text, theme);
                let style = Style::default()
                    .fg(theme.get_text_color())
                    .add_modifier(Modifier::BOLD);
                if parsed.is_empty() {
                    lines.push(Line::from(Span::styled(text.clone(), style)));
                } else {
                    let styled: Vec<Span<'static>> = parsed
                        .into_iter()
                        .map(|mut s| {
                            s.style = style.patch(s.style);
                            s
                        })
                        .collect();
                    lines.push(Line::from(styled));
                }
            }
            MarkdownBlock::Heading3(text) => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.heading3(text) {
                        lines.push(custom);
                        return;
                    }
                }
                let parsed = parse_inline_formatting(text, theme);
                let style = Style::default()
                    .fg(theme.get_secondary_color())
                    .add_modifier(Modifier::BOLD);
                if parsed.is_empty() {
                    lines.push(Line::from(Span::styled(text.clone(), style)));
                } else {
                    let styled: Vec<Span<'static>> = parsed
                        .into_iter()
                        .map(|mut s| {
                            s.style = style.patch(s.style);
                            s
                        })
                        .collect();
                    lines.push(Line::from(styled));
                }
            }
            MarkdownBlock::Paragraph(paragraph_lines) => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.paragraph(paragraph_lines) {
                        lines.extend(custom);
                        return;
                    }
                }
                for pline in paragraph_lines {
                    let wrapped = self.wrap_text_with_inline_formatting(pline, theme);
                    lines.extend(wrapped);
                }
            }
            MarkdownBlock::CodeBlock {
                lang,
                code,
                header_override,
                footer_override,
                prefix_override,
            } => {
                let code = code.replace('\t', "    ");
                if let Some(h) = hooks {
                    if let Some(custom) = h.render_code_block(lang, &code) {
                        lines.extend(custom);
                        return;
                    }
                }

                if lang == LANG_MERMAID {
                    #[cfg(feature = "mermaid")]
                    {
                        let mermaid_width = self.max_width.saturating_sub(2);
                        let rendered =
                            crate::mermaid::render_mermaid(&code, mermaid_width, None, theme);
                        if let Some(mermaid_lines) = rendered {
                            let border_style = Style::default().fg(theme.get_muted_text_color());

                            lines.push(Line::from(Span::styled(
                                format!("{ROUNDED_TL}{HLINE} mermaid"),
                                border_style,
                            )));

                            let prefix = format!("{VLINE} ");
                            for ml in mermaid_lines {
                                let mut spans: Vec<Span<'static>> =
                                    vec![Span::styled(prefix.clone(), border_style)];
                                spans.extend(ml.spans);
                                lines.push(Line::from(spans));
                            }

                            lines.push(Line::from(Span::styled(
                                format!("{ROUNDED_BL}{HLINE}"),
                                border_style,
                            )));
                            return;
                        }
                    }
                    return;
                }

                let content_lines: Vec<&str> = code.lines().collect();
                let content_line_count = content_lines.len();

                if let Some(ref hdr) = header_override {
                    lines.push(Line::from(Span::styled(
                        hdr.clone(),
                        Style::default().fg(theme.get_muted_text_color()),
                    )));
                } else if let Some(h) = hooks {
                    if let Some(custom) = h.code_block_header(lang) {
                        lines.push(custom);
                    } else {
                        lines.push(self.default_code_block_header(lang, theme));
                    }
                } else {
                    lines.push(self.default_code_block_header(lang, theme));
                }

                let prefix = if let Some(ref pfx) = prefix_override {
                    pfx.clone()
                } else if let Some(h) = hooks {
                    h.code_block_line_prefix(lang)
                        .unwrap_or_else(|| format!("{VLINE} "))
                } else {
                    format!("{VLINE} ")
                };

                for (idx, code_line) in content_lines.iter().enumerate() {
                    if let Some(h) = hooks {
                        if let Some(custom) = h.code_block_line(code_line, idx, content_line_count)
                        {
                            lines.push(custom);
                            continue;
                        }
                    }
                    lines.push(Line::from(vec![
                        Span::styled(
                            prefix.clone(),
                            Style::default().fg(theme.get_muted_text_color()),
                        ),
                        Span::styled(
                            code_line.to_string(),
                            Style::default().fg(theme.get_accent_yellow()),
                        ),
                    ]));
                }

                if let Some(ref ftr) = footer_override {
                    lines.push(Line::from(Span::styled(
                        ftr.clone(),
                        Style::default().fg(theme.get_muted_text_color()),
                    )));
                } else if let Some(h) = hooks {
                    if let Some(custom) = h.code_block_footer(lang, content_line_count) {
                        lines.push(custom);
                    } else {
                        lines.push(self.default_code_block_footer(theme));
                    }
                } else {
                    lines.push(self.default_code_block_footer(theme));
                }
            }
            MarkdownBlock::InlineCode(code) => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.inline_code(code) {
                        lines.push(custom);
                        return;
                    }
                }
                lines.push(Line::from(Span::styled(
                    format!("`{}`", code),
                    Style::default().fg(theme.get_accent_yellow()),
                )));
            }
            MarkdownBlock::ListItem(text, indent) => {
                let (is_last, ancestors_are_last, index_in_group) =
                    Self::find_list_context(block_idx, blocks);

                if let Some(h) = hooks {
                    let marker =
                        h.list_item_marker(*indent, is_last, &ancestors_are_last, index_in_group);
                    if marker.is_some() || h.list_item_content(text, *indent).is_some() {
                        let marker_str = marker.unwrap_or_else(|| "\u{2022} ".to_string());
                        if let Some(custom_content) = h.list_item_content(text, *indent) {
                            for mut cline in custom_content {
                                let mut new_spans = vec![Span::raw(marker_str.clone())];
                                new_spans.append(&mut cline.spans);
                                cline.spans = new_spans;
                                lines.push(cline);
                            }
                        } else {
                            let marker_width = Self::string_width(&marker_str);
                            let cont_indent = h
                                .tree_continuation_prefix(*indent, &ancestors_are_last)
                                .unwrap_or_else(|| " ".repeat(marker_width));
                            let content_width = self.max_width.saturating_sub(marker_width);
                            let wrapped = if content_width > 0 {
                                let mut text_lines = Vec::new();
                                for text_line in text.split('\n') {
                                    let spans = parse_inline_formatting(text_line, theme);
                                    let wrapped_spans =
                                        Self::wrap_styled_spans_to_width(spans, content_width);
                                    text_lines.extend(wrapped_spans);
                                }
                                text_lines
                            } else {
                                vec![parse_inline_formatting(text, theme)]
                            };
                            for (i, span_line) in wrapped.into_iter().enumerate() {
                                let prefix = if i == 0 {
                                    Span::raw(marker_str.clone())
                                } else {
                                    Span::raw(cont_indent.clone())
                                };
                                let mut new_spans = vec![prefix];
                                new_spans.extend(span_line);
                                lines.push(Line::from(new_spans));
                            }
                        }
                        return;
                    }
                }

                let indent_str = "  ".repeat(*indent as usize);
                let wrapped = self.wrap_text_with_inline_formatting(
                    &format!("{}\u{2022}  {}", indent_str, text),
                    theme,
                );
                lines.extend(wrapped);
            }
            MarkdownBlock::TaskItem {
                text,
                indent,
                checked,
            } => {
                let indent_str = "  ".repeat(*indent as usize);
                let checkbox = if *checked { "☑ " } else { "☐ " };
                let wrapped = self.wrap_text_with_inline_formatting(
                    &format!("{}{}{}", indent_str, checkbox, text),
                    theme,
                );
                lines.extend(wrapped);
            }
            MarkdownBlock::Blockquote {
                level,
                children,
                header_override,
                footer_override,
            } => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.blockquote(*level, children) {
                        lines.extend(custom);
                        return;
                    }
                }

                let prefix_str = "│ ".repeat(*level as usize);
                let prefix_style = Style::default().fg(theme.get_muted_text_color());

                if let Some(ref hdr) = header_override {
                    lines.push(Line::from(Span::styled(
                        format!("{}{}", prefix_str, hdr),
                        prefix_style,
                    )));
                }

                let mut inner_lines = Vec::new();
                for (child_idx, child) in children.iter().enumerate() {
                    self.render_block(child, child_idx, theme, children, &mut inner_lines);
                }

                for mut line in inner_lines {
                    line.spans
                        .insert(0, Span::styled(prefix_str.clone(), prefix_style));
                    for span in line.spans.iter_mut().skip(1) {
                        let new_style = span
                            .style
                            .fg(theme.get_muted_text_color())
                            .add_modifier(Modifier::ITALIC);
                        span.style = new_style;
                    }
                    lines.push(line);
                }

                if let Some(ref ftr) = footer_override {
                    lines.push(Line::from(Span::styled(
                        format!("{}{}", prefix_str, ftr),
                        prefix_style,
                    )));
                }
            }
            MarkdownBlock::HorizontalRule => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.horizontal_rule() {
                        lines.push(custom);
                        return;
                    }
                }
                lines.push(Line::from(Span::styled(
                    HLINE.repeat(self.max_width.min(80)),
                    Style::default().fg(theme.get_muted_text_color()),
                )));
            }
            MarkdownBlock::BlankLine => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.blank_line() {
                        lines.push(custom);
                        return;
                    }
                }
                lines.push(Line::raw(""));
            }
            MarkdownBlock::Table { headers, rows } => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.table(headers, rows) {
                        lines.extend(custom);
                        return;
                    }
                }
                let table_lines = self.render_table(headers, rows, theme);
                lines.extend(table_lines);
            }
            MarkdownBlock::Image { alt, path } => {
                if let Some(h) = hooks {
                    if let Some(custom) = h.image_fallback(alt, path) {
                        lines.extend(custom);
                        return;
                    }
                }
                lines.push(default_image_fallback(alt, path));
            }
        }
    }

    fn find_list_context(block_idx: usize, blocks: &[MarkdownBlock]) -> (bool, Vec<bool>, usize) {
        let group_start = (0..=block_idx)
            .rev()
            .find(|&i| !matches!(blocks.get(i), Some(MarkdownBlock::ListItem(_, _))))
            .map(|i| i + 1)
            .unwrap_or(0);
        let group_end = (block_idx..blocks.len())
            .find(|&i| !matches!(blocks.get(i), Some(MarkdownBlock::ListItem(_, _))))
            .unwrap_or(blocks.len());

        let items: Vec<(usize, u8)> = blocks[group_start..group_end]
            .iter()
            .enumerate()
            .filter_map(|(i, b)| match b {
                MarkdownBlock::ListItem(_, indent) => Some((group_start + i, *indent)),
                _ => None,
            })
            .collect();

        let our_pos = match items.iter().position(|&(i, _)| i == block_idx) {
            Some(p) => p,
            None => return (true, Vec::new(), 0),
        };

        let our_indent = items[our_pos].1;

        let is_last = !has_sibling_after(&items, our_pos);

        let mut ancestors_are_last = Vec::new();
        let mut anc_pos = find_parent_pos(&items, our_pos);
        while let Some(p) = anc_pos {
            ancestors_are_last.push(!has_sibling_after(&items, p));
            anc_pos = find_parent_pos(&items, p);
        }
        ancestors_are_last.reverse();

        let our_parent = find_parent_pos(&items, our_pos);
        let index_in_group = items[..our_pos]
            .iter()
            .enumerate()
            .filter(|&(_, &(_, ind))| ind == our_indent)
            .filter(|&(pos, _)| find_parent_pos(&items, pos) == our_parent)
            .count();

        (is_last, ancestors_are_last, index_in_group)
    }

    fn default_code_block_header(&self, lang: &str, theme: &impl RichTextTheme) -> Line<'static> {
        if !lang.is_empty() {
            Line::from(Span::styled(
                format!("{ROUNDED_TL}{HLINE} {}", lang),
                Style::default().fg(theme.get_muted_text_color()),
            ))
        } else {
            Line::from(Span::styled(
                format!("{ROUNDED_TL}{HLINE}"),
                Style::default().fg(theme.get_muted_text_color()),
            ))
        }
    }

    fn default_code_block_footer(&self, theme: &impl RichTextTheme) -> Line<'static> {
        Line::from(Span::styled(
            format!("{ROUNDED_BL}{HLINE}"),
            Style::default().fg(theme.get_muted_text_color()),
        ))
    }

    fn render_table(
        &self,
        headers: &[String],
        rows: &[Vec<String>],
        theme: &impl RichTextTheme,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let col_count = headers
            .len()
            .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
        let padding_per_cell: usize = 2;
        let border_overhead = col_count + 1;
        let total_padding = col_count * padding_per_cell;
        let available = if self.max_width > border_overhead + total_padding {
            self.max_width
                .saturating_sub(border_overhead + total_padding)
        } else {
            80_usize.saturating_sub(border_overhead + total_padding)
        };

        fn longest_token_width(text: &str) -> usize {
            MarkdownRenderer::tokenize(text)
                .into_iter()
                .filter_map(|t| match t {
                    TextToken::Word(w) => Some(MarkdownRenderer::string_width(&w)),
                    _ => None,
                })
                .max()
                .unwrap_or(0)
        }

        let header_widths: Vec<usize> = (0..col_count)
            .map(|c| headers.get(c).map(|h| Self::string_width(h)).unwrap_or(0))
            .collect();

        let min_widths: Vec<usize> = (0..col_count)
            .map(|c| {
                let h_longest = headers.get(c).map(|h| longest_token_width(h)).unwrap_or(0);
                let d_longest = rows
                    .iter()
                    .filter_map(|r| r.get(c))
                    .map(|cell| longest_token_width(cell))
                    .max()
                    .unwrap_or(0);
                h_longest.max(d_longest).max(3)
            })
            .collect();

        let natural_widths: Vec<usize> = (0..col_count)
            .map(|c| {
                let hw = header_widths[c];
                let rw = rows
                    .iter()
                    .filter_map(|r| r.get(c))
                    .map(|cell| Self::string_width(cell))
                    .max()
                    .unwrap_or(0);
                hw.max(rw)
            })
            .collect();

        let target_lines: usize = 3;
        let ideal_widths: Vec<usize> = (0..col_count)
            .map(|c| {
                let natural = natural_widths[c];
                if natural == 0 {
                    return min_widths[c];
                }
                let wrapped = natural.div_ceil(target_lines);
                wrapped.max(min_widths[c])
            })
            .collect();

        let total_ideal: usize = ideal_widths.iter().sum();
        let mut col_widths: Vec<usize> = if total_ideal <= available {
            (0..col_count)
                .map(|c| ideal_widths[c].max(min_widths[c]))
                .collect()
        } else {
            (0..col_count)
                .map(|c| {
                    let proportional =
                        (available as u64 * ideal_widths[c] as u64 / total_ideal as u64) as usize;
                    proportional.max(min_widths[c])
                })
                .collect()
        };

        let mut total_allocated: usize = col_widths.iter().sum();
        if total_allocated > available {
            let deficit = total_allocated - available;
            let mut remaining = deficit;
            let mut sorted: Vec<usize> = (0..col_count).collect();
            sorted.sort_by_key(|&i| std::cmp::Reverse(col_widths[i] - min_widths[i]));
            for idx in sorted {
                if remaining == 0 {
                    break;
                }
                let shrinkable = col_widths[idx] - min_widths[idx];
                let take = shrinkable.min(remaining);
                col_widths[idx] -= take;
                remaining -= take;
            }
            total_allocated = col_widths.iter().sum();
        }
        if total_allocated < available {
            let mut surplus = available - total_allocated;
            let total_natural: usize = natural_widths.iter().sum::<usize>().max(1);
            while surplus > 0 {
                let mut gave_this_round = false;
                for idx in 0..col_count {
                    if surplus == 0 {
                        break;
                    }
                    let share = ((surplus * natural_widths[idx]) / total_natural).max(1);
                    let take = share.min(surplus);
                    col_widths[idx] += take;
                    surplus -= take;
                    gave_this_round = true;
                }
                if !gave_this_round {
                    break;
                }
            }
        }

        let border_style = Style::default().fg(theme.get_muted_text_color());

        lines.push(Line::from(Span::styled(
            Self::build_table_hline(&col_widths, CORNER_TL, TOP_MID, CORNER_TR),
            border_style,
        )));
        let header_line_spans: Vec<Vec<Vec<Span<'static>>>> = (0..col_count)
            .map(|c| {
                let text = headers.get(c).map(|s| s.as_str()).unwrap_or("");
                let inner = col_widths[c].saturating_sub(2);
                let base_style = Style::default()
                    .fg(theme.get_text_color())
                    .add_modifier(Modifier::BOLD);
                let parsed = parse_inline_formatting(text, theme);
                let spans = if parsed.is_empty() {
                    vec![Span::styled(text.to_string(), base_style)]
                } else {
                    parsed
                        .into_iter()
                        .map(|mut s| {
                            s.style = base_style.patch(s.style);
                            s
                        })
                        .collect()
                };
                Self::wrap_styled_spans_to_width(spans, inner)
            })
            .collect();
        let header_height = header_line_spans
            .iter()
            .map(|l| l.len().max(1))
            .max()
            .unwrap_or(1);
        for line_idx in 0..header_height {
            let line_cells: Vec<Vec<Span<'static>>> = (0..col_count)
                .map(|c| {
                    if line_idx < header_line_spans[c].len() {
                        header_line_spans[c][line_idx].clone()
                    } else {
                        vec![]
                    }
                })
                .collect();
            lines.push(Self::build_table_row_from_spans(
                &col_widths,
                &line_cells,
                theme,
                true,
            ));
        }
        lines.push(Line::from(Span::styled(
            Self::build_table_hline(&col_widths, MID_LEFT, CROSS_MID, MID_RIGHT),
            border_style,
        )));

        for row in rows {
            let cell_line_spans: Vec<Vec<Vec<Span<'static>>>> = (0..col_count)
                .map(|c| {
                    let text = row.get(c).map(|s| s.as_str()).unwrap_or("");
                    let inner = col_widths[c].saturating_sub(2);
                    let base_style = Style::default().fg(theme.get_text_color());
                    let parsed = parse_inline_formatting(text, theme);
                    let spans = if parsed.is_empty() {
                        vec![Span::styled(text.to_string(), base_style)]
                    } else {
                        parsed
                    };
                    Self::wrap_styled_spans_to_width(spans, inner)
                })
                .collect();
            let row_height = cell_line_spans
                .iter()
                .map(|l| l.len().max(1))
                .max()
                .unwrap_or(1);
            for line_idx in 0..row_height {
                let line_cells: Vec<Vec<Span<'static>>> = (0..col_count)
                    .map(|c| {
                        if line_idx < cell_line_spans[c].len() {
                            cell_line_spans[c][line_idx].clone()
                        } else {
                            vec![]
                        }
                    })
                    .collect();
                lines.push(Self::build_table_row_from_spans(
                    &col_widths,
                    &line_cells,
                    theme,
                    false,
                ));
            }
            lines.push(Line::from(Span::styled(
                Self::build_table_hline(&col_widths, MID_LEFT, CROSS_MID, MID_RIGHT),
                border_style,
            )));
        }

        let last_sep_idx = lines.len() - 1;
        let last_hline = Self::build_table_hline(&col_widths, CORNER_BL, BOTTOM_MID, CORNER_BR);
        lines[last_sep_idx] = Line::from(Span::styled(last_hline, border_style));

        lines
    }

    pub(crate) fn build_table_hline(
        col_widths: &[usize],
        left: &str,
        mid: &str,
        right: &str,
    ) -> String {
        let mut parts = vec![left.to_string()];
        for width in col_widths.iter() {
            parts.push(HLINE.repeat(*width));
            parts.push(mid.to_string());
        }
        parts.pop();
        parts.push(right.to_string());
        parts.join("")
    }

    pub(crate) fn wrap_styled_spans_to_width(
        spans: Vec<Span<'static>>,
        max_w: usize,
    ) -> Vec<Vec<Span<'static>>> {
        if max_w == 0 || spans.is_empty() {
            return if spans.is_empty() {
                vec![vec![]]
            } else {
                vec![spans]
            };
        }
        let mut lines: Vec<Vec<Span<'static>>> = Vec::new();
        let mut current_line: Vec<Span<'static>> = Vec::new();
        let mut current_width: usize = 0;
        let mut pending_space = false;

        for span in spans {
            let style = span.style;
            let text = span.content.to_string();
            let tokens = Self::tokenize(&text);

            for token in tokens {
                match token {
                    TextToken::Newline => {
                        lines.push(std::mem::take(&mut current_line));
                        current_width = 0;
                        pending_space = false;
                    }
                    TextToken::Space => {
                        pending_space = true;
                    }
                    TextToken::Word(word) => {
                        let word_w = Self::string_width(&word);
                        let space_w: usize = if pending_space && current_width > 0 {
                            1
                        } else {
                            0
                        };
                        let needs_wrap = if current_width == 0 {
                            false
                        } else if space_w > 0 && current_width + space_w >= max_w {
                            true
                        } else {
                            current_width + space_w + word_w > max_w
                        };
                        if needs_wrap && !current_line.is_empty() {
                            lines.push(std::mem::take(&mut current_line));
                            current_width = 0;
                            pending_space = false;
                        }
                        let final_space = if pending_space && current_width > 0 {
                            pending_space = false;
                            1
                        } else {
                            0
                        };
                        if final_space > 0 {
                            current_line.push(Span::styled(" ".to_string(), style));
                            current_width += final_space;
                        }
                        if current_width == 0 && word_w > max_w && max_w > 0 {
                            let mut chars: Vec<char> = word.chars().collect();
                            let mut char_w = 0;
                            let mut chunk = String::new();
                            for ch in chars.drain(..) {
                                let cw = Self::string_width(&ch.to_string());
                                if char_w + cw > max_w && !chunk.is_empty() {
                                    current_line.push(Span::styled(chunk, style));
                                    lines.push(std::mem::take(&mut current_line));
                                    chunk = String::new();
                                    char_w = 0;
                                }
                                chunk.push(ch);
                                char_w += cw;
                            }
                            if !chunk.is_empty() {
                                current_line.push(Span::styled(chunk, style));
                                current_width += char_w;
                            }
                        } else {
                            current_line.push(Span::styled(word, style));
                            current_width += word_w;
                        }
                    }
                }
            }
        }
        if !current_line.is_empty() || lines.is_empty() {
            lines.push(current_line);
        }
        lines
    }

    pub(crate) fn build_table_row_from_spans(
        col_widths: &[usize],
        cell_spans: &[Vec<Span<'static>>],
        theme: &impl RichTextTheme,
        is_header: bool,
    ) -> Line<'static> {
        let base_style = if is_header {
            Style::default()
                .fg(theme.get_text_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.get_text_color())
        };

        let mut spans = Vec::new();
        spans.push(Span::styled(
            VLINE.to_string(),
            Style::default().fg(theme.get_muted_text_color()),
        ));
        for (i, width) in col_widths.iter().enumerate() {
            let cell_spans_ref = cell_spans.get(i).map(|v| v.as_slice()).unwrap_or(&[]);
            let total_cell_w: usize = cell_spans_ref
                .iter()
                .map(|s| Self::string_width(&s.content))
                .sum();
            let inner_w = width.saturating_sub(2);

            spans.push(Span::styled("  ".to_string(), base_style));
            spans.extend_from_slice(cell_spans_ref);

            let padding = inner_w.saturating_sub(total_cell_w);
            if padding > 0 {
                spans.push(Span::styled(" ".repeat(padding), base_style));
            }
            spans.push(Span::styled(
                VLINE.to_string(),
                Style::default().fg(theme.get_muted_text_color()),
            ));
        }
        Line::from(spans)
    }
}
