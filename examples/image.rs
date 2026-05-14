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
use ratatui_image::{
    StatefulImage,
    picker::{Capability, Picker, ProtocolType},
    protocol::StatefulProtocol,
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

fn fix_protocol_override(picker: &mut Picker) {
    let caps = picker.capabilities();
    let has_kitty = caps.contains(&Capability::Kitty);
    if has_kitty && picker.protocol_type() != ProtocolType::Kitty {
        picker.set_protocol_type(ProtocolType::Kitty);
    }
}

fn safe_font_size(picker: &Picker) -> (u16, u16) {
    let (fw, fh) = picker.font_size();
    if fw == 0 || fh == 0 { (8, 16) } else { (fw, fh) }
}

fn pixel_to_cell(pw: u32, ph: u32, picker: &Picker) -> (u16, u16) {
    let (fw, fh) = safe_font_size(picker);
    let cell_w = ((pw as f64 / fw as f64).ceil() as u16).max(1);
    let height_div = match picker.protocol_type() {
        ProtocolType::Halfblocks => fh as f64 * 2.0,
        _ => fh as f64,
    };
    let cell_h = ((ph as f64 / height_div).ceil() as u16).max(1);
    (cell_w, cell_h)
}

struct FsImageResolver {
    base_dir: std::path::PathBuf,
    font_w: u16,
    font_h: u16,
    protocol_type: ProtocolType,
}

impl FsImageResolver {
    fn new(base_dir: &str, picker: &Picker) -> Self {
        let (fw, fh) = safe_font_size(picker);
        Self {
            base_dir: std::path::PathBuf::from(base_dir),
            font_w: fw,
            font_h: fh,
            protocol_type: picker.protocol_type(),
        }
    }
}

impl ImageResolver for FsImageResolver {
    fn resolve(&mut self, path: &str) -> Option<image::DynamicImage> {
        let full_path = self.base_dir.join(path);
        image::ImageReader::open(&full_path).ok()?.decode().ok()
    }

    fn cell_dimensions(
        &self,
        img: &image::DynamicImage,
        max_width: u16,
        _max_height: u16,
    ) -> (u16, u16) {
        let pw = img.width();
        let ph = img.height();
        if pw == 0 || ph == 0 || max_width == 0 {
            return (0, 0);
        }
        let cell_w = ((pw as f64 / self.font_w as f64).ceil() as u16).max(1);
        let w = cell_w.min(max_width);
        let ratio = ph as f64 * w as f64 / (pw as f64).max(1.0);
        let height_div = match self.protocol_type {
            ProtocolType::Halfblocks => self.font_h as f64 * 2.0,
            _ => self.font_h as f64,
        };
        let h = ((ratio / height_div).ceil() as u16).max(1);
        (w, h)
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

This example renders images via `ratatui-image` using the
terminal's native graphics protocol (kitty, iTerm2, sixels, or
halfblocks — auto-detected).

## Logo (loaded from disk)

![ratatui-markdown Logo](logo.webp)

## Demo Screenshot (loaded from disk)

![Demo Screenshot](demo.webp)

## Missing Image (fallback)

![Missing Image](nonexistent.webp)

Press `q` to quit.
"#;

struct RenderedImage {
    protocol: Option<StatefulProtocol>,
    cell_w: u16,
    cell_h: u16,
    failed: bool,
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76);

    let mut picker = match Picker::from_query_stdio() {
        Ok(mut p) => {
            fix_protocol_override(&mut p);
            p
        },
        Err(_) => Picker::halfblocks(),
    };

    eprintln!(
        "[image] protocol={:?}, font_size={:?}",
        picker.protocol_type(),
        picker.font_size()
    );

    let mut resolver = FsImageResolver::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples"),
        &picker,
    );
    let (blocks, resolved) = renderer.parse_with_images(MARKDOWN, &mut resolver);
    let output = renderer.render_full(&blocks, &theme, &resolved, &resolver, 70, 20);

    let mut rendered_images: Vec<Option<RenderedImage>> = Vec::new();
    for placement in &output.images {
        let proto = picker.new_resize_protocol(placement.image.clone());
        let (cw, ch) =
            pixel_to_cell(placement.image.width(), placement.image.height(), &picker);
        rendered_images.push(Some(RenderedImage {
            protocol: Some(proto),
            cell_w: cw.min(placement.width_cells),
            cell_h: ch.min(placement.height_cells),
            failed: false,
        }));
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
                let img = match rendered_images.get_mut(i) {
                    Some(Some(ref mut img)) => img,
                    _ => continue,
                };
                if img.failed || img.cell_h == 0 || img.cell_w == 0 {
                    continue;
                }
                let render_h = img.cell_h.min(inner.height);
                let render_w = img.cell_w.min(inner.width.saturating_sub(4));
                if render_h < 2 || render_w < 2 {
                    continue;
                }
                let mut proto = match img.protocol.take() {
                    Some(p) => p,
                    None => continue,
                };

                let rect = Rect::new(
                    inner.x + 2,
                    inner.y
                        + (placement.row as u16).min(inner.height.saturating_sub(render_h)),
                    render_w,
                    render_h,
                );
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let widget = StatefulImage::default();
                    f.render_stateful_widget(widget, rect, &mut proto);
                }));

                if result.is_err() {
                    img.failed = true;
                    continue;
                }

                if let Some(enc_result) = proto.last_encoding_result() {
                    match enc_result {
                        Ok(()) => {
                            img.protocol = Some(proto);
                        }
                        Err(e) => {
                            eprintln!("[image] encoding error: {}", e);
                            img.failed = true;
                        }
                    }
                } else {
                    img.protocol = Some(proto);
                }
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
