//! **THE MACHINE IS NOT AN INPUT TO THE LAYOUT.**
//!
//! The module this grades exists because one settings readout — a
//! [`SettingKind::Path`] row's — is written by the reader's filesystem rather
//! than by awl, and the value column it shares is one column. So its length was
//! reaching the picker's width arithmetic, and two readers at the same window
//! size got different layouts (see the module doc for the measured damage).
//!
//! The three claims here are separate on purpose, because each is satisfiable
//! while the others are broken:
//!
//!   * **BOUND** — no drawn path cell is wider than the allowance the roster
//!     itself sets. This is what removes the machine from the arithmetic.
//!   * **INVARIANCE** — two machines whose paths BOTH overflow the allowance
//!     draw the identical column. A bound alone permits a column that still
//!     shifts with the path, just less.
//!   * **PRESENCE** — the cell still names the folder it is about. A bound is
//!     satisfied perfectly by printing nothing, and an invariance is satisfied
//!     perfectly by printing the same nothing twice; the floor here is set under
//!     the roster's own tightest real cell rather than at a written-down width.
//!
//! Both corpus doors are swept (the Settings menu's `visible_value_cells` and
//! the command palette's `palette_value_cells`), because they take different
//! subsets of the roster through the same owner and a fix applied at one door
//! only would read as fixed from the other.
//!
//! A fourth claim stands on its own below — the allowance carries no LIVE state,
//! so a reader's dictionary cannot change how much of their path they see. It is
//! a separate test because it sweeps a different axis (the variant roster, at one
//! fixed path) rather than a different cell of this one.

use super::*;
use crate::settings::{SettingKind, SettingRow, SettingsValues, value_for};

fn values(project_root: &str) -> SettingsValues {
    SettingsValues {
        page_width_prose: 70,
        page_width_code: 100,
        zoom: 1.0,
        scroll_sensitivity: 1.0,
        default_folder: project_root.to_string(),
        workspace: project_root.to_string(),
        project_root: project_root.to_string(),
        autosave: true,
        history: true,
        session_restore: true,
        keymap: "native".to_string(),
        today_ymd: crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
    }
}

/// The PATH rows, asked of the roster rather than named. A row that becomes a
/// path row enrols itself; a roster that stops having any fails the law outright
/// rather than passing it vacuously.
fn path_rows() -> Vec<&'static SettingRow> {
    crate::settings::SETTINGS
        .iter()
        .filter(|r| r.kind == SettingKind::Path)
        .collect()
}

/// The two corpus doors, by name, each with the rows it takes and the cells it
/// draws for them — so a failure says which door it came out of.
fn doors(v: &SettingsValues) -> Vec<(&'static str, Vec<&'static SettingRow>, Vec<String>)> {
    vec![
        (
            "settings menu (visible_value_cells)",
            crate::settings::visible_rows(),
            crate::settings::visible_value_cells(v),
        ),
        (
            "command palette (palette_value_cells)",
            crate::settings::palette_rows(),
            crate::settings::palette_value_cells(v),
        ),
    ]
}

/// Roots a real reader can have, from "nearly nothing" through the shape that
/// produced the report — a home directory several folders deep with a
/// descriptive project name on the end — plus one that is absurd on purpose and
/// one whose final segment ALONE overflows the allowance, which is `elide_path`'s
/// other branch and the one that middle-truncates the name itself.
const ROOTS: &[&str] = &[
    "/p",
    "/tmp/notes",
    "/Users/someone/Documents",
    "/Users/someone/Documents/writing/projects/2026/the-long-novel-working-draft",
    concat!(
        "/Users/someone/Documents/writing/projects/2026/drafts/chapters/",
        "revisions/final/really/quite/deep/indeed"
    ),
    "/Users/someone/a-single-enormously-descriptive-folder-name-with-no-parents-to-drop",
    "/Users/someone/仕事/原稿/長編小説の作業中の草稿",
];

/// **ONE PATH CELL, GRADED** — the BOUND, and where something was actually cut,
/// the PRESENCE floor. Returns whether this cell was elided, which is what makes
/// it a member of the arm the invariance claim compares.
fn grade_path_cell(
    what: &str,
    cell: &str,
    whole: &str,
    allowance: usize,
    widest_authored: usize,
) -> bool {
    // **BOUND.**
    let n = cell.chars().count();
    assert!(
        n <= allowance,
        "{what}: the drawn cell is {n} chars against the roster's own {allowance}-char allowance \
         — the value column's width is reading off the reader's filesystem again. cell {cell:?}"
    );

    if whole.chars().count() <= allowance {
        // A path that fits is drawn whole: the elision is pressure relief, not a
        // haircut everyone gets.
        assert_eq!(
            cell, whole,
            "{what}: a path already inside the allowance was elided anyway"
        );
        return false;
    }

    // **PRESENCE**, on the arm where something was actually cut.
    assert_eq!(
        n, allowance,
        "{what}: an elided cell should spend the whole allowance — {n} of {allowance} chars \
         used, cell {cell:?}"
    );
    assert!(
        cell.contains('…'),
        "{what}: the cell was shortened without saying so. cell {cell:?}"
    );
    assert!(
        cell.contains('/'),
        "{what}: an elided directory must still read as a path, but every slash vanished. \
         cell {cell:?}"
    );
    // The floor is the ROSTER'S OWN TIGHTEST REAL CELL, not a number: whatever
    // the widest authored readout is, a path gets at least that much room, so
    // "elide the path" can never degrade into a stub beside full-width words.
    assert!(
        n >= widest_authored,
        "{what}: the path cell got {n} chars while an authored readout in the same column got \
         {widest_authored} — the path is the row whose value the reader cannot guess, so it may \
         not be the narrowest thing in the column"
    );
    // The FOLDER the row is about survives. When the last segment fits the
    // allowance beside an ellipsis it is carried whole; when the segment itself
    // overflows, its own tail is.
    let last = whole.rsplit('/').next().unwrap_or(whole);
    let kept = if last.chars().count() + 2 <= allowance {
        last.to_string()
    } else {
        let tail = allowance - 3; // `…/` plus the leaf's own `…`
        let tail = tail / 2 + tail % 2;
        last.chars().skip(last.chars().count() - tail).collect()
    };
    assert!(
        cell.ends_with(&kept),
        "{what}: the cell no longer ends in the folder it names — expected to keep {kept:?}, \
         cell {cell:?}"
    );
    true
}

/// Every claim, over both doors and the whole root ladder. One test because the
/// three claims are graded per cell and reporting them apart would mean sweeping
/// the same ladder three times for nothing.
#[test]
fn a_path_cell_is_bounded_by_the_roster_invariant_across_machines_and_still_names_its_folder() {
    let _g = crate::testlock::serial();

    let paths = path_rows();
    assert!(
        !paths.is_empty(),
        "no row enrolled — this law's subject is the roster's `SettingKind::Path` rows, and \
         an enrolment that matches nothing grades nothing"
    );

    // The DEEP arm: the roots whose readout actually overflows the allowance and
    // so are the ones the elision is about. Collected per (door, row) so the
    // invariance below compares like with like.
    let mut deep: std::collections::BTreeMap<(&str, &str), Vec<(&str, String)>> =
        Default::default();
    let (mut bounded, mut untouched) = (0usize, 0usize);

    let allowance = allowance();
    assert!(
        allowance > 0,
        "the roster's allowance came out at 0 chars, so every path cell would be elided to \
         nothing — it is the widest display name in SETTINGS and one of those must have gone \
         empty"
    );

    for root in ROOTS {
        let v = values(root);

        for (door, rows, cells) in doors(&v) {
            assert_eq!(
                rows.len(),
                cells.len(),
                "{door}: {} rows against {} cells",
                rows.len(),
                cells.len()
            );
            let widest_authored = rows
                .iter()
                .zip(&cells)
                .filter(|(r, _)| r.kind != SettingKind::Path)
                .map(|(_, c)| c.chars().count())
                .max()
                .unwrap_or(0);
            for (row, cell) in rows.iter().zip(&cells) {
                let whole = value_for(row, &v);
                let what = format!("{door}, row {:?}, root {root:?}", row.name);

                if row.kind != SettingKind::Path {
                    // **THE ELISION NEVER LEAKS.** An authored readout is awl's
                    // own words and is drawn exactly as `value_for` gives it —
                    // otherwise a fix aimed at paths would be quietly shortening
                    // "English (Australia)" too.
                    assert_eq!(
                        *cell, whole,
                        "{what}: an authored readout was rewritten on its way to the column"
                    );
                    untouched += 1;
                    continue;
                }

                bounded += 1;
                if grade_path_cell(&what, cell, &whole, allowance, widest_authored) {
                    deep.entry((door, row.name))
                        .or_default()
                        .push((root, cell.clone()));
                }
            }
        }
    }

    // **INVARIANCE.** Two readers whose paths both overflow draw the same column.
    assert!(
        !deep.is_empty(),
        "no root in the ladder overflowed the {allowance}-char allowance, so the elision was \
         never exercised and the bound above is a statement about paths that already fit"
    );
    let mut compared = 0usize;
    for ((door, row), seen) in &deep {
        let widths: std::collections::BTreeSet<usize> =
            seen.iter().map(|(_, c)| c.chars().count()).collect();
        assert_eq!(
            widths.len(),
            1,
            "{door}, row {row:?}: {} different drawn widths across machines whose paths ALL \
             overflow the allowance — the column still moves with the filesystem. {seen:?}",
            widths.len()
        );
        compared += seen.len();
    }

    eprintln!(
        "settings value cells: {} path rows enrolled ({:?}), {} roots, allowance {allowance} \
         chars; {bounded} path cells bounded, {untouched} authored cells untouched, {compared} \
         deep cells compared across {} (door, row) pairs",
        paths.len(),
        paths.iter().map(|r| r.name).collect::<Vec<_>>(),
        ROOTS.len(),
        deep.len(),
    );
}

/// **THE ALLOWANCE CARRIES NO LIVE STATE.**
///
/// An earlier draft derived it from the widest AUTHORED READOUT, which made it
/// move with the dictionary: pick a shorter language and the column narrows, and
/// with it every width decision downstream — which name elides, whether the
/// accessory column is granted, how wide a pane must be before the workspace
/// shows two regions. The signature (`allowance()` takes no `values`) says as
/// much structurally; this says it of the BEHAVIOUR, over the whole variant
/// roster rather than a chosen pair, because a reader changing their dictionary
/// must not change how much of their project path they can see.
#[test]
fn changing_a_live_setting_does_not_change_how_much_of_a_path_is_drawn() {
    let _g = crate::testlock::serial();
    let ambient_dict = crate::spell::active_variant();
    let deep_root = ROOTS
        .iter()
        .max_by_key(|r| r.chars().count())
        .expect("the ladder is not empty");
    let mut across_state: std::collections::BTreeSet<Vec<usize>> = Default::default();
    for variant in crate::spell::DictVariant::ALL {
        crate::spell::set_active_variant(variant);
        let v = values(deep_root);
        across_state.insert(
            crate::settings::visible_rows()
                .iter()
                .zip(crate::settings::visible_value_cells(&v))
                .filter(|(r, _)| r.kind == SettingKind::Path)
                .map(|(_, c)| c.chars().count())
                .collect(),
        );
    }
    crate::spell::set_active_variant(ambient_dict);
    assert_eq!(
        across_state.len(),
        1,
        "the drawn path cells changed width across {} dictionary variants at one fixed path — a \
         live setting is reaching the geometry through the value column: {across_state:?}",
        crate::spell::DictVariant::ALL.len()
    );
    assert!(
        !crate::spell::DictVariant::ALL.is_empty(),
        "no dictionary variant enrolled, so the state axis swept nothing"
    );
}
