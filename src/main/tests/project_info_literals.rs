// ── The capture's project location has ONE derivation ────────────────────
//
// `capture_screenshot` reports the project location TWICE (launch root, then
// again on an accepted Project-picker row) and `story.rs` a third time. Those
// were three hand-rolled `ProjectInfo` literals, and the accept site carried
// the LAUNCH root's `workspace` forward while re-deriving everything else from
// the accepted root — the exact half-derivation in the harness's own
// copy of the rule, after the App's copy was fixed. A capture of a
// Switch-project therefore reported a workspace the running editor no longer
// had. `run::project_info` is now the one builder; the parity law pinning it to
// the live `App` is `app::files::tests::
// the_capture_sidecars_project_location_equals_the_live_apps`.
//
// This is the OTHER half of that law, and the half the parity test cannot do:
// a parity test proves the BUILDER is right, never that every site USES it.
// The bug was in a call site, so the guard has to be structural.

/// Per-file counts of a whitespace-collapsed needle under `src/`. Mirrors
/// `app::tests::source_audit::scan_dir_collapsed` (whose module is private to
/// `crate::app`): stripping whitespace first means a line-wrapped literal can
/// never dodge the scan by being reformatted.
fn scan_src_collapsed(needle: &str) -> std::collections::BTreeMap<String, usize> {
    fn walk(
        base: &std::path::Path,
        dir: &std::path::Path,
        needle: &str,
        counts: &mut std::collections::BTreeMap<String, usize>,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, needle, counts);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let collapsed: String = text.chars().filter(|c| !c.is_whitespace()).collect();
            let n = collapsed.matches(needle).count();
            if n == 0 {
                continue;
            }
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            *counts.entry(rel).or_insert(0) += n;
        }
    }
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut counts = Default::default();
    walk(&base, &base, needle, &mut counts);
    counts
}

/// Every `ProjectInfo` LITERAL in the tree is individually accounted for, so a
/// new hand-rolled one — the shape that caused the stale-workspace sidecar —
/// fails here until its author consciously chooses between routing through
/// `run::project_info` and declaring why it must not.
#[test]
fn every_capture_project_info_literal_is_accounted_for() {
    // NOTE ON THE NEEDLE: built at RUNTIME from two literals rather than
    // spelled contiguously anywhere in this file, or this guard's own source
    // text would match itself and inflate the count it guards (the same
    // precaution `app/tests/source_audit.rs` takes for its own needle).
    // Whitespace is stripped before matching, so the needle also catches the
    // struct DECLARATION and the one builder's RETURN TYPE — both accounted
    // for below rather than filtered out, since an unaccounted-for match is
    // exactly what this law wants to be loud about.
    let needle = ["Project", "Info{"].concat();
    let hits = scan_src_collapsed(&needle);
    let expected: &[(&str, usize)] = &[
        // The struct's own declaration.
        ("capture/opts.rs", 1),
        // A doc-drift ORACLE, not a capture: every field deliberately `Some`,
        // so the sidecar writer emits its whole key set and CAPTURE.md's
        // `project` row can be held to it. Routing this through
        // `run::project_info` would make the oracle depend on what a real
        // launch happens to populate, which is the opposite of what it needs.
        ("capture/tests/capture_md_drift.rs", 1),
        // A hand-built LEAK fixture: every path field anchored under the real
        // `$HOME` BY CONSTRUCTION, so the home-redaction sweep enrols the same
        // way on a developer machine and on a runner whose checkout sits
        // outside its home. `run::project_info` would derive those paths from
        // wherever this checkout happens to be — exactly the
        // configuration-dependent enrolment that sweep must not have.
        ("capture/tests/redact_law.rs", 1),
        // A hand-built sidecar FIXTURE: no root, no filesystem, no derivation
        // to get wrong — it exists to pin the JSON schema's chrome block.
        ("capture/tests/schema_chrome.rs", 1),
        // The two deliberately LOCATION-FREE capture modes. `--capture-timeline`
        // and `--capture-held` report `default_folder: None, workspace: None`
        // on purpose: neither takes a `--workspace`/`--default-folder` flag,
        // and a `Some(..)` there would invent a workspace their sidecars have
        // never carried. They are not half-derivations — they derive nothing.
        ("main/run.rs", 2),
        // THE ONE BUILDER (`run::project_info`): its return type + its body;
        // plus `ReplaySession::current_project_info`'s typed return. That
        // session method contains no literal: it supplies the current private
        // root/workspace inputs and delegates straight to the builder, so a
        // storyboard step cannot cache or hand-roll the location derivation.
        ("main/run/location.rs", 3),
    ];
    assert_eq!(
        hits,
        expected
            .iter()
            .map(|(f, n)| ((*f).to_string(), *n))
            .collect::<std::collections::BTreeMap<_, _>>(),
        "a new `ProjectInfo` literal appeared (or an accounted one moved). \
         Route it through `run::project_info` — the ONE derivation of a \
         capture's project location — or add it above with the reason it \
         cannot be. Found: {hits:?}"
    );
}
