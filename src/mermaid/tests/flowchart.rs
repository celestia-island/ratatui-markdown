use super::common::*;

static SIMPLE_TD: &str = "graph TD\n    A[Start] --> B[End]";
static SIMPLE_TD_EXPECTED: &str = "                                   ┌───────┐\n                                   │ Start │\n                                   └───────┘\n                                       │\n                                       │\n                                       ▼\n                                    ┌─────┐\n                                    │ End │\n                                    └─────┘";

static FORK_TD: &str = "graph TD\n    A[Start] --> B[Left]\n    A --> C[Right]";
static FORK_TD_EXPECTED: &str = "                                   ┌───────┐\n                                   │ Start │\n                                   └───────┘\n                                       │\n                                 ┌─────┬─────┐\n                                 ▼           ▼\n                             ┌──────┐    ┌───────┐\n                             │ Left │    │ Right │\n                             └──────┘    └───────┘";

static LR: &str = "graph LR\n    A --> B";
static LR_EXPECTED: &str = "┌────┐    ┌────┐\n│    │    │    │\n│ A  │───►│ B  │\n│    │    │    │\n└────┘    └────┘";

#[test]
fn simple_td() {
    let buf = render_to_buffer(SIMPLE_TD, 80, 10);
    assert_buffer_eq(&buf, SIMPLE_TD_EXPECTED);
}

#[test]
fn fork_td() {
    let buf = render_to_buffer(FORK_TD, 80, 10);
    assert_buffer_eq(&buf, FORK_TD_EXPECTED);
}

#[test]
fn lr() {
    let buf = render_to_buffer(LR, 20, 5);
    assert_buffer_eq(&buf, LR_EXPECTED);
}
