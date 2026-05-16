use super::common::*;

static SIMPLE_QUADRANT: &str = "quadrantChart
    x-axis Low --> High
    y-axis Low --> High
    A: [0.3, 0.6]
    B: [0.45, 0.23]";

static SIMPLE_QUADRANT_EXPECTED: &str = "
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │              ●          │
               │                         │
               │─────────────────────────┼────────────────────────
               │                         │
               │                         │
               │                         │
               │                         │
               │                      ●  │
               │                         │
               │                         │
               │                         │
               │                         │
               │─────────────────────────┴────────────────────────
               Low -- ▲ -- High

 ● A (0.30, 0.60)
 ● B (0.45, 0.23)";

static THREE_POINTS: &str = "quadrantChart
    x-axis Low --> High
    y-axis Low --> High
    A: [0.1, 0.9]
    B: [0.8, 0.8]
    C: [0.5, 0.2]";

static THREE_POINTS_EXPECTED: &str = "
               │                         │
               │    ●                    │
               │                         │
               │                         │             ●
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │─────────────────────────┼────────────────────────
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                        ●│
               │                         │
               │                         │
               │                         │
               │─────────────────────────┴────────────────────────
               Low -- ▲ -- High

 ● A (0.10, 0.90)
 ● B (0.80, 0.80)
 ● C (0.50, 0.20)";

static ONE_POINT: &str = "quadrantChart
    x-axis L --> R
    y-axis B --> T
    X: [0.5, 0.5]";

static ONE_POINT_EXPECTED: &str = "
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │────────────────────────●┼────────────────────────
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │                         │
               │─────────────────────────┴────────────────────────
               L -- ▲ -- R

 ● X (0.50, 0.50)";

#[test]
fn simple_quadrant() {
    let buf = render_to_buffer(SIMPLE_QUADRANT, 80, 26);
    assert_buffer_eq(&buf, SIMPLE_QUADRANT_EXPECTED);
}

#[test]
fn three_points() {
    let buf = render_to_buffer(THREE_POINTS, 80, 26);
    assert_buffer_eq(&buf, THREE_POINTS_EXPECTED);
}

#[test]
fn one_point() {
    let buf = render_to_buffer(ONE_POINT, 80, 26);
    assert_buffer_eq(&buf, ONE_POINT_EXPECTED);
}
