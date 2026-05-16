use super::common::*;

static SIMPLE_GANTT: &str = "gantt
title Project
section Phase 1
Task 1 :a1, 7d
Task 2 :a2, after a1, 5d";

static SIMPLE_GANTT_EXPECTED: &str = "
                                  Project

 Phase 1
  Task 1           ███████████████████████░░░░░░░░░░░░░░░░░ 7d
  Task 2                                  █████████████████ 5d";

static MULTI_SECTION: &str = "gantt
title Plan
section Q1
Design :d, 14d
section Q2
Build :b, after d, 30d";

static MULTI_SECTION_EXPECTED: &str = "
                                    Plan

 Q1
  Design           █████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░ 14d
 Q2
  Build                         ███████████████████████████ 30d";

static SINGLE_TASK: &str = "gantt
title Solo
section Work
Task :t, 5d";

static SINGLE_TASK_EXPECTED: &str = "
                                    Solo

 Work
  Task             ████████████████████████████████████████ 5d";

#[test]
fn simple_gantt() {
    let buf = render_to_buffer(SIMPLE_GANTT, 80, 6);
    assert_buffer_eq(&buf, SIMPLE_GANTT_EXPECTED);
}

#[test]
fn multi_section() {
    let buf = render_to_buffer(MULTI_SECTION, 80, 8);
    assert_buffer_eq(&buf, MULTI_SECTION_EXPECTED);
}

#[test]
fn single_task() {
    let buf = render_to_buffer(SINGLE_TASK, 80, 5);
    assert_buffer_eq(&buf, SINGLE_TASK_EXPECTED);
}
