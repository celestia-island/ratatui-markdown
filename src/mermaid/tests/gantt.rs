use super::common::*;

static SIMPLE_GANTT: &str = "gantt\ntitle Project\nsection Phase 1\nTask 1 :a1, 7d\nTask 2 :a2, after a1, 5d";
static SIMPLE_GANTT_EXPECTED: &str = "                                  Project\n\n Phase 1\n  Task 1           ████████████████████░░░░░░░░░░░░░░░░░░░░ 7d\n  Task 2                 ████████████████████░░░░░░░░░░░░░░ 5d";

#[test]
fn simple_gantt() {
    let buf = render_to_buffer(SIMPLE_GANTT, 80, 6);
    assert_buffer_eq(&buf, SIMPLE_GANTT_EXPECTED);
}
