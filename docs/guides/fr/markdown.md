# Module Markdown

> Analyse et rend le texte Markdown en `ratatui::text::Line`s stylisées.

## Aperçu

Le module `markdown` fournit un analyseur et un moteur de rendu Markdown personnalisés conçus pour la sortie terminal. Il ne s'agit **pas** d'un analyseur conforme à CommonMark — il cible le sous-ensemble de Markdown le plus utile dans les IHM terminal.

Gardé derrière le drapeau de fonctionnalité `markdown` (activé par défaut).

## MarkdownRenderer

```rust
pub struct MarkdownRenderer {
    max_width: usize,
}

impl MarkdownRenderer {
    pub fn new(max_width: usize) -> Self;
    pub fn parse(&self, markdown: &str) -> Vec<MarkdownBlock>;
    pub fn render(&self, blocks: &[MarkdownBlock], theme: &impl RichTextTheme) -> Vec<Line<'static>>;
}
```

### Constructeur

`MarkdownRenderer::new(max_width)` prend la largeur de contenu disponible en colonnes. Cette largeur est utilisée pour le retour à la ligne des paragraphes (compatible CJK) et le dimensionnement des colonnes de tableaux.

### Analyse

`parse()` prend un `&str` de texte Markdown et retourne un `Vec<MarkdownBlock>`. L'analyseur est orienté ligne et traite les blocs séquentiellement.

### Rendu

`render()` prend les blocs analysés et un thème, produisant un `Vec<Line<'static>>` utilisable directement dans les widgets ratatui.

## Enum MarkdownBlock

```rust
pub enum MarkdownBlock {
    Heading1(String),              // # Titre
    Heading2(String),              // ## Titre
    Heading3(String),              // ### Titre
    Paragraph(Vec<String>),        // Lignes de paragraphe avec retour à la ligne
    CodeBlock(String, String),     // (langage, contenu)
    InlineCode(String),            // `code en ligne`
    ListItem(String, u8),          // (contenu, niveau_d'indentation)
    Blockquote(String),            // > texte cité
    HorizontalRule,                // --- ou *** ou ___
    BlankLine,                     // ligne vide
    Table {                        // | col1 | col2 |
        headers: Vec<String>,      //   |------|------|
        rows: Vec<Vec<String>>,    //   | val1 | val2 |
    },
}
```

### Détails des Blocs

**Titres** (H1-H3) : Rendu avec `primary_color`, le H1 utilisant `Modifier::BOLD`.

**Paragraphes** : Le texte est soumis à un retour à la ligne compatible CJK jusqu'à `max_width`. Chaque ligne coupée devient une entrée dans le `Vec<String>`.

**Blocs de code** (délimités par ` ``` `) : Rendu avec `muted_text_color` dans des boîtes bordées utilisant des caractères de dessin de boîte. Les blocs de code Mermaid sont silencieusement ignorés.

**Code en ligne** : Rendu avec `secondary_color` et `Modifier::DIM`.

**Listes** : Non ordonnées (`-`, `*`, `+`) et ordonnées (`1.`, `2.`). Chaque élément conserve son niveau d'indentation. Les sous-éléments sont indentés visuellement.

**Citations** : Préfixées par une barre `│` colorée et rendues en `muted_text_color`.

**Tableaux** : Les colonnes sont dimensionnées proportionnellement en fonction de la largeur du contenu. Les cellules sont coupées, les en-têtes utilisent `Modifier::BOLD` et les bordures utilisent des caractères de dessin de boîte.

## Formatage en Ligne

Le formatage en ligne est appliqué **à l'intérieur** du texte des paragraphes et des éléments de liste :

| Markdown        | Effet Rendu                        |
|-----------------|------------------------------------|
| `**texte**`     | **Gras** (`Modifier::BOLD`)       |
| `*texte*`       | *Italique* (`Modifier::ITALIC`)   |
| `***texte***`   | ***Gras+Italique***               |
| `` `code` ``    | Style `Code en ligne`             |

```rust
pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>>;
```

Cette fonction autonome est également réexportée pour une utilisation en dehors de `MarkdownRenderer`.

## Exemple

```rust
use ratatui_markdown::markdown::MarkdownRenderer;

let md = r#"
# Titre

Ceci est un paragraphe avec du texte en **gras** et en *italique*.

## Code

```rust
fn main() {
    println!("Bonjour !");
}
```

| Nom     | Version |
|---------|---------|
| ratatui | 0.29    |
| serde   | 1.0     |
"#;

let renderer = MarkdownRenderer::new(80);
let blocks = renderer.parse(md);
let lines = renderer.render(&blocks, theme);
// utiliser les lignes dans un widget ratatui
```
