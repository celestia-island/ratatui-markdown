# Contribuir

## Configuración de Desarrollo

```bash
git clone https://github.com/celestia-island/ratatui-markdown.git
cd ratatui-markdown
cargo build
```

## Verificaciones de CI

Antes de enviar un PR, asegúrese de que pasen las siguientes verificaciones:

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

## Convenciones del Proyecto

### Estilo de Código

- Siga los modismos estándar de Rust (`cargo fmt` y `cargo clippy` lo aplican)
- Sin comentarios en el código a menos que sean estrictamente necesarios
- Use `pub(crate)` para visibilidad interna; `pub` solo para la superficie de la API pública

### Organización de Módulos

Cada módulo de funcionalidad reside en `src/{modulo}/`:

```
src/markdown/       # Funcionalidad: markdown
  ├── mod.rs        # Reexporta tipos públicos, define MarkdownRenderer
  ├── parser.rs     # Analizador a nivel de bloque (impl MarkdownRenderer)
  ├── types.rs      # Enums MarkdownBlock, TextToken
  ├── render.rs     # Renderizador a nivel de bloque
  ├── inline.rs     # Analizador de formato en línea
  ├── text.rs       # Utilidades de ajuste de texto
  ├── tests.rs      # Pruebas de analizador/integración
  └── render_tests.rs  # Pruebas de instantáneas de salida de renderizado
```

Las pruebas residen junto a los archivos fuente dentro de bloques `#[cfg(test)] mod tests { }`, con suites de pruebas más grandes en archivos dedicados `tests.rs` / `render_tests.rs`.

### Banderas de Funcionalidades

Todas las funcionalidades están habilitadas por defecto. Use `cfg(feature = "X")` para controlar el código:

```rust
#[cfg(feature = "markdown")]
pub mod markdown;
```

Las dependencias de funcionalidades se expresan en `Cargo.toml`:

```toml
tree = ["dep:serde_json", "dep:toml", "scroll"]
preview = ["markdown", "scroll", "tree"]
```

### Pruebas

Ejecute todas las pruebas:

```bash
cargo test --all-features
```

Pruebe cada combinación de funcionalidades:

```bash
cargo test --no-default-features
cargo test --no-default-features --features markdown
cargo test --no-default-features --features scroll
cargo test --no-default-features --features tree
cargo test --no-default-features --features mermaid
cargo test --no-default-features --features image
cargo test --no-default-features --features viewer
cargo test --no-default-features --features preview  # implica markdown, scroll, tree
```

### Documentación

- La documentación de la API sigue las convenciones de doc-comments de Rust
- La documentación orientada al usuario reside en `docs/guides/`
- `docs/guides/en/` es la documentación canónica (inglés)
- Las traducciones son bienvenidas para otros idiomas en `docs/guides/{idioma}/`

## Estilo de Mensajes de Commit

Siga el formato de commit convencional:

```
tipo: descripción corta

tipo: feat, fix, refactor, test, docs, chore, ci, style
```

## Proceso de Publicación

La publicación en crates.io se maneja mediante el flujo de trabajo `publish.yml` de GitHub Actions al enviar una etiqueta.

```bash
# Incrementar la versión en Cargo.toml, luego:
git tag v0.1.1
git push origin v0.1.1
```
