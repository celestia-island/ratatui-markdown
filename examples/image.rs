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
        image::ImageReader::open(&full_path)
            .ok()?
            .decode()
            .ok()
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

This example demonstrates image resolution and rendering.

## With Image

![Rust Logo](rust-logo.png)

## Without Image (fallback)

![Missing Image](nonexistent.png)

The first image shows a loaded file if `assets/rust-logo.png` exists.
The second shows the fallback span for missing images.

## Regular content

Images are resolved via the `ImageResolver` trait. When an image is
found, its position is recorded in `MarkdownRenderOutput`. When not
found, the fallback `Span` is displayed inline.

Press `q` to quit.
"#;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76);

    let mut resolver = FsImageResolver::new("./assets");
    let blocks = renderer.parse_with_images(MARKDOWN, &mut resolver);
    let output = renderer.render_full(&blocks, &theme);

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let inner = Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), area.height.saturating_sub(2));
            let paragraph = Paragraph::new(output.lines.clone())
                .block(Block::default().borders(Borders::ALL).title(" Image Example "))
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, inner);

            // Note: To render actual images, use ratatui-image in your draw loop:
            // for placement in &output.images {
            //     let protocol = picker.new_protocol(placement.image, size, Resize::Fit(None))?;
            //     let rect = Rect::new(placement.col, placement.row as u16, placement.width_cells, placement.height_cells);
            //     f.render_widget(ratatui_image::Image::new(&protocol), rect);
            // }
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
