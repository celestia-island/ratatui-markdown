# Démarrage

## Prérequis

- **Rust** 1.74 ou ultérieur
- **ratatui** 0.29 (automatiquement récupéré comme dépendance)

## Installation

Ajoutez à votre `Cargo.toml` :

```toml
[dependencies]
ratatui-markdown = "0.1"
```

Cela active toutes les fonctionnalités par défaut (`markdown`, `scroll`, `tree`, `preview`).

### Fonctionnalités Sélectives

Pour réduire le temps de compilation et les dépendances, activez uniquement ce dont vous avez besoin :

```toml
# Rendu Markdown uniquement
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }

# Système de défilement uniquement
ratatui-markdown = { version = "0.1", default-features = false, features = ["scroll"] }

# Vue arborescente (inclut scroll, serde_json et toml)
ratatui-markdown = { version = "0.1", default-features = false, features = ["tree"] }
```

## Utilisation de Base

### Rendu Markdown

```rust
use ratatui_markdown::markdown::MarkdownRenderer;
use ratatui_markdown::theme::RichTextTheme;

// Créer un moteur de rendu avec une largeur de contenu maximale
let renderer = MarkdownRenderer::new(80);

// Analyser le texte Markdown en blocs
let blocks = renderer.parse("# Bonjour\n\nCeci est du texte en **gras** et en *italique*.");

// Rendre les blocs en ratatui::text::Line<'static>
let lines = renderer.render(&blocks, &my_theme);
```

### Parcourir un Arbre

```rust
use ratatui_markdown::tree::CollapsibleTree;

// Analyser du JSON en un arbre rétractable
let json_str = r#"{"nom": "projet", "deps": {"ratatui": "0.29", "serde": "1.0"}}"#;
let mut tree = CollapsibleTree::from_json_str(json_str).unwrap();

// Rendre les lignes de l'arbre
let lines = tree.render_lines(80, &my_theme);

// Obtenir les éléments focalisables pour la navigation
let items = tree.build_focusable_items();

// Basculer un nœud
tree.toggle("deps/serde");
```

### Utiliser le Widget MarkdownPreview

Le widget `MarkdownPreview` combine tout en une seule vue défilable :

```rust
use ratatui_markdown::preview::MarkdownPreview;
use ratatui_markdown::theme::RichTextTheme;

let mut preview = MarkdownPreview::new()
    .with_left_padding(true);

// Définir le contenu Markdown
preview.set_content("# Bienvenue\n\n- Élément un\n- Élément deux\n\n```rust\nlet x = 42;\n```");

// Définir un arbre rétractable (optionnel)
let tree = CollapsibleTree::from_json_str(r#"{"config": {"port": 8080}}"#).unwrap();
preview.set_tree(Some(tree));

// Gérer les entrées clavier
preview.scroll_up();
preview.scroll_down();
preview.page_up(10);
preview.page_down(10);
preview.toggle_tree_node(); // Touche Entrée

// Rendu dans votre boucle de dessin ratatui
fn draw(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}
```

## Implémenter un Thème

La bibliothèque utilise un trait pour obtenir toutes les couleurs :

```rust
use ratatui::style::Color;
use ratatui_markdown::theme::{Generation, RichTextTheme};

struct MonTheme;

impl RichTextTheme for MonTheme {
    fn generation(&self) -> Generation { Generation(1) }
    fn get_text_color(&self) -> Color { Color::White }
    fn get_muted_text_color(&self) -> Color { Color::Gray }
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
```

Modifiez la valeur de retour de `generation()` pour invalider le cache du widget d'aperçu et forcer un nouveau rendu (par exemple, lorsque l'utilisateur change de thème à l'exécution).

## Prochaines Étapes

- [Module Markdown](markdown.md) — API complète d'analyse et de rendu Markdown
- [Système de Défilement](scroll.md) — comprendre l'architecture du défilement hybride
- [Vue Arborescente](tree.md) — rendu et interaction avec les arbres JSON/TOML
- [Widget Aperçu](preview.md) — le widget unifié de haut niveau
- [Thème](theme.md) — guide complet de personnalisation des thèmes
