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
                        "len={len} victim={victim} active={active}: no active slot while {} files remain",
                        ws.len()
                    ),
                    Some(a) => assert!(
                        a < ws.len(),
                        "len={len} victim={victim} active={active}: active slot {a} is past the end ({})",
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
                "active={active} victim={victim}: closing an inactive row changed the active file"
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
                        "{label:?} at budget {budget} produced {s:?}, losing the nearest-to-root segment"
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
