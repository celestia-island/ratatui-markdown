use super::common::*;

static SIMPLE_BLOCK: &str = "block-beta
    A B C
    D E F";

static SIMPLE_BLOCK_EXPECTED: &str = "
                  ╭────╮
                  │ A  │
                  ╰────╯
                     │
                     │
                     │
                  ╭────╮
                  │ B  │
                  ╰────╯
                     │
                     │
                     │
                  ╭────╮
                  │ C  │
                  ╰────╯
                     │
                     │
                     │
                  ╭────╮
                  │ D  │
                  ╰────╯
                     │
                     │
                     │
                  ╭────╮
                  │ E  │
                  ╰────╯
                     │
                     │
                     │
                  ╭────╮
                  │ F  │
                  ╰────╯";

static GRID_2X2: &str = "block-beta
    columns 2
    A B
    C D";

static GRID_2X2_EXPECTED: &str = "
            ╭────╮
            │ A  │
            ╰────╯
               │
          ┌────┴────┐
          │         │
       ╭────╮    ╭────╮
       │ B  │    │ C  │
       ╰────╯    ╰────╯
          │         │
          └────┬────┘
               │
            ╭────╮
            │ D  │
            ╰────╯";

static SINGLE: &str = "block-beta
    X";

static SINGLE_EXPECTED: &str = "
       ╭────╮
       │ X  │
       ╰────╯";

#[test]
fn simple_block() {
    let buf = render_to_buffer(SIMPLE_BLOCK, 42, 35);
    assert_buffer_eq(&buf, SIMPLE_BLOCK_EXPECTED);
}

#[test]
fn grid_2x2() {
    let buf = render_to_buffer(GRID_2X2, 30, 20);
    assert_buffer_eq(&buf, GRID_2X2_EXPECTED);
}

#[test]
fn single() {
    let buf = render_to_buffer(SINGLE, 20, 5);
    assert_buffer_eq(&buf, SINGLE_EXPECTED);
}
