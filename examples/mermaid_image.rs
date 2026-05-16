#[path = "utils/mod.rs"]
mod common;

use common::{lorem, Theme};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::Rect,
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
use ratatui_markdown::{
    markdown::{ImageResolver, MarkdownRenderer, RenderHooks},
    theme::RichTextTheme,
};

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

const MARKDOWN_TEMPLATE: &str = r#"
# Mermaid Image Rendering

This demo renders mermaid diagrams as **actual images** using the
embedded `mermaid-rs-renderer` crate (pure Rust, no external CLI).

## Cache Flow

```mermaid
graph TD
    A{Cache Hit?} -->|No| B[Compute Result]
    B --> C[Update Cache]
    A -->|Yes| D[Return Cached]
    C --> D
    D --> E[Response]
```

LOREM_2

## Pipeline

```mermaid
graph LR
    Input --> Parser --> AST --> Renderer --> Output
```

LOREM_2

## Pie Chart

```mermaid
pie title Languages
    "Rust" : 40
    "TypeScript" : 25
    "Python" : 20
    "Go" : 15
```

LOREM_2

## Sequence Diagram

```mermaid
sequenceDiagram
    participant Client
    participant Server
    participant DB
    Client->>Server: HTTP Request
    Server->>DB: Query
    DB-->>Server: Result
    Server-->>Client: Response
```

LOREM_3
"#;

fn to_renderer_theme(
    src: &ratatui_markdown::mermaid::theme::MermaidTheme,
) -> mermaid_rs_renderer::Theme {
    use ratatui_markdown::mermaid::theme::{color_to_hex, GIT_COLORS_HSL};
    let hex = |c: ratatui::style::Color| format!("#{}", color_to_hex(c));
    let pie: [String; 12] = src.pie_colors.map(hex);
    mermaid_rs_renderer::Theme {
        font_family: "sans-serif".to_string(),
        font_size: 14.0,
        primary_color: hex(src.primary_color),
        primary_text_color: hex(src.primary_text_color),
        primary_border_color: hex(src.primary_border_color),
        line_color: hex(src.line_color),
        secondary_color: hex(src.secondary_color),
        tertiary_color: hex(src.tertiary_color),
        edge_label_background: hex(src.edge_label_background),
        cluster_background: hex(src.cluster_background),
        cluster_border: hex(src.cluster_border),
        background: hex(src.background),
        sequence_actor_fill: hex(src.actor_fill),
        sequence_actor_border: hex(src.actor_border),
        sequence_actor_line: hex(src.actor_line),
        sequence_note_fill: hex(src.note_fill),
        sequence_note_border: hex(src.note_border),
        sequence_activation_fill: hex(src.activation_fill),
        sequence_activation_border: hex(src.activation_border),
        text_color: hex(src.text_color),
        git_colors: GIT_COLORS_HSL.map(|v| v.to_string()),
        git_inv_colors: [
            "hsl(60, 100%, 3.7%)",
            "rgb(0,0,160)",
            "rgb(49,0,147)",
            "rgb(147,73,0)",
            "rgb(147,0,0)",
            "rgb(147,0,73)",
            "rgb(0,147,0)",
            "rgb(0,147,147)",
        ]
        .map(|v| v.to_string()),
        git_branch_label_colors: [
            "#ffffff", "black", "black", "#ffffff", "black", "black", "black", "black",
        ]
        .map(|v| v.to_string()),
        git_commit_label_color: hex(src.git_commit_label_color),
        git_commit_label_background: hex(src.git_commit_label_bg),
        git_tag_label_color: hex(src.git_tag_label_color),
        git_tag_label_background: hex(src.git_tag_label_bg),
        git_tag_label_border: "hsl(240, 60%, 86.3%)".to_string(),
        pie_colors: pie,
        pie_title_text_size: 25.0,
        pie_title_text_color: hex(src.text_color),
        pie_section_text_size: 17.0,
        pie_section_text_color: hex(src.text_color),
        pie_legend_text_size: 17.0,
        pie_legend_text_color: hex(src.text_color),
        pie_stroke_color: hex(src.pie_stroke_color),
        pie_stroke_width: 2.0,
        pie_outer_stroke_width: 2.0,
        pie_outer_stroke_color: hex(src.pie_outer_stroke_color),
        pie_opacity: 0.75,
    }
}

fn render_mermaid_to_image(
    source: &str,
    mermaid_theme: &ratatui_markdown::mermaid::theme::MermaidTheme,
    font_w: u32,
    font_h: u32,
) -> Option<image::DynamicImage> {
    let renderer_theme = to_renderer_theme(mermaid_theme);
    let opts = mermaid_rs_renderer::RenderOptions {
        theme: renderer_theme,
        layout: mermaid_rs_renderer::LayoutConfig::default(),
    };
    let svg = mermaid_rs_renderer::render_with_options(source, opts).ok()?;

    let mut font_db = fontdb::Database::new();

    let detected_family = detect_system_font();
    if let Some(ref family) = detected_family {
        if !try_load_font_by_name(&mut font_db, family) {
            font_db.load_system_fonts();
        }
    } else {
        font_db.load_system_fonts();
    }

    for path in &[
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
    ] {
        if std::path::Path::new(path).exists() {
            let _ = font_db.load_font_file(path);
        }
    }

    let fallback = detected_family.as_deref().unwrap_or("Deja Vu Sans");
    font_db.set_sans_serif_family(fallback);
    font_db.set_serif_family(fallback);
    font_db.set_monospace_family(fallback);

    let usvg_opts = usvg::Options {
        fontdb: std::sync::Arc::new(font_db),
        ..usvg::Options::default()
    };

    let tree = usvg::Tree::from_str(&svg, &usvg_opts).ok()?;
    let size = tree.size();
    let svg_w = size.width() as f64;
    let svg_h = size.height() as f64;
    if svg_w <= 0.0 || svg_h <= 0.0 || font_w == 0 || font_h == 0 {
        return None;
    }

    let aspect = svg_w / svg_h;
    let cell_h = (svg_h / font_h as f64).ceil() as u32;
    let px_h = cell_h * font_h;
    let px_w = (px_h as f64 * aspect).round() as u32;
    let cell_w = (px_w as f64 / font_w as f64).floor().max(1.0) as u32;
    let px_w = cell_w * font_w;

    let mut pixmap = tiny_skia::Pixmap::new(px_w, px_h)?;
    let bg_fill = match mermaid_theme.background {
        ratatui::style::Color::Rgb(r, g, b) => tiny_skia::Color::from_rgba8(r, g, b, 255),
        ratatui::style::Color::Black => tiny_skia::Color::from_rgba8(0, 0, 0, 255),
        _ => tiny_skia::Color::from_rgba8(0, 0, 0, 255),
    };
    pixmap.fill(bg_fill);

    let scale = px_h as f32 / svg_h as f32;
    let ts = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, ts, &mut pixmap.as_mut());

    let rgba = pixmap.data().to_vec();
    image::RgbaImage::from_raw(px_w, px_h, rgba).map(image::DynamicImage::ImageRgba8)
}

fn detect_system_font() -> Option<String> {
    use font_kit::family_name::FamilyName;
    let source = font_kit::source::SystemSource::new();
    let handle = source
        .select_best_match(
            &[FamilyName::SansSerif],
            &font_kit::properties::Properties::new(),
        )
        .ok()?;
    let font = handle.load().ok()?;
    Some(font.full_name())
}

fn try_load_font_by_name(font_db: &mut fontdb::Database, name: &str) -> bool {
    use font_kit::family_name::FamilyName;
    let source = font_kit::source::SystemSource::new();
    let handle = match source.select_best_match(
        &[FamilyName::Title(name.to_string())],
        &font_kit::properties::Properties::new(),
    ) {
        Ok(h) => h,
        Err(_) => return false,
    };
    let font = match handle.load() {
        Ok(f) => f,
        Err(_) => return false,
    };
    if let Some(arc_bytes) = font.copy_font_data() {
        font_db.load_font_data((*arc_bytes).clone());
        true
    } else {
        false
    }
}

struct MermaidImage {
    image: image::DynamicImage,
    resolved_cells: (u16, u16),
    protocol: Option<Protocol>,
    dirty: bool,
    failed: bool,
}

impl MermaidImage {
    fn cell_size(&self) -> (u16, u16) {
        self.resolved_cells
    }
}

struct MermaidImageHooks {
    mermaid_theme: ratatui_markdown::mermaid::theme::MermaidTheme,
    font_w: u32,
    font_h: u32,
}

impl RenderHooks for MermaidImageHooks {
    fn render_mermaid_image(&self, source: &str) -> Option<image::DynamicImage> {
        render_mermaid_to_image(source, &self.mermaid_theme, self.font_w, self.font_h)
    }
}

struct MermaidResolver {
    font_w: u16,
    font_h: u16,
    proto: ProtocolType,
}

impl MermaidResolver {
    fn new(picker: &Picker) -> Self {
        let (fw, fh) = safe_font_size(picker);
        Self {
            font_w: fw,
            font_h: fh,
            proto: picker.protocol_type(),
        }
    }
}

impl ImageResolver for MermaidResolver {
    fn resolve(&mut self, _path: &str) -> Option<image::DynamicImage> {
        None
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
            self.proto,
        );
        let w = cw.min(max_width);
        let h = if w < cw {
            let ratio = img.height() as f64 * w as f64 / (img.width() as f64).max(1.0);
            (ratio / height_divisor(self.font_h, self.proto)).ceil() as u16
        } else {
            ch
        };
        let h = h.min(max_height);
        (w.max(1), h.max(1))
    }
}

struct AppState {
    renderer: MarkdownRenderer,
    theme: Theme,
    picker: Picker,
    resolver: MermaidResolver,
    blocks: Vec<ratatui_markdown::markdown::MarkdownBlock>,
    images: Vec<MermaidImage>,
    scroll: u16,
    font_w: u16,
    font_h: u16,
    max_w: u16,
}

impl AppState {
    fn rebuild_output(&mut self) -> ratatui_markdown::markdown::image::MarkdownRenderOutput {
        self.renderer.render_full(
            &self.blocks,
            &self.theme,
            &[],
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
    let picker = match Picker::from_query_stdio() {
        Ok(mut p) => {
            fix_protocol_override(&mut p);
            p
        }
        Err(_) => Picker::halfblocks(),
    };

    let (font_w, font_h) = safe_font_size(&picker);

    let area = terminal.size()?;
    let max_w = area.width.saturating_sub(4);
    let content_width = max_w as usize;

    let mermaid_theme = theme.get_mermaid_theme();
    let hooks = MermaidImageHooks {
        mermaid_theme: mermaid_theme.clone(),
        font_w: font_w as u32,
        font_h: font_h as u32,
    };
    let renderer = MarkdownRenderer::new(content_width).with_render_hooks(Box::new(hooks));

    let md = MARKDOWN_TEMPLATE
        .replace("LOREM_2", &lorem(100))
        .replace("LOREM_3", &lorem(150));

    let blocks = renderer.parse(&md);
    let resolver = MermaidResolver::new(&picker);

    let mut state = AppState {
        renderer,
        theme,
        picker,
        resolver,
        blocks,
        images: Vec::new(),
        scroll: 0,
        font_w,
        font_h,
        max_w,
    };

    let output = state.rebuild_output();

    for placement in &output.images {
        state.images.push(MermaidImage {
            image: placement.image.clone(),
            resolved_cells: (placement.width_cells, placement.height_cells),
            protocol: None,
            dirty: true,
            failed: false,
        });
    }

    loop {
        terminal.draw(|f| {
            let area = f.area();
            let block_area = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));

            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Mermaid Image Rendering ")
                .padding(Padding::new(1, 1, 0, 0));

            let inner = block.inner(block_area);
            let text_top = inner.y;
            let text_bot = inner.y + inner.height.saturating_sub(1);
            let text_left = inner.x;
            let sb_col = block_area.x + block_area.width.saturating_sub(1);
            let content_w = inner.width;
            let content_h = inner.height;

            let mut doc_h = output.lines.len() as u16;
            for (i, img) in state.images.iter().enumerate() {
                if img.failed {
                    continue;
                }
                let (_, img_h) = img.cell_size();
                let placement = &output.images[i];
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
                let mi = match state.images.get_mut(i) {
                    Some(m) if !m.failed => m,
                    _ => continue,
                };

                let (img_w, img_h) = mi.cell_size();
                if img_h < 1 || img_w < 1 {
                    continue;
                }

                let img_l = text_left as i32 + placement.col as i32;
                let img_t = text_top as i32 + placement.row as i32 - state.scroll as i32;
                let img_r = img_l + img_w as i32 - 1;
                let img_b = img_t + img_h as i32 - 1;

                let vp_l = text_left as i32;
                let vp_t = text_top as i32;
                let vp_r = (text_left as i32 + content_w as i32 - 1).max(vp_l);
                let vp_b = text_bot as i32;

                if img_r < vp_l || img_l > vp_r || img_b < vp_t || img_t > vp_b {
                    continue;
                }

                let clip_l = img_l.max(vp_l);
                let clip_t = img_t.max(vp_t);
                let clip_r = img_r.min(vp_r);
                let clip_b = img_b.min(vp_b);

                let vis_w = (clip_r - clip_l + 1) as u16;
                let vis_h = (clip_b - clip_t + 1) as u16;

                let crop_cells_l = (clip_l - img_l) as u32;
                let crop_cells_t = (clip_t - img_t) as u32;
                let crop_cells_r = (img_r - clip_r) as u32;
                let crop_cells_b = (img_b - clip_b) as u32;

                let fw = state.font_w as u32;
                let fh = state.font_h as u32;
                let total_px_w = mi.image.width();
                let total_px_h = mi.image.height();

                if mi.dirty || mi.protocol.is_none() {
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
                        mi.image
                            .crop_imm(crop_px_x, crop_px_y, crop_px_w, crop_px_h)
                    } else {
                        mi.image.clone()
                    };

                    let target_px_w = vis_w as u32 * fw;
                    let target_px_h = vis_h as u32 * fh;
                    let final_img = if img_for_proto.width() == target_px_w
                        && img_for_proto.height() == target_px_h
                    {
                        img_for_proto
                    } else {
                        let mut canvas = image::RgbaImage::new(target_px_w, target_px_h);
                        let ox = (target_px_w.saturating_sub(img_for_proto.width())) / 2;
                        let oy = (target_px_h.saturating_sub(img_for_proto.height())) / 2;
                        image::imageops::overlay(
                            &mut canvas,
                            &img_for_proto.to_rgba8(),
                            ox as i64,
                            oy as i64,
                        );
                        image::DynamicImage::ImageRgba8(canvas)
                    };

                    let rect_for_proto = Rect::new(0, 0, vis_w, vis_h);
                    match state
                        .picker
                        .new_protocol(final_img, rect_for_proto, Resize::Fit(None))
                    {
                        Ok(p) => mi.protocol = Some(p),
                        Err(_) => {
                            mi.failed = true;
                            continue;
                        }
                    }
                    mi.dirty = false;
                }

                let proto_ref = match &mi.protocol {
                    Some(p) => p,
                    None => continue,
                };
                let rect = Rect::new(clip_l as u16, clip_t as u16, vis_w, vis_h);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let widget = Image::new(proto_ref);
                    f.render_widget(widget, rect);
                }));
                if result.is_err() {
                    mi.failed = true;
                    continue;
                }
            }

            if doc_h > content_h && content_h > 0 {
                let sb_area = Rect::new(sb_col, inner.y, 1, content_h);
                let ratatui_content_len = doc_h.saturating_sub(content_h).saturating_add(1);
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

            let info_area = Rect::new(area.x, area.height.saturating_sub(1), area.width, 1);
            f.render_widget(
                Paragraph::new(vec![Line::from(Span::styled(
                    " ↑↓ scroll · PgUp/PgDn · Home/End · q quit",
                    Style::default().fg(Color::DarkGray),
                ))]),
                info_area,
            );
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        state.scroll = state.scroll.saturating_sub(1);
                        mark_dirty(&mut state.images);
                    }
                    KeyCode::Down => {
                        state.scroll = state.scroll.saturating_add(1);
                        mark_dirty(&mut state.images);
                    }
                    KeyCode::PageUp => {
                        let ch = terminal.get_frame().area().height.saturating_sub(3);
                        state.scroll = state.scroll.saturating_sub(ch.max(1));
                        mark_dirty(&mut state.images);
                    }
                    KeyCode::PageDown => {
                        let ch = terminal.get_frame().area().height.saturating_sub(3);
                        state.scroll = state.scroll.saturating_add(ch.max(1));
                        mark_dirty(&mut state.images);
                    }
                    KeyCode::Home => {
                        state.scroll = 0;
                        mark_dirty(&mut state.images);
                    }
                    KeyCode::End => {
                        state.scroll = u16::MAX;
                        mark_dirty(&mut state.images);
                    }
                    _ => {}
                },
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        state.scroll = state.scroll.saturating_sub(3);
                        mark_dirty(&mut state.images);
                    }
                    MouseEventKind::ScrollDown => {
                        state.scroll = state.scroll.saturating_add(3);
                        mark_dirty(&mut state.images);
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

fn mark_dirty(images: &mut [MermaidImage]) {
    for img in images.iter_mut() {
        if !img.failed {
            img.dirty = true;
        }
    }
}
