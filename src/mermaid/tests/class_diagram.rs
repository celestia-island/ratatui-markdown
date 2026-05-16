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

static TWO_CLASSES: &str = "classDiagram
    class Dog {
        +bark()
    }
    class Cat {
        +meow(): String
    }
    Dog --> Cat";

static TWO_CLASSES_EXPECTED: &str = "
                              ┌─────────────┐
                              │     Dog     │
                              ├─────────────┤
                              │ + bark()    │
                              └─────────────┘
                                     uses
                                       │
                                       ▼
                            ┌─────────────────┐
                            │       Cat       │
                            ├─────────────────┤
                            │ + meow(): String│
                            └─────────────────┘";

static EMPTY_CLASS: &str = "classDiagram
    class Empty {
    }";

static EMPTY_CLASS_EXPECTED: &str = "
                                 ┌───────┐
                                 │ Empty │
                                 ├───────┤
                                 └───────┘";

#[test]
fn simple_class() {
    let buf = render_to_buffer(SIMPLE_CLASS, 80, 10);
    assert_buffer_eq(&buf, SIMPLE_CLASS_EXPECTED);
}

#[test]
fn two_classes() {
    let buf = render_to_buffer(TWO_CLASSES, 80, 20);
    assert_buffer_eq(&buf, TWO_CLASSES_EXPECTED);
}

#[test]
fn empty_class() {
    let buf = render_to_buffer(EMPTY_CLASS, 80, 6);
    assert_buffer_eq(&buf, EMPTY_CLASS_EXPECTED);
}
