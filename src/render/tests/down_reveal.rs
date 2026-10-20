use super::{headless_pipeline, view_md};
use crate::actions::LayoutOracle;

#[test]
fn down_arrow_lands_nearest_the_remembered_x_after_target_markup_reveals() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let _page = crate::page::PagePin::snapshot();
    let was_wysiwyg = crate::markdown::wysiwyg_on();
    crate::markdown::set_wysiwyg_on(true);
    crate::page::set_page_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping arrow reveal regression: no GPU adapter");
        return;
    };
    let mut failures = Vec::new();
    for world in crate::theme::world_names() {
        crate::theme::set_active_by_name(world).unwrap();
        for measure in [12, 30, 70] {
            crate::page::set_measure(measure);
            p.set_size(1200.0, 800.0);
            for target in [
                "abc",
                "abcdefghijklmnopqrstuv",
                "**abcdefghijklmnopqrstuv**",
                "- abcdefghijklmnopqrstuv",
                "- [ ] abcdefghijklmnopqrstuv",
                "- parent\n  - nested **body** with words to wrap",
                "**日本語の文章をもう少し長くします**",
                "`abcdefghijklmnopqrstuv`",
                "[abcdefghijklmnopqrstuv](https://example.invalid)",
                "# abcdefghijklmnopqrstuv",
                "e\u{301} 日本語 العربية עברית",
                "> quoted **prose** with words",
                "a paragraph with many words that will wrap onto another visual row",
            ] {
                for col in [0, 3, 8, 14, 22] {
                    let text = format!("abcdefghijklmnopqrstuv\n{target}\nabcdefghijklmnopqrstuv");
                    for down in [true, false] {
                        let origin = if down { 0 } else { text.lines().count() - 1 };
                        p.set_view(&view_md(&text, origin, col));
                        // Start from the boundary visual row so the step enters
                        // the next logical line even at the narrowest measure.
                        let origin_rows = p.visual_rows(origin);
                        let row = if down { origin_rows.len() - 1 } else { 0 };
                        let start_col = col.clamp(
                            origin_rows[row].start_col,
                            origin_rows[row].end_col.saturating_sub(1),
                        );
                        p.set_view(&view_md(&text, origin, start_col));
                        let goal =
                            p.visual_x_of(origin, start_col, crate::caret::Affinity::Downstream);
                        let (line, landed) = if down {
                            p.visual_line_down(
                                origin,
                                start_col,
                                goal,
                                crate::caret::Affinity::Downstream,
                            )
                        } else {
                            p.visual_line_up(
                                origin,
                                start_col,
                                goal,
                                crate::caret::Affinity::Downstream,
                            )
                        };
                        assert_ne!(line, origin, "the test must cross a logical-line boundary");
                        p.set_view(&view_md(&text, line, landed));
                        let rows = p.visual_rows(line);
                        let row = if down { 0 } else { rows.len() - 1 };
                        let expected = crate::render::col_on_row(&rows, row, goal);
                        if landed != expected {
                            failures.push(format!(
                                "{world}/{measure} {target:?} col {col} down={down}: \
                                     landed {landed}, expected {expected}"
                            ));
                        }
                    }
                }
            }
        }
    }
    crate::markdown::set_wysiwyg_on(was_wysiwyg);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn arrow_destination_shape_work_is_local_to_concealed_source() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let Some(mut p) = headless_pipeline() else {
        return;
    };
    let was_wysiwyg = crate::markdown::wysiwyg_on();
    crate::markdown::set_wysiwyg_on(true);
    let text = format!(
        "# Heading\n{}**concealed**\n",
        "ordinary prose\n".repeat(2000)
    );
    p.set_view(&view_md(&text, 0, 0));
    assert!(
        !p.needs_reveal_shape(2000),
        "ordinary destination never enters attrs rebuilding or shaping"
    );
    assert!(p.needs_reveal_shape(2001));
    let mut selected = view_md(&text, 0, 0);
    selected.selection = Some(((2001, 0), (2001, 8)));
    p.set_view(&selected);
    assert!(
        !p.needs_reveal_shape(2001),
        "selection already reveals the source"
    );
    p.set_view(&view_md(&text, 2001, 2));
    assert!(
        !p.needs_reveal_shape(2001),
        "already active source retains geometry"
    );
    crate::markdown::set_wysiwyg_on(false);
    p.set_view(&view_md(&text, 0, 0));
    assert!(
        !p.needs_reveal_shape(2001),
        "literal Markdown has no conceal transition"
    );
    crate::markdown::set_wysiwyg_on(was_wysiwyg);
}
