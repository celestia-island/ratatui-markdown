mod node_ops;
mod rendering;

use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KeyStyle {
    #[default]
    Json,
    Toml,
}

pub struct CollapsibleTree {
    pub root: serde_json::Value,
    pub expanded_paths: HashSet<String>,
    pub key_style: KeyStyle,
    pub base_indent: usize,
    pub show_root: bool,
    pub root_label: String,
}

#[derive(Debug, Clone)]
pub struct FlatEntry {
    pub path: String,
    pub depth: usize,
    pub is_last_stack: Vec<bool>,
    pub kind: EntryKind,
}

#[derive(Debug, Clone)]
pub enum EntryKind {
    Collapsed {
        label: String,
        count_str: String,
    },
    Expanded {
        label: String,
        count_str: String,
    },
    Leaf {
        key: String,
        value: String,
        value_type: ValueType,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Null,
}

impl CollapsibleTree {
    pub fn from_value(value: serde_json::Value) -> Self {
        Self {
            root: value,
            expanded_paths: HashSet::new(),
            key_style: KeyStyle::Json,
            base_indent: 0,
            show_root: true,
            root_label: String::new(),
        }
    }

    pub fn from_json_str(s: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(s).ok()?;
        Some(Self::from_value(value))
    }

    pub fn from_toml_str(s: &str) -> Option<Self> {
        let toml_value: toml::Value = s.parse().ok()?;
        let json_value = toml_to_json(&toml_value);
        Some(Self {
            root: json_value,
            expanded_paths: HashSet::new(),
            key_style: KeyStyle::Toml,
            base_indent: 0,
            show_root: true,
            root_label: String::new(),
        })
    }

    pub fn with_key_style(mut self, style: KeyStyle) -> Self {
        self.key_style = style;
        self
    }

    pub fn with_base_indent(mut self, indent: usize) -> Self {
        self.base_indent = indent;
        self
    }

    pub fn with_show_root(mut self, show: bool) -> Self {
        self.show_root = show;
        self
    }

    pub fn with_root_label(mut self, label: &str) -> Self {
        self.root_label = label.to_string();
        self
    }

    pub fn toggle(&mut self, path: &str) {
        if self.expanded_paths.contains(path) {
            self.expanded_paths.remove(path);
        } else {
            self.expanded_paths.insert(path.to_string());
        }
    }

    pub fn expand_all(&mut self) {
        self.expanded_paths = Self::collect_expandable_paths(&self.root, "");
    }

    pub fn collapse_all(&mut self) {
        self.expanded_paths.clear();
    }
}

fn toml_to_json(toml_value: &toml::Value) -> serde_json::Value {
    match toml_value {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Array(arr) => serde_json::Value::Array(arr.iter().map(toml_to_json).collect()),
        toml::Value::Table(table) => {
            let map: serde_json::Map<String, serde_json::Value> = table
                .iter()
                .map(|(k, v)| (k.clone(), toml_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeConfig;
    use ratatui::style::Color;
    use ratatui::text::Span;

    fn test_theme() -> ThemeConfig {
        ThemeConfig::default()
            .with_focused_border_color(Color::Cyan)
            .with_json_key_color(Color::Cyan)
            .with_info_color(Color::Blue)
    }

    #[test]
    fn from_json_simple() -> anyhow::Result<()> {
        let json = r#"{"name": "exec", "port": 8080}"#;
        let tree = CollapsibleTree::from_json_str(json)
            .ok_or_else(|| anyhow::anyhow!("failed to parse json"))?;
        assert_eq!(tree.total_lines(), 1);
        Ok(())
    }

    #[test]
    fn from_toml_simple() -> anyhow::Result<()> {
        let toml = r#"name = "exec"
agent = "skemma""#;
        let tree = CollapsibleTree::from_toml_str(toml)
            .ok_or_else(|| anyhow::anyhow!("failed to parse toml"))?;
        assert_eq!(tree.total_lines(), 1);
        Ok(())
    }

    #[test]
    fn expand_all_json() -> anyhow::Result<()> {
        let json = r#"{"name": "exec", "agent": "skemma"}"#;
        let mut tree = CollapsibleTree::from_json_str(json)
            .ok_or_else(|| anyhow::anyhow!("failed to parse json"))?;
        tree.expand_all();
        assert_eq!(tree.total_lines(), 3);
        Ok(())
    }

    #[test]
    fn nested_expand_collapse() -> anyhow::Result<()> {
        let json = r#"{"name": "test", "desc": {"en": "hello", "zhs": "你好"}}"#;
        let mut tree = CollapsibleTree::from_json_str(json)
            .ok_or_else(|| anyhow::anyhow!("failed to parse json"))?;
        tree.expand_all();
        assert_eq!(tree.total_lines(), 5);
        tree.toggle("desc");
        assert_eq!(tree.total_lines(), 3);
        Ok(())
    }

    #[test]
    fn array_of_objects() -> anyhow::Result<()> {
        let json = r#"{"tools": [{"name": "a"}, {"name": "b"}]}"#;
        let mut tree = CollapsibleTree::from_json_str(json)
            .ok_or_else(|| anyhow::anyhow!("failed to parse json"))?;
        tree.expand_all();
        assert_eq!(tree.total_lines(), 6);
        Ok(())
    }

    #[test]
    fn show_root_false() -> anyhow::Result<()> {
        let json = r#"{"name": "exec"}"#;
        let mut tree = CollapsibleTree::from_json_str(json)
            .ok_or_else(|| anyhow::anyhow!("failed to parse json"))?
            .with_show_root(false);
        tree.expand_all();
        assert_eq!(tree.total_lines(), 1);
        Ok(())
    }

    #[test]
    fn handle_toggle_leaf_returns_false() -> anyhow::Result<()> {
        let json = r#"{"name": "exec"}"#;
        let mut tree = CollapsibleTree::from_json_str(json)
            .ok_or_else(|| anyhow::anyhow!("failed to parse json"))?;
        tree.expand_all();
        assert!(!tree.handle_toggle("name"));
        Ok(())
    }

    #[test]
    fn handle_toggle_collapsible() -> anyhow::Result<()> {
        let json = r#"{"desc": {"en": "hello"}}"#;
        let mut tree = CollapsibleTree::from_json_str(json)
            .ok_or_else(|| anyhow::anyhow!("failed to parse json"))?;
        tree.expand_all();
        assert!(tree.handle_toggle("desc"));
        assert_eq!(tree.total_lines(), 2);
        Ok(())
    }

    #[test]
    fn count_expanded_lines() -> anyhow::Result<()> {
        let json = r#"{"a": 1, "b": {"c": 2}}"#;
        assert_eq!(
            CollapsibleTree::count_expanded_lines(&serde_json::from_str(json)?),
            4
        );
        Ok(())
    }

    #[test]
    fn toml_key_style() -> anyhow::Result<()> {
        let toml = r#"name = "exec""#;
        let mut tree = CollapsibleTree::from_toml_str(toml)
            .ok_or_else(|| anyhow::anyhow!("failed to parse toml"))?
            .with_show_root(false);
        tree.expand_all();
        let lines = tree.render_lines(80, &test_theme());
        let line_str = spans_to_string(&lines[0].spans);
        assert!(line_str.contains("name = "));
        assert!(!line_str.contains("\"name\""));
        Ok(())
    }

    #[test]
    fn json_key_style() -> anyhow::Result<()> {
        let json = r#"{"name": "exec"}"#;
        let mut tree = CollapsibleTree::from_json_str(json)
            .ok_or_else(|| anyhow::anyhow!("failed to parse json"))?
            .with_key_style(KeyStyle::Json)
            .with_show_root(false);
        tree.expand_all();
        let lines = tree.render_lines(80, &test_theme());
        let line_str = spans_to_string(&lines[0].spans);
        assert!(line_str.contains("\"name\": "));
        Ok(())
    }

    fn spans_to_string(spans: &[Span]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }
}
