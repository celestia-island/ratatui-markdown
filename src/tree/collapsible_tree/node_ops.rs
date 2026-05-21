use std::collections::HashSet;

use super::{CollapsibleTree, EntryKind, FlatEntry, ValueType};

pub(crate) struct FlattenContext<'a> {
    pub is_last_stack: &'a mut Vec<bool>,
    pub expanded_paths: &'a HashSet<String>,
    pub entries: &'a mut Vec<FlatEntry>,
}

impl CollapsibleTree {
    pub(crate) fn collect_expandable_paths(
        value: &serde_json::Value,
        path: &str,
    ) -> HashSet<String> {
        let mut paths = HashSet::new();
        match value {
            serde_json::Value::Object(map) if !map.is_empty() => {
                paths.insert(path.to_string());
                for (key, val) in map {
                    let child = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    paths.extend(Self::collect_expandable_paths(val, &child));
                }
            }
            serde_json::Value::Array(arr) if !arr.is_empty() => {
                paths.insert(path.to_string());
                for (i, item) in arr.iter().enumerate() {
                    let child = format!("{}[{}]", path, i);
                    paths.extend(Self::collect_expandable_paths(item, &child));
                }
            }
            _ => {}
        }
        paths
    }

    pub fn expand_to_depth(&mut self, max_depth: usize) {
        self.expanded_paths = Self::collect_expandable_paths_to_depth(&self.root, "", 0, max_depth);
    }

    fn collect_expandable_paths_to_depth(
        value: &serde_json::Value,
        path: &str,
        current_depth: usize,
        max_depth: usize,
    ) -> HashSet<String> {
        let mut paths = HashSet::new();
        match value {
            serde_json::Value::Object(map) if !map.is_empty() => {
                paths.insert(path.to_string());
                if current_depth < max_depth {
                    for (key, val) in map {
                        let child = if path.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", path, key)
                        };
                        paths.extend(Self::collect_expandable_paths_to_depth(
                            val,
                            &child,
                            current_depth + 1,
                            max_depth,
                        ));
                    }
                }
            }
            serde_json::Value::Array(arr) if !arr.is_empty() => {
                paths.insert(path.to_string());
                if current_depth < max_depth {
                    for (i, item) in arr.iter().enumerate() {
                        let child = format!("{}[{}]", path, i);
                        paths.extend(Self::collect_expandable_paths_to_depth(
                            item,
                            &child,
                            current_depth + 1,
                            max_depth,
                        ));
                    }
                }
            }
            _ => {}
        }
        paths
    }

    pub fn flatten(&self) -> Vec<FlatEntry> {
        let mut entries = Vec::new();
        let mut is_last_stack = Vec::new();
        let mut ctx = FlattenContext {
            is_last_stack: &mut is_last_stack,
            expanded_paths: &self.expanded_paths,
            entries: &mut entries,
        };
        Self::flatten_node(
            &self.root,
            "",
            0,
            self.show_root,
            &self.root_label,
            &mut ctx,
        );
        entries
    }

    pub(crate) fn flatten_node(
        value: &serde_json::Value,
        path: &str,
        depth: usize,
        render_header: bool,
        label_override: &str,
        ctx: &mut FlattenContext,
    ) {
        match value {
            serde_json::Value::Object(map) if !map.is_empty() => {
                let is_expanded = ctx.expanded_paths.contains(path) || !render_header;
                let count = map.len();
                let label = if !label_override.is_empty() {
                    label_override.to_string()
                } else {
                    path.to_string()
                };
                let count_str = format!("{{{}}}", count);

                if render_header {
                    ctx.entries.push(FlatEntry {
                        path: path.to_string(),
                        depth,
                        is_last_stack: ctx.is_last_stack.clone(),
                        kind: if is_expanded {
                            EntryKind::Expanded {
                                label: label.clone(),
                                count_str,
                            }
                        } else {
                            EntryKind::Collapsed {
                                label: label.clone(),
                                count_str,
                            }
                        },
                    });
                }

                if is_expanded {
                    let keys: Vec<_> = map.keys().collect();
                    for (i, key) in keys.iter().enumerate() {
                        let is_last = i == count - 1;
                        ctx.is_last_stack.push(is_last);
                        let child_path = if path.is_empty() {
                            (*key).clone()
                        } else {
                            format!("{}.{}", path, key)
                        };
                        Self::flatten_node(
                            &map[*key],
                            &child_path,
                            if render_header { depth + 1 } else { depth },
                            true,
                            "",
                            ctx,
                        );
                        ctx.is_last_stack.pop();
                    }
                }
            }
            serde_json::Value::Array(arr) if !arr.is_empty() => {
                let is_expanded = ctx.expanded_paths.contains(path) || !render_header;
                let count = arr.len();
                let label = if !label_override.is_empty() {
                    label_override.to_string()
                } else {
                    path.to_string()
                };
                let count_str = format!("[{}]", count);

                if render_header {
                    ctx.entries.push(FlatEntry {
                        path: path.to_string(),
                        depth,
                        is_last_stack: ctx.is_last_stack.clone(),
                        kind: if is_expanded {
                            EntryKind::Expanded {
                                label: label.clone(),
                                count_str,
                            }
                        } else {
                            EntryKind::Collapsed {
                                label: label.clone(),
                                count_str,
                            }
                        },
                    });
                }

                if is_expanded {
                    for (i, item) in arr.iter().enumerate() {
                        let is_last = i == count - 1;
                        ctx.is_last_stack.push(is_last);
                        let item_path = format!("{}[{}]", path, i);
                        let item_label = format!("[{}]", i);

                        match item {
                            serde_json::Value::Object(m) if !m.is_empty() => {
                                Self::flatten_node(
                                    item,
                                    &item_path,
                                    if render_header { depth + 1 } else { depth },
                                    true,
                                    &item_label,
                                    ctx,
                                );
                            }
                            serde_json::Value::Array(a) if !a.is_empty() => {
                                Self::flatten_node(
                                    item,
                                    &item_path,
                                    if render_header { depth + 1 } else { depth },
                                    true,
                                    &item_label,
                                    ctx,
                                );
                            }
                            _ => {
                                let (value_str, value_type) = format_primitive(item);
                                ctx.entries.push(FlatEntry {
                                    path: item_path,
                                    depth: if render_header { depth + 1 } else { depth },
                                    is_last_stack: ctx.is_last_stack.clone(),
                                    kind: EntryKind::Leaf {
                                        key: item_label,
                                        value: value_str,
                                        value_type,
                                    },
                                });
                            }
                        }
                        ctx.is_last_stack.pop();
                    }
                }
            }
            serde_json::Value::Object(_) => {
                ctx.entries.push(FlatEntry {
                    path: path.to_string(),
                    depth,
                    is_last_stack: ctx.is_last_stack.clone(),
                    kind: EntryKind::Leaf {
                        key: if !label_override.is_empty() {
                            label_override.to_string()
                        } else {
                            path.to_string()
                        },
                        value: "{}".to_string(),
                        value_type: ValueType::Null,
                    },
                });
            }
            serde_json::Value::Array(_) => {
                ctx.entries.push(FlatEntry {
                    path: path.to_string(),
                    depth,
                    is_last_stack: ctx.is_last_stack.clone(),
                    kind: EntryKind::Leaf {
                        key: if !label_override.is_empty() {
                            label_override.to_string()
                        } else {
                            path.to_string()
                        },
                        value: "[]".to_string(),
                        value_type: ValueType::Null,
                    },
                });
            }
            _ => {
                let (value_str, value_type) = format_primitive(value);
                ctx.entries.push(FlatEntry {
                    path: path.to_string(),
                    depth,
                    is_last_stack: ctx.is_last_stack.clone(),
                    kind: EntryKind::Leaf {
                        key: if !label_override.is_empty() {
                            label_override.to_string()
                        } else {
                            path.to_string()
                        },
                        value: value_str,
                        value_type,
                    },
                });
            }
        }
    }
}

pub(crate) fn format_primitive(value: &serde_json::Value) -> (String, ValueType) {
    match value {
        serde_json::Value::String(s) => (
            format!("\"{}\"", s.replace('\t', "    ")),
            ValueType::String,
        ),
        serde_json::Value::Number(n) => (n.to_string(), ValueType::Number),
        serde_json::Value::Bool(b) => (b.to_string(), ValueType::Boolean),
        serde_json::Value::Null => ("null".to_string(), ValueType::String),
        _ => (value.to_string().replace('\t', "    "), ValueType::String),
    }
}
