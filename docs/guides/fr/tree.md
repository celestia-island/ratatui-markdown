# Vue Arborescente

> Arbre rétractable interactif pour les données JSON et TOML.

## Aperçu

Le module `tree` analyse du JSON ou du TOML en un arbre rétractable interactif. Les utilisateurs peuvent déplier/replier les nœuds et naviguer au clavier dans une IHM terminal.

Gardé derrière le drapeau de fonctionnalité `tree` (nécessite les dépendances `scroll`, `serde_json` et `toml`).

## CollapsibleTree

```rust
pub struct CollapsibleTree { /* champs */ }

impl CollapsibleTree {
    // Constructeurs
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_toml_str(toml: &str) -> Result<Self, Box<dyn Error>>;
    pub fn from_value(value: serde_json::Value, style: KeyStyle) -> Self;

    // Manipulation de l'arbre
    pub fn toggle(&mut self, path: &str) -> bool;
    pub fn handle_toggle(&mut self, id: &str) -> bool;
    pub fn expand_all(&mut self);
    pub fn collapse_all(&mut self);
    pub fn expand_to_depth(&mut self, depth: usize);

    // Rendu
    pub fn render_lines(&self, width: usize, theme: &impl RichTextTheme) -> Vec<Line<'static>>;
    pub fn flatten(&self) -> Vec<FlatEntry>;
    pub fn build_focusable_items(&self) -> Vec<FocusableItemRange>;
}
```

### Constructeurs

- **`from_json_str`** : Analyse une chaîne JSON en un arbre. Les clés utilisent `KeyStyle::Json` (entre guillemets, avec séparateur `:`).
- **`from_toml_str`** : Analyse une chaîne TOML (convertit en interne en JSON). Les clés utilisent `KeyStyle::Toml` (nues, avec séparateur `=`).
- **`from_value`** : Construit un arbre à partir d'un `serde_json::Value` existant avec un style de clé choisi.

### Manipulation de l'Arbre

```rust
// Basculer en utilisant un chemin séparé par des barres obliques
tree.toggle("dependencies");           // basculer la clé racine
tree.toggle("dependencies/serde");     // basculer la clé imbriquée

// Méthodes pratiques
tree.expand_all();
tree.collapse_all();
tree.expand_to_depth(2);

// Dans un contexte de défilement — utilise l'ID de l'élément sélectionné
if scroll.selected_item_id().is_some() {
    tree.handle_toggle(selected_id);
}
```

### Rendu

- **`render_lines`** : Produit des `Line`s stylisées avec des connecteurs d'arbre et des valeurs colorées.
- **`flatten`** : Retourne une liste plate de toutes les entrées visibles (respecte l'état replié).
- **`build_focusable_items`** : Retourne des plages focalisables pour l'intégration avec `HybridScrollView`, avec des IDs correspondant aux chemins de l'arbre.

## Types de Données

```rust
pub struct FlatEntry {
    pub depth: usize,
    pub key: String,
    pub value: String,
    pub kind: EntryKind,
    pub value_type: ValueType,
    pub path: String,          // ex. "dependencies/serde"
    pub is_last: bool,
}

pub enum EntryKind {
    Collapsed,  // a des enfants, actuellement replié : [+]
    Expanded,   // a des enfants, actuellement déplié : [-]
    Leaf,       // pas d'enfants
}

pub enum ValueType {
    String, Number, Boolean, Null, Object, Array,
}

pub enum KeyStyle {
    Json,  // "clé": valeur
    Toml,  // clé = valeur
}
```

### Couleurs par Type de Valeur

Chaque `ValueType` correspond à une méthode de couleur de thème :

| ValueType | Méthode de Thème          |
|-----------|---------------------------|
| String    | `get_json_string_color()` |
| Number    | `get_json_number_color()` |
| Boolean   | `get_json_bool_color()`   |
| Null      | `get_json_null_color()`   |

## Assistants de Lignes d'Arbre

Le module `tree_lines` (réexporté depuis `crate::tree`) fournit des fonctions de construction de lignes bas niveau :

```rust
pub fn make_body_line(key: &str, value: &str, style: &Style, value_style: &Style) -> Line<'static>;
pub fn make_status_line(status: &str, style: &Style) -> Line<'static>;
pub fn make_branch_dispatch_line(kind: EntryKind, ...) -> Line<'static>;
```

## Exemple

```rust
use ratatui_markdown::tree::CollapsibleTree;

let toml_content = r#"
[package]
name = "mon-appli"
version = "0.1.0"

[dependencies]
ratatui = "0.29"
serde = { version = "1.0", features = ["derive"] }

[features]
default = ["std"]
std = []
"#;

let mut tree = CollapsibleTree::from_toml_str(toml_content).unwrap();

// Tout déplier
tree.expand_all();

// Rendu en lignes
let lines = tree.render_lines(80, theme);

// Replier le sous-arbre des dépendances
tree.toggle("dependencies");

// Nouveau rendu — les dépendances sont maintenant repliées
let lines = tree.render_lines(80, theme);

// Obtenir les éléments focalisables pour la navigation
let items = tree.build_focusable_items();
```
