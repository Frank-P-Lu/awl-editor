use std::path::PathBuf;

use super::*;
use crate::buffers::BufferKey;

fn open(ws: &mut WorkingSet, root: &str, rel: &str) {
    let root = PathBuf::from(root);
    let path = root.join(rel);
    ws.open(BufferKey::path(&path), Some(path), root);
}

fn file_labels(view: &PrototypeView) -> Vec<String> {
    view.rows
        .iter()
        .filter(|row| matches!(row.kind, StackRowKind::File))
        .map(|row| format!("{}{}", row.parent, row.leaf))
        .collect()
}

/// The whole undecided axis: every population from one through twelve and every
/// active slot. A five-row implementation that simply takes the prefix passes
/// any fixture whose active file happens to be inside that prefix; this sweep
/// forces the active file through every hidden position.
#[test]
fn collapsed_candidate_always_represents_the_active_file_and_counts_every_hidden_open_buffer() {
    for len in 1..=12 {
        for active in 0..len {
            let mut ws = WorkingSet::default();
            for at in 0..len {
                open(&mut ws, "/notes", &format!("f{at}.md"));
            }
            assert!(ws.set_active(active));
            let view = ws.prototype_view(PrototypeSpec::Collapsed { hover: None });
            assert_eq!(
                view.rows.iter().filter(|row| row.active).count(),
                1,
                "len={len} active={active}: active file fell out of the resting window"
            );
            assert_eq!(view.report.visible_file_rows, len.min(RESTING_FILES));
            assert_eq!(
                view.report.hidden,
                len.saturating_sub(RESTING_FILES),
                "len={len} active={active}: generic count is not all hidden files"
            );
            let more = view.rows.iter().find_map(|row| match row.kind {
                StackRowKind::More { hidden } => Some(hidden),
                _ => None,
            });
            assert_eq!(more, (view.report.hidden > 0).then_some(view.report.hidden));
        }
    }

    // Other roots are not a second counter. They join the SAME generic count.
    let mut ws = WorkingSet::default();
    open(&mut ws, "/notes", "only.md");
    open(&mut ws, "/archive", "old-a.md");
    open(&mut ws, "/archive", "old-b.md");
    assert!(ws.set_active(0));
    let view = ws.prototype_view(PrototypeSpec::Collapsed { hover: None });
    assert_eq!(file_labels(&view), ["only.md"]);
    assert_eq!(view.report.hidden, 2);
    assert!(matches!(
        view.rows[1].kind,
        StackRowKind::More { hidden: 2 }
    ));
}

#[test]
fn expanded_scroll_is_clamped_inside_the_active_group_and_never_reorders_it() {
    let mut ws = WorkingSet::default();
    for at in 0..13 {
        open(&mut ws, "/notes", &format!("journal/f{at}.md"));
    }
    let opening = ws
        .files()
        .iter()
        .map(|file| format!("{}{}", file.parent_label().unwrap_or_default(), file.leaf()))
        .collect::<Vec<_>>();
    for requested in [0, 1, 4, 5, 6, usize::MAX] {
        let view = ws.prototype_view(PrototypeSpec::Expanded {
            scroll: requested,
            hover: None,
        });
        let expected_start = requested.min(13 - EXPANDED_FILES);
        assert_eq!(view.report.scroll, expected_start);
        assert_eq!(
            file_labels(&view),
            opening[expected_start..expected_start + EXPANDED_FILES],
            "scroll={requested}: the working-set viewport escaped or reordered its source"
        );
        assert_eq!(view.report.visible_file_rows, EXPANDED_FILES);
        assert_eq!(view.report.hidden, 13 - EXPANDED_FILES);
    }
}

#[test]
fn grouped_view_names_every_real_root_once_and_marks_the_active_group_and_file() {
    let mut ws = WorkingSet::default();
    open(&mut ws, "/workspace/notes", "opening.md");
    open(&mut ws, "/workspace/notes", "journal/field.md");
    open(&mut ws, "/workspace/archive", "2019/log.md");
    open(&mut ws, "/workspace/archive", "2020/log.md");
    assert!(ws.set_active(1));

    let view = ws.prototype_view(PrototypeSpec::Grouped { hover: None });
    let groups = view
        .rows
        .iter()
        .filter_map(|row| match row.kind {
            StackRowKind::Group { active } => Some((row.leaf.as_str(), active)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(groups, [("notes", true), ("archive", false)]);
    assert_eq!(
        file_labels(&view),
        [
            "opening.md",
            "journal/field.md",
            "2019/log.md",
            "2020/log.md"
        ]
    );
    assert_eq!(view.rows.iter().filter(|row| row.active).count(), 1);
    assert_eq!(view.report.hidden, 0);
    assert_eq!(view.report.total_open, 4);
}

#[test]
fn prototype_hover_can_reveal_only_a_real_file_row() {
    let mut ws = WorkingSet::default();
    open(&mut ws, "/notes", "one.md");
    open(&mut ws, "/notes", "two.md");
    for at in 0..6 {
        open(&mut ws, "/archive", &format!("old-{at}.md"));
    }
    assert!(ws.set_active(0));
    let more_at = ws
        .prototype_view(PrototypeSpec::Collapsed { hover: None })
        .rows
        .iter()
        .position(|row| matches!(row.kind, StackRowKind::More { .. }))
        .expect("the fixture has overflow");
    let rejected = ws.prototype_view(PrototypeSpec::Collapsed {
        hover: Some(more_at),
    });
    assert_eq!(rejected.report.hovered_row, None);
    assert!(rejected.rows.iter().all(|row| !row.prototype_hovered));

    let shown = ws.prototype_view(PrototypeSpec::Collapsed { hover: Some(0) });
    assert_eq!(shown.report.hovered_row, Some(0));
    assert!(shown.rows[0].prototype_hovered);
}
