use super::common::*;

static SIMPLE_BLOCK: &str = "block-beta\n    A B C\n    D E F";
static SIMPLE_BLOCK_EXPECTED: &str = "                  ╭────╮\n                  │ A  │\n                  ╰────╯\n                     │\n                     │\n                     │\n                  ╭────╮\n                  │ B  │\n                  ╰────╯\n                     │\n                     │\n                     │\n                  ╭────╮\n                  │ C  │\n                  ╰────╯\n                     │\n                     │\n                     │\n                  ╭────╮\n                  │ D  │\n                  ╰────╯\n                     │\n                     │\n                     │\n                  ╭────╮\n                  │ E  │\n                  ╰────╯\n                     │\n                     │\n                     │\n                  ╭────╮\n                  │ F  │\n                  ╰────╯";

#[test]
fn simple_block() {
    let buf = render_to_buffer(SIMPLE_BLOCK, 42, 35);
    assert_buffer_eq(&buf, SIMPLE_BLOCK_EXPECTED);
}
