# Módulo Markdown

> Analiza y renderiza texto Markdown en `ratatui::text::Line`s con estilo.

## Descripción General

El módulo `markdown` proporciona un analizador y renderizador de Markdown personalizado diseñado para salida de terminal. **No** es un analizador compatible con CommonMark — se enfoca en el subconjunto de Markdown más útil en interfaces de usuario de terminal.

Controlado por la bandera de funcionalidad `markdown` (habilitada por defecto).

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

### Constructor

`MarkdownRenderer::new(max_width)` toma el ancho de contenido disponible en columnas. Este ancho se usa para el ajuste de texto de párrafos (compatible con CJK) y el dimensionamiento de columnas de tablas.

### Análisis

`parse()` toma un `&str` de texto Markdown y devuelve un `Vec<MarkdownBlock>`. El analizador está orientado a líneas y procesa los bloques en secuencia.

### Renderizado

`render()` toma los bloques analizados y un tema, produciendo un `Vec<Line<'static>>` adecuado para uso directo en widgets de ratatui.

## Enum MarkdownBlock

```rust
pub enum MarkdownBlock {
    Heading1(String),              // # Encabezado
    Heading2(String),              // ## Encabezado
    Heading3(String),              // ### Encabezado
    Paragraph(Vec<String>),        // Líneas de párrafo ajustadas
    CodeBlock(String, String),     // (lenguaje, contenido)
    InlineCode(String),            // `código en línea`
    ListItem(String, u8),          // (contenido, nivel_de_indentación)
    Blockquote(String),            // > texto citado
    HorizontalRule,                // --- o *** o ___
    BlankLine,                     // línea vacía
    Table {                        // | col1 | col2 |
        headers: Vec<String>,      //   |------|------|
        rows: Vec<Vec<String>>,    //   | val1 | val2 |
    },
}
```

### Detalles de Bloques

**Encabezados** (H1-H3): Renderizados con `primary_color`, con H1 usando `Modifier::BOLD`.

**Párrafos**: El texto se ajusta con soporte CJK al `max_width`. Cada línea ajustada se convierte en una entrada en el `Vec<String>`.

**Bloques de Código** (delimitados con ` ``` `): Renderizados con `muted_text_color` dentro de cajas con bordes usando caracteres de dibujo de caja. Los bloques de código Mermaid se omiten silenciosamente.

**Código en Línea**: Renderizado con `secondary_color` y `Modifier::DIM`.

**Listas**: No ordenadas (`-`, `*`, `+`) y ordenadas (`1.`, `2.`). Cada elemento conserva su nivel de indentación. Los sub-elementos se indentan visualmente.

**Citas**: Precedidas por una barra `│` coloreada y renderizadas en `muted_text_color`.

**Tablas**: Las columnas se dimensionan proporcionalmente según el ancho del contenido. Las celdas se ajustan, los encabezados usan `Modifier::BOLD` y los bordes usan caracteres de dibujo de caja.

## Formato en Línea

El formato en línea se aplica **dentro** del texto de párrafos y elementos de lista:

| Markdown        | Efecto Renderizado                |
|-----------------|-----------------------------------|
| `**texto**`      | **Negrita** (`Modifier::BOLD`)   |
| `*texto*`        | *Cursiva* (`Modifier::ITALIC`)   |
| `***texto***`    | ***Negrita+Cursiva***             |
| `` `código` ``    | Estilo `Código en Línea`         |

```rust
pub fn parse_inline_formatting(text: &str, theme: &impl RichTextTheme) -> Vec<Span<'static>>;
```

Esta función independiente también se reexporta para uso fuera del `MarkdownRenderer`.

## Ejemplo

```rust
use ratatui_markdown::markdown::MarkdownRenderer;

let md = r#"
# Título

Esto es un párrafo con texto en **negrita** y *cursiva*.

## Código

```rust
fn main() {
    println!("¡Hola!");
}
```

| Nombre | Versión |
|--------|---------|
| ratatui | 0.29 |
| serde | 1.0 |
"#;

let renderer = MarkdownRenderer::new(80);
let blocks = renderer.parse(md);
let lines = renderer.render(&blocks, theme);
// usar lines en un widget de ratatui
```
