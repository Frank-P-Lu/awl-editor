use super::*;

fn root() -> PathBuf {
    PathBuf::from("/proj/notes")
}

fn opened(ws: &mut WorkingSet, rel: &str) -> usize {
    let p = root().join(rel);
    ws.open(BufferKey::path(&p), Some(p.clone()), root())
}

fn drawn(ws: &WorkingSet) -> Vec<String> {
    ws.files()
        .iter()
        .map(|f| format!("{}{}", f.parent_label().unwrap_or_default(), f.leaf()))
        .collect()
}

/// **STABLE OPEN ORDER, swept rather than sampled.** The item's own reason for
/// the rule is a pointer already reaching for a row, so the failure to guard
/// against is not "one switch reordered the list" but "SOME switch did".
///
/// A hand-picked switch is exactly the imagined case that lets an MRU
/// implementation pass: re-activating the file that is already last leaves an
/// MRU list unchanged too. So this sweeps EVERY member as the re-activation
/// target, from every starting active slot, and requires the drawn order to be
/// byte-identical to the opening order in all of them.
#[test]
fn reactivating_any_member_from_any_slot_never_moves_a_row() {
    let names = ["index.md", "journal/field-notes.md", "research/sources.md"];
    let opening: Vec<String> = {
        let mut ws = WorkingSet::default();
        for n in names {
            opened(&mut ws, n);
        }
        drawn(&ws)
    };
    assert_eq!(
        opening,
        vec!["index.md", "journal/field-notes.md", "research/sources.md"],
        "the opening order is the drawn order"
    );

    for from in 0..names.len() {
        for to in 0..names.len() {
            let mut ws = WorkingSet::default();
            for n in names {
                opened(&mut ws, n);
            }
            assert!(ws.set_active(from));
            let at = opened(&mut ws, names[to]);
            assert_eq!(at, to, "a re-open returns the file's ORIGINAL slot");
            assert_eq!(ws.active_index(), Some(to));
            assert_eq!(
                drawn(&ws),
                opening,
                "re-activating {} from slot {from} moved a row",
                names[to]
            );
        }
    }
}

/// Closing sweeps every (length, victim) cell rather than one comfortable case.
/// The invariants: the victim is gone, every survivor keeps its relative order,
/// and the active slot still points at a real file — never past the end, which
/// is the off-by-one a `remove` in front of the active index produces and which
/// a test that only ever closes the LAST row cannot see.
#[test]
fn closing_any_row_of_any_length_leaves_the_active_slot_on_a_real_file() {
    for len in 1..=6usize {
        let names: Vec<String> = (0..len).map(|i| format!("f{i}.md")).collect();
        for victim in 0..len {
            for active in 0..len {
                let mut ws = WorkingSet::default();
                for n in &names {
                    opened(&mut ws, n);
                }
                assert!(ws.set_active(active));
                let gone = ws.close(victim).expect("the victim existed");
                assert_eq!(gone.leaf(), names[victim]);
                assert_eq!(ws.len(), len - 1);

                let surviving: Vec<String> = ws.files().iter().map(|f| f.leaf()).collect();
                let expected: Vec<String> = names
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != victim)
                    .map(|(_, n)| n.clone())
                    .collect();
                assert_eq!(surviving, expected, "survivors keep their relative order");

                match ws.active_index() {
                    None => assert!(
                        ws.is_empty(),
                        "len={len} victim={victim} active={active}: no active slot \
                         while {} files remain",
                        ws.len()
                    ),
                    Some(a) => assert!(
                        a < ws.len(),
                        "len={len} victim={victim} active={active}: active slot {a} \
                         is past the end ({})",
                        ws.len()
                    ),
                }
            }
        }
    }
}

/// The zero-document state is REACHABLE from the model: closing the last file
/// leaves no active slot at all, rather than an active index pointing at
/// nothing or a fabricated unnamed member.
#[test]
fn closing_the_last_file_leaves_no_active_file() {
    let mut ws = WorkingSet::default();
    opened(&mut ws, "only.md");
    assert_eq!(ws.active_index(), Some(0));
    ws.close(0);
    assert!(ws.is_empty());
    assert_eq!(ws.active_index(), None);
    assert!(ws.active_file().is_none());
    assert!(ws.active_root().is_none());
}

/// **A row reads by its location, not its leaf.** Two files that share a name
/// under different subfolders must produce different rows — the failure the
/// item names ("not by a leaf name that throws away the location"), and one a
/// leaf-only implementation passes every other test with.
#[test]
fn same_named_files_in_different_folders_are_different_rows() {
    let mut ws = WorkingSet::default();
    opened(&mut ws, "notes.md");
    opened(&mut ws, "journal/notes.md");
    opened(&mut ws, "research/notes.md");
    assert_eq!(ws.len(), 3, "three slots, not one collapsed leaf");
    let rows = drawn(&ws);
    assert_eq!(
        rows,
        vec!["notes.md", "journal/notes.md", "research/notes.md"]
    );
    let unique: std::collections::HashSet<&String> = rows.iter().collect();
    assert_eq!(unique.len(), 3, "every row reads differently");

    // A file directly under the root draws NO location half — there is nothing
    // to add, and an empty span would reserve width for nothing.
    assert_eq!(ws.files()[0].parent_label(), None);
    assert_eq!(ws.files()[1].parent_label().as_deref(), Some("journal/"));
}

/// A row's location is relative to ITS OWN root, not to whatever root is
/// active. The grouped view puts rows from several roots on screen at once, so
/// a label computed against the active root would describe files in the other
/// groups wrongly — and would do it silently, since the string still looks like
/// a path.
#[test]
fn a_rows_location_is_relative_to_its_own_root() {
    let mut ws = WorkingSet::default();
    let a = PathBuf::from("/proj/notes/journal/field.md");
    let b = PathBuf::from("/elsewhere/archive/2019/log.md");
    ws.open(
        BufferKey::path(&a),
        Some(a.clone()),
        PathBuf::from("/proj/notes"),
    );
    ws.open(
        BufferKey::path(&b),
        Some(b.clone()),
        PathBuf::from("/elsewhere/archive"),
    );
    assert_eq!(ws.files()[0].parent_label().as_deref(), Some("journal/"));
    assert_eq!(
        ws.files()[1].parent_label().as_deref(),
        Some("2019/"),
        "the second row is relative to /elsewhere/archive, not to the first row's root"
    );

    // …and the groups partition the stable order without reordering it.
    assert_eq!(ws.group(Path::new("/proj/notes")), vec![0]);
    assert_eq!(ws.group(Path::new("/elsewhere/archive")), vec![1]);
    assert_eq!(
        ws.active_root(),
        Some(Path::new("/elsewhere/archive")),
        "the active file's own root is what a switch must restore"
    );
}

/// **`rekey_active` keeps the moved file's slot, and updates its label from
/// the new path — not `open`, which would read the moved file's new
/// (path-derived) key as a DIFFERENT identity and push a second row.**
#[test]
fn rekey_active_keeps_the_slot_and_updates_the_label() {
    let mut ws = WorkingSet::default();
    let root = PathBuf::from("/proj/notes");
    opened(&mut ws, "a.md");
    let moved = opened(&mut ws, "journal/field.md"); // slot 1, the active one
    opened(&mut ws, "c.md");
    assert_eq!(ws.active_index(), Some(2), "opening c.md moved the pointer");
    assert!(ws.set_active(moved));
    assert_eq!(
        ws.files()[1].parent_label().as_deref(),
        Some("journal/"),
        "precondition: the row starts nested"
    );

    let new_path = root.join("field.md"); // moved UP a level, to the root
    ws.rekey_active(BufferKey::path(&new_path), Some(new_path.clone()));

    assert_eq!(
        drawn(&ws),
        vec!["a.md", "field.md", "c.md"],
        "the slot never moved — only its own label changed"
    );
    assert_eq!(
        ws.files()[1].parent_label(),
        None,
        "the row now reads as living directly under its root"
    );
    assert_eq!(ws.files()[1].path.as_deref(), Some(new_path.as_path()));
    assert_eq!(
        ws.index_of(&BufferKey::path(&new_path)),
        Some(1),
        "the row is findable by its NEW identity"
    );
    assert_eq!(
        ws.index_of(&BufferKey::path(&root.join("journal/field.md"))),
        None,
        "and no longer findable by the old one"
    );
    // The root itself never moved (Move stays bounded to the source file's
    // owning root), so the row's group membership is untouched.
    assert_eq!(ws.group(&root), vec![0, 1, 2]);
}

#[test]
fn rekey_active_is_a_no_op_with_nothing_active() {
    let mut ws = WorkingSet::default();
    let p = PathBuf::from("/proj/notes/a.md");
    ws.rekey_active(BufferKey::path(&p), Some(p));
    assert!(ws.is_empty(), "nothing was opened, nothing to rekey");
}

/// **Closing an INACTIVE row closes that named buffer without activating it.**
/// The pointer route the design decision spells out, and the one an
/// implementation that routes every close through "activate, then close the
/// active one" fails — that implementation leaves the reader looking at a
/// document they never asked for, having briefly loaded it.
///
/// Swept over which row is closed and which is active, because the interesting
/// disagreement (the active file MOVED) only appears for victims before the
/// active slot.
#[test]
fn closing_an_inactive_row_never_changes_which_file_is_active() {
    let names = ["a.md", "b.md", "c.md", "d.md"];
    for active in 0..names.len() {
        for victim in 0..names.len() {
            if victim == active {
                continue;
            }
            let mut ws = WorkingSet::default();
            for n in names {
                opened(&mut ws, n);
            }
            assert!(ws.set_active(active));
            let key = ws.files()[victim].key.clone();
            let before = ws.active_file().unwrap().leaf();
            let gone = ws.close_key(&key).expect("the named row existed");
            assert_eq!(gone.leaf(), names[victim], "closed exactly its target");
            assert_eq!(
                ws.active_file().unwrap().leaf(),
                before,
                "active={active} victim={victim}: closing an inactive row \
                 changed the active file"
            );
            assert!(
                ws.index_of(&key).is_none(),
                "the closed row is gone from the order"
            );
        }
    }
}

/// **`root_for` keeps a file's root across a visit to another project** — the
/// one cell a "use the active root" implementation gets wrong, and gets wrong
/// silently, because every other cell agrees with it.
///
/// The sweep is over the three inputs that can disagree: whether the file is
/// under the remembered root, whether it is under the active one, and whether
/// it is under neither.
#[test]
fn root_for_keeps_a_file_with_its_own_project() {
    let notes = Path::new("/proj/notes");
    let archive = Path::new("/proj/archive");
    let under_notes = Path::new("/proj/notes/journal/field.md");
    let under_archive = Path::new("/proj/archive/log.md");
    let orphan = Path::new("/elsewhere/loose.md");

    // First open: no memory yet, the active root contains it.
    assert_eq!(root_for(under_notes, notes, None), notes);

    // THE CELL THAT MATTERS: re-activated while another project is current.
    // The active root must NOT win — it does not contain the file, and letting
    // it win would erase the only record of where the file belongs.
    assert_eq!(
        root_for(under_notes, archive, Some(notes)),
        notes,
        "a remembered root that still contains the file survives a visit elsewhere"
    );

    // A remembered root that no longer contains the file (it moved out) yields
    // to the active root, rather than pinning a stale project forever.
    assert_eq!(root_for(under_archive, archive, Some(notes)), archive);

    // Under neither: the file stands on its own parent, never borrowing a root
    // it is not inside — a borrowed root would make `parent_label` compute a
    // relative path against a directory the file has nothing to do with.
    assert_eq!(root_for(orphan, notes, None), Path::new("/elsewhere"));
    assert_eq!(
        root_for(orphan, notes, Some(archive)),
        Path::new("/elsewhere")
    );
}

/// `fit_parent` swept across EVERY budget from 0 to past the label's length,
/// asserting three invariants at once rather than checking one comfortable
/// width. The third is the one a "just truncate it" implementation fails: an
/// elided label must still say that it was elided.
#[test]
fn fit_parent_never_overruns_its_budget_and_never_lies_about_depth() {
    for label in [
        "journal/",
        "research/sources/",
        "research/sources/drafts/",
        "a/b/c/d/e/f/",
    ] {
        let full = label.chars().count();
        for budget in 0..full + 3 {
            let got = fit_parent(label, budget);
            match &got {
                Some(s) => {
                    assert!(
                        s.chars().count() <= budget,
                        "{label:?} at budget {budget} produced {s:?}, which is longer"
                    );
                    let first = label.split('/').next().unwrap();
                    assert!(
                        s.starts_with(first),
                        "{label:?} at budget {budget} produced {s:?}, \
                         losing the nearest-to-root segment"
                    );
                    if s != label {
                        assert!(
                            s.contains('…'),
                            "{label:?} at budget {budget} produced {s:?} — shorter than the real \
                             location but saying nothing about the depth it dropped"
                        );
                    }
                }
                None => assert!(
                    budget
                        < format!("{}/…/", label.split('/').next().unwrap())
                            .chars()
                            .count(),
                    "{label:?} at budget {budget} gave up while the elided form still fits"
                ),
            }
        }
        assert_eq!(
            fit_parent(label, full).as_deref(),
            Some(label),
            "exactly at its own width, a label is drawn whole"
        );
    }
}

/// **THE ONE-FILE CONTRACT, AND IT IS ABOUT THE GROUP RATHER THAN THE SET.**
///
/// The margin widens only when the ACTIVE ROOT holds more than one file, so the
/// count that decides it is [`WorkingSet::group`]'s, never [`WorkingSet::len`]'s.
/// The two agree in the easy case and part company in the one this surface
/// exists for: a buffer retained from another project keeps its slot in the set
/// while contributing nothing to this project's stack. A `len()`-based gate
/// passes every single-root test ever written and then draws a stack of one the
/// first time a second root is involved — so this asserts the empty answer at
/// `len() == 2`, where the two rules disagree.
#[test]
fn a_stack_appears_only_once_the_active_root_holds_two_files() {
    let mut ws = WorkingSet::default();
    opened(&mut ws, "index.md");
    assert!(
        ws.stack_rows(&root()).is_empty(),
        "one file under the root must draw no stack"
    );

    // A second file, but under ANOTHER root: the set now holds two, the active
    // root's group still holds one, and the margin must stay a single line.
    let other = PathBuf::from("/proj/archive");
    let far = other.join("old.md");
    ws.open(BufferKey::path(&far), Some(far.clone()), other.clone());
    assert_eq!(ws.len(), 2, "the set holds both files");
    assert!(
        ws.stack_rows(&root()).is_empty(),
        "a file parked under another root must not summon this root's stack"
    );
    assert!(
        ws.stack_rows(&other).is_empty(),
        "nor the other root's, which also holds one"
    );

    // The second file under the ACTIVE root is what widens it.
    opened(&mut ws, "journal/field-notes.md");
    let rows = ws.stack_rows(&root());
    // The FILE rows are still exactly the group, in opening order, excluding
    // the other root — the group filter is unchanged by residual 3.
    let file_rows: Vec<&StackRow> = rows
        .iter()
        .filter(|r| matches!(r.kind, StackRowKind::File))
        .collect();
    assert_eq!(
        file_rows
            .iter()
            .map(|r| format!("{}{}", r.parent, r.leaf))
            .collect::<Vec<_>>(),
        vec!["index.md", "journal/field-notes.md"],
        "the FILE rows are the group, in opening order, and exclude the other root"
    );
    assert_eq!(
        file_rows.iter().filter(|r| r.active).count(),
        1,
        "exactly one row is the reader's current file"
    );
    assert!(
        file_rows[1].active,
        "the file just opened is the active one"
    );
    // RESIDUAL 3's overflow row: the OTHER root's file is hidden from this
    // group, but it is still an OPEN buffer nowhere else on screen, so the
    // generic `+ N more…` row must count it — "same-root overflow and other
    // roots alike" (the queue item's own wording). Before residual 3 this
    // group drew no overflow row at all; the trailing row is the deliberate
    // change, not a regression.
    assert_eq!(
        rows.last().map(|r| &r.kind),
        Some(&StackRowKind::More { hidden: 1 }),
        "the one file parked under the other root is hidden, and counted"
    );
}

/// **WHICH ROW IS MARKED ACTIVE, SWEPT OVER EVERY SLOT.** A stack that marked
/// slot 0, or the last slot, would pass any test that only ever activates the
/// file it just opened — which is the natural way to write this by hand.
#[test]
fn exactly_the_activated_row_is_marked_in_every_slot() {
    let names = ["index.md", "journal/field-notes.md", "research/sources.md"];
    for target in 0..names.len() {
        let mut ws = WorkingSet::default();
        for n in names {
            opened(&mut ws, n);
        }
        assert!(ws.set_active(target));
        let rows = ws.stack_rows(&root());
        let marked: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.active)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(
            marked,
            vec![target],
            "activating slot {target} marked {marked:?}"
        );
    }
}

/// Ten files under one root, named `f0.md`..`f9.md` — the fixture every
/// residual-3 law below sweeps, standing in for the judged gallery's
/// `opening.md`..`archive.md` set at the same count.
fn ten(ws: &mut WorkingSet) {
    for i in 0..10 {
        opened(ws, &format!("f{i}.md"));
    }
}

fn file_leaves(rows: &[StackRow]) -> Vec<String> {
    rows.iter()
        .filter(|r| matches!(r.kind, StackRowKind::File))
        .map(|r| r.leaf.clone())
        .collect()
}

/// THE GALLERY'S REJECTED CANDIDATE, reproduced inline for the non-vacuity
/// proof below: a STATELESS window that re-derives itself from nothing but
/// the active file's index EVERY time, with no memory of where it sat a
/// moment ago — exactly the formula `collapsed-jitter.png` caught jumping an
/// already-visible row to the opposite end of the window.
fn stateless_start(active_in_group: usize, group_len: usize) -> usize {
    let max_start = group_len.saturating_sub(RESTING_FILES);
    active_in_group
        .saturating_sub(RESTING_FILES.saturating_sub(1))
        .min(max_start)
}

/// **THE HOLD-STILL LAW, reproducing the exact `collapsed-jitter.png`
/// sequence the gallery rejected the stateless candidate on.**
///
/// `f7` (the gallery's `journal/entry.md`) activates last, at the BOTTOM of
/// its five-row window; `f3` (the gallery's `plan.md`) sits at the window's
/// TOP row and is already fully visible. Activating it next must leave every
/// drawn row exactly where it was — the row the reader was just looking at
/// does not jump to the opposite end of a shifted window.
///
/// **Non-vacuity, proved without touching the shipped formula**: this asks
/// [`stateless_start`] — the rejected candidate's own formula — the identical
/// question and shows it DOES move, proving the fixture reproduces the real
/// bug and the fix is not testing a formula so close to the old one that both
/// would pass by accident.
#[test]
fn resting_window_holds_still_when_the_newly_active_file_is_already_visible() {
    let mut ws = WorkingSet::default();
    ten(&mut ws);
    // Opening ten files one at a time already slides the window as each new
    // file becomes active in turn (the SAME hold-still/minimal-slide rule
    // this law is about) — so `f0` re-establishes a KNOWN fresh baseline
    // window (`[0..5)`) before the actual jitter-reproduction sequence below,
    // rather than the test depending on wherever the incidental open sequence
    // happened to leave it.
    assert!(ws.set_active(0));
    assert_eq!(
        file_leaves(&ws.stack_rows(&root())),
        vec!["f0.md", "f1.md", "f2.md", "f3.md", "f4.md"],
        "known baseline window"
    );

    assert!(ws.set_active(7), "f7 exists");
    let window_a = file_leaves(&ws.stack_rows(&root()));
    assert_eq!(
        window_a,
        vec!["f3.md", "f4.md", "f5.md", "f6.md", "f7.md"],
        "f7 active lands at the bottom row of a fresh five-row reveal"
    );

    assert!(ws.set_active(3), "f3 exists");
    let window_b = file_leaves(&ws.stack_rows(&root()));
    assert_eq!(
        window_b, window_a,
        "f3 was already the window's own top row — activating it must not move a single drawn row"
    );

    // THE RED-ARM REFERENCE: the same two activations, asked of the rejected
    // stateless formula, on the SAME ten-file group.
    let group_len = 10;
    let leaves_from = |start: usize| -> Vec<String> {
        (start..(start + RESTING_FILES).min(group_len))
            .map(|at| format!("f{at}.md"))
            .collect()
    };
    let rejected_a = leaves_from(stateless_start(7, group_len));
    let rejected_b = leaves_from(stateless_start(3, group_len));
    assert_ne!(
        rejected_a, rejected_b,
        "the rejected stateless formula must actually reproduce the jitter on this exact \
         sequence, or this law is not proving anything about the fix"
    );
}

/// **MINIMAL SLIDE: when the active file leaves the window, it moves by
/// exactly enough to reveal it — never re-centring, never jumping further
/// than the file itself required.**
#[test]
fn resting_window_slides_the_minimum_distance_when_the_active_file_leaves_it() {
    let mut ws = WorkingSet::default();
    ten(&mut ws);
    assert!(ws.set_active(0));
    assert_eq!(
        file_leaves(&ws.stack_rows(&root())),
        vec!["f0.md", "f1.md", "f2.md", "f3.md", "f4.md"],
        "fresh window anchored at the top"
    );

    // f9 is four rows past the window's bottom edge — the slide must land it
    // EXACTLY at the new bottom row, not recentre the window around it.
    assert!(ws.set_active(9));
    assert_eq!(
        file_leaves(&ws.stack_rows(&root())),
        vec!["f5.md", "f6.md", "f7.md", "f8.md", "f9.md"],
        "the window slides down by the minimum distance that reveals f9"
    );

    // f5 is already the window's own top row: hold still.
    assert!(ws.set_active(5));
    assert_eq!(
        file_leaves(&ws.stack_rows(&root())),
        vec!["f5.md", "f6.md", "f7.md", "f8.md", "f9.md"],
        "f5 was already visible; the window must not move"
    );

    // f0 is five rows above the window's top edge — the slide must land it
    // EXACTLY at the new top row, the same minimal-distance law in the other
    // direction.
    assert!(ws.set_active(0));
    assert_eq!(
        file_leaves(&ws.stack_rows(&root())),
        vec!["f0.md", "f1.md", "f2.md", "f3.md", "f4.md"],
        "the window slides up by the minimum distance that reveals f0"
    );
}

/// **THE ACTIVE FILE IS ALWAYS REPRESENTED, and the overflow row's count is
/// EXACT — swept over every slot rather than one hand-picked activation.**
/// Two roots are open at once so the count has to include a hidden buffer
/// that is not even in this root's own group ("same-root overflow and other
/// roots alike", the queue item's own wording).
#[test]
fn overflow_count_is_exact_and_the_active_file_is_always_in_the_visible_window() {
    let mut ws = WorkingSet::default();
    ten(&mut ws);
    let other = PathBuf::from("/proj/archive");
    for i in 0..2 {
        let p = other.join(format!("g{i}.md"));
        ws.open(BufferKey::path(&p), Some(p), other.clone());
    }
    // Re-activate a `notes` file so the active root's group is `notes` again
    // (opening `archive`'s files last made `archive` active).
    assert!(ws.set_active(0));
    assert_eq!(ws.len(), 12, "ten notes files plus two archive files");

    for target in 0..10 {
        assert!(ws.set_active(target), "f{target} exists");
        let rows = ws.stack_rows(&root());
        let visible_files = file_leaves(&rows);
        assert!(
            visible_files.len() <= RESTING_FILES,
            "target={target}: {} visible file rows exceeds the cap",
            visible_files.len()
        );
        assert!(
            visible_files.contains(&format!("f{target}.md")),
            "target={target}: the active file is not in the drawn window {visible_files:?}"
        );
        let more = rows.iter().find_map(|r| match r.kind {
            StackRowKind::More { hidden } => Some(hidden),
            _ => None,
        });
        let expected_hidden = ws.len() - visible_files.len();
        assert_eq!(
            more,
            Some(expected_hidden),
            "target={target}: the +N more row must count every open buffer this window \
             does not draw, across both roots"
        );
    }
}

/// **REVEAL ON OPEN**: the expanded panel opens scrolled so the active row is
/// visible, by the minimal jump — never re-centred arbitrarily.
#[test]
fn expand_opens_scrolled_so_the_active_row_is_visible() {
    let mut ws = WorkingSet::default();
    ten(&mut ws);
    assert!(ws.set_active(9), "the last-opened file is deep in the list");
    assert!(!ws.is_expanded());
    ws.expand();
    assert!(ws.is_expanded());
    let rows = ws.expanded_rows();
    assert!(
        rows.iter().any(|r| r.active),
        "the active row must be inside the panel's own first drawn window: {rows:?}"
    );
    // f9 is the group's last file — the panel's own viewport (EXPANDED_VIEWPORT
    // rows, one heading + ten files = 11 total) cannot show all ten files AND
    // the heading in 8 rows, so the reveal must have scrolled forward rather
    // than defaulting to scroll 0.
    let leaf_at_active = rows
        .iter()
        .find(|r| r.active)
        .map(|r| r.leaf.as_str())
        .unwrap();
    assert_eq!(leaf_at_active, "f9.md");
}

/// **A READER'S OWN SCROLL IS NEVER FOUGHT: the panel does not clamp back
/// toward the active row once open, only to the panel's own bounds.**
#[test]
fn scroll_expanded_never_reverts_toward_the_active_row_and_clamps_only_to_bounds() {
    let mut ws = WorkingSet::default();
    ten(&mut ws);
    assert!(ws.set_active(0));
    ws.expand();
    assert!(
        ws.expanded_rows().iter().any(|r| r.active),
        "opens revealing f0"
    );

    // Scroll far past the end — an oversized delta must clamp to the panel's
    // own max, not error or wrap.
    ws.scroll_expanded(1000);
    let scrolled_rows = ws.expanded_rows();
    assert!(
        !scrolled_rows.iter().any(|r| r.active),
        "f0 must have scrolled OFF screen — nothing pulls it back"
    );

    // A further activation-free scroll must not creep past the same bound —
    // clamped, not merely "still off-screen by luck".
    ws.scroll_expanded(1);
    let still = ws.expanded_rows();
    assert_eq!(
        still, scrolled_rows,
        "scrolling past the bottom bound is a no-op, not a further slide"
    );

    // And the bottom bound is real: scrolling all the way back to 0 and past
    // it clamps at 0 rather than going negative.
    ws.scroll_expanded(-1000);
    assert!(
        ws.expanded_rows().iter().any(|r| r.active),
        "scrolling back past the top bound must land at 0, revealing f0 again"
    );
}

/// **ANY ACTIVATION RE-REVEALS the expanded panel** — the brief's own second
/// scroll clause, read together with the first: opening reveals, a reader's
/// OWN scroll is never fought, but a NEW activation while the panel remains
/// open re-centres exactly the way opening did.
#[test]
fn activating_a_different_file_while_expanded_re_reveals_it() {
    let mut ws = WorkingSet::default();
    ten(&mut ws);
    assert!(ws.set_active(0));
    ws.expand();
    ws.scroll_expanded(1000); // scroll f0 away, as the previous law proved
    assert!(!ws.expanded_rows().iter().any(|r| r.active));

    assert!(
        ws.set_active(9),
        "a fresh activation while the panel is open"
    );
    assert!(
        ws.is_expanded(),
        "the panel stays open across the activation"
    );
    assert!(
        ws.expanded_rows().iter().any(|r| r.active),
        "the newly active file must be re-revealed, not left behind the stale scroll"
    );
}

/// **THE PANEL NEVER OUTLIVES A WORKING SET THAT CAN NO LONGER SHOW ONE.**
/// Closing down to a single file collapses the expanded panel outright — an
/// open panel over a working set with nothing left to browse would be a
/// summoned surface with no reason to exist.
#[test]
fn closing_down_to_one_file_collapses_an_open_panel() {
    let mut ws = WorkingSet::default();
    opened(&mut ws, "a.md");
    opened(&mut ws, "b.md");
    ws.expand();
    assert!(ws.is_expanded());
    let key_a = ws.files()[0].key.clone();
    ws.close_key(&key_a);
    assert!(
        !ws.is_expanded(),
        "the panel must not survive the working set dropping below two files"
    );
    assert!(ws.expanded_rows().is_empty());
}

/// **ROW→FILE RESOLUTION AGREES WITH THE DRAWN ROW**, in the expanded panel's
/// own multi-root, scrolled index space — the click-resolution counterpart to
/// `expanded_rows`, swept the same way `stack_rows`' row→file door is swept
/// elsewhere in this file.
#[test]
fn expanded_row_open_file_resolves_the_exact_row_expanded_rows_draws() {
    let mut ws = WorkingSet::default();
    for n in ["a.md", "b.md"] {
        opened(&mut ws, n);
    }
    let other = PathBuf::from("/proj/archive");
    let far = other.join("c.md");
    ws.open(BufferKey::path(&far), Some(far.clone()), other.clone());
    ws.expand();
    let rows = ws.expanded_rows();
    for (row, drawn) in rows.iter().enumerate() {
        match drawn.kind {
            StackRowKind::File => {
                let file = ws
                    .expanded_row_open_file(row)
                    .unwrap_or_else(|| panic!("row {row} names a file"));
                assert_eq!(
                    file.leaf(),
                    drawn.leaf,
                    "row {row} resolves to a different file than the one drawn"
                );
            }
            StackRowKind::Group { .. } => {
                assert!(
                    ws.expanded_row_open_file(row).is_none(),
                    "row {row} is a heading and must name no file"
                );
            }
            StackRowKind::More { .. } => unreachable!("the expanded panel draws no More row"),
        }
    }
    assert!(
        ws.expanded_row_open_file(rows.len()).is_none(),
        "a row past the end of the drawn panel names no file"
    );
}

/// **`resting_row_index` AGREES WITH THE DRAWN WINDOW AFTER IT SLIDES** — the
/// window-offset bug a naive `group(root)[row]` resolution carries: once the
/// hold-still window has moved away from the top, row 0 of the drawn stack is
/// `group[start]`, not `group[0]`.
///
/// Non-vacuity: the naive resolution is computed alongside the real one and
/// asserted to DISAGREE at this exact window, so the fixture is proved to
/// actually exercise the slid case rather than one where `start` happens to
/// still be `0`.
#[test]
fn resting_row_index_agrees_with_the_drawn_window_after_it_slides() {
    let mut ws = WorkingSet::default();
    ten(&mut ws);
    // A known fresh baseline window ([0..5)) before the real sequence, exactly
    // like `resting_window_holds_still_when_the_newly_active_file_is_already_visible`'s
    // own setup — opening ten files one at a time already slides the window as
    // each becomes active in turn.
    assert!(ws.set_active(0));
    assert!(ws.set_active(7), "f7 exists");
    // The hold-still law (asserted elsewhere) puts the window at [3..8).
    assert_eq!(
        file_leaves(&ws.stack_rows(&root())),
        vec!["f3.md", "f4.md", "f5.md", "f6.md", "f7.md"],
        "precondition: the window has slid to start=3"
    );

    let naive: Vec<usize> = ws.group(&root());
    for (row, &naive_at) in naive.iter().enumerate().take(RESTING_FILES) {
        let resolved = ws
            .resting_row_index(&root(), row)
            .unwrap_or_else(|| panic!("row {row} of a full window must resolve"));
        assert_eq!(
            ws.files()[resolved].leaf(),
            format!("f{}.md", row + 3),
            "row {row} of the slid window must name f{}.md",
            row + 3
        );
        assert_ne!(
            resolved, naive_at,
            "row {row}: the window-aware and the naive (unslid) resolution must \
             disagree here, or this fixture proves nothing about the slide"
        );
    }
    // `RESTING_FILES` (row 5, the +more row's own slot in a 10-file group)
    // still resolves — `group[start + 5] = group[8]`, f8's real slot, one
    // past the visible window but still inside the group. Only a row far
    // enough to run past the group's own length resolves to nothing.
    assert_eq!(
        ws.resting_row_index(&root(), RESTING_FILES)
            .map(|at| ws.files()[at].leaf()),
        Some("f8.md".to_string())
    );
    assert_eq!(
        ws.resting_row_index(&root(), 50),
        None,
        "a row far past the group's own length names no file"
    );

    // A group of exactly one still resolves row 0 — the single-file identity
    // line's own door, which draws no STACK but must still resolve a row.
    let mut lone = WorkingSet::default();
    opened(&mut lone, "only.md");
    assert_eq!(
        lone.resting_row_index(&root(), 0)
            .map(|at| lone.files()[at].leaf()),
        Some("only.md".to_string())
    );
}

/// A file interleaved from a foreign root, mid-list — the arrangement that
/// makes a resolution which forgot the group filter wrong from index 1 on,
/// matching the pattern `app/input/gutter/tests.rs` already sweeps this
/// module's click routes with.
fn foreign_interleaved(ws: &mut WorkingSet) -> PathBuf {
    let other = PathBuf::from("/proj/archive");
    opened(ws, "index.md");
    let far = other.join("outside.md");
    ws.open(BufferKey::path(&far), Some(far), other.clone());
    opened(ws, "alpha.md");
    opened(ws, "beta.md");
    opened(ws, "gamma.md");
    other
}

/// **`reorder_in_group`, SWEPT OVER EVERY (from, to, starting-active) CELL.**
/// The three invariants the item's own brief names: the group lands in the
/// exact order a plain `remove`+`insert` on its own sequence would produce,
/// every OTHER root's absolute slot is untouched (in-group only holds at the
/// storage layer), and reordering never changes WHICH file is active — only
/// where it sits.
#[test]
fn reorder_in_group_moves_the_file_and_leaves_every_foreign_slot_untouched() {
    let group_names = ["index.md", "alpha.md", "beta.md", "gamma.md"];
    for from in 0..group_names.len() {
        for to in 0..group_names.len() {
            for active_at in 0..group_names.len() {
                let mut ws = WorkingSet::default();
                let other = foreign_interleaved(&mut ws);
                let group_before = ws.group(&root());
                assert!(ws.set_active(group_before[active_at]));
                let active_key_before = ws.active_file().unwrap().key.clone();
                let outside_key = ws
                    .files()
                    .iter()
                    .find(|f| f.root == other)
                    .unwrap()
                    .key
                    .clone();
                let outside_slot_before = ws.index_of(&outside_key).unwrap();

                ws.reorder_in_group(&root(), from, to);

                let mut expected: Vec<&str> = group_names.to_vec();
                let moved = expected.remove(from);
                expected.insert(to.min(expected.len()), moved);
                let got: Vec<String> = ws
                    .group(&root())
                    .iter()
                    .map(|&at| ws.files()[at].leaf())
                    .collect();
                assert_eq!(
                    got, expected,
                    "from={from} to={to}: group order after reorder"
                );

                assert_eq!(
                    ws.index_of(&outside_key),
                    Some(outside_slot_before),
                    "from={from} to={to}: the foreign root's own absolute slot moved"
                );

                assert_eq!(
                    ws.active_file().map(|f| f.key.clone()),
                    Some(active_key_before),
                    "from={from} to={to} active_at={active_at}: reorder changed WHICH \
                     file is active, not just where it sits"
                );
            }
        }
    }
}

/// **`reorder_in_group` IS A NO-OP off a single-member group, and on an
/// out-of-range `from`** — a stale drop target (the group shrank underneath a
/// held drag, or `from` never named a real slot) must never panic or corrupt
/// the list.
#[test]
fn reorder_in_group_is_a_no_op_off_range_or_a_single_file_group() {
    let mut ws = WorkingSet::default();
    opened(&mut ws, "only.md");
    ws.reorder_in_group(&root(), 0, 0);
    assert_eq!(
        drawn(&ws),
        vec!["only.md"],
        "a single-file group never moves"
    );

    let mut ws2 = WorkingSet::default();
    for n in ["a.md", "b.md", "c.md"] {
        opened(&mut ws2, n);
    }
    let before = drawn(&ws2);
    ws2.reorder_in_group(&root(), 99, 1);
    assert_eq!(
        before,
        drawn(&ws2),
        "an out-of-range `from` changes nothing"
    );
}

/// **`reorder_target` IN THE RESTING STACK is window-aware**, mirroring
/// `resting_row_index`'s own fix: a drop row resolves against the SLID
/// window, not group index `0..RESTING_FILES` unconditionally.
#[test]
fn reorder_target_in_the_resting_stack_follows_the_slid_window() {
    let mut ws = WorkingSet::default();
    ten(&mut ws);
    assert!(ws.set_active(0));
    assert!(ws.set_active(7));
    // Window is [3..8) — see the hold-still law elsewhere in this file.
    for row in 0..RESTING_FILES {
        assert_eq!(
            ws.reorder_target(&root(), row),
            row + 3,
            "row {row} of the slid resting window must target group slot {}",
            row + 3
        );
    }
    // A row far past the visible window clamps to the group's own last slot
    // rather than reaching past it.
    assert_eq!(
        ws.reorder_target(&root(), 6),
        9,
        "row 6: exactly the last slot"
    );
    assert_eq!(
        ws.reorder_target(&root(), 999),
        9,
        "row far past the group: clamped"
    );
}

/// **`reorder_target` IN THE EXPANDED PANEL clamps DIRECTIONALLY into the
/// origin root's own block** — swept over EVERY drawn row for BOTH roots as
/// the origin, so a clamp that only ever tested one direction (or one origin)
/// cannot pass by accident. Two roots, `notes` (3 files) opened first and
/// `archive` (2 files) opened last — `expanded_full` heads them
/// `[Group(notes), a0, a1, a2, Group(archive), b0, b1]`, seven rows.
#[test]
fn reorder_target_in_the_expanded_panel_clamps_into_the_source_block() {
    let mut ws = WorkingSet::default();
    for n in ["a0.md", "a1.md", "a2.md"] {
        opened(&mut ws, n);
    }
    let other = PathBuf::from("/proj/archive");
    for n in ["b0.md", "b1.md"] {
        let p = other.join(n);
        ws.open(BufferKey::path(&p), Some(p), other.clone());
    }
    ws.expand();
    assert!(ws.is_expanded());
    assert_eq!(
        ws.expanded_rows().len(),
        7,
        "precondition: all seven rows fit inside one unscrolled viewport"
    );

    // origin = `notes` (its own block spans rows 0..=3: its heading + a0/a1/a2).
    let expected_for_notes = [
        0, // row 0: notes' own heading -> top of its own group
        0, // row 1: a0 -> its own slot
        1, // row 2: a1
        2, // row 3: a2 (last)
        2, // row 4: archive's heading -> below notes' block -> clamps to bottom
        2, // row 5: b0 -> same clamp
        2, // row 6: b1 -> same clamp
    ];
    for (row, &want) in expected_for_notes.iter().enumerate() {
        assert_eq!(
            ws.reorder_target(&root(), row),
            want,
            "origin=notes row={row}"
        );
    }

    // origin = `archive` (its own block spans rows 4..=6).
    let expected_for_archive = [
        0, // row 0: notes' heading -> above archive's block -> clamps to top
        0, // row 1: a0 -> same clamp
        0, // row 2: a1 -> same clamp
        0, // row 3: a2 -> same clamp
        0, // row 4: archive's OWN heading -> top of its own group
        0, // row 5: b0 -> its own slot
        1, // row 6: b1 (last)
    ];
    for (row, &want) in expected_for_archive.iter().enumerate() {
        assert_eq!(
            ws.reorder_target(&other, row),
            want,
            "origin=archive row={row}"
        );
    }
}
