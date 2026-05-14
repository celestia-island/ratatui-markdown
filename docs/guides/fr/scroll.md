# Système de Défilement

> Défilement hybride intelligent avec navigation par éléments focalisables.

## Aperçu

Le module `scroll` fournit un système de défilement hybride prenant en charge deux modes :

1. **Défilement libre** — lorsqu'aucun élément focalisable n'est en vue, le contenu défile librement
2. **Engagé** — lorsque des éléments focalisables entrent au centre de la zone de visualisation, le curseur s'accroche au premier élément pour la navigation clavier

Gardé derrière le drapeau de fonctionnalité `scroll` (activé par défaut).

## HybridScrollView

Le widget principal qui gère le défilement, les régions de focus et le rendu :

```rust
pub struct HybridScrollView { /* champs */ }

impl HybridScrollView {
    pub fn new() -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;
    pub fn with_cursor_indicator(self, show: bool) -> Self;

    // Gestion du contenu
    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>);
    pub fn set_lines(&mut self, lines: Vec<Line<'static>>);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // État du défilement
    pub fn total_lines(&self) -> usize;
    pub fn get_scroll_offset(&self) -> usize;
    pub fn get_viewport_height(&self) -> usize;
    pub fn set_scroll_offset(&mut self, offset: usize);

    // Navigation
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // Engagement
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;
    pub fn engage_first(&mut self);
    pub fn engage_by_id(&mut self, id: &str) -> bool;

    // Rendu
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

### Configuration

- **with_left_padding** : Ajoute 1 colonne de marge à gauche sur toutes les lignes affichées
- **with_cursor_indicator** : Affiche `> ` (2 colonnes) sur la ligne du curseur engagé (prend le pas sur `left_padding`)

La méthode `effective_padding()` retourne la marge réellement utilisée :
- `2` si l'indicateur de curseur est activé
- `1` si seule la marge gauche est activée
- `0` sinon

## Régions et Éléments Focalisables

```rust
pub struct FocusableItemRange {
    pub start_line: usize,    // inclusif
    pub end_line: usize,      // exclusif
    pub id: String,           // identifiant unique
}

pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}
```

Les régions définissent des étendues de lignes qui deviennent interactives. Lorsque le centre de la zone de visualisation passe sur une région, la vue de défilement s'engage automatiquement et le curseur se positionne sur le premier élément.

### Comportement d'Engagement

- Défiler **vers le bas** dans une région engage le **premier** élément
- Défiler **vers le haut** dans une région engage le **dernier** élément
- Naviguer au-delà du dernier élément d'une région **désengage** et revient au défilement libre
- `scroll_to_top()` et `scroll_to_bottom()` désengagent toujours
- Au sein d'une région, `scroll_up`/`scroll_down` déplacent le curseur entre les éléments

## Autres Widgets de Défilement

### ScrollableList<T>

Une liste défilable générique avec navigation souris/clavier et rendu bordé optionnel :

```rust
pub trait ListItemRenderer {
    fn render_item(&self, index: usize, is_selected: bool, width: usize) -> Line<'static>;
}
```

### ArrowScrollbar

Une barre de défilement personnalisée dessinée avec des symboles de flèches Unicode :

```rust
pub fn render_arrow_scrollbar(
    area: Rect,
    buf: &mut Buffer,
    top: usize,
    bottom: usize,
    theme: &impl RichTextTheme,
);
```

### FollowScrollState

Pour le suivi automatique du contenu (par exemple, sortie en continu) :

```rust
pub struct FollowScrollState {
    // suit si la zone de visualisation est en bas
}
```

### ScrollableRenderResult

Un simple conteneur de panneau défilable :

```rust
pub fn render_scrollable(
    lines: &[Line],
    scroll_offset: usize,
    area: Rect,
    buf: &mut Buffer,
) -> ScrollableRenderResult;
```

## Exemple

```rust
use ratatui_markdown::scroll::{HybridScrollView, FocusableItemRange, FocusableRegion};

let mut scroll = HybridScrollView::new()
    .with_cursor_indicator(true);

// Lignes de contenu
let lines: Vec<Line> = (0..100)
    .map(|i| Line::raw(format!("Ligne {}", i)))
    .collect();

// Rendre les lignes 30-32 focalisables
let region = FocusableRegion {
    items: vec![
        FocusableItemRange { start_line: 30, end_line: 31, id: "element-a".into() },
        FocusableItemRange { start_line: 31, end_line: 32, id: "element-b".into() },
        FocusableItemRange { start_line: 32, end_line: 33, id: "element-c".into() },
    ],
};

scroll.set_content(lines, vec![region]);

// Défilement libre vers le bas ; s'engage automatiquement quand la ligne 30 entre au centre
scroll.scroll_down();

// Naviguer entre les éléments
while scroll.scroll_down() == InputResult::Continue {
    if let Some(id) = scroll.selected_item_id() {
        println!("Sélectionné : {}", id);
    }
}
```
