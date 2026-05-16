<div align="center">
  <img src="../../../examples/logo.webp" alt="ratatui-markdown logo" width="200" />
</div>

# ratatui-markdown

> Une bibliothèque Rust offrant un rendu Markdown, des diagrammes Mermaid, la coloration syntaxique, des arbres JSON/TOML rétractables et des widgets de défilement riches pour ratatui.
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

ratatui-markdown est une bibliothèque de rendu riche pour les interfaces utilisateur terminal construites avec [ratatui](https://github.com/ratatui/ratatui). Elle fournit plusieurs modules fonctionnels qui peuvent être utilisés indépendamment ou combinés via les widgets `MarkdownPreview` / `MarkdownViewer`.

## Modules Principaux

### Rendu Markdown

Analyse et rend le texte Markdown en sortie terminal stylisée :

- **Titres**: H1 (`#`), H2 (`##`), H3 (`###`)
- **Paragraphes** avec retour à la ligne automatique compatible CJK
- **Formatage en ligne**: `**gras**`, `*italique*`, `***gras+italique***`, `` `code en ligne` ``
- **Blocs de code** avec étiquettes de langage optionnelles (les blocs mermaid sont rendus en diagrammes)
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

### Diagrammes Mermaid

Rendu direct de diagrammes Mermaid dans le terminal :

- **Diagrammes de séquence**, **camemberts**, **diagrammes de Gantt**, **diagrammes d'état**
- Déclenchés par les blocs ` ```mermaid `
- Drapeau de fonctionnalité : `mermaid`

### Coloration Syntaxique

Coloration syntaxique des blocs de code basée sur tree-sitter :

- Drapeaux par langage (`highlight-lang-rust`, `highlight-lang-python`, etc.)
- `highlight-lang-all` active tous les langages
- Personnalisable via `HighlightHooks`

### Widgets MarkdownPreview / MarkdownViewer

Le widget de haut niveau qui intègre tout :

- Rend le contenu Markdown, les vues arborescentes et les éléments d'action dans une seule mise en page défilable
- **Cache**: reconstruit la sortie uniquement lorsque le contenu, la largeur ou la génération du thème change
- **Suppression du préambule TOML**: supprime automatiquement le préambule TOML délimité par `+++`
- **Éléments d'action**: éléments étiquetés sélectionnables au clavier avec IDs d'action
- Délègue toute la navigation à `HybridScrollView`

## Démarrage Rapide

```toml
[dependencies]
ratatui-markdown = "0.2"
```

### Exemples

| Exemple              | Description                          | Fonctionnalités requises       |
|----------------------|--------------------------------------|-------------------------------|
| `basic`              | Rendu Markdown minimal               | —                             |
| `code`               | Blocs de code avec coloration        | `highlight-lang-all`          |
| `custom_code_block`  | Hooks de rendu de blocs personnalisés | —                             |
| `image`              | Intégration et zoom d'images         | `image`                       |
| `mermaid`            | Rendu de diagrammes Mermaid          | `mermaid`                     |
| `tree_list`          | Arbre JSON/TOML rétractable          | —                             |

```bash
cargo run --example basic
cargo run --example code --features highlight-lang-all
cargo run --example image --features image
cargo run --example mermaid --features mermaid
cargo run --example tree_list
```

## Drapeaux de Fonctionnalités

Toutes les fonctionnalités sont activées par défaut. Désactivez les fonctionnalités par défaut pour n'activer que ce dont vous avez besoin :

```toml
[dependencies]
ratatui-markdown = { version = "0.2", default-features = false, features = ["markdown"] }
```

| Fonctionnalité       | Dépend de                           | Description                                     |
|----------------------|--------------------------------------|-------------------------------------------------|
| `markdown`           | —                                    | Analyseur et moteur de rendu Markdown           |
| `image`              | —                                    | Résolution d'images via `ImageResolver`         |
| `scroll`             | —                                    | HybridScrollView, listes défilables, barre      |
| `tree`               | `scroll`, `serde_json`, `toml`       | Arbre JSON/TOML rétractable                     |
| `preview`            | `markdown`, `scroll`, `tree`         | Widget unifié MarkdownPreview                   |
| `mermaid`            | `markdown`                           | Rendu de diagrammes Mermaid                     |
| `viewer`             | `markdown`, `scroll`                 | Widget MarkdownViewer                           |
| `highlight`          | —                                    | Coloration syntaxique via tree-sitter           |
| `highlight-lang-*`   | `highlight`                          | Grammaires individuelles par langage            |
| `highlight-lang-all` | `highlight`                          | Toutes les grammaires bundlées                  |

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
