use super::SpanTree;

pub(super) fn navigate_up(tree: &mut SpanTree) {
    if tree.entries.is_empty() {
        return;
    }

    let current = tree.selected_index().unwrap_or(0);
    if current > 0 {
        let new_idx = current - 1;
        tree.selected_id = Some(tree.entries[new_idx].id.clone());
        tree.scroll_to_selected();
    } else if current == 0 {
        tree.scroll_offset = tree.scroll_offset.saturating_sub(1);
    }
}

pub(super) fn navigate_down(tree: &mut SpanTree) {
    if tree.entries.is_empty() {
        return;
    }

    let current = tree.selected_index();
    match current {
        Some(idx) if idx + 1 < tree.entries.len() => {
            let new_idx = idx + 1;
            tree.selected_id = Some(tree.entries[new_idx].id.clone());
            tree.scroll_to_selected();
        }
        Some(_) => {
            let max = tree.max_scroll_offset();
            if tree.scroll_offset < max {
                tree.scroll_offset += 1;
            }
        }
        None => {
            tree.selected_id = Some(tree.entries[0].id.clone());
            tree.scroll_to_selected();
        }
    }
}

pub(super) fn navigate_to_first(tree: &mut SpanTree) {
    if tree.entries.is_empty() {
        return;
    }
    tree.selected_id = Some(tree.entries[0].id.clone());
    tree.scroll_offset = 0;
}

pub(super) fn navigate_to_last(tree: &mut SpanTree) {
    if tree.entries.is_empty() {
        return;
    }
    let last_idx = tree.entries.len() - 1;
    tree.selected_id = Some(tree.entries[last_idx].id.clone());
    tree.scroll_to_selected();
}

pub(super) fn scroll_up(tree: &mut SpanTree, lines: usize) {
    for _ in 0..lines {
        tree.scroll_offset = tree.scroll_offset.saturating_sub(1);
    }
}

pub(super) fn scroll_down(tree: &mut SpanTree, lines: usize) {
    let max = tree.max_scroll_offset();
    tree.scroll_offset = (tree.scroll_offset + lines).min(max);
}
