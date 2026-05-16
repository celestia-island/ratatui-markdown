use super::common::*;

static SIMPLE_PIE: &str = "pie title Pets
    \"Dogs\" : 386
    \"Cats\" : 85";

static SIMPLE_PIE_EXPECTED: &str = "
                                    Pets

 Dogs           ██████████████████████████████ 82%
 Cats           ███████░░░░░░░░░░░░░░░░░░░░░░░ 18%";

static THREE_SLICES: &str = "pie title Fruits
    \"Apple\" : 40
    \"Banana\" : 30
    \"Cherry\" : 20";

static THREE_SLICES_EXPECTED: &str = "
                                   Fruits

 Apple          ██████████████████████████████ 44%
 Banana         ███████████████████████░░░░░░░ 33%
 Cherry         ███████████████░░░░░░░░░░░░░░░ 22%";

static NO_TITLE: &str = "pie
    \"X\" : 10
    \"Y\" : 5";

static NO_TITLE_EXPECTED: &str = "
 X              ██████████████████████████████ 67%
 Y              ███████████████░░░░░░░░░░░░░░░ 33%";

#[test]
fn simple_pie() {
    let buf = render_to_buffer(SIMPLE_PIE, 80, 5);
    assert_buffer_eq(&buf, SIMPLE_PIE_EXPECTED);
}

#[test]
fn three_slices() {
    let buf = render_to_buffer(THREE_SLICES, 80, 6);
    assert_buffer_eq(&buf, THREE_SLICES_EXPECTED);
}

#[test]
fn no_title() {
    let buf = render_to_buffer(NO_TITLE, 80, 5);
    assert_buffer_eq(&buf, NO_TITLE_EXPECTED);
}
