use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Terminal,
};
use ratatui_image::{
    StatefulImage,
    picker::{Picker, ProtocolType},
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
    use ratatui_image::picker::Capability;
    let caps = picker.capabilities();
    if caps.contains(&Capability::Kitty) && picker.protocol_type() != ProtocolType::Kitty {
        picker.set_protocol_type(ProtocolType::Kitty);
    }
}

fn safe_font_size(picker: &Picker) -> (u16, u16) {
    let (fw, fh) = picker.font_size();
    if fw == 0 || fh == 0 { (8, 16) } else { (fw, fh) }
}

fn pixel_to_cell(pw: u32, ph: u32, font_w: u16, font_h: u16, proto: ProtocolType) -> (u16, u16) {
    if pw == 0 || ph == 0 || font_w == 0 || font_h == 0 {
        return (0, 0);
    }
    let height_div = match proto {
        ProtocolType::Halfblocks => font_h as f64 * 2.0,
        _ => font_h as f64,
    };
    let cw = (pw as f64 / font_w as f64).ceil() as u16;
    let ch = (ph as f64 / height_div).ceil() as u16;
    (cw.max(1), ch.max(1))
}

fn scale_to_fit_rows(
    pw: u32, ph: u32, target_rows: u16,
    font_w: u16, font_h: u16, proto: ProtocolType, max_w: u16,
) -> f64 {
    let height_div = match proto {
        ProtocolType::Halfblocks => font_h as f64 * 2.0,
        _ => font_h as f64,
    };
    let natural_h = (ph as f64 / height_div).ceil();
    if natural_h <= target_rows as f64 {
        let natural_w = (pw as f64 / font_w as f64).ceil();
        if natural_w <= max_w as f64 {
            return 1.0;
        }
        return max_w as f64 * font_w as f64 / pw as f64;
    }
    target_rows as f64 * height_div / ph as f64
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

`j` / `k` — zoom selected image in/out
`Tab` — cycle selected image
`q` — quit
"#;

struct ScaledImage {
    original: image::DynamicImage,
    scaled: image::DynamicImage,
    protocol: Option<StatefulProtocol>,
    scale: f64,
    failed: bool,
}

impl ScaledImage {
    fn new(img: image::DynamicImage) -> Self {
        Self {
            original: img.clone(),
            scaled: img,
            protocol: None,
            scale: 1.0,
            failed: false,
        }
    }

    fn apply_scale(&mut self, picker: &mut Picker) {
        let sw = ((self.original.width() as f64 * self.scale).ceil() as u32).max(1);
        let sh = ((self.original.height() as f64 * self.scale).ceil() as u32).max(1);
        self.scaled = self.original.resize_exact(sw, sh, image::imageops::FilterType::Triangle);
        self.protocol = None;
        self.rebuild_proto(picker);
    }

    fn zoom_in(&mut self, picker: &mut Picker) {
        self.scale = (self.scale * 1.25).min(5.0);
        self.apply_scale(picker);
    }

    fn zoom_out(&mut self, picker: &mut Picker) {
        self.scale = (self.scale / 1.25).max(0.05);
        self.apply_scale(picker);
    }

    fn rebuild_proto(&mut self, picker: &mut Picker) {
        self.protocol = Some(picker.new_resize_protocol(self.scaled.clone()));
    }
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
        &mut self,
        img: &image::DynamicImage,
        max_width: u16,
        _max_height: u16,
    ) -> (u16, u16) {
        let (cw, ch) = pixel_to_cell(img.width(), img.height(), self.font_w, self.font_h, self.protocol_type);
        let w = cw.min(max_width);
        if w < cw {
            let ratio = img.height() as f64 * w as f64 / (img.width() as f64).max(1.0);
            let height_div = match self.protocol_type {
                ProtocolType::Halfblocks => self.font_h as f64 * 2.0,
                _ => self.font_h as f64,
            };
            let h = (ratio / height_div).ceil() as u16;
            (w.max(1), h.max(1))
        } else {
            (w.max(1), ch.max(1))
        }
    }

    fn fallback(&self, path: &str, alt: &str) -> ratatui::text::Span<'static> {
        let label = if alt.is_empty() { path } else { alt };
        Span::styled(format!("[no image: {label}]"), Style::default().italic().fg(Color::Gray))
    }
}

struct AppState {
    renderer: MarkdownRenderer,
    theme: Theme,
    picker: Picker,
    resolver: FsImageResolver,
    blocks: Vec<ratatui_markdown::markdown::MarkdownBlock>,
    resolved_paths: Vec<String>,
    scaled_images: Vec<ScaledImage>,
    selected: usize,
    need_rerender: bool,
}

impl AppState {
    fn rebuild_output(&mut self) -> ratatui_markdown::markdown::image::MarkdownRenderOutput {
        let resolved_scaled: Vec<ratatui_markdown::markdown::image::ResolvedImage> = self
            .resolved_paths
            .iter()
            .zip(self.scaled_images.iter())
            .map(|(path, si)| ratatui_markdown::markdown::image::ResolvedImage {
                path: path.clone(),
                image: si.scaled.clone(),
            })
            .collect();
        self.renderer.render_full(
            &self.blocks,
            &self.theme,
            &resolved_scaled,
            &mut self.resolver,
            70,
            20,
        )
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

    let (font_w, font_h) = safe_font_size(&picker);
    let proto = picker.protocol_type();

    let mut resolver = FsImageResolver::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples"),
        &picker,
    );

    let (blocks, resolved) = renderer.parse_with_images(MARKDOWN, &mut resolver);

    let default_rows: Vec<u16> = vec![2, 3];
    let mut scaled_images: Vec<ScaledImage> = Vec::new();
    for (i, ri) in resolved.iter().enumerate() {
        let target_rows = default_rows.get(i).copied().unwrap_or(3);
        let init_scale = scale_to_fit_rows(
            ri.image.width(), ri.image.height(), target_rows,
            font_w, font_h, proto, 70,
        );
        let mut si = ScaledImage::new(ri.image.clone());
        si.scale = init_scale;
        si.apply_scale(&mut picker);
        scaled_images.push(si);
    }

    let resolved_paths: Vec<String> = resolved.iter().map(|r| r.path.clone()).collect();

    let mut state = AppState {
        renderer,
        theme,
        picker,
        resolver,
        blocks,
        resolved_paths,
        scaled_images,
        selected: 0,
        need_rerender: true,
    };

    let mut output = state.rebuild_output();

    loop {
        let sel_count = state.scaled_images.iter().filter(|i| !i.failed).count();
        let selected_idx = if sel_count > 0 { state.selected % sel_count } else { 0 };

        if state.need_rerender {
            output = state.rebuild_output();
            state.need_rerender = false;
        }

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

            let mut img_render_idx = 0;
            for (i, placement) in output.images.iter().enumerate() {
                let si = match state.scaled_images.get_mut(i) {
                    Some(s) if !s.failed => s,
                    _ => continue,
                };

                let cell_w = placement.width_cells;
                let cell_h = placement.height_cells;
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

                if si.protocol.is_none() {
                    si.rebuild_proto(&mut state.picker);
                }
                let mut proto = match si.protocol.take() { Some(p) => p, None => continue };

                let rect = Rect::new(base_x, base_y, render_w, render_h);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let widget = StatefulImage::default();
                    f.render_stateful_widget(widget, rect, &mut proto);
                }));

                if result.is_err() { si.failed = true; continue; }

                if let Some(enc_result) = proto.last_encoding_result() {
                    match enc_result {
                        Ok(()) => si.protocol = Some(proto),
                        Err(_) => { si.failed = true; continue; }
                    }
                } else {
                    si.protocol = Some(proto);
                }

                if img_render_idx == selected_idx {
                    if show_v_scroll {
                        let sb_area = Rect::new(inner.x + inner.width - 1, base_y, 1, render_h);
                        let sb = Scrollbar::default()
                            .orientation(ScrollbarOrientation::VerticalRight)
                            .thumb_symbol("█")
                            .track_symbol(Some("│"))
                            .style(Style::default().fg(Color::DarkGray))
                            .thumb_style(Style::default().fg(Color::Cyan));
                        let mut sb_state = ScrollbarState::default()
                            .content_length(cell_h as usize)
                            .viewport_content_length(vp_h as usize)
                            .position(0);
                        f.render_stateful_widget(sb, sb_area, &mut sb_state);
                    }

                    if show_h_scroll {
                        let sb_area = Rect::new(base_x, base_y + render_h, render_w, 1);
                        let sb = Scrollbar::default()
                            .orientation(ScrollbarOrientation::HorizontalBottom)
                            .thumb_symbol("█")
                            .track_symbol(Some("─"))
                            .style(Style::default().fg(Color::DarkGray))
                            .thumb_style(Style::default().fg(Color::Cyan));
                        let mut sb_state = ScrollbarState::default()
                            .content_length(cell_w as usize)
                            .viewport_content_length(vp_w as usize)
                            .position(0);
                        f.render_stateful_widget(sb, sb_area, &mut sb_state);
                    }
                }

                img_render_idx += 1;
            }

            let zoom_pct = state.scaled_images.iter()
                .filter(|i| !i.failed)
                .nth(selected_idx)
                .map(|i| i.scale * 100.0)
                .unwrap_or(0.0);
            let info_text = format!(
                "img {}/{} | zoom {:.0}% | j/k resize | Tab cycle | q quit",
                selected_idx + 1, sel_count, zoom_pct,
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
                    KeyCode::Char('j') => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                si.zoom_out(&mut state.picker);
                                state.need_rerender = true;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    KeyCode::Char('k') => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                si.zoom_in(&mut state.picker);
                                state.need_rerender = true;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    KeyCode::Tab => {
                        state.selected = if sel_count > 0 { (state.selected + 1) % sel_count } else { 0 };
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
