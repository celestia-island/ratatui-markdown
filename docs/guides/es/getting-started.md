# Primeros Pasos

## Requisitos Previos

- **Rust** 1.74 o superior
- **ratatui** 0.29 (se obtiene automáticamente como dependencia)

## Instalación

Agregue a su `Cargo.toml`:

```toml
[dependencies]
ratatui-markdown = "0.1"
```

Esto habilita todas las funcionalidades por defecto (`markdown`, `scroll`, `tree`, `preview`, `mermaid`, `image`, `viewer`).

### Funcionalidades Selectivas

Para reducir el tiempo de compilación y las dependencias, habilite solo lo que necesite:

```toml
# Solo renderizado de Markdown
ratatui-markdown = { version = "0.1", default-features = false, features = ["markdown"] }

# Solo sistema de desplazamiento
ratatui-markdown = { version = "0.1", default-features = false, features = ["scroll"] }

# Vista de árbol (incluye scroll, serde_json y toml)
ratatui-markdown = { version = "0.1", default-features = false, features = ["tree"] }
```

## Uso Básico

### Renderizar Markdown

```rust
use ratatui_markdown::markdown::MarkdownRenderer;
use ratatui_markdown::theme::RichTextTheme;

// Crear un renderizador con el ancho máximo de contenido
let renderer = MarkdownRenderer::new(80);

// Analizar texto Markdown en bloques
let blocks = renderer.parse("# Hola\n\nEsto es texto en **negrita** y *cursiva*.");

// Renderizar bloques en ratatui::text::Line<'static>
let lines = renderer.render(&blocks, &my_theme);
```

### Navegar un Árbol

```rust
use ratatui_markdown::tree::CollapsibleTree;

// Analizar JSON en un árbol colapsable
let json_str = r#"{"name": "proyecto", "deps": {"ratatui": "0.29", "serde": "1.0"}}"#;
let mut tree = CollapsibleTree::from_json_str(json_str).unwrap();

// Renderizar líneas del árbol
let lines = tree.render_lines(80, &my_theme);

// Obtener elementos enfocables para navegación
let items = tree.build_focusable_items();

// Alternar un nodo
tree.toggle("deps/serde");
```

### Usar el Widget MarkdownPreview

El widget `MarkdownPreview` combina todo en una sola vista desplazable:

```rust
use ratatui_markdown::preview::MarkdownPreview;
use ratatui_markdown::theme::RichTextTheme;

let mut preview = MarkdownPreview::new()
    .with_left_padding(true);

// Establecer contenido Markdown
preview.set_content("# Bienvenido\n\n- Elemento uno\n- Elemento dos\n\n```rust\nlet x = 42;\n```");

// Establecer un árbol colapsable (opcional)
let tree = CollapsibleTree::from_json_str(r#"{"config": {"port": 8080}}"#).unwrap();
preview.set_tree(Some(tree));

// Manejar entrada de teclado
preview.scroll_up();
preview.scroll_down();
preview.page_up(10);
preview.page_down(10);
preview.toggle_tree_node(); // Tecla Enter

// Renderizar en su bucle de dibujo de ratatui
fn draw(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}
```

## Implementar un Tema

La biblioteca usa un trait para buscar todos los colores:

```rust
use ratatui::style::Color;
use ratatui_markdown::theme::{Generation, RichTextTheme};

struct MiTema;

impl RichTextTheme for MiTema {
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

Cambie el valor de retorno de `generation()` para invalidar la caché del widget de vista previa y forzar un re-renderizado (por ejemplo, cuando el usuario cambia de tema en tiempo de ejecución).

## Próximos Pasos

- [Módulo Markdown](markdown.md) — API completa de análisis y renderizado de Markdown
- [Sistema de Desplazamiento](scroll.md) — comprender la arquitectura de desplazamiento híbrido
- [Vista de Árbol](tree.md) — renderizado e interacción con árboles JSON/TOML
- [Widget de Vista Previa](preview.md) — el widget unificado de alto nivel
- [Tema](theme.md) — guía completa de personalización de temas
