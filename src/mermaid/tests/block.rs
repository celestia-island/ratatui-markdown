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

#[test]
fn simple_block() {
    let buf = render_to_buffer(SIMPLE_BLOCK, 42, 35);
    assert_buffer_eq(&buf, SIMPLE_BLOCK_EXPECTED);
}
