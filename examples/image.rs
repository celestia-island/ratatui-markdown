use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    prelude::Stylize,
    style::Color,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use ratatui_image::picker::Picker;
use ratatui_markdown::{
    markdown::{ImageResolver, MarkdownRenderer},
    theme::{Generation, RichTextTheme},
};

struct Theme;

impl RichTextTheme for Theme {
    fn generation(&self) -> Generation { Generation(1) }
    fn get_text_color(&self) -> Color { Color::White }
    fn get_muted_text_color(&self) -> Color { Color::DarkGray }
    fn get_primary_color(&self) -> Color { Color::Cyan }
    fn get_secondary_color(&self) -> Color { Color::Blue }
    fn get_info_color(&self) -> Color { Color::LightBlue }
    fn get_background_color(&self) -> Color { Color::Black }
    fn get_border_color(&self) -> Color { Color::DarkGray }
    fn get_focused_border_color(&self) -> Color { Color::White }
    fn get_popup_selected_background(&self) -> Color { Color::DarkGray }
    fn get_popup_selected_text_color(&self) -> Color { Color::White }
    fn get_json_key_color(&self) -> Color { Color::LightCyan }
    fn get_json_string_color(&self) -> Color { Color::Green }
    fn get_json_number_color(&self) -> Color { Color::Yellow }
    fn get_json_bool_color(&self) -> Color { Color::Magenta }
    fn get_json_null_color(&self) -> Color { Color::DarkGray }
    fn get_accent_yellow(&self) -> Color { Color::Yellow }
}

struct FsImageResolver {
    base_dir: std::path::PathBuf,
}

impl FsImageResolver {
    fn new(base_dir: &str) -> Self {
        Self {
            base_dir: std::path::PathBuf::from(base_dir),
        }
    }
}

impl ImageResolver for FsImageResolver {
    fn resolve(&mut self, path: &str) -> Option<image::DynamicImage> {
        let full_path = self.base_dir.join(path);
        image::ImageReader::open(&full_path).ok()?.decode().ok()
    }

    fn fallback(&self, path: &str, alt: &str) -> ratatui::text::Span<'static> {
        let label = if alt.is_empty() { path } else { alt };
        ratatui::text::Span::styled(
            format!("[no image: {label}]"),
            ratatui::style::Style::default().italic().fg(Color::Gray),
        )
    }
}

const MARKDOWN: &str = r#"
# Image Rendering Example

This example demonstrates image resolution and rendering via
`ratatui-image`. The terminal's graphics protocol (kitty, iTerm2,
sixels, or halfblocks) is auto-detected by `Picker`.

## Logo (loaded from disk)

![ratatui-markdown Logo](logo.webp)

## Demo Screenshot (loaded from disk)

![Demo Screenshot](demo.webp)

## Missing Image (fallback)

![Missing Image](nonexistent.webp)

The first two images are loaded from `examples/` using a filesystem
resolver. The third shows the fallback span.

Press `q` to quit.
"#;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76);

    let mut resolver = FsImageResolver::new(concat!(env!("CARGO_MANIFEST_DIR"), "/examples"));
    let (blocks, resolved) = renderer.parse_with_images(MARKDOWN, &mut resolver);

    let output = renderer.render_full(&blocks, &theme, &resolved, &resolver, 70, 20);

    let mut picker =
        Picker::from_termios().map_err(|e| anyhow::anyhow!("picker init failed: {:?}", e))?;

    let mut protocols: Vec<Option<Box<dyn ratatui_image::protocol::Protocol>>> = Vec::new();
    for placement in &output.images {
        let rect = Rect::new(0, 0, placement.width_cells, placement.height_cells);
        match picker.new_protocol(
            placement.image.clone(),
            rect,
            ratatui_image::Resize::Fit(Some(image::imageops::FilterType::Triangle)),
        ) {
            Ok(p) => protocols.push(Some(p)),
            Err(_) => protocols.push(None),
        }
    }

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let inner = Rect::new(
                area.x + 1,
                area.y + 1,
                area.width.saturating_sub(2),
                area.height.saturating_sub(2),
            );

            f.render_widget(
                Paragraph::new(output.lines.clone())
                    .block(Block::default().borders(Borders::ALL).title(" Image Example "))
                    .wrap(Wrap { trim: false }),
                inner,
            );

            for (i, placement) in output.images.iter().enumerate() {
                let Some(ref proto) = protocols.get(i).and_then(|o| o.as_ref()) else {
                    continue;
                };
                if placement.height_cells == 0 || placement.width_cells == 0 {
                    continue;
                }
                let render_h = placement.height_cells.min(inner.height);
                let render_w = placement.width_cells.min(inner.width.saturating_sub(4));
                if render_h < 2 || render_w < 2 {
                    continue;
                }
                let rect = Rect::new(
                    inner.x + 2,
                    inner.y + (placement.row as u16).min(inner.height.saturating_sub(render_h)),
                    render_w,
                    render_h,
                );
                f.render_widget(ratatui_image::Image::new(proto.as_ref()), rect);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
