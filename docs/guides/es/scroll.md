# Sistema de Desplazamiento

> Desplazamiento híbrido inteligente con navegación por elementos enfocables.

## Descripción General

El módulo `scroll` proporciona un sistema de desplazamiento híbrido que soporta dos modos:

1. **Desplazamiento libre** — cuando no hay elementos enfocables a la vista, el contenido se desplaza libremente
2. **Activado** — cuando los elementos enfocables entran en el centro de la vista, el cursor se fija en el primer elemento para navegación por teclado

Controlado por la bandera de funcionalidad `scroll` (habilitada por defecto).

## HybridScrollView

El widget principal que gestiona el desplazamiento, las regiones de foco y el renderizado:

```rust
pub struct HybridScrollView { /* campos */ }

impl HybridScrollView {
    pub fn new() -> Self;
    pub fn with_left_padding(self, padding: bool) -> Self;
    pub fn with_cursor_indicator(self, show: bool) -> Self;

    // Gestión de contenido
    pub fn set_content(&mut self, lines: Vec<Line<'static>>, regions: Vec<FocusableRegion>);
    pub fn set_lines(&mut self, lines: Vec<Line<'static>>);
    pub fn clear(&mut self);
    pub fn is_empty(&self) -> bool;

    // Estado de desplazamiento
    pub fn total_lines(&self) -> usize;
    pub fn get_scroll_offset(&self) -> usize;
    pub fn get_viewport_height(&self) -> usize;
    pub fn set_scroll_offset(&mut self, offset: usize);

    // Navegación
    pub fn scroll_up(&mut self);
    pub fn scroll_down(&mut self);
    pub fn scroll_to_top(&mut self);
    pub fn scroll_to_bottom(&mut self);
    pub fn page_up(&mut self, lines: usize);
    pub fn page_down(&mut self, lines: usize);

    // Activación
    pub fn is_engaged(&self) -> bool;
    pub fn engaged_cursor(&self) -> Option<(usize, usize)>;
    pub fn selected_item_id(&self) -> Option<&str>;
    pub fn engage_first(&mut self);
    pub fn engage_by_id(&mut self, id: &str) -> bool;

    // Renderizado
    pub fn render(&mut self, f: &mut Frame, inner_area: Rect, outer_area: Rect, theme: &impl RichTextTheme);
}
```

### Configuración

- **with_left_padding**: Agrega 1 columna de relleno izquierdo a todas las líneas mostradas
- **with_cursor_indicator**: Muestra `> ` (2 columnas) en la línea del cursor activado (tiene prioridad sobre `left_padding`)

El método `effective_padding()` devuelve el relleno real utilizado:
- `2` si el indicador de cursor está habilitado
- `1` si solo hay relleno izquierdo
- `0` en caso contrario

## Regiones y Elementos Enfocables

```rust
pub struct FocusableItemRange {
    pub start_line: usize,    // inclusivo
    pub end_line: usize,      // exclusivo
    pub id: String,           // identificador único
}

pub struct FocusableRegion {
    pub items: Vec<FocusableItemRange>,
}
```

Las regiones definen tramos de líneas que se vuelven interactivos. Cuando el centro de la vista pasa sobre una región, la vista de desplazamiento se activa automáticamente y el cursor se posiciona en el primer elemento.

### Comportamiento de Activación

- Desplazarse **hacia abajo** dentro de una región activa el **primer** elemento
- Desplazarse **hacia arriba** dentro de una región activa el **último** elemento
- Navegar más allá del último elemento de una región **desactiva** y vuelve al desplazamiento libre
- `scroll_to_top()` y `scroll_to_bottom()` siempre desactivan
- Dentro de una región, `scroll_up`/`scroll_down` mueven el cursor entre elementos

## Otros Widgets de Desplazamiento

### ScrollableList<T>

Una lista desplazable genérica con navegación por ratón/teclado y renderizado con bordes opcional:

```rust
pub trait ListItemRenderer {
    fn render_item(&self, index: usize, is_selected: bool, width: usize) -> Line<'static>;
}
```

### ArrowScrollbar

Una barra de desplazamiento personalizada dibujada con símbolos de flecha Unicode:

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

Para seguimiento automático de contenido (por ejemplo, salida en streaming):

```rust
pub struct FollowScrollState {
    // rastrea si la vista está en la parte inferior
}
```

### ScrollableRenderResult

Un envoltorio de panel desplazable simple:

```rust
pub fn render_scrollable(
    lines: &[Line],
    scroll_offset: usize,
    area: Rect,
    buf: &mut Buffer,
) -> ScrollableRenderResult;
```

## Ejemplo

```rust
use ratatui_markdown::scroll::{HybridScrollView, FocusableItemRange, FocusableRegion};

let mut scroll = HybridScrollView::new()
    .with_cursor_indicator(true);

// Líneas de contenido
let lines: Vec<Line> = (0..100)
    .map(|i| Line::raw(format!("Línea {}", i)))
    .collect();

// Hacer las líneas 30-32 enfocables
let region = FocusableRegion {
    items: vec![
        FocusableItemRange { start_line: 30, end_line: 31, id: "item-a".into() },
        FocusableItemRange { start_line: 31, end_line: 32, id: "item-b".into() },
        FocusableItemRange { start_line: 32, end_line: 33, id: "item-c".into() },
    ],
};

scroll.set_content(lines, vec![region]);

// Desplazamiento libre hacia abajo; se activa automáticamente cuando la línea 30 entra en el centro de la vista
scroll.scroll_down();

// Navegar a través de los elementos
while scroll.scroll_down() == InputResult::Continue {
    if let Some(id) = scroll.selected_item_id() {
        println!("Seleccionado: {}", id);
    }
}
```
