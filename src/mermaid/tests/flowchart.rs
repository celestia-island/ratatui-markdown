use super::common::*;

static SIMPLE_TD: &str = "graph TD
    A[Start] --> B[End]";

static SIMPLE_TD_EXPECTED: &str = "
                                   ┌───────┐
                                   │ Start │
                                   └───────┘
                                       │
                                       │
                                       ▼
                                    ┌─────┐
                                    │ End │
                                    └─────┘";

static FORK_TD: &str = "graph TD
    A[Start] --> B[Left]
    A --> C[Right]";

static FORK_TD_EXPECTED: &str = "
                                   ┌───────┐
                                   │ Start │
                                   └───────┘
                                       │
                                 ┌─────┴─────┐
                                 ▼           ▼
                             ┌──────┐    ┌───────┐
                             │ Left │    │ Right │
                             └──────┘    └───────┘";

static LR: &str = "graph LR
    A --> B";

static LR_EXPECTED: &str = "
┌────┐    ┌────┐
│    │    │    │
│ A  │───►│ B  │
│    │    │    │
└────┘    └────┘";

static CHAIN: &str = "graph TD
    A --> B --> C";

static CHAIN_EXPECTED: &str = "
                                     ┌────┐
                                     │ A  │
                                     └────┘
                                        │
                                        │
                                        ▼
                                     ┌────┐
                                     │ B  │
                                     └────┘
                                        │
                                        │
                                        ▼
                                     ┌────┐
                                     │ C  │
                                     └────┘";

static DIAMOND: &str = "graph TD
    A{Start} -->|yes| B[Yes]
    A -->|no| C[No]";

static DIAMOND_EXPECTED: &str = "
                                   ╭───────╮
                                   │ Start │
                                   ╰───────╯
                                 yes   │    no
                                  ┌────┴─────┐
                                  ▼          ▼
                               ┌─────┐    ┌────┐
                               │ Yes │    │ No │
                               └─────┘    └────┘";

static LABELED_LR: &str = "graph LR
    A -->|hello| B -->|world| C";

static LABELED_LR_EXPECTED: &str = "
┌────┐    ┌────┐    ┌────┐
│    │ello│    │orld│    │
│ A  │───►│ B  │───►│ C  │
│    │    │    │    │    │
└────┘    └────┘    └────┘";

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

#[test]
fn chain() {
    let buf = render_to_buffer(CHAIN, 80, 15);
    assert_buffer_eq(&buf, CHAIN_EXPECTED);
}

#[test]
fn diamond() {
    let buf = render_to_buffer(DIAMOND, 80, 12);
    assert_buffer_eq(&buf, DIAMOND_EXPECTED);
}

#[test]
fn labeled_lr() {
    let buf = render_to_buffer(LABELED_LR, 40, 5);
    assert_buffer_eq(&buf, LABELED_LR_EXPECTED);
}
