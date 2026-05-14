# Widget MarkdownPreview

> Le widget unifié de haut niveau combinant Markdown, arbres et actions dans une seule vue défilable.

## Aperçu

`MarkdownPreview` est le widget d'intégration de plus haut niveau. Il combine le rendu Markdown, l'affichage d'arbre rétractable et les éléments d'action dans un seul `HybridScrollView`. C'est le point d'entrée recommandé pour la plupart des applications.

Gardé derrière le drapeau de fonctionnalité `preview` (activé par défaut, nécessite toutes les autres fonctionnalités).

## API

```rust
pub struct ActionItem {
    pub id: String,
    pub label: String,
}

impl MarkdownPreview {
    pub fn new() -> Self;

    // Configuration
    pub fn with_strip_frontmatter(self, strip: bool) -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;

    // Contenu
    pub fn set_content(&mut self, content: &str);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // Arbre
    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>);
    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree>;
    pub fn has_tree(&self) -> bool;
    pub fn toggle_tree_node(&mut self) -> bool;

    // Éléments d'action
    pub fn set_action_items(&mut self, items: Vec<ActionItem>);
    pub fn selected_action_id(&self) -> Option<&str>;

    // Navigation
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // État
    pub fn total_lines(&self) -> usize;
    pub fn scroll_offset(&self) -> usize;
    pub fn visible_height(&self) -> usize;
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;

    // Rendu
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

## Configuration

### with_strip_frontmatter

Lorsqu'il est activé (par défaut), le contenu délimité par `+++` est traité comme du préambule TOML et supprimé avant le rendu :

```
+++
titre = "Mon Document"
auteur = "Jean Dupont"
+++

# Contenu Réel
Ceci est le corps.
```

Seuls `# Contenu Réel` et le paragraphe suivant apparaîtraient dans la sortie.

### with_left_padding

Ajoute 1 colonne de marge à gauche au contenu rendu (transmis à `HybridScrollView`).

## Disposition du Contenu

Le widget rend le contenu dans l'ordre vertical :

1. **Arbre** (si présent) — lignes d'arbre avec nœuds focalisables
2. **Markdown** — contenu analysé et rendu
3. **Éléments d'action** — étiquettes d'action focalisables avec des crochets `[étiquette]`

Une ligne vide est insérée entre les sections, et chaque section reçoit sa propre `FocusableRegion` dans la vue de défilement.

## Cache

`MarkdownPreview` met en cache la sortie rendue et ne reconstruit que lorsque :

- Le contenu change (`set_content` avec un texte différent)
- La largeur change (redimensionnement du terminal)
- La génération du thème change (`theme.generation()` retourne une nouvelle valeur)
- L'arbre est modifié (`set_tree`, `toggle_tree_node`)
- Les éléments d'action sont modifiés (`set_action_items`)

Utilisez `theme.generation()` pour déclencher un nouveau rendu après des changements de thème.

## Gestion du Préambule TOML

Le préambule est supposé être du TOML. Il n'est **pas analysé** — il est simplement retiré de la sortie rendue. La première ligne `+++` démarre le mode préambule, la seconde `+++` le termine. Les lignes avant et après le bloc de préambule sont rendues normalement.

Si le contenu ne commence pas par `+++`, aucune suppression n'a lieu.

## Éléments d'Action

Les éléments d'action fournissent des options sélectionnables au clavier en bas de la vue :

```rust
preview.set_action_items(vec![
    ActionItem { id: "confirmer".into(), label: " Confirmer ".into() },
    ActionItem { id: "annuler".into(), label: " Annuler ".into() },
]);

// Dans votre gestionnaire d'entrées :
if let Some("confirmer") = preview.selected_action_id() {
    // gérer la confirmation
}
```

Les IDs des éléments d'action sont préfixés par `action:` en interne pour éviter les collisions avec les chemins de nœuds d'arbre.

## Exemple

```rust
use ratatui_markdown::preview::{MarkdownPreview, ActionItem};
use ratatui_markdown::tree::CollapsibleTree;

let mut preview = MarkdownPreview::new()
    .with_strip_frontmatter(true)
    .with_left_padding(true);

// Définir le contenu Markdown (avec préambule TOML)
preview.set_content(concat!(
    "+++\n",
    "titre = \"Mon Doc\"\n",
    "+++\n",
    "\n",
    "# Bonjour\n\nContenu ici.\n",
));

// Définir un arbre
let tree = CollapsibleTree::from_json_str(r#"{"config": {"theme": "sombre"}}"#).unwrap();
preview.set_tree(Some(tree));

// Définir les éléments d'action
preview.set_action_items(vec![
    ActionItem { id: "modifier".into(), label: " Modifier ".into() },
    ActionItem { id: "enregistrer".into(), label: " Enregistrer ".into() },
]);

// Dans la fonction de rendu de votre application ratatui :
fn render_ui(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}

// Gérer les entrées
fn handle_input(key: rataui::crossterm::event::KeyCode, preview: &mut MarkdownPreview) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => preview.scroll_up(),
        KeyCode::Down | KeyCode::Char('j') => preview.scroll_down(),
        KeyCode::PageUp => preview.page_up(20),
        KeyCode::PageDown => preview.page_down(20),
        KeyCode::Home => preview.scroll_to_top(),
        KeyCode::End => preview.scroll_to_bottom(),
        KeyCode::Enter => { preview.toggle_tree_node(); }
        _ => {}
    }
}
```
