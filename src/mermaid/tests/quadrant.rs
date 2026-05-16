use super::common::*;

static SIMPLE_QUADRANT: &str = "quadrantChart\n    x-axis Low --> High\n    y-axis Low --> High\n    A: [0.3, 0.6]\n    B: [0.45, 0.23]";
static SIMPLE_QUADRANT_EXPECTED: &str = "               │                         │\n               │                         │\n               │                         │\n               │                         │\n               │                         │\n               │                         │\n               │                         │\n               │                         │\n               │              ●          │\n               │                         │\n               │─────────────────────────┼────────────────────────\n               │                         │\n               │                         │\n               │                         │\n               │                         │\n               │                      ●  │\n               │                         │\n               │                         │\n               │                         │\n               │                         │\n               │─────────────────────────┴────────────────────────\n               Low -- ▲ -- High\n\n ● A (0.30, 0.60)\n ● B (0.45, 0.23)";

#[test]
fn simple_quadrant() {
    let buf = render_to_buffer(SIMPLE_QUADRANT, 80, 26);
    assert_buffer_eq(&buf, SIMPLE_QUADRANT_EXPECTED);
}
