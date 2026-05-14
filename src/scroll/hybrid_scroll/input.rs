use super::HybridScrollView;

impl HybridScrollView {
    pub(super) fn check_engage_down(&mut self) {
        let center = self.viewport_center();
        for (i, region) in self.regions.iter().enumerate() {
            if region.items.is_empty() {
                continue;
            }
            let first_item_center = region.item_center(0);
            if center >= first_item_center && self.last_center < first_item_center {
                self.engaged_region = Some(i);
                self.cursor_item = 0;
                self.center_on_item(i, 0);
                return;
            }
        }
        self.last_center = center;
    }

    pub(super) fn check_engage_up(&mut self) {
        let center = self.viewport_center();
        for (i, region) in self.regions.iter().enumerate() {
            if region.items.is_empty() {
                continue;
            }
            let last_idx = region.items.len() - 1;
            let last_item_center = region.item_center(last_idx);
            if center <= last_item_center && self.last_center > last_item_center {
                self.engaged_region = Some(i);
                self.cursor_item = last_idx;
                self.center_on_item(i, last_idx);
                return;
            }
        }
        self.last_center = center;
    }

    pub fn scroll_down(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        if let Some(region_idx) = self.engaged_region {
            let item_count = self.regions[region_idx].items.len();
            if self.cursor_item < item_count - 1 {
                self.cursor_item += 1;
                self.center_on_item(region_idx, self.cursor_item);
            } else {
                let is_last_region = region_idx == self.regions.len() - 1;
                if is_last_region {
                    if self.scroll_offset < self.max_offset() {
                        self.scroll_offset += 1;
                        self.last_center = self.viewport_center();
                    }
                } else {
                    self.engaged_region = None;
                    self.last_center = self.viewport_center();
                    if self.scroll_offset < self.max_offset() {
                        self.scroll_offset += 1;
                        self.last_center = self.viewport_center();
                    }
                }
            }
        } else if self.scroll_offset < self.max_offset() {
            self.scroll_offset += 1;
            self.check_engage_down();
        }
    }

    pub fn scroll_up(&mut self) {
        if self.lines.is_empty() {
            return;
        }

        if let Some(region_idx) = self.engaged_region {
            if self.cursor_item > 0 {
                self.cursor_item -= 1;
                self.center_on_item(region_idx, self.cursor_item);
            } else {
                let is_first_region = region_idx == 0;
                if is_first_region {
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                        self.last_center = self.viewport_center();
                    }
                } else {
                    self.engaged_region = None;
                    self.last_center = self.viewport_center();
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                        self.last_center = self.viewport_center();
                    }
                }
            }
        } else if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            self.check_engage_up();
        }
    }

    pub fn engage_first(&mut self) {
        for (i, region) in self.regions.iter().enumerate() {
            if !region.items.is_empty() {
                self.engaged_region = Some(i);
                self.cursor_item = 0;
                self.center_on_item(i, 0);
                return;
            }
        }
    }

    pub fn engage_by_id(&mut self, id: &str) -> bool {
        for (ri, region) in self.regions.iter().enumerate() {
            for (ci, item) in region.items.iter().enumerate() {
                if item.id == id {
                    self.engaged_region = Some(ri);
                    self.cursor_item = ci;
                    return true;
                }
            }
        }
        false
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
        self.last_center = self.viewport_center();
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.engaged_region = None;
        self.cursor_item = 0;
        self.last_center = self.viewport_center();
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_offset();
        self.engaged_region = None;
        self.cursor_item = 0;
        self.last_center = self.viewport_center();
    }

    pub fn page_down(&mut self, lines: usize) {
        for _ in 0..lines {
            self.scroll_down();
        }
    }

    pub fn page_up(&mut self, lines: usize) {
        for _ in 0..lines {
            self.scroll_up();
        }
    }
}
