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
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Terminal,
};
use ratatui_image::{
    picker::{Picker, ProtocolType},
    protocol::Protocol,
    Image, Resize,
};
use ratatui_markdown::markdown::{ImageResolver, MarkdownRenderer};

#[path = "utils/mod.rs"]
mod common;

use common::{lorem, Theme};

fn fix_protocol_override(picker: &mut Picker) {
    use ratatui_image::picker::Capability;
    let caps = picker.capabilities();
    if caps.contains(&Capability::Kitty) && picker.protocol_type() != ProtocolType::Kitty {
        picker.set_protocol_type(ProtocolType::Kitty);
    }
}

fn safe_font_size(picker: &Picker) -> (u16, u16) {
    let (fw, fh) = picker.font_size();
    if fw == 0 || fh == 0 {
        (8, 16)
    } else {
        (fw, fh)
    }
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

const MARKDOWN_TEMPLATE: &str = r#"
# Image Rendering Example

Images render via `ratatui-image` using the terminal's native
graphics protocol (kitty, iTerm2, sixels, or halfblocks).

## Logo (loaded from disk)

![ratatui-markdown Logo](logo.webp)

## Demo Screenshot (loaded from disk)

![Demo Screenshot](demo.webp)

LOREM_4

## Missing Image (fallback)

![Missing Image](nonexistent.webp)

LOREM_3
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
        }
        .max(1);

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

        self.scaled = self
            .original
            .resize_exact(sw, sh, image::imageops::FilterType::Triangle);
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
        pixel_to_cell(
            self.scaled.width(),
            self.scaled.height(),
            font_w,
            font_h,
            proto,
        )
    }

    fn display_percent(&self) -> f64 {
        if self.natural_rows == 0 {
            return 100.0;
        }
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
        let (cw, ch) = pixel_to_cell(
            img.width(),
            img.height(),
            self.font_w,
            self.font_h,
            self.protocol_type,
        );
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
        Span::styled(
            format!("[no image: {label}]"),
            Style::default().italic().fg(Color::Gray),
        )
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
            .map(
                |(path, si)| ratatui_markdown::markdown::image::ResolvedImage {
                    path: path.clone(),
                    image: si.scaled.clone(),
                },
            )
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
    crossterm::execute!(
        std::io::stdout(),
        EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let theme = Theme;
    let max_w: u16 = 70;
    let renderer = MarkdownRenderer::new(76);

    let picker = match Picker::from_query_stdio() {
        Ok(mut p) => {
            fix_protocol_override(&mut p);
            p
        }
        Err(_) => Picker::halfblocks(),
    };

    let (font_w, font_h) = safe_font_size(&picker);
    let proto = picker.protocol_type();

    let mut resolver =
        FsImageResolver::new(concat!(env!("CARGO_MANIFEST_DIR"), "/examples"), &picker);

    let md = MARKDOWN_TEMPLATE
        .replace("LOREM_3", &lorem(150))
        .replace("LOREM_4", &lorem(200));

    let (blocks, resolved) = renderer.parse_with_images(&md, &mut resolver);

    let config: Vec<(u16, bool)> = vec![(3, true), (16, false), (2, false)];
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
            let block_area = Rect::new(
                area.x,
                area.y,
                area.width,
                area.height.saturating_sub(1),
            );

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Image Viewer ")
                .padding(Padding::new(1, 1, 0, 0));

            let inner = block.inner(block_area);
            let text_top = inner.y;
            let text_bot = inner.y + inner.height.saturating_sub(1);
            let text_left = inner.x;
            let sb_col = block_area.x + block_area.width.saturating_sub(1);
            let content_w = inner.width;
            let content_h = inner.height;

            let mut doc_h = output.lines.len() as u16;
            for (i, placement) in output.images.iter().enumerate() {
                let si = match state.scaled_images.get(i) {
                    Some(s) if !s.failed => s,
                    _ => continue,
                };
                let (_img_w, img_h) = si.cell_size(state.font_w, state.font_h, state.proto);
                let img_end = placement.row as u16 + img_h;
                if img_end > doc_h {
                    doc_h = img_end;
                }
            }
            let max_scroll = doc_h.saturating_sub(content_h);
            if state.scroll > max_scroll {
                state.scroll = max_scroll;
            }

            f.render_widget(
                Paragraph::new(output.lines.clone())
                    .block(block)
                    .scroll((state.scroll, 0)),
                block_area,
            );

            for (i, placement) in output.images.iter().enumerate() {
                let si = match state.scaled_images.get_mut(i) {
                    Some(s) if !s.failed => s,
                    _ => continue,
                };

                let (img_w, img_h) = si.cell_size(state.font_w, state.font_h, state.proto);
                if img_h < 1 || img_w < 1 {
                    continue;
                }

                // Image rect in screen coordinates (i32 so negatives = above/left of viewport)
                let img_l = text_left as i32;
                let img_t = text_top as i32 + placement.row as i32 - state.scroll as i32;
                let img_r = img_l + img_w as i32 - 1;
                let img_b = img_t + img_h as i32 - 1;

                // Viewport rect
                let vp_l = text_left as i32;
                let vp_t = text_top as i32;
                let vp_r = (text_left as i32 + content_w as i32 - 1).max(vp_l);
                let vp_b = text_bot as i32;

                // Skip if no overlap at all
                if img_r < vp_l || img_l > vp_r || img_b < vp_t || img_t > vp_b {
                    continue;
                }

                // Clipped rect = intersection of image and viewport
                let clip_l = img_l.max(vp_l);
                let clip_t = img_t.max(vp_t);
                let clip_r = img_r.min(vp_r);
                let clip_b = img_b.min(vp_b);

                let vis_w = (clip_r - clip_l + 1) as u16;
                let vis_h = (clip_b - clip_t + 1) as u16;

                // Cells cropped from each edge of the image
                let crop_cells_l = (clip_l - img_l) as u32;
                let crop_cells_t = (clip_t - img_t) as u32;
                let crop_cells_r = (img_r - clip_r) as u32;
                let crop_cells_b = (img_b - clip_b) as u32;

                let fw = state.font_w as u32;
                let fh = state.font_h as u32;
                let total_px_w = si.scaled.width();
                let total_px_h = si.scaled.height();

                if si.dirty || si.protocol.is_none() {
                    let crop_px_x = crop_cells_l * fw;
                    let crop_px_y = crop_cells_t * fh;
                    let crop_px_w = total_px_w
                        .saturating_sub(crop_cells_l * fw)
                        .saturating_sub(crop_cells_r * fw)
                        .max(1);
                    let crop_px_h = total_px_h
                        .saturating_sub(crop_cells_t * fh)
                        .saturating_sub(crop_cells_b * fh)
                        .max(1);

                    let need_crop = crop_cells_l > 0
                        || crop_cells_t > 0
                        || crop_cells_r > 0
                        || crop_cells_b > 0;

                    let img_for_proto = if need_crop {
                        si.scaled.crop_imm(crop_px_x, crop_px_y, crop_px_w, crop_px_h)
                    } else {
                        si.scaled.clone()
                    };

                    let rect_for_proto = Rect::new(0, 0, vis_w, vis_h);
                    match state
                        .picker
                        .new_protocol(img_for_proto, rect_for_proto, Resize::Fit(None))
                    {
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
                let rect = Rect::new(clip_l as u16, clip_t as u16, vis_w, vis_h);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let widget = Image::new(proto_ref);
                    f.render_widget(widget, rect);
                }));
                if result.is_err() {
                    si.failed = true;
                    continue;
                }
            }

            if doc_h > content_h && content_h > 0 {
                let sb_area = Rect::new(sb_col, inner.y, 1, content_h);
                let ratatui_content_len =
                    doc_h.saturating_sub(content_h).saturating_add(1);
                let sb = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .thumb_symbol("█")
                    .track_symbol(Some("│"))
                    .style(Style::default().fg(Color::DarkGray))
                    .thumb_style(Style::default().fg(Color::Cyan));
                let mut sb_state = ScrollbarState::default()
                    .content_length(ratatui_content_len as usize)
                    .viewport_content_length(content_h as usize)
                    .position(state.scroll as usize);
                f.render_stateful_widget(sb, sb_area, &mut sb_state);
            }

            let demo_si = state
                .scaled_images
                .iter()
                .find(|si| !si.frozen && !si.failed);
            let info = match demo_si {
                Some(si) => {
                    let pct = si.display_percent();
                    format!(
                        " demo {} rows ({:.0}%) \u{00b7} k/j zoom \u{00b7} \u{2191}\u{2193} scroll \u{00b7} q quit",
                        si.target_rows, pct,
                    )
                }
                None => " \u{2191}\u{2193} scroll \u{00b7} q quit".into(),
            };
            let info_area = Rect::new(area.x, area.height.saturating_sub(1), area.width, 1);
            f.render_widget(
                Paragraph::new(vec![Line::from(Span::styled(
                    info,
                    Style::default().fg(Color::DarkGray),
                ))]),
                info_area,
            );
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('k') => {
                        for si in state.scaled_images.iter_mut() {
                            if si.failed || si.frozen {
                                continue;
                            }
                            si.grow(state.font_w, state.font_h, state.proto, state.max_w);
                            state.need_rerender = true;
                            break;
                        }
                    }
                    KeyCode::Char('j') => {
                        for si in state.scaled_images.iter_mut() {
                            if si.failed || si.frozen {
                                continue;
                            }
                            si.shrink(state.font_w, state.font_h, state.proto, state.max_w);
                            state.need_rerender = true;
                            break;
                        }
                    }
                    KeyCode::Up => {
                        state.scroll = state.scroll.saturating_sub(1);
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed {
                                si.dirty = true;
                            }
                        }
                    }
                    KeyCode::Down => {
                        state.scroll = state.scroll.saturating_add(1);
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed {
                                si.dirty = true;
                            }
                        }
                    }
                    KeyCode::PageUp => {
                        let content_h = terminal.get_frame().area().height.saturating_sub(3);
                        state.scroll = state.scroll.saturating_sub(content_h.max(1));
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed {
                                si.dirty = true;
                            }
                        }
                    }
                    KeyCode::PageDown => {
                        let content_h = terminal.get_frame().area().height.saturating_sub(3);
                        state.scroll = state.scroll.saturating_add(content_h.max(1));
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed {
                                si.dirty = true;
                            }
                        }
                    }
                    KeyCode::Home => {
                        state.scroll = 0;
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed {
                                si.dirty = true;
                            }
                        }
                    }
                    KeyCode::End => {
                        state.scroll = u16::MAX;
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed {
                                si.dirty = true;
                            }
                        }
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        state.scroll = state.scroll.saturating_sub(3);
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed {
                                si.dirty = true;
                            }
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        state.scroll = state.scroll.saturating_add(3);
                        for si in state.scaled_images.iter_mut() {
                            if !si.failed {
                                si.dirty = true;
                            }
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
