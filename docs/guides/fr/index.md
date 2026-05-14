<div align="center">
  <img src="../../../examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

# ratatui-markdown

> Une bibliothèque Rust offrant un rendu Markdown, des arbres JSON/TOML rétractables et des widgets de défilement riches pour ratatui.
>
> **Construit avec**: [ratatui](https://github.com/ratatui/ratatui) 0.29 + Rust pur
>
> **Version Rust minimale**: 1.74

<div align="center">
  <p>
    <a href="../../README.md">English</a> |
    <a href="../zhs/index.md">简体中文</a> |
    <a href="../zht/index.md">繁體中文</a> |
    <a href="../ja/index.md">日本語</a> |
    <a href="../ko/index.md">한국어</a> |
    <a href="../fr/index.md">Français</a> |
    <a href="../es/index.md">Español</a> |
    <a href="../ru/index.md">Русский</a> |
    <a href="../ar/index.md">العربية</a>
  </p>
</div>

## Qu'est-ce que ratatui-markdown ?

ratatui-markdown est une bibliothèque de rendu riche pour les interfaces utilisateur terminal construites avec [ratatui](https://github.com/ratatui/ratatui). Elle fournit quatre modules fonctionnels principaux qui peuvent être utilisés indépendamment ou combinés via le widget `MarkdownPreview`.

## Modules Principaux

### Rendu Markdown

Analyse et rend le texte Markdown en sortie terminal stylisée :

- **Titres**: H1 (`#`), H2 (`##`), H3 (`###`)
- **Paragraphes** avec retour à la ligne automatique compatible CJK
- **Formatage en ligne**: `**gras**`, `*italique*`, `***gras+italique***`, `` `code en ligne` ``
- **Blocs de code** avec étiquettes de langage optionnelles (les blocs mermaid sont ignorés)
- **Citations** (`>`)
- **Listes non ordonnées** (`-`, `*`, `+`) et ordonnées (`1.`, `2.`)
- **Lignes horizontales** (`---`, `***`, `___`)
- **Tableaux** avec largeurs de colonnes proportionnelles et retour à la ligne des cellules

### Vue Arborescente Rétractable

Analyse et navigation interactive dans des données structurées :

- Analyse **JSON** et **TOML** en arbres rétractables
- **Déplier / Replier** nœuds individuels, tout déplier, tout replier, déplier par profondeur
- **Clés stylisées**: mode JSON (clés entre guillemets + `:`) ou mode TOML (clés nues + `=`)
- **Navigation clavier**: sélection et basculement par curseur
- **Coloration par type de valeur**: chaînes, nombres, booléens, null — chacun avec sa couleur de thème

### Système de Défilement Hybride

Défilement intelligent gérant à la fois la navigation libre et la navigation par éléments :

- **Mode défilement libre**: parcourez le contenu librement
- **Mode engagé**: s'active automatiquement lorsque des éléments focalisables entrent dans la vue
- **Navigation par curseur**: déplacez-vous entre les éléments focalisables au clavier
- **Indicateur de curseur**: préfixe visuel `> ` sur les lignes engagées
- **Barre de défilement**: superposition à base de flèches
- **Pagination**: support de `page_up` / `page_down`

### Widget MarkdownPreview

Le widget de haut niveau qui intègre tout :

- Rend le contenu Markdown, les vues arborescentes et les éléments d'action dans une seule mise en page défilable
- **Cache**: reconstruit la sortie uniquement lorsque le contenu, la largeur ou la génération du thème change
- **Suppression du préambule TOML**: supprime automatiquement le préambule TOML délimité par `+++`
- **Éléments d'action**: éléments étiquetés sélectionnables au clavier avec IDs d'action
- Délègue toute la navigation à `HybridScrollView`

## Démarrage Rapide

```toml
[dependencies]
ratatui-markdown = "0.1"
```

```rust
use ratatui_markdown::preview::MarkdownPreview;

let mut preview = MarkdownPreview::new();
preview.set_content("# Bonjour le monde !\n\nCeci est un paragraphe.");
// rendu et gestion des entrées dans la boucle d'application ratatui
```

## Drapeaux de Fonctionnalités

Toutes les fonctionnalités sont activées par défaut. Désactivez les fonctionnalités par défaut pour n'activer que ce dont vous avez besoin :

```toml
[dependencies]
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }
```

| Fonctionnalité | Dépendances       | Description                                     |
|----------------|--------------------|-------------------------------------------------|
| `markdown`     | —                  | Analyseur et moteur de rendu Markdown           |
| `scroll`       | —                  | HybridScrollView, listes défilables, barre de défilement |
| `tree`         | `scroll`, `serde_json`, `toml` | Arbre JSON/TOML rétractable        |
| `preview`      | `markdown`, `scroll`, `tree` | Widget unifié MarkdownPreview        |

## Structure du Projet

```
ratatui-markdown/
  src/
   ├── lib.rs                  # Point d'entrée : modules avec portes de fonctionnalités
   ├── theme.rs                # Trait RichTextTheme, jeton Generation
   ├── constants/
   │   ├── mod.rs              # Réexportations
   │   ├── box_chars.rs        # Constantes de caractères de boîte
   │   └── list_prefix.rs      # Connecteurs d'arbre, flèches, marqueurs
   ├── markdown/
   │   ├── mod.rs              # Structure MarkdownRenderer
   │   ├── parser.rs           # Analyseur Markdown niveau bloc
   │   ├── types.rs            # Enum MarkdownBlock, TextToken
   │   ├── render.rs           # Moteur de rendu niveau bloc (+ tableaux)
   │   ├── inline.rs           # Analyseur de formatage en ligne
   │   └── text.rs             # Retour à la ligne compatible CJK
   ├── scroll/
   │   ├── mod.rs              # Réexportations
   │   ├── hybrid_scroll/      # HybridScrollView (widget principal)
   │   ├── scrollable_list.rs  # ScrollableList<T> générique
   │   ├── scrollable_panel.rs # Assistant de défilement simple
   │   ├── focusable_list.rs   # Rendu FocusableItemList
   │   ├── follow_scroll.rs    # FollowScrollState
   │   └── scrollbar.rs        # Widget ArrowScrollbar
   ├── tree/
   │   ├── mod.rs              # Réexportations
   │   ├── tree_lines.rs       # Construction de lignes d'arbre
   │   └── collapsible_tree/   # CollapsibleTree + opérations + rendu
   └── preview/
       └── mod.rs              # Widget unifié MarkdownPreview
```

## Documentation

| Guide | Description |
|-------|-------------|
| [Démarrage](getting-started.md) | Installation et premier rendu |
| [Markdown](markdown.md) | Analyse et rendu Markdown |
| [Système de Défilement](scroll.md) | Défilement hybride, navigation |
| [Vue Arborescente](tree.md) | Arbres JSON/TOML, déplier/replier |
| [Widget Aperçu](preview.md) | Tout combiner avec MarkdownPreview |
| [Thème](theme.md) | Implémentation de RichTextTheme |
| [Contribuer](contributing.md) | Guide de développement et de contribution |

## Licence

Double licence MIT OR Apache-2.0.
