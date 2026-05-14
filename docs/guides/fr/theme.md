# Personnalisation du Thème

> Implémentation de schémas de couleurs personnalisés via le trait `RichTextTheme`.

## Aperçu

Toutes les recherches de couleurs et de styles passent par le trait `RichTextTheme`, ce qui découple la logique de rendu de l'apparence. Implémentez ce trait une fois et passez-le à toutes les fonctions de rendu.

## Trait RichTextTheme

```rust
pub trait RichTextTheme {
    fn generation(&self) -> Generation;

    // Texte général
    fn get_text_color(&self) -> Color;
    fn get_muted_text_color(&self) -> Color;
    fn get_background_color(&self) -> Color;

    // Accentuation / emphase
    fn get_primary_color(&self) -> Color;
    fn get_secondary_color(&self) -> Color;
    fn get_info_color(&self) -> Color;

    // Bordures
    fn get_border_color(&self) -> Color;
    fn get_focused_border_color(&self) -> Color;

    // Popups / superpositions
    fn get_popup_selected_background(&self) -> Color;
    fn get_popup_selected_text_color(&self) -> Color;

    // Valeurs JSON/Arbre
    fn get_json_key_color(&self) -> Color;
    fn get_json_string_color(&self) -> Color;
    fn get_json_number_color(&self) -> Color;
    fn get_json_bool_color(&self) -> Color;
    fn get_json_null_color(&self) -> Color;

    // Divers
    fn get_accent_yellow(&self) -> Color;
}
```

## Jeton Generation

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Generation(pub u64);

impl Generation {
    pub fn next(&self) -> Self { Self(self.0 + 1) }
}
```

Le jeton `Generation` est utilisé par `MarkdownPreview` pour l'invalidation du cache. Lorsque `generation()` du thème retourne une valeur différente de celle en cache, l'aperçu reconstruit toute la sortie rendue.

**Incrémentez la génération chaque fois que l'utilisateur change de thème** à l'exécution :

```rust
struct ThemeApp {
    generation: Generation,
    mode_sombre: bool,
}

impl RichTextTheme for ThemeApp {
    fn generation(&self) -> Generation { self.generation }

    fn get_text_color(&self) -> Color {
        if self.mode_sombre { Color::White } else { Color::Black }
    }

    // ...
}

impl ThemeApp {
    fn basculer_mode_sombre(&mut self) {
        self.mode_sombre = !self.mode_sombre;
        self.generation = self.generation.next();
    }
}
```

## Utilisation des Emplacements de Couleur

Voici comment chaque couleur de thème est utilisée en pratique :

| Méthode                      | Utilisation                                          |
|------------------------------|------------------------------------------------------|
| `get_text_color`             | Texte de paragraphe par défaut, texte d'élément de liste |
| `get_muted_text_color`       | Blocs de code, texte de citation, bordures de tableaux |
| `get_primary_color`          | Titres (H1-H3), accent du texte en gras              |
| `get_secondary_color`        | Code en ligne, accents secondaires                   |
| `get_info_color`             | Surbrillances de niveau info                         |
| `get_background_color`       | Remplissages d'arrière-plan généraux                  |
| `get_border_color`           | Bordures non focalisées (blocs de code, tableaux)    |
| `get_focused_border_color`   | Bordures d'éléments focalisés/engagés, barre de défilement |
| `get_popup_selected_background` | Arrière-plan des éléments de popup sélectionnés   |
| `get_popup_selected_text_color` | Couleur du texte des éléments de popup sélectionnés |
| `get_json_key_color`         | Clés des nœuds d'arbre                               |
| `get_json_string_color`      | Valeurs chaînes dans les arbres                      |
| `get_json_number_color`      | Valeurs numériques dans les arbres                   |
| `get_json_bool_color`        | Valeurs booléennes dans les arbres                   |
| `get_json_null_color`        | Valeurs nulles dans les arbres                       |
| `get_accent_yellow`          | Accents d'avertissement/surbrillance                 |

## Exemple : Thème Complet

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

## Notes de Performance

- Toutes les méthodes de thème sont appelées fréquemment pendant le rendu. Gardez-les légères — retournez des constantes ou de simples consultations de champs.
- La comparaison du jeton `Generation` est une simple vérification d'égalité entière, donc l'incrémenter est rapide.
