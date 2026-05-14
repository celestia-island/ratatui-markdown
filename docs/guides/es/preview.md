# Widget MarkdownPreview

> El widget unificado de alto nivel que combina Markdown, árboles y acciones en una sola vista desplazable.

## Descripción General

`MarkdownPreview` es el widget de integración de nivel superior. Combina el renderizado de Markdown, la visualización de árboles colapsables y elementos de acción en un solo `HybridScrollView`. Este es el punto de entrada recomendado para la mayoría de las aplicaciones.

Controlado por la bandera de funcionalidad `preview` (habilitada por defecto, requiere todas las demás funcionalidades).

## API

```rust
pub struct ActionItem {
    pub id: String,
    pub label: String,
}

impl MarkdownPreview {
    pub fn new() -> Self;

    // Configuración
    pub fn with_strip_frontmatter(self, strip: bool) -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;

    // Contenido
    pub fn set_content(&mut self, content: &str);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // Árbol
    pub fn set_tree(&mut self, tree: Option<CollapsibleTree>);
    pub fn tree_mut(&mut self) -> Option<&mut CollapsibleTree>;
    pub fn has_tree(&self) -> bool;
    pub fn toggle_tree_node(&mut self) -> bool;

    // Elementos de acción
    pub fn set_action_items(&mut self, items: Vec<ActionItem>);
    pub fn selected_action_id(&self) -> Option<&str>;

    // Navegación
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // Estado
    pub fn total_lines(&self) -> usize;
    pub fn scroll_offset(&self) -> usize;
    pub fn visible_height(&self) -> usize;
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;

    // Renderizado
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

## Configuración

### with_strip_frontmatter

Cuando está habilitado (por defecto), el contenido delimitado por `+++` se trata como preámbulo TOML y se elimina antes del renderizado:

```
+++
title = "Mi Documento"
author = "Fulano de Tal"
+++

# Contenido Real
Este es el cuerpo.
```

Solo `# Contenido Real` y el párrafo siguiente aparecerían en la salida.

### with_left_padding

Agrega 1 columna de relleno izquierdo al contenido renderizado (se pasa a `HybridScrollView`).

## Diseño del Contenido

El widget renderiza el contenido en orden vertical:

1. **Árbol** (si está presente) — líneas del árbol con nodos enfocables
2. **Markdown** — contenido analizado y renderizado
3. **Elementos de acción** — etiquetas de acción enfocables con envoltorios de corchetes `[etiqueta]`

Se inserta una línea en blanco entre secciones, y cada sección obtiene su propia `FocusableRegion` en la vista de desplazamiento.

## Caché

`MarkdownPreview` almacena en caché la salida renderizada y solo reconstruye cuando:

- El contenido cambia (`set_content` con texto diferente)
- El ancho cambia (redimensionamiento del terminal)
- La generación del tema cambia (`theme.generation()` devuelve un nuevo valor)
- El árbol se modifica (`set_tree`, `toggle_tree_node`)
- Los elementos de acción se modifican (`set_action_items`)

Use `theme.generation()` para activar un re-renderizado después de cambios de tema.

## Manejo de Preámbulo TOML

Se asume que el preámbulo es TOML. **No se analiza** — simplemente se elimina de la salida renderizada. La primera línea `+++` inicia el modo preámbulo, la segunda `+++` lo finaliza. Las líneas antes y después del bloque de preámbulo se renderizan normalmente.

Si el contenido no comienza con `+++`, no se realiza ninguna eliminación.

## Elementos de Acción

Los elementos de acción proporcionan opciones seleccionables por teclado en la parte inferior de la vista:

```rust
preview.set_action_items(vec![
    ActionItem { id: "confirmar".into(), label: " Confirmar ".into() },
    ActionItem { id: "cancelar".into(), label: " Cancelar ".into() },
]);

// En su manejador de entrada:
if let Some("confirmar") = preview.selected_action_id() {
    // manejar confirmación
}
```

Los IDs de los elementos de acción tienen el prefijo `action:` internamente para evitar colisiones con las rutas de nodos del árbol.

## Ejemplo

```rust
use ratatui_markdown::preview::{MarkdownPreview, ActionItem};
use ratatui_markdown::tree::CollapsibleTree;

let mut preview = MarkdownPreview::new()
    .with_strip_frontmatter(true)
    .with_left_padding(true);

// Establecer contenido Markdown (con preámbulo TOML)
preview.set_content(concat!(
    "+++\n",
    "title = \"Mi Doc\"\n",
    "+++\n",
    "\n",
    "# Hola\n\nContenido aquí.\n",
));

// Establecer un árbol
let tree = CollapsibleTree::from_json_str(r#"{"config": {"theme": "dark"}}"#).unwrap();
preview.set_tree(Some(tree));

// Establecer elementos de acción
preview.set_action_items(vec![
    ActionItem { id: "editar".into(), label: " Editar ".into() },
    ActionItem { id: "guardar".into(), label: " Guardar ".into() },
]);

// En la función de renderizado de su aplicación ratatui:
fn render_ui(f: &mut ratatui::Frame, preview: &mut MarkdownPreview, theme: &impl RichTextTheme) {
    preview.render(f, f.area(), f.area(), theme);
}

// Manejar entrada
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
