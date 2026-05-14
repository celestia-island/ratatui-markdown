use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use crate::theme::RichTextTheme;

pub fn make_body_line(
    connector: &str,
    line: Line<'static>,
    theme: &impl RichTextTheme,
) -> Line<'static> {
    Line::from(
        vec![Span::styled(
            connector.to_string(),
            Style::default().fg(theme.get_muted_text_color()),
        )]
        .into_iter()
        .chain(line.spans)
        .collect::<Vec<Span<'static>>>(),
    )
}

pub fn make_status_line(connector: &str, text: &str, theme: &impl RichTextTheme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            connector.to_string(),
            Style::default().fg(theme.get_muted_text_color()),
        ),
        Span::styled(
            text.to_string(),
            Style::default()
                .fg(theme.get_muted_text_color())
                .add_modifier(Modifier::ITALIC),
        ),
    ])
}

pub fn make_branch_dispatch_line(
    connector: &str,
    text: &str,
    theme: &impl RichTextTheme,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            connector.to_string(),
            Style::default().fg(theme.get_muted_text_color()),
        ),
        Span::styled(
            text.to_string(),
            Style::default()
                .fg(theme.get_info_color())
                .add_modifier(Modifier::BOLD),
        ),
    ])
}
