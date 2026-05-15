use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Terminal,
};
use ratatui_image::{
    Image,
    Resize,
    picker::{Picker, ProtocolType},
    protocol::Protocol,
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

fn height_divisor(font_h: u16, proto: ProtocolType) -> f64 {
    match proto {
        ProtocolType::Halfblocks => font_h as f64 * 2.0,
        _ => font_h as f64,
    }
}

fn pixel_to_cell(pw: u32, ph: u32, font_w: u16, font_h: u16, proto: ProtocolType) -> (u16, u16) {
    if pw == 0 || ph == 0 || font_w == 0 {
        return (0, 0);
    }
    let cw = (pw as f64 / font_w as f64).ceil() as u16;
    let ch = (ph as f64 / height_divisor(font_h, proto)).ceil() as u16;
    (cw.max(1), ch.max(1))
}

fn rows_to_pixel_height(rows: u16, font_h: u16, proto: ProtocolType) -> u32 {
    (rows as f64 * height_divisor(font_h, proto)).ceil() as u32
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

`k` / `j` — grow / shrink demo image   `↑↓` / mouse — scroll document
`q` — quit
"#;

struct ScaledImage {
    original: image::DynamicImage,
    scaled: image::DynamicImage,
    protocol: Option<Protocol>,
    target_rows: u16,
    natural_rows: u16,
    failed: bool,
    dirty: bool,
    frozen: bool,
}

impl ScaledImage {
    fn new(
        img: image::DynamicImage,
        initial_rows: u16,
        font_w: u16,
        font_h: u16,
        proto: ProtocolType,
        max_w: u16,
        frozen: bool,
    ) -> Self {
        let pw = img.width();
        let ph = img.height();
        let nat_cw = (pw as f64 / font_w as f64).ceil() as u16;
        let nat_ch = pixel_to_cell(pw, ph, font_w, font_h, proto).1;
        let w = nat_cw.min(max_w);
        let nat = if w < nat_cw {
            let ratio = ph as f64 * w as f64 / (pw as f64).max(1.0);
            (ratio / height_divisor(font_h, proto)).ceil() as u16
        } else {
            nat_ch
        }.max(1);

        let rows = initial_rows.max(1);
        let mut s = Self {
            original: img.clone(),
            scaled: img,
            protocol: None,
            target_rows: rows,
            natural_rows: nat,
            failed: false,
            dirty: true,
            frozen,
        };
        s.resize_to_target(font_w, font_h, proto, max_w);
        s
    }

    fn resize_to_target(&mut self, font_w: u16, font_h: u16, proto: ProtocolType, max_w: u16) {
        let pw = self.original.width();
        let ph = self.original.height();

        let target_px_h = rows_to_pixel_height(self.target_rows, font_h, proto);
        let scale_h = target_px_h as f64 / ph as f64;

        let nat_cw = (pw as f64 / font_w as f64).ceil() as u16;
        let scale_w = if nat_cw > max_w {
            max_w as f64 * font_w as f64 / pw as f64
        } else {
            1.0
        };

        let scale = scale_h.min(scale_w);
        let sw = ((pw as f64 * scale).ceil() as u32).max(1);
        let sh = ((ph as f64 * scale).ceil() as u32).max(1);

        self.scaled = self.original.resize_exact(sw, sh, image::imageops::FilterType::Triangle);
        self.dirty = true;
    }

    fn grow(&mut self, font_w: u16, font_h: u16, proto: ProtocolType, max_w: u16) {
        self.target_rows = self.target_rows.saturating_add(1).min(200);
        self.resize_to_target(font_w, font_h, proto, max_w);
    }

    fn shrink(&mut self, font_w: u16, font_h: u16, proto: ProtocolType, max_w: u16) {
        self.target_rows = self.target_rows.saturating_sub(1).max(1);
        self.resize_to_target(font_w, font_h, proto, max_w);
    }

    fn cell_size(&self, font_w: u16, font_h: u16, proto: ProtocolType) -> (u16, u16) {
        pixel_to_cell(self.scaled.width(), self.scaled.height(), font_w, font_h, proto)
    }

    fn display_percent(&self) -> f64 {
        if self.natural_rows == 0 { return 100.0; }
        self.target_rows as f64 / self.natural_rows as f64 * 100.0
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
        max_height: u16,
    ) -> (u16, u16) {
        let (cw, ch) = pixel_to_cell(img.width(), img.height(), self.font_w, self.font_h, self.protocol_type);
        let w = cw.min(max_width);
        let h = if w < cw {
            let ratio = img.height() as f64 * w as f64 / (img.width() as f64).max(1.0);
            (ratio / height_divisor(self.font_h, self.protocol_type)).ceil() as u16
        } else {
            ch
        };
        let h = h.min(max_height);
        (w.max(1), h.max(1))
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
    need_rerender: bool,
    scroll: u16,
    font_w: u16,
    font_h: u16,
    proto: ProtocolType,
    max_w: u16,
}

impl AppState {
    fn rebuild_output(&mut self) -> ratatui_markdown::markdown::image::MarkdownRenderOutput {
        let resolved_images: Vec<ratatui_markdown::markdown::image::ResolvedImage> = self
            .resolved_paths
            .iter()
            .zip(self.scaled_images.iter())
            .map(|(path, si)| {
                ratatui_markdown::markdown::image::ResolvedImage {
                    path: path.clone(),
                    image: si.scaled.clone(),
                }
            })
            .collect();
        self.renderer.render_full(
            &self.blocks,
            &self.theme,
            &resolved_images,
            &mut self.resolver,
            self.max_w,
            999,
        )
    }
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let max_w: u16 = 70;
    let renderer = MarkdownRenderer::new(76);

    let picker = match Picker::from_query_stdio() {
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

    // Logo: frozen at 3 rows.  Demo: starts at 3 rows, zoomable.  Missing: 2 rows.
    let config: Vec<(u16, bool)> = vec![(3, true), (3, false), (2, false)];
    let mut scaled_images: Vec<ScaledImage> = Vec::new();
    for (i, ri) in resolved.iter().enumerate() {
        let (rows, frozen) = config.get(i).copied().unwrap_or((3, false));
        let si = ScaledImage::new(ri.image.clone(), rows, font_w, font_h, proto, max_w, frozen);
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
        need_rerender: true,
        scroll: 0,
        font_w,
        font_h,
        proto,
        max_w,
    };

    let mut output = state.rebuild_output();

    loop {
        if state.need_rerender {
            output = state.rebuild_output();
            state.need_rerender = false;
        }

        terminal.draw(|f| {
            let area = f.area();
            let pad_t: u16 = 1;
            let pad_b: u16 = 2;
            let pad_l: u16 = 1;
            let pad_r: u16 = 2;
            let inner = Rect::new(
                area.x + pad_l,
                area.y + pad_t,
                area.width.saturating_sub(pad_l + pad_r),
                area.height.saturating_sub(pad_t + pad_b),
            );

            let text_top = inner.y + 1;
            let text_bot = inner.y + inner.height.saturating_sub(2);
            let text_left = inner.x + 1;
            let sb_col = inner.x + inner.width.saturating_sub(1);
            let text_right = sb_col.saturating_sub(1);
            let content_w = text_right.saturating_sub(text_left);
            let content_h = text_bot.saturating_sub(text_top).saturating_add(1);

            // Compute total document height (text lines + image extensions)
            let mut doc_h = output.lines.len() as u16;
            // Verify image placements don't exceed text lines
            for (i, placement) in output.images.iter().enumerate() {
                let si = match state.scaled_images.get(i) {
                    Some(s) if !s.failed => s,
                    _ => continue,
                };
                let (_img_w, img_h) = si.cell_size(state.font_w, state.font_h, state.proto);
                let img_end = placement.row as u16 + img_h;
                if img_end > doc_h {
                    use std::io::Write;
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .append(true).create(true).open("/tmp/image_debug.log")
                    {
                        let _ = writeln!(f, "OVERFLOW img[{i}] row={} h_cells={} img_h={} doc_h={} content_h={content_h} target_rows={} zoom={} pw={} ph={}",
                            placement.row, placement.height_cells, img_h, doc_h,
                            si.target_rows, si.display_percent(),
                            si.scaled.width(), si.scaled.height());
                    }
                    doc_h = img_end;
                }
            }
            let max_scroll = doc_h.saturating_sub(content_h);
            if state.scroll > max_scroll {
                state.scroll = max_scroll;
            }

            // Render text with document scroll
            f.render_widget(
                Paragraph::new(output.lines.clone())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Image Viewer "),
                    )
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .scroll((state.scroll, 0)),
                inner,
            );

            // Render images, offset by scroll
            for (i, placement) in output.images.iter().enumerate() {
                let si = match state.scaled_images.get_mut(i) {
                    Some(s) if !s.failed => s,
                    _ => continue,
                };

                let (img_w, img_h) = si.cell_size(state.font_w, state.font_h, state.proto);
                if img_h < 1 || img_w < 1 { continue; }

                let abs_y = text_top + placement.row as u16;
                let base_y = abs_y.saturating_sub(state.scroll);
                let img_x = text_left;

                // Skip if fully scrolled out of view (above or below)
                let img_bottom = base_y.saturating_add(img_h).saturating_sub(1);
                if img_bottom < text_top || base_y > text_bot {
                    continue;
                }

                // Compute visible portion of the image
                let visible_top = base_y.max(text_top);
                let visible_bot = (base_y + img_h).min(text_bot.saturating_add(1));
                let vis_h = visible_bot.saturating_sub(visible_top);

                let render_w = img_w.min(content_w);

                if si.dirty || si.protocol.is_none() {
                    // Crop: top (image scrolled above viewport)
                    let crop_y = if base_y < text_top {
                        ((text_top - base_y) as u32 * state.font_h as u32)
                            .min(si.scaled.height().saturating_sub(1))
                    } else {
                        0
                    };
                    // Crop: bottom (image extends below viewport)
                    let crop_px_h = (vis_h as u32 * state.font_h as u32)
                        .min(si.scaled.height().saturating_sub(crop_y));
                    // Crop: right (image wider than render area)
                    let crop_px_w = (render_w as u32 * state.font_w as u32)
                        .min(si.scaled.width());

                    let img_for_proto = if crop_y > 0
                        || crop_px_w < si.scaled.width()
                        || crop_px_h < si.scaled.height()
                    {
                        si.scaled.crop_imm(0, crop_y, crop_px_w, crop_px_h.max(1))
                    } else {
                        si.scaled.clone()
                    };

                    let rect_for_proto = Rect::new(0, 0, render_w, vis_h);
                    match state.picker.new_protocol(
                        img_for_proto,
                        rect_for_proto,
                        Resize::Fit(None),
                    ) {
                        Ok(proto) => si.protocol = Some(proto),
                        Err(_) => {
                            si.failed = true;
                            continue;
                        }
                    }
                    si.dirty = false;
                }

                let proto_ref = match &si.protocol {
                    Some(p) => p,
                    None => continue,
                };
                let rect = Rect::new(img_x, visible_top, render_w, vis_h);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let widget = Image::new(proto_ref);
                    f.render_widget(widget, rect);
                }));
                if result.is_err() {
                    si.failed = true;
                    continue;
                }
            }

            // Vertical scrollbar
            if doc_h > content_h && content_h > 0 {
                let sb_area = Rect::new(sb_col, text_top, 1, content_h);
                let sb = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .thumb_symbol("█")
                    .track_symbol(Some("│"))
                    .style(Style::default().fg(Color::DarkGray))
                    .thumb_style(Style::default().fg(Color::Cyan));
                // Ratatui's internal thumb-size formula:
                //   thumb = V * track / ((C-1) + V)
                // We want: thumb = V * track / doc_h
                // → set C = doc_h - V + 1, so denominator = doc_h
                let ratatui_content_len = doc_h
                    .saturating_sub(content_h)
                    .saturating_add(1);
                let mut sb_state = ScrollbarState::default()
                    .content_length(ratatui_content_len as usize)
                    .viewport_content_length(content_h as usize)
                    .position(state.scroll as usize);
                f.render_stateful_widget(sb, sb_area, &mut sb_state);
            }

            // Info bar
            let demo_si = state.scaled_images.iter().find(|si| !si.frozen && !si.failed);
            let info = match demo_si {
                Some(si) => {
                    let pct = si.display_percent();
                    format!(
                        "demo {} rows ({:.0}%) | k/j zoom | ↑↓ scroll | q quit",
                        si.target_rows, pct,
                    )
                }
                None => "no zoomable image | q quit".into(),
            };
            f.render_widget(
                Paragraph::new(vec![Line::from(vec![Span::styled(
                    info,
                    Style::default().fg(Color::DarkGray),
                )])]),
                Rect::new(
                    area.x + 1,
                    area.height.saturating_sub(1),
                    area.width.saturating_sub(2),
                    1,
                ),
            );
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('k') => {
                        for si in state.scaled_images.iter_mut() {
                            if si.failed || si.frozen { continue; }
                            si.grow(state.font_w, state.font_h, state.proto, state.max_w);
                            state.need_rerender = true;
                            break;
                        }
                    }
                    KeyCode::Char('j') => {
                        for si in state.scaled_images.iter_mut() {
                            if si.failed || si.frozen { continue; }
                            si.shrink(state.font_w, state.font_h, state.proto, state.max_w);
                            state.need_rerender = true;
                            break;
                        }
                    }
                    KeyCode::Up => {
                        state.scroll = state.scroll.saturating_sub(1);
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed { si.dirty = true; }
                        }
                    }
                    KeyCode::Down => {
                        state.scroll = state.scroll.saturating_add(1);
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed { si.dirty = true; }
                        }
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        state.scroll = state.scroll.saturating_sub(1);
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed { si.dirty = true; }
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        state.scroll = state.scroll.saturating_add(1);
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed { si.dirty = true; }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    Ok(())
}
