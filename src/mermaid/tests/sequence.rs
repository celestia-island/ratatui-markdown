use super::common::*;

static SIMPLE_SEQ: &str = "sequenceDiagram
    Alice->>Bob: Hello
    Bob-->>Alice: Hi";

static SIMPLE_SEQ_EXPECTED: &str = "
       Alice                 Bob
          │                    │
          │        Hello       │
          │───────────────────▶│
          │                    │
          │         Hi         │
          │◀╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌│
          │                    │";

#[test]
fn simple_sequence() {
    let buf = render_to_buffer(SIMPLE_SEQ, 80, 10);
    assert_buffer_eq(&buf, SIMPLE_SEQ_EXPECTED);
}
