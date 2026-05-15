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
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
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

fn natural_rows(pw: u32, ph: u32, font_w: u16, font_h: u16, proto: ProtocolType, max_w: u16) -> u16 {
    let (cw, ch) = pixel_to_cell(pw, ph, font_w, font_h, proto);
    let w = cw.min(max_w);
    if w < cw {
        let ratio = ph as f64 * w as f64 / (pw as f64).max(1.0);
        let h = (ratio / height_divisor(font_h, proto)).ceil() as u16;
        h.max(1)
    } else {
        ch.max(1)
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

`k` / `j` — grow / shrink zoom   `↑↓←→` — pan
`Tab` — cycle image   `q` — quit
"#;

struct ScaledImage {
    original: image::DynamicImage,
    scaled: image::DynamicImage,
    protocol: Option<Protocol>,
    target_rows: u16,
    natural_rows: u16,
    failed: bool,
    scroll_x: u16,
    scroll_y: u16,
    dirty: bool,
}

impl ScaledImage {
    fn new(
        img: image::DynamicImage,
        initial_rows: u16,
        font_w: u16,
        font_h: u16,
        proto: ProtocolType,
        max_w: u16,
    ) -> Self {
        let nat = natural_rows(img.width(), img.height(), font_w, font_h, proto, max_w);
        let rows = initial_rows.max(1);
        let mut s = Self {
            original: img.clone(),
            scaled: img,
            protocol: None,
            target_rows: rows,
            natural_rows: nat,
            failed: false,
            scroll_x: 0,
            scroll_y: 0,
            dirty: true,
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
        self.scroll_x = 0;
        self.scroll_y = 0;
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

    fn crop_to_viewport(
        &self,
        vis_w: u16,
        vis_h: u16,
        font_w: u16,
        font_h: u16,
        _proto: ProtocolType,
    ) -> image::DynamicImage {
        let (full_cw, full_ch) = self.cell_size(font_w, font_h, _proto);

        let sx = if full_cw > vis_w {
            self.scroll_x.min(full_cw.saturating_sub(vis_w))
        } else {
            0
        };
        let sy = if full_ch > vis_h {
            self.scroll_y.min(full_ch.saturating_sub(vis_h))
        } else {
            0
        };

        let target_px_w = (vis_w as u32 * font_w as u32).max(1);
        let target_px_h = (vis_h as u32 * font_h as u32).max(1);

        let src_x = (sx as u32 * font_w as u32).min(self.scaled.width().saturating_sub(1));
        let src_y = (sy as u32 * font_h as u32).min(self.scaled.height().saturating_sub(1));
        let src_w = target_px_w.min(self.scaled.width().saturating_sub(src_x));
        let src_h = target_px_h.min(self.scaled.height().saturating_sub(src_y));

        if src_w == 0 || src_h == 0 {
            return self.scaled.clone();
        }

        let cropped = self.scaled.crop_imm(src_x, src_y, src_w, src_h);
        cropped.resize_exact(target_px_w, target_px_h, image::imageops::FilterType::Triangle)
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
    selected: usize,
    need_rerender: bool,
    font_w: u16,
    font_h: u16,
    proto: ProtocolType,
    max_w: u16,
    vp_h: u16,
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
            self.vp_h,
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

    let default_rows: Vec<u16> = vec![2, 3];
    let mut scaled_images: Vec<ScaledImage> = Vec::new();
    for (i, ri) in resolved.iter().enumerate() {
        let rows = default_rows.get(i).copied().unwrap_or(3);
        let si = ScaledImage::new(ri.image.clone(), rows, font_w, font_h, proto, max_w);
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
        font_w,
        font_h,
        proto,
        max_w,
        vp_h: 20,
    };

    let mut output = state.rebuild_output();
    let mut last_vp_w: u16 = 70;
    let mut last_vp_h: u16 = 20;

    loop {
        if state.need_rerender {
            output = state.rebuild_output();
            state.need_rerender = false;
        }

        let sel_count = state.scaled_images.iter().filter(|i| !i.failed).count();
        let selected_idx = if sel_count > 0 { state.selected % sel_count } else { 0 };

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

            let vp_w = inner.width.saturating_sub(pad_l + pad_r + 1);
            let vp_h = inner.height.saturating_sub(pad_t + pad_b);
            last_vp_w = vp_w;
            last_vp_h = vp_h;

            if state.vp_h != vp_h {
                state.vp_h = vp_h;
                state.need_rerender = true;
            }

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

                let placement_w = placement.width_cells;
                let placement_h = placement.height_cells;
                if placement_h < 1 || placement_w < 1 { continue; }

                let render_w = placement_w.min(vp_w);

                let base_y = inner.y + pad_t
                    + (placement.row as u16).min(inner.height.saturating_sub(1));
                let border_bottom_y = inner.y + inner.height.saturating_sub(1);
                let max_render_h = border_bottom_y.saturating_sub(base_y);
                let render_h = placement_h.min(max_render_h).min(vp_h);

                let base_x = inner.x + pad_l;

                if si.dirty || si.protocol.is_none() {
                    let cropped = si.crop_to_viewport(
                        render_w, render_h,
                        state.font_w, state.font_h, state.proto,
                    );
                    let rect_for_proto = Rect::new(0, 0, render_w, render_h);
                    match state.picker.new_protocol(cropped, rect_for_proto, Resize::Fit(None)) {
                        Ok(proto) => si.protocol = Some(proto),
                        Err(_) => { si.failed = true; continue; }
                    }
                    si.dirty = false;
                }

                let proto_ref = match &si.protocol {
                    Some(p) => p,
                    None => continue,
                };
                let rect = Rect::new(base_x, base_y, render_w, render_h);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let widget = Image::new(proto_ref);
                    f.render_widget(widget, rect);
                }));

                if result.is_err() { si.failed = true; continue; }

                let (full_cw, full_ch) = si.cell_size(state.font_w, state.font_h, state.proto);

                let show_v_scroll = full_ch > render_h;
                let show_h_scroll = full_cw > render_w;

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
                            .content_length(full_ch as usize)
                            .viewport_content_length(render_h as usize)
                            .position(si.scroll_y as usize);
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
                            .content_length(full_cw as usize)
                            .viewport_content_length(render_w as usize)
                            .position(si.scroll_x as usize);
                        f.render_stateful_widget(sb, sb_area, &mut sb_state);
                    }
                }

                img_render_idx += 1;
            }

            let si_info = state.scaled_images.iter()
                .filter(|i| !i.failed)
                .nth(selected_idx);
            let info = match si_info {
                Some(si) => {
                    let pct = si.display_percent();
                    let (full_cw, full_ch) = si.cell_size(state.font_w, state.font_h, state.proto);
                    let overflow = if full_cw > last_vp_w || full_ch > last_vp_h {
                        format!(" | overflow {}x{}", full_cw, full_ch)
                    } else {
                        String::new()
                    };
                    format!(
                        "img {}/{} | {} rows ({:.0}%){} | k/j zoom ↑↓←→ pan | Tab | q",
                        selected_idx + 1, sel_count, si.target_rows, pct, overflow,
                    )
                }
                None => format!("img 0/{} | q quit", sel_count),
            };
            let info_line = Line::from(vec![
                Span::styled(info, Style::default().fg(Color::DarkGray)),
            ]);
            let info_y = area.y + area.height - 1;
            f.render_widget(Paragraph::new(vec![info_line]), Rect::new(area.x + 1, info_y, area.width - 2, 1));
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('k') => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                si.grow(state.font_w, state.font_h, state.proto, state.max_w);
                                state.need_rerender = true;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    KeyCode::Char('j') => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                si.shrink(state.font_w, state.font_h, state.proto, state.max_w);
                                state.need_rerender = true;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    KeyCode::Left => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                si.scroll_x = si.scroll_x.saturating_sub(1);
                                si.dirty = true;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    KeyCode::Right => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                let (full_cw, _) = si.cell_size(state.font_w, state.font_h, state.proto);
                                si.scroll_x = si.scroll_x.saturating_add(1).min(full_cw.saturating_sub(last_vp_w));
                                si.dirty = true;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    KeyCode::Up => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                si.scroll_y = si.scroll_y.saturating_sub(1);
                                si.dirty = true;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    KeyCode::Down => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                let (_, full_ch) = si.cell_size(state.font_w, state.font_h, state.proto);
                                si.scroll_y = si.scroll_y.saturating_add(1).min(full_ch.saturating_sub(1));
                                si.dirty = true;
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
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                si.scroll_y = si.scroll_y.saturating_sub(1);
                                si.dirty = true;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        let mut idx = 0;
                        for si in state.scaled_images.iter_mut() {
                            if si.failed { continue; }
                            if idx == selected_idx {
                                let (_, full_ch) = si.cell_size(state.font_w, state.font_h, state.proto);
                                si.scroll_y = si.scroll_y.saturating_add(1).min(full_ch.saturating_sub(1));
                                si.dirty = true;
                                break;
                            }
                            idx += 1;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen, event::DisableMouseCapture)?;
    Ok(())
}
