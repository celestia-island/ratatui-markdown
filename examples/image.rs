use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    prelude::Stylize,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
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
    if caps.contains(&Capability::Kitty) && picker.protocol_type() != ProtocolType::Kitty {
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
        Span::styled(format!("[no image: {label}]",), Style::default().italic().fg(Color::Gray))
    }
}

const MARKDOWN: &str = r#"
# Image Rendering Example

Images render via `ratatui-image` using the terminal's native
graphics protocol (kitty, iTerm2, sixels, or halfblocks).

## Logo (loaded from disk)

![ratatui-markdown Logo](logo.webp)

## Demo Screenshot (loaded from disk)

![Demo Screenshot](demo.webp)

## Missing Image (fallback)

![Missing Image](nonexistent.webp)

`[` / `]` — zoom selected image in/out
`q` — quit
"#;

struct RenderedImage {
    original_img: image::DynamicImage,
    protocol: Option<StatefulProtocol>,
    natural_cell_w: u16,
    natural_cell_h: u16,
    scale: f64,
    failed: bool,
}

impl RenderedImage {
    fn new(img: image::DynamicImage, picker: &Picker) -> Self {
        let (cw, ch) = pixel_to_cell(img.width(), img.height(), picker);
        Self {
            original_img: img,
            protocol: None,
            natural_cell_w: cw,
            natural_cell_h: ch,
            scale: 1.0,
            failed: false,
        }
    }

    fn scaled_px(&self) -> (u32, u32) {
        let w = self.original_img.width();
        let h = self.original_img.height();
        let sw = (w as f64 * self.scale) as u32;
        let sh = (h as f64 * self.scale) as u32;
        (sw.max(1), sh.max(1))
    }

    fn scaled_cells(&self, picker: &Picker) -> (u16, u16) {
        let (sw, sh) = self.scaled_px();
        pixel_to_cell(sw, sh, picker)
    }

    fn zoom_in(&mut self) { self.scale *= 1.25; self.protocol = None; }

    fn zoom_out(&mut self) {
        self.scale /= 1.25;
        self.scale = self.scale.max(0.05);
        self.protocol = None;
    }

    fn rebuild_proto(&mut self, picker: &mut Picker) {
        let (sw, sh) = self.scaled_px();
        let resized = self.original_img.resize_exact(
            sw, sh, image::imageops::FilterType::Triangle,
        );
        self.protocol = Some(picker.new_resize_protocol(resized));
    }
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let renderer = MarkdownRenderer::new(76);

    let mut picker = match Picker::from_query_stdio() {
        Ok(mut p) => { fix_protocol_override(&mut p); p }
        Err(_) => Picker::halfblocks(),
    };

    let mut resolver = FsImageResolver::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples"),
        &picker,
    );
    let (blocks, resolved) = renderer.parse_with_images(MARKDOWN, &mut resolver);
    let output = renderer.render_full(&blocks, &theme, &resolved, &resolver, 70, 20);

    let mut images: Vec<Option<RenderedImage>> = Vec::new();
    for placement in &output.images {
        let mut ri = RenderedImage::new(placement.image.clone(), &picker);
        ri.rebuild_proto(&mut picker);
        images.push(Some(ri));
    }

    let mut selected: usize = 0;
    let v_scroll: usize = 0;
    let h_scroll: usize = 0;

    loop {
        let sel_count = images.iter().filter(|o| o.is_some()).count();
        let selected_idx = if sel_count > 0 { selected % sel_count } else { 0 };

        terminal.draw(|f| {
            let area = f.area();
            let pad_t: u16 = 1;
            let pad_b: u16 = 3;
            let pad_l: u16 = 1;
            let pad_r: u16 = 2;
            let inner = Rect::new(
                area.x + pad_l,
                area.y + pad_t,
                area.width.saturating_sub(pad_l + pad_r),
                area.height.saturating_sub(pad_t + pad_b),
            );

            f.render_widget(
                Paragraph::new(output.lines.clone())
                    .block(Block::default().borders(Borders::ALL).title(" Image Example "))
                    .wrap(ratatui::widgets::Wrap { trim: false }),
                inner,
            );

            let mut img_idx = 0;
            for (i, placement) in output.images.iter().enumerate() {
                let img = match images.get_mut(i) { Some(Some(ref mut im)) => im, _ => continue };
                if img.failed { continue; }
                if img_idx != selected_idx { continue; }

                let (cell_w, cell_h) = img.scaled_cells(&picker);
                if cell_h < 1 || cell_w < 1 { continue; }

                let vp_w = inner.width.saturating_sub(pad_l + pad_r + 1);
                let vp_h = inner.height.saturating_sub(pad_t + pad_b);
                let show_v_scroll = cell_h > vp_h && vp_h > 0;
                let show_h_scroll = cell_w > vp_w && vp_w > 0;

                let render_w = if show_h_scroll { vp_w } else { cell_w.min(vp_w) };
                let render_h = if show_v_scroll { vp_h } else { cell_h.min(vp_h) };

                if render_h < 1 || render_w < 1 { continue; }

                let base_x = inner.x + pad_l;
                let base_y = inner.y + pad_t
                    + (placement.row as u16).min(inner.height.saturating_sub(render_h));

                if img.protocol.is_none() {
                    img.rebuild_proto(&mut picker);
                }
                let mut proto = match img.protocol.take() { Some(p) => p, None => continue };

                let rect = Rect::new(base_x, base_y, render_w, render_h);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let widget = StatefulImage::default();
                    f.render_stateful_widget(widget, rect, &mut proto);
                }));

                if result.is_err() { img.failed = true; continue; }

                if let Some(enc_result) = proto.last_encoding_result() {
                    match enc_result {
                        Ok(()) => img.protocol = Some(proto),
                        Err(_) => { img.failed = true; continue; }
                    }
                } else {
                    img.protocol = Some(proto);
                }

                if show_v_scroll {
                    let sb_area = Rect::new(inner.x + inner.width - 1, base_y, 1, render_h);
                    let max_pos = cell_h.saturating_sub(1);
                    let thumb_pos = if vp_h >= cell_h { 0 } else {
                        let max_off = cell_h - vp_h;
                        if max_off > 0 && max_pos > 0 {
                            (v_scroll as u64 * max_pos as u64 / max_off as u64) as usize
                        } else { 0 }
                    };
                    let sb = Scrollbar::default()
                        .orientation(ScrollbarOrientation::VerticalRight)
                        .thumb_symbol("█")
                        .track_symbol(Some("│"))
                        .style(Style::default().fg(Color::DarkGray))
                        .thumb_style(Style::default().fg(Color::Cyan));
                    let mut sb_state = ScrollbarState::default()
                        .content_length(cell_h as usize)
                        .viewport_content_length(vp_h as usize)
                        .position(thumb_pos.min(max_pos as usize));
                    f.render_stateful_widget(sb, sb_area, &mut sb_state);
                }

                if show_h_scroll {
                    let sb_area = Rect::new(base_x, base_y + render_h, render_w, 1);
                    let max_pos = cell_w.saturating_sub(1);
                    let thumb_pos = if vp_w >= cell_w { 0 } else {
                        let max_off = cell_w - vp_w;
                        if max_off > 0 && max_pos > 0 {
                            (h_scroll as u64 * max_pos as u64 / max_off as u64) as usize
                        } else { 0 }
                    };
                    let sb = Scrollbar::default()
                        .orientation(ScrollbarOrientation::HorizontalBottom)
                        .thumb_symbol("█")
                        .track_symbol(Some("─"))
                        .style(Style::default().fg(Color::DarkGray))
                        .thumb_style(Style::default().fg(Color::Cyan));
                    let mut sb_state = ScrollbarState::default()
                        .content_length(cell_w as usize)
                        .viewport_content_length(vp_w as usize)
                        .position(thumb_pos.min(max_pos as usize));
                    f.render_stateful_widget(sb, sb_area, &mut sb_state);
                }

                break;
            }

            let info_text = format!(
                "img {}/{} | zoom {:.0}% | [ ] resize | q quit",
                selected_idx + 1, sel_count,
                images.iter()
                    .filter_map(|o| o.as_ref())
                    .nth(selected_idx)
                    .map(|i| i.scale * 100.0)
                    .unwrap_or(0.0),
            );
            let info_line = Line::from(vec![
                Span::styled(info_text, Style::default().fg(Color::DarkGray)),
            ]);
            let info_y = area.y + area.height - 1;
            f.render_widget(Paragraph::new(vec![info_line]), Rect::new(area.x + 1, info_y, area.width - 2, 1));
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('[') => {
                        let mut idx = 0;
                        for (i, opt) in images.iter_mut().enumerate() {
                            if opt.is_some() {
                                if idx == selected_idx {
                                    opt.as_mut().unwrap().zoom_in();
                                    break;
                                }
                                idx += 1;
                            }
                        }
                    }
                    KeyCode::Char(']') => {
                        let mut idx = 0;
                        for (i, opt) in images.iter_mut().enumerate() {
                            if opt.is_some() {
                                if idx == selected_idx {
                                    opt.as_mut().unwrap().zoom_out();
                                    break;
                                }
                                idx += 1;
                            }
                        }
                    }
                    KeyCode::Tab => {
                        selected = if sel_count > 0 { (selected + 1) % sel_count } else { 0 };
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
