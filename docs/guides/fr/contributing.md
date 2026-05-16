# Contribuer

## Configuration de Développement

```bash
git clone https://github.com/celestia-island/ratatui-markdown.git
cd ratatui-markdown
cargo build
```

## Vérifications CI

Avant de soumettre une PR, assurez-vous que les vérifications suivantes passent :

```bash
cargo test --all-features
cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
```

## Conventions du Projet

### Style de Code

- Suivez les idiomes Rust standard (`cargo fmt` et `cargo clippy` les appliquent)
- Pas de commentaires dans le code sauf si strictement nécessaire
- Utilisez `pub(crate)` pour la visibilité interne ; `pub` uniquement pour la surface d'API publique

### Organisation des Modules

Chaque module de fonctionnalité se trouve sous `src/{module}/` :

```
src/markdown/       # Fonctionnalité : markdown
  ├── mod.rs        # Réexporte les types publics, définit MarkdownRenderer
  ├── parser.rs     # Analyseur niveau bloc (impl MarkdownRenderer)
  ├── types.rs      # Enums MarkdownBlock, TextToken
  ├── render.rs     # Moteur de rendu niveau bloc
  ├── inline.rs     # Analyseur de formatage en ligne
  ├── text.rs       # Utilitaires de retour à la ligne
  ├── tests.rs      # Tests d'analyseur/intégration
  └── render_tests.rs  # Tests de snapshot de sortie de rendu
```

Les tests cohabitent avec les fichiers source dans des blocs `#[cfg(test)] mod tests { }`, les suites de tests plus volumineuses se trouvant dans des fichiers dédiés `tests.rs` / `render_tests.rs`.

### Drapeaux de Fonctionnalités

Toutes les fonctionnalités sont activées par défaut. Utilisez `cfg(feature = "X")` pour garder le code :

```rust
#[cfg(feature = "markdown")]
pub mod markdown;
```

Les dépendances de fonctionnalités sont exprimées dans `Cargo.toml` :

```toml
tree = ["dep:serde_json", "dep:toml", "scroll"]
preview = ["markdown", "scroll", "tree"]
```

### Tests

Lancer tous les tests :

```bash
cargo test --all-features
```

Tester chaque combinaison de fonctionnalités :

```bash
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
cargo test --no-default-features --features preview  # implique markdown, scroll, tree
```

### Documentation

- La documentation API suit les conventions de commentaires doc de Rust
- La documentation utilisateur se trouve sous `docs/guides/`
- `docs/guides/en/` est la documentation canonique (anglaise)
- Les traductions sont bienvenues pour d'autres langues sous `docs/guides/{lang}/`

## Style des Messages de Commit

Suivez le format de commit conventionnel :

```
type: description courte

type: feat, fix, refactor, test, docs, chore, ci, style
```

## Processus de Publication

La publication sur crates.io est gérée par le workflow GitHub Actions `publish.yml` lors de l'envoi d'un tag.

```bash
# Incrémenter la version dans Cargo.toml, puis :
git tag v0.1.1
git push origin v0.1.1
```
