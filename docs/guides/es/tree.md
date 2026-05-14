# Vista de Árbol

> Árbol colapsable interactivo para datos JSON y TOML.

## Descripción General

El módulo `tree` analiza JSON o TOML en un árbol colapsable interactivo. Los usuarios pueden expandir/colapsar nodos y navegar con el teclado en una interfaz de terminal.

Controlado por la bandera de funcionalidad `tree` (requiere las dependencias `scroll`, `serde_json` y `toml`).

## CollapsibleTree

```rust
pub struct CollapsibleTree { /* campos */ }

impl CollapsibleTree {
    // Constructores
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_toml_str(toml: &str) -> Result<Self, Box<dyn Error>>;
    pub fn from_value(value: serde_json::Value, style: KeyStyle) -> Self;

    // Manipulación del árbol
    pub fn toggle(&mut self, path: &str) -> bool;
    pub fn handle_toggle(&mut self, id: &str) -> bool;
    pub fn expand_all(&mut self);
    pub fn collapse_all(&mut self);
    pub fn expand_to_depth(&mut self, depth: usize);

    // Renderizado
    pub fn render_lines(&self, width: usize, theme: &impl RichTextTheme) -> Vec<Line<'static>>;
    pub fn flatten(&self) -> Vec<FlatEntry>;
    pub fn build_focusable_items(&self) -> Vec<FocusableItemRange>;
}
```

### Constructores

- **`from_json_str`**: Analiza una cadena JSON en un árbol. Las claves usan `KeyStyle::Json` (entre comillas, con separador `:`).
- **`from_toml_str`**: Analiza una cadena TOML (convierte internamente a JSON). Las claves usan `KeyStyle::Toml` (sin comillas, con separador `=`).
- **`from_value`**: Construye un árbol a partir de un `serde_json::Value` existente con un estilo de clave elegido.

### Manipulación del Árbol

```rust
// Alternar usando una ruta separada por barras
tree.toggle("dependencies");          // alternar clave raíz
tree.toggle("dependencies/serde");    // alternar clave anidada

// Métodos de conveniencia
tree.expand_all();
tree.collapse_all();
tree.expand_to_depth(2);

// En un contexto de desplazamiento — usa el ID del elemento seleccionado
if scroll.selected_item_id().is_some() {
    tree.handle_toggle(selected_id);
}
```

### Renderizado

- **`render_lines`**: Produce `Line`s con estilo, conectores de árbol y valores coloreados.
- **`flatten`**: Devuelve una lista plana de todas las entradas visibles (respeta el estado de colapso).
- **`build_focusable_items`**: Devuelve rangos enfocables para integración con `HybridScrollView`, con IDs que coinciden con las rutas del árbol.

## Tipos de Datos

```rust
pub struct FlatEntry {
    pub depth: usize,
    pub key: String,
    pub value: String,
    pub kind: EntryKind,
    pub value_type: ValueType,
    pub path: String,          // ej. "dependencies/serde"
    pub is_last: bool,
}

pub enum EntryKind {
    Collapsed,  // tiene hijos, actualmente colapsado: [+]
    Expanded,   // tiene hijos, actualmente expandido: [-]
    Leaf,       // sin hijos
}

pub enum ValueType {
    String, Number, Boolean, Null, Object, Array,
}

pub enum KeyStyle {
    Json,  // "clave": valor
    Toml,  // clave = valor
}
```

### Colores por Tipo de Valor

Cada `ValueType` se asigna a un método de color del tema correspondiente:

| ValueType | Método del Tema          |
|-----------|--------------------------|
| String    | `get_json_string_color()` |
| Number    | `get_json_number_color()` |
| Boolean   | `get_json_bool_color()`   |
| Null      | `get_json_null_color()`   |

## Ayudantes de Líneas de Árbol

El módulo `tree_lines` (reexportado desde `crate::tree`) proporciona construcción de líneas de bajo nivel:

```rust
pub fn make_body_line(key: &str, value: &str, style: &Style, value_style: &Style) -> Line<'static>;
pub fn make_status_line(status: &str, style: &Style) -> Line<'static>;
pub fn make_branch_dispatch_line(kind: EntryKind, ...) -> Line<'static>;
```

## Ejemplo

```rust
use ratatui_markdown::tree::CollapsibleTree;

let toml_content = r#"
[package]
name = "my-app"
version = "0.1.0"

[dependencies]
ratatui = "0.29"
serde = { version = "1.0", features = ["derive"] }

[features]
default = ["std"]
std = []
"#;

let mut tree = CollapsibleTree::from_toml_str(toml_content).unwrap();

// Expandir todo
tree.expand_all();

// Renderizar a líneas
let lines = tree.render_lines(80, theme);

// Colapsar el subárbol de dependencias
tree.toggle("dependencies");

// Re-renderizar — las dependencias ahora están colapsadas
let lines = tree.render_lines(80, theme);

// Obtener elementos enfocables para navegación por desplazamiento
let items = tree.build_focusable_items();
```
