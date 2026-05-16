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

static THREE_PART: &str = "sequenceDiagram
    Alice->>Bob: Hi
    Bob->>Carol: Hey
    Carol-->>Alice: Yo";

static THREE_PART_EXPECTED: &str = "
       Alice                 Bob                 Carol
          │                    │                    │
          │         Hi         │                    │
          │───────────────────▶│                    │
          │                    │                    │
          │                    │         Hey        │
          │                    │───────────────────▶│
          │                    │                    │
          │                   Y│                    │
          │◀╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌│╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌│
          │                    │                    │";

static SELF_MSG: &str = "sequenceDiagram
    Alice->>Alice: Loop";

static SELF_MSG_EXPECTED: &str = "
       Alice
          │
        Lo│p
         <│◀
          │";

#[test]
fn simple_sequence() {
    let buf = render_to_buffer(SIMPLE_SEQ, 80, 10);
    assert_buffer_eq(&buf, SIMPLE_SEQ_EXPECTED);
}

#[test]
fn three_participants() {
    let buf = render_to_buffer(THREE_PART, 80, 12);
    assert_buffer_eq(&buf, THREE_PART_EXPECTED);
}

#[test]
fn self_message() {
    let buf = render_to_buffer(SELF_MSG, 80, 10);
    assert_buffer_eq(&buf, SELF_MSG_EXPECTED);
}
