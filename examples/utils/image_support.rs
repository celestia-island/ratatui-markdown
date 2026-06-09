use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use ratatui_image::picker::{Picker, ProtocolType};

#[allow(dead_code)]
pub fn fix_protocol_override(picker: &mut Picker) {
    use ratatui_image::picker::Capability;
    let caps = picker.capabilities();
    if caps.contains(&Capability::Kitty) && picker.protocol_type() != ProtocolType::Kitty {
        picker.set_protocol_type(ProtocolType::Kitty);
    }
}

pub fn safe_font_size(picker: &Picker) -> (u16, u16) {
    let (fw, fh) = picker.font_size();
    if fw == 0 || fh == 0 {
        (8, 16)
    } else {
        (fw, fh)
    }
}

pub fn height_divisor(font_h: u16, proto: ProtocolType) -> f64 {
    match proto {
        ProtocolType::Halfblocks => font_h as f64 * 2.0,
        _ => font_h as f64,
    }
}

pub fn pixel_to_cell(pw: u32, ph: u32, font_w: u16, font_h: u16, proto: ProtocolType) -> (u16, u16) {
    if pw == 0 || ph == 0 || font_w == 0 {
        return (0, 0);
    }
    let cw = (pw as f64 / font_w as f64).ceil() as u16;
    let ch = (ph as f64 / height_divisor(font_h, proto)).ceil() as u16;
    (cw.max(1), ch.max(1))
}

#[allow(dead_code)]
pub fn rows_to_pixel_height(rows: u16, font_h: u16, proto: ProtocolType) -> u32 {
    (rows as f64 * height_divisor(font_h, proto)).ceil() as u32
}

pub fn cell_dimensions(
    img: &image::DynamicImage,
    max_width: u16,
    max_height: u16,
    font_w: u16,
    font_h: u16,
    proto: ProtocolType,
) -> (u16, u16) {
    let (cw, ch) = pixel_to_cell(img.width(), img.height(), font_w, font_h, proto);
    let w = cw.min(max_width);
    let h = if w < cw {
        let ratio = img.height() as f64 * w as f64 / (img.width() as f64).max(1.0);
        (ratio / height_divisor(font_h, proto)).ceil() as u16
    } else {
        ch
    };
    let h = h.min(max_height);
    (w.max(1), h.max(1))
}

pub struct ClipResult {
    pub screen_x: u16,
    pub screen_y: u16,
    pub vis_w: u16,
    pub vis_h: u16,
    pub crop_cells_l: u32,
    pub crop_cells_t: u32,
    pub crop_cells_r: u32,
    pub crop_cells_b: u32,
}

#[allow(clippy::too_many_arguments)]
pub fn calculate_clip(
    img_l: i32,
    img_t: i32,
    img_w: u16,
    img_h: u16,
    text_left: u16,
    text_top: u16,
    content_w: u16,
    text_bot: u16,
) -> Option<ClipResult> {
    let img_r = img_l + img_w as i32 - 1;
    let img_b = img_t + img_h as i32 - 1;

    let vp_l = text_left as i32;
    let vp_t = text_top as i32;
    let vp_r = (text_left as i32 + content_w as i32 - 1).max(vp_l);
    let vp_b = text_bot as i32;

    if img_r < vp_l || img_l > vp_r || img_b < vp_t || img_t > vp_b {
        return None;
    }

    let clip_l = img_l.max(vp_l);
    let clip_t = img_t.max(vp_t);
    let clip_r = img_r.min(vp_r);
    let clip_b = img_b.min(vp_b);

    Some(ClipResult {
        screen_x: clip_l as u16,
        screen_y: clip_t as u16,
        vis_w: (clip_r - clip_l + 1) as u16,
        vis_h: (clip_b - clip_t + 1) as u16,
        crop_cells_l: (clip_l - img_l) as u32,
        crop_cells_t: (clip_t - img_t) as u32,
        crop_cells_r: (img_r - clip_r) as u32,
        crop_cells_b: (img_b - clip_b) as u32,
    })
}

pub fn render_scrollbar(
    f: &mut Frame,
    doc_h: u16,
    content_h: u16,
    scroll: u16,
    sb_col: u16,
    inner_y: u16,
) {
    if doc_h <= content_h || content_h == 0 {
        return;
    }
    let sb_area = Rect::new(sb_col, inner_y, 1, content_h);
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
        .position(scroll as usize);
    f.render_stateful_widget(sb, sb_area, &mut sb_state);
}

pub trait Dirtyable {
    fn is_failed(&self) -> bool;
    fn set_dirty(&mut self);
}

pub fn mark_all_dirty<T: Dirtyable>(items: &mut [T]) {
    for item in items.iter_mut() {
        if !item.is_failed() {
            item.set_dirty();
        }
    }
}
