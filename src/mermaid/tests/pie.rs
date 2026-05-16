use super::common::*;

static SIMPLE_PIE: &str = "pie title Pets
    \"Dogs\" : 386
    \"Cats\" : 85";

static SIMPLE_PIE_EXPECTED: &str = "
                                    Pets

 Dogs           ██████████████████████████████ 82%
 Cats           ███████░░░░░░░░░░░░░░░░░░░░░░░ 18%";

#[test]
fn simple_pie() {
    let buf = render_to_buffer(SIMPLE_PIE, 80, 5);
    assert_buffer_eq(&buf, SIMPLE_PIE_EXPECTED);
}
