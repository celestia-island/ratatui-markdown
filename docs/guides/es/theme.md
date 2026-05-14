# Personalización de Temas

> Implementación de esquemas de color personalizados mediante el trait `RichTextTheme`.

## Descripción General

Todas las búsquedas de color y estilo pasan por el trait `RichTextTheme`, manteniendo la lógica de renderizado desacoplada de la apariencia. Implemente este trait una vez y páselo a todas las funciones de renderizado.

## Trait RichTextTheme

```rust
pub trait RichTextTheme {
    fn generation(&self) -> Generation;

    // Texto general
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_background_color(&self) -> Color;

    // Acento / énfasis
    fn get_primary_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;

    // Bordes
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;

    // Popups / superposiciones
    fn get_popup_selected_background(&self) -> Color;
    fn get_popup_selected_text_color(&self) -> Color;

    // Valores JSON/Árbol
    fn get_json_key_color(&self) -> Color;
    fn get_json_string_color(&self) -> Color;
    fn get_json_number_color(&self) -> Color;
    fn get_json_bool_color(&self) -> Color;
    fn get_json_null_color(&self) -> Color;

    // Misceláneo
    fn get_accent_yellow(&self) -> Color;
}
```

## Token de Generación

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(pub u64);

impl Generation {
    pub fn next(&self) -> Self { Self(self.0 + 1) }
}
```

El token `Generation` es usado por `MarkdownPreview` para la invalidación de caché. Cuando `generation()` del tema devuelve un valor diferente al almacenado en caché, la vista previa reconstruye toda la salida renderizada.

**Incremente la generación cada vez que el usuario cambie de tema** en tiempo de ejecución:

```rust
struct AppTheme {
    generation: Generation,
    dark_mode: bool,
}

impl RichTextTheme for AppTheme {
    fn generation(&self) -> Generation { self.generation }

    fn get_text_color(&self) -> Color {
        if self.dark_mode { Color::White } else { Color::Black }
    }

    // ...
}

impl AppTheme {
    fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
        self.generation = self.generation.next();
    }
}
```

## Uso de los Espacios de Color

A continuación se muestra cómo se usa cada color del tema en la práctica:

| Método | Usado Para |
|--------|------------|
| `get_text_color` | Texto de párrafo predeterminado, texto de elementos de lista |
| `get_muted_text_color` | Bloques de código, texto de citas, bordes de tabla |
| `get_primary_color` | Encabezados (H1-H3), acento de texto en negrita |
| `get_secondary_color` | Código en línea, acentos secundarios |
| `get_info_color` | Resaltados de nivel informativo |
| `get_background_color` | Rellenos de fondo generales |
| `get_border_color` | Bordes sin foco (bloques de código, tablas) |
| `get_focused_border_color` | Bordes de elementos enfocados/activados, barra de desplazamiento |
| `get_popup_selected_background` | Fondo de elementos de popup seleccionados |
| `get_popup_selected_text_color` | Color de texto de elementos de popup seleccionados |
| `get_json_key_color` | Claves de nodos del árbol |
| `get_json_string_color` | Valores de cadena en árboles |
| `get_json_number_color` | Valores numéricos en árboles |
| `get_json_bool_color` | Valores booleanos en árboles |
| `get_json_null_color` | Valores nulos en árboles |
| `get_accent_yellow` | Acentos de advertencia/resaltado |

## Ejemplo: Tema Completo

```rust
use ratatui::style::Color;
use ratatui_markdown::theme::{Generation, RichTextTheme};

struct CatppuccinMocha;

impl RichTextTheme for CatppuccinMocha {
    fn generation(&self) -> Generation { Generation(0) }

    fn get_text_color(&self) -> Color { Color::new(205, 214, 244) }       // text
    fn get_muted_text_color(&self) -> Color { Color::new(147, 153, 178) } // overlay2
    fn get_background_color(&self) -> Color { Color::new(30, 30, 46) }    // base
    fn get_primary_color(&self) -> Color { Color::new(137, 180, 250) }    // blue
    fn get_secondary_color(&self) -> Color { Color::new(203, 166, 247) }  // mauve
    fn get_info_color(&self) -> Color { Color::new(137, 220, 235) }       // sky
    fn get_border_color(&self) -> Color { Color::new(69, 71, 90) }        // surface1
    fn get_focused_border_color(&self) -> Color { Color::new(137, 180, 250) } // blue
    fn get_popup_selected_background(&self) -> Color { Color::new(69, 71, 90) }
    fn get_popup_selected_text_color(&self) -> Color { Color::new(205, 214, 244) }
    fn get_json_key_color(&self) -> Color { Color::new(137, 220, 235) }    // sky
    fn get_json_string_color(&self) -> Color { Color::new(166, 227, 161) } // green
    fn get_json_number_color(&self) -> Color { Color::new(250, 179, 135) } // peach
    fn get_json_bool_color(&self) -> Color { Color::new(203, 166, 247) }   // mauve
    fn get_json_null_color(&self) -> Color { Color::new(108, 112, 134) }   // surface2
    fn get_accent_yellow(&self) -> Color { Color::new(249, 226, 175) }     // yellow
}
```

## Notas de Rendimiento

- Todos los métodos del tema se llaman con frecuencia durante el renderizado. Manténgalos ligeros — devuelva constantes o búsquedas simples de campos.
- La comparación del token `Generation` es una simple verificación de igualdad de enteros, por lo que incrementarlo es rápido.
