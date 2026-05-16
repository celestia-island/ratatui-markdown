use super::common::*;

static SIMPLE_SEQ: &str = "sequenceDiagram\n    Alice->>Bob: Hello\n    Bob-->>Alice: Hi";
static SIMPLE_SEQ_EXPECTED: &str = "       Alice                 Bob\n          │                    │\n          │        Hello       │\n          │───────────────────▶│\n          │                    │\n          │         Hi         │\n          │◀╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌│\n          │                    │";

#[test]
fn simple_sequence() {
    let buf = render_to_buffer(SIMPLE_SEQ, 80, 10);
    assert_buffer_eq(&buf, SIMPLE_SEQ_EXPECTED);
}
