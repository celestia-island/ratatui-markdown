use super::common::*;

static SIMPLE_STATE: &str = "stateDiagram-v2\n    [*] --> Idle\n    Idle --> Running\n    Running --> Idle";

static SIMPLE_STATE_EXPECTED: &str = "
                             ╭────▲────╮    ╭────╮
                             │ Running │    │ ●  │
                             ╰─────────╯    ╰────╯
                                  │            │
                                  ├─────┬──────┘
                                  │     ▼
                                  │ ╭──────╮
                                  │ │ Idle │
                                  │ ╰──────╯";

#[test]
fn simple_state() {
    let buf = render_to_buffer(SIMPLE_STATE, 80, 10);
    assert_buffer_eq(&buf, SIMPLE_STATE_EXPECTED);
}
