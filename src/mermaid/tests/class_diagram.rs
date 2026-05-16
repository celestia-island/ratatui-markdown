use super::common::*;

static SIMPLE_CLASS: &str = "classDiagram
    class Animal {
        +name: String
        +speak()
    }";

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
