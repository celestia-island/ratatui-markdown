use super::common::*;

static SIMPLE_CLASS: &str = "classDiagram\n    class Animal {\n        +name: String\n        +speak()\n    }";

static SIMPLE_CLASS_EXPECTED: &str = "
                             ┌───────────────┐
                             │    Animal     │
                             ├───────────────┤
                             │ + name: String│
                             │               │
                             │ + speak()     │
                             └───────────────┘";

#[test]
fn simple_class() {
    let buf = render_to_buffer(SIMPLE_CLASS, 80, 10);
    assert_buffer_eq(&buf, SIMPLE_CLASS_EXPECTED);
}
