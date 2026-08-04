//! ITEM 255 — THE FONT LICENCE ROSTER LAW. `assets/fonts/LICENSES.md` claimed
//! "the copyright line for each is in its `name` table", and a hasty
//! `strings`-based audit (2026-08-04) concluded that was false for 38 of 45
//! bundled faces — its own tool couldn't see it: plain `strings` only
//! decodes a Macintosh-platform (ASCII) `name` record, and macOS's `strings`
//! has no `-e` flag to decode the Windows-platform UTF-16BE record every
//! other bundled face actually uses, so that half of the check silently
//! produced nothing for all 45 files. Three more of its "7 found" hits were
//! themselves false positives — `post`-table glyph names like
//! `copyright.sc` (a small-caps variant name), not a `name`-table record at
//! all. A fresh read via [`ttf_parser::Face::names`] — the SAME table
//! `fontdb`/`skrifa` decode at runtime — found every one of the 45 bundled
//! `.ttf` files carries a real copyright-bearing record: nameID 0 for 43 of
//! them, and nameID 7 (trademark) for Monaspace Xenon's two instances, which
//! `LICENSES.md`'s own per-face table already flags as reading its copyright
//! from nameID 7. The claim was true; the measurement was not. This law
//! makes the claim keep being true, on the axis the original audit could
//! not see and a hand-kept list would not catch: a face added later, or an
//! existing face re-subset by a pipeline change that drops its `name`
//! table's copyright record.
//!
//! Both checks below are DERIVED FROM THE DIRECTORY: the roster is
//! `std::fs::read_dir("assets/fonts")`, filtered to `.ttf`, never a
//! hand-kept list a new face could be added without touching. `LICENSES.md`
//! is read from [`crate::embedded_docs::FONT_LICENSES_MD`] — the exact copy
//! that ships — not a second read of the file on disk.
//!
//! MUTATION PROOF (recorded here; see the item's report for the exact panic
//! text): dropping one `.ttf`'s row from a scratch copy of the embedded
//! table's text and re-running failed `missing_rows` by name; truncating a
//! face's `name` table to strip its nameID-0/7 records and re-running failed
//! `missing_copyright` by name. Both were reverted after observing red.

use std::collections::BTreeSet;
use std::path::Path;

use ttf_parser::Face;

const FONTS_DIR: &str = "assets/fonts";

/// Every `.ttf` filename physically present in `assets/fonts/`, sorted — the
/// DIRECTORY, not a hand-kept roster. A face landing here without a matching
/// `LICENSES.md` row, or losing its embedded copyright to a re-subset, fails
/// below by name.
fn bundled_ttf_files() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(FONTS_DIR)
        .unwrap_or_else(|e| panic!("{FONTS_DIR} must be readable from the repo root: {e}"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".ttf"))
        .collect();
    names.sort();
    names
}

/// Every filename `LICENSES.md`'s per-face table names in backticks (e.g.
/// `` `Bitter-Regular.ttf` ``), parsed from the SHIPPED embedded copy so the
/// law checks what actually ships, not a second file read.
fn licensed_ttf_files() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in crate::embedded_docs::FONT_LICENSES_MD.lines() {
        let trimmed = line.trim_start();
        let Some(after_pipe) = trimmed.strip_prefix("| `") else {
            continue;
        };
        if let Some(end) = after_pipe.find('`') {
            let name = &after_pipe[..end];
            if name.ends_with(".ttf") {
                names.insert(name.to_string());
            }
        }
    }
    names
}

/// True when this face's `name` table carries a non-empty copyright-bearing
/// record — nameID 0 (the normal case) or nameID 7 (Monaspace Xenon's
/// documented trademark-field exception). Decoded exactly as `ttf_parser`
/// decodes any Windows- or Unicode-platform record; a face's ADDITIONAL
/// Macintosh-platform (plain-ASCII) duplicate, which this crate does not
/// decode by design, is not required — every bundled face's real content
/// lives in a decodable record.
fn has_copyright_record(face: &Face) -> bool {
    face.names().into_iter().any(|name| {
        matches!(
            name.name_id,
            ttf_parser::name_id::COPYRIGHT_NOTICE | ttf_parser::name_id::TRADEMARK
        ) && name.to_string().is_some_and(|s| !s.trim().is_empty())
    })
}

#[test]
fn every_bundled_face_has_a_licenses_md_row_and_a_copyright_record() {
    let on_disk = bundled_ttf_files();
    assert!(
        on_disk.len() >= 40,
        "{FONTS_DIR}/*.ttf looks empty or the directory read failed — found \
         {} files, expected the ~45-face roster (non-vacuity floor)",
        on_disk.len()
    );

    let documented = licensed_ttf_files();
    assert!(
        documented.len() >= 40,
        "LICENSES.md's per-face table looks empty or unparseable — found {} \
         backtick-quoted `.ttf` names (non-vacuity floor)",
        documented.len()
    );

    let mut missing_rows = Vec::new();
    let mut missing_copyright = Vec::new();
    let mut checked = 0usize;

    for file in &on_disk {
        if !documented.contains(file) {
            missing_rows.push(file.clone());
            continue;
        }

        let path = Path::new(FONTS_DIR).join(file);
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("{file}: could not read {path:?}: {e}"));
        let face = Face::parse(&bytes, 0)
            .unwrap_or_else(|e| panic!("{file}: ttf_parser could not parse this face: {e:?}"));

        if !has_copyright_record(&face) {
            missing_copyright.push(file.clone());
        }
        checked += 1;
    }

    assert!(
        checked >= 40,
        "too few faces were actually checked ({checked}) — the roster \
         intersection with LICENSES.md collapsed, which would hide every \
         other assertion in this test"
    );

    assert!(
        missing_rows.is_empty(),
        "LICENSES.md's per-face table has no row for: {missing_rows:?} — \
         every bundled .ttf needs a documented copyright holder and \
         license, derived from the directory, never a hand-kept list \
         (item 255)"
    );
    assert!(
        missing_copyright.is_empty(),
        "these bundled faces carry NO copyright-bearing name-table record \
         (neither nameID 0 nor nameID 7): {missing_copyright:?} — \
         LICENSES.md's compliance story depends on this being true for \
         every face; a re-subset that silently strips it is the exact \
         regression item 255 was opened to check for"
    );
}
