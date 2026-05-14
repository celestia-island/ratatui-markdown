#[derive(Debug, Clone, Default)]
pub struct FollowScrollState {
    pub manual_offset: Option<usize>,
    last_non_streaming_total: usize,
}

impl FollowScrollState {
    pub fn new() -> Self {
        Self {
            manual_offset: None,
            last_non_streaming_total: 0,
        }
    }

    pub fn resolve(&self, total_lines: usize, viewport_height: usize, is_streaming: bool) -> usize {
        let max = total_lines.saturating_sub(viewport_height);
        match self.manual_offset {
            Some(offset) => offset.min(max),
            None => {
                if is_streaming {
                    max
                } else {
                    max.max(
                        self.last_non_streaming_total
                            .saturating_sub(viewport_height),
                    )
                    .min(max)
                }
            }
        }
    }

    pub fn resolve_always_follow(&self, total_lines: usize, viewport_height: usize) -> usize {
        let max = total_lines.saturating_sub(viewport_height);
        match self.manual_offset {
            None => max,
            Some(offset) => offset.min(max),
        }
    }

    pub fn scroll_up(&mut self, step: usize, total_lines: usize, viewport_height: usize) {
        let current = self.resolve(total_lines, viewport_height, false);
        let new_offset = current.saturating_sub(step);
        self.manual_offset = Some(new_offset);
    }

    pub fn scroll_down(&mut self, step: usize, total_lines: usize, viewport_height: usize) {
        let max = total_lines.saturating_sub(viewport_height);
        let current = self.resolve(total_lines, viewport_height, false);
        let new_offset = (current + step).min(max);
        if new_offset >= max {
            self.manual_offset = None;
        } else {
            self.manual_offset = Some(new_offset);
        }
    }

    pub fn page_up(&mut self, total_lines: usize, viewport_height: usize) {
        let page = viewport_height.max(1);
        self.scroll_up(page, total_lines, viewport_height);
    }

    pub fn page_down(&mut self, total_lines: usize, viewport_height: usize) {
        let page = viewport_height.max(1);
        self.scroll_down(page, total_lines, viewport_height);
    }

    pub fn is_auto_following(&self) -> bool {
        self.manual_offset.is_none()
    }

    pub fn reset(&mut self) {
        self.manual_offset = None;
        self.last_non_streaming_total = 0;
    }

    pub fn update_non_streaming_baseline(&mut self, total_lines: usize) {
        if self.manual_offset.is_none() {
            self.last_non_streaming_total = total_lines;
        }
    }
}
