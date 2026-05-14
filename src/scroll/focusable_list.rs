use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::{scroll::render_arrow_scrollbar, theme::RichTextTheme};

#[derive(Debug)]
pub struct FocusableItemLines<'a> {
    pub lines: Vec<Line<'a>>,
}

pub struct FocusableItemList;

#[derive(Debug, Clone)]
pub struct RenderContext<'a, T: RichTextTheme> {
    pub inner_area: Rect,
    pub outer_area: Rect,
    pub items: &'a [FocusableItemLines<'a>],
    pub selected_index: usize,
    pub add_separator: bool,
    pub no_items_text: &'a str,
    pub theme: &'a T,
}

impl FocusableItemList {
    pub fn render<T: RichTextTheme>(f: &mut Frame, ctx: RenderContext<'_, T>) {
        let visible_height = ctx.inner_area.height as usize;

        if ctx.items.is_empty() {
            let placeholder = Paragraph::new(Line::from(Span::styled(
                format!("     {}", ctx.no_items_text),
                Style::default()
                    .fg(ctx.theme.get_muted_text_color())
                    .add_modifier(Modifier::ITALIC),
            )))
            .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(placeholder, ctx.inner_area);
            return;
        }

        let mut all_lines: Vec<Line> = Vec::new();
        let mut item_line_ranges: Vec<(usize, usize)> = Vec::new();

        for (idx, item) in ctx.items.iter().enumerate() {
            let start_line = all_lines.len();
            all_lines.extend(item.lines.iter().cloned());

            if ctx.add_separator && idx < ctx.items.len() - 1 {
                all_lines.push(Line::raw(""));
            }

            let end_line = all_lines.len();
            item_line_ranges.push((start_line, end_line));
        }

        let total_lines = all_lines.len();
        let selected_index = ctx.selected_index.min(ctx.items.len().saturating_sub(1));
        let (selected_start, selected_end) = item_line_ranges[selected_index];
        let selected_height = selected_end - selected_start;
        let max_scroll_offset = total_lines.saturating_sub(visible_height);

        let scroll_offset = if selected_height >= visible_height {
            selected_start.min(max_scroll_offset)
        } else if selected_start < visible_height / 2 {
            0
        } else if selected_end > total_lines.saturating_sub(visible_height / 2) {
            max_scroll_offset
        } else {
            selected_start
                .saturating_sub(visible_height / 2 - selected_height / 2)
                .min(max_scroll_offset)
        };

        let end_line = (scroll_offset + visible_height).min(total_lines);
        let visible_lines: Vec<Line> = all_lines[scroll_offset..end_line].to_vec();
        let paragraph = Paragraph::new(visible_lines);
        f.render_widget(paragraph, ctx.inner_area);

        if total_lines > visible_height {
            render_arrow_scrollbar(
                f,
                ctx.outer_area,
                total_lines,
                visible_height,
                scroll_offset,
                ctx.theme,
            );
        }
    }
}
