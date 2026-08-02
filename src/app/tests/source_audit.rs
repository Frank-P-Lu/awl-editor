// ── HERMETICITY STRUCTURAL GUARD ────────────────────────────────────────
//
// Rust's privacy model can express "visible to production plus every
// descendant module" (what a private `fn new` already gets — every test
// submodule under `app/` is a descendant of `app`, so it already sees the
// raw constructor) but NOT "visible to production plus this ONE helper
// function's own body" — there is no `pub(in path)` spelling that grants
// access to `new_hermetic`'s definition while denying every sibling test
// module. So the raw constructor's door can't be sealed at compile time
// without also blocking the small set of tests that deliberately need
// the REAL disk (see `App::new_hermetic`'s own doc for that list). This
// is the honest fallback: a SOURCE-SCAN law test, in the same spirit as
// `rowlayout.rs`'s / `theme/`'s no-wildcard enumerations — a structural
// fact asserted at test time, cheap to keep honest because the count it
// guards is small and curated, not a general-purpose linter.
//
// NOTE ON THE NEEDLE: the pattern this scan looks for is built at RUNTIME
// (`app_new_needle`, four separate literals concatenated) rather than
// spelled out as one contiguous string anywhere in this file — otherwise
// this very guard's own source text would match itself and inflate its
// own count. Keep every comment/message below phrased without writing
// the raw constructor's name directly followed by an open paren.
//
// Exact per-file occurrence counts of the needle across the whole crate.
// Every entry below is individually accounted for (see each call site's
// own inline comment): either the ONE real production call, a real-disk
// test that explicitly disables `session_restore` (can't use
// `new_hermetic` because it needs `Buffer::from_file` to see genuine
// bytes), or a test already wrapped in `fs::with_fs`/`FsGuard::install`
// with a controlled fake `InMemoryFs` (hermetic by construction,
// independent of `session_restore`'s value — `app/session.rs`'s own
// tests, which specifically exercise session restore, cannot use
// `new_hermetic` at all since it forces `session_restore: Some(false)`).
// A test that only needs a plain, don't-care-about-disk `App` must go
// through `App::new_hermetic` instead, which never contributes to this
// count at all (its name has an extra `_hermetic` between `new` and the
// open paren, so it never matches the needle).
//
// Adding a NEW raw call anywhere — including a new file — fails this
// test until the count below is consciously updated, which forces the
// same two-way choice every existing site already made.
#[test]
fn real_fs_app_new_calls_are_all_accounted_for() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    scan_dir_for_app_new(&root, &root, &mut counts);

    let expected: &[(&str, usize)] = &[
        // 1 production call (in `crate::app::run`), the ONLY raw `App::new`
        // left in `app.rs` proper now the test module lives in its own file.
        ("app.rs", 1),
        // The 4 test-side raw calls retain their former accounting after the
        // feature split: 2 real-disk lifecycle tests with `session_restore`
        // disabled inline, 1 real-disk chdir/buffer test (same treatment),
        // and the `app_on` helper (every caller installs its own fake FS).
        ("app/tests/lifecycle.rs", 2),
        ("app/tests/buffers.rs", 1),
        ("app/tests/common.rs", 1),
        // 1 real-disk test (`finish_buffer_saves_...`), session_restore
        // disabled inline.
        ("app/daemon.rs", 1),
        // 8 calls, every one inside a `crate::fs::with_fs(fake, || ..)`
        // closure seeded with its own `InMemoryFs` — these tests exist
        // specifically to prove what `apply_session_restore` reads back (5),
        // plus item 76's 3 more: `switch_project` eagerly flushing the new
        // ONE-OWNER root without waiting for blur/quit (1 call), and the
        // A→B→A folder/document/view round-trip test (2 calls — a first App
        // that switches + flushes, a second bare-launch App that resumes from
        // the flushed state) — so they can't use a constructor that forces
        // `session_restore` off.
        ("app/session.rs", 8),
        // 3 store tests (2 recent-projects + 1 recent-files), each inside its
        // own `fs::with_fs(fake, ..)` closure seeded with an `InMemoryFs` — they
        // exist specifically to prove what `App::switch_project` / `App::load_path`
        // / `App::new` write to and read back from the recent-projects /
        // recent-files stores, so they need to CONTROL + INSPECT the injected fs
        // (which `new_hermetic`'s private internal fs hides), never real disk.
        // Same treatment as `app/session.rs` above. Plus 3 NO-PATH-PASTE-SAVES-
        // FIRST tests (`ensure_note_named_before_paste_*`), each also inside its
        // own `fs::with_fs(fake, ..)` closure with an `InMemoryFs` handle kept by
        // the test — they exist specifically to prove what
        // `App::ensure_note_named_before_paste` writes to disk (the promoted
        // note's derived path + its saved bytes), so they need the same
        // CONTROL + INSPECT access `new_hermetic` hides. Same treatment. Plus 1
        // CJK-priority persist test
        // (`persist_cjk_priority_writes_the_whole_ordered_ladder_to_config`),
        // inside its own `fs::with_fs(fake, ..)` closure with an `InMemoryFs`
        // handle — proves what `App::persist_cjk_priority` writes to
        // `config.path` on disk, same CONTROL + INSPECT need. Plus 2
        // SPELLCHECK x CONFIG-RELOAD tests (the spell-toggle-x-theme
        // investigation, 2026-07-18:
        // `reload_config_absent_spellcheck_key_leaves_global_untouched` +
        // `reload_config_reapplies_a_persisted_spellcheck_value_immediately`),
        // each inside its own `fs::with_fs(fake, ..)` closure with an
        // `InMemoryFs` handle — they exist specifically to prove what
        // `App::reload_config` reads back from `config.path` on disk (and,
        // for the absent-key case, that it must NOT force a default), same
        // CONTROL + INSPECT need `new_hermetic` hides. (Item 76 REMOVED the 4
        // two-desk "Notes" flip tests that used to be accounted for here — the
        // command is retired; there is now exactly one active folder.) Plus 2
        // ADD-TO-DICTIONARY tests (item 39:
        // `add_to_dictionary_persists_the_word_and_silences_it_live` +
        // `startup_loads_the_personal_dictionary_so_an_added_word_never_squiggles_across_a_restart`),
        // each inside its own `fs::with_fs(fake, ..)` closure with an `InMemoryFs`
        // handle — they prove what `App::add_to_dictionary` writes to (and
        // `App::new` → `load_user_dictionary` reads back from) `dictionary.txt`
        // beside `config.toml` on disk, the same CONTROL + INSPECT need
        // `new_hermetic` hides. Item 56: this test module now lives in
        // `app/files/tests.rs` (the former `app/files.rs` monolith's split
        // moved its `#[cfg(test)] mod tests` verbatim into its own file).
        // Plus item 180's 5 `ProjectLocation`-derivation tests (the
        // different-parent repro, the same-parent/filesystem-root/explicit-
        // config-workspace/round-trip axis sweep), each inside its own
        // `fs::with_fs(fake, ..)` closure with an `InMemoryFs` handle — they
        // exist specifically to prove what `App::switch_project` leaves
        // `workspace_root` pointing at, so they need the same CONTROL +
        // INSPECT access `new_hermetic` hides. Plus item 183's 2: the live/
        // headless PARITY law (it must compare a live `App`'s derivation against
        // the capture builder's over the SAME injected tree) and the real-chord
        // Switch-project law (it drives `App::apply` through the whole picker
        // journey, so it needs a controlled workspace to navigate) — same
        // CONTROL + INSPECT need, same `fs::with_fs` + `InMemoryFs` treatment.
        ("app/files/tests.rs", 18),
        // 9 LIFETIME STATS + USAGE LEDGER + DISCOVERABILITY tests, each inside its own
        // `fs::with_fs(fake, ..)` closure seeded with an `InMemoryFs` — they exist
        // specifically to prove what the tracking hooks / the ledger's
        // `ledger_note_dispatch` + `stats_flush` write to and read back from
        // `stats.toml`, so they need to CONTROL + INSPECT the injected fs (which
        // `new_hermetic`'s private internal fs hides). Same treatment as
        // `app/session.rs` / `app/files/tests.rs` above. (The 3 added by the ledger:
        // door-attribution round-trip, graduation-candidate ranking, kill-switch;
        // the 2 added by the discoverability round: peek/footer ranking from a fake
        // ledger, and the fresh-ledger-empty case.)
        ("app/stats.rs", 9),
        // 6 WRITING STREAKS tests, each inside its own `fs::with_fs(fake, ..)`
        // closure seeded with an `InMemoryFs` — they exist specifically to prove
        // what `streaks_flush` writes to / reads back from `streaks.toml` (and
        // that the kill switch never writes), so they need to CONTROL + INSPECT
        // the injected fs (which `new_hermetic`'s private fs hides). Same
        // treatment as `app/stats.rs` above. `new_hermetic` also won't do here:
        // it restores the real backend on construction return, but these tests
        // keep driving the fs AFTER construction (`new_document`, the summon flush),
        // so the fake must stay active across the whole closure. (The 3 added by
        // the anchor-swallow fix: fresh-note + fresh-scratch record words typed
        // before the first flush, and the card-summon-freshness flush.)
        ("app/streaks.rs", 6),
        // input.rs's click tests all moved onto `App::new_hermetic` —
        // zero raw calls left.
    ];
    let mut expected_map: std::collections::BTreeMap<String, usize> =
        expected.iter().map(|(k, v)| (k.to_string(), *v)).collect();
    // Any file not listed above must have ZERO occurrences.
    for (file, count) in &counts {
        let want = expected_map.remove(file).unwrap_or(0);
        assert_eq!(
            *count, want,
            "unexpected raw-constructor count in {file}: found {count}, expected {want} — \
             either route the new call through App::new_hermetic, or (if it genuinely needs \
             real disk) disable session_restore inline / wrap it in fs::with_fs and update \
             this test's expected count with a comment explaining why"
        );
    }
    for (file, want) in expected_map {
        assert_eq!(
            0, want,
            "expected {want} raw-constructor call(s) in {file} but found none — did it move to new_hermetic or a different file?"
        );
    }

    // The ONE production call site must still exist exactly once, naming
    // its real argument list (guards against the count staying right by
    // coincidence while the actual production call moved or was deleted).
    let mut production_hits = 0usize;
    count_substr_in_dir(&root, &production_call_needle(), &mut production_hits);
    assert_eq!(
        production_hits, 1,
        "the production App::new call in crate::app::run must exist exactly once"
    );
}

/// Built from separate literals at runtime — see the module-doc note
/// above the guard test for why this can't be one contiguous literal.
#[cfg(test)]
fn app_new_needle() -> String {
    ["App", "::", "new", "("].concat()
}

#[cfg(test)]
fn production_call_needle() -> String {
    format!(
        "{}file, root, cli_workspace, cli_default_folder, config);",
        app_new_needle()
    )
}

#[cfg(test)]
fn scan_dir_for_app_new(
    base: &std::path::Path,
    dir: &std::path::Path,
    counts: &mut std::collections::BTreeMap<String, usize>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let needle = app_new_needle();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_for_app_new(base, &path, counts);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let n = text.matches(&needle).count();
        if n == 0 {
            continue;
        }
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        counts.insert(rel, n);
    }
}

// ── DOCUMENT SLOT OWNERSHIP LAW ──────────────────────────────────────
//
// The manual field-by-field EXTRA-STATE copy-out/copy-back pair this round
// RETIRED (the two former per-buffer-bookkeeping helpers this module's own
// needles below name only via runtime string concatenation, so this comment
// block and the assertions after it can discuss them without self-matching
// their own scan — see the App::new guard's module-doc note above for the
// same technique) must never come back — a whole-slot move (a raw
// `mem::replace`, or a struct-literal assignment) is the only way
// `App::active` may change hands, so a future buffer-scoped field travels
// correctly by construction with no matching edit needed anywhere. This is
// the same source-scan-law spirit as the guard above: a structural fact
// asserted at test time, cheap to keep honest.
#[test]
fn source_audit_the_active_slot_has_one_owner() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let app_root = root.join("app");

    // The two retired helpers (named only by runtime concatenation — see
    // this test function's own doc above) must never reappear anywhere.
    let snapshot_needle = ["snap", "shot", "_ex", "tra"].concat();
    let mut snapshot_hits = 0usize;
    count_substr_in_dir(&root, &snapshot_needle, &mut snapshot_hits);
    assert_eq!(
        snapshot_hits, 0,
        "the retired per-buffer extra-state copy-OUT helper must never come back"
    );
    let restore_needle = ["res", "tore", "_ex", "tra"].concat();
    let mut restore_hits = 0usize;
    count_substr_in_dir(&root, &restore_needle, &mut restore_hits);
    assert_eq!(
        restore_hits, 0,
        "the retired per-buffer extra-state copy-BACK helper must never come back"
    );

    // The WHOLE-SLOT mem::replace that parks the outgoing active buffer
    // exists exactly once, in `document.rs` — the sole place permitted
    // to tear the slot apart. Whitespace-COLLAPSED search (the real call
    // wraps its arguments onto their own lines), so line-wrap reflow can
    // never dodge this scan the way a raw substring match would.
    let mut mem_replace_hits: std::collections::BTreeMap<String, usize> = Default::default();
    // Split across two literals with NO trailing/leading whitespace inside
    // either (the quote+comma noise between them in THIS file's own raw text
    // breaks the contiguous match, so this scan's own source can't trip its
    // own needle — only the RUNTIME-concatenated value can).
    let mem_replace_needle = ["mem::replace(&mut", "self.active"].concat();
    scan_dir_collapsed(&root, &app_root, &mem_replace_needle, &mut mem_replace_hits);
    assert_eq!(
        mem_replace_hits.keys().collect::<Vec<_>>(),
        vec!["app/document.rs"],
        "the whole-slot mem::replace must exist ONLY in document.rs, found in: {mem_replace_hits:?}"
    );
    assert_eq!(mem_replace_hits.get("app/document.rs"), Some(&1));

    // `BufferRegistry::take` — the ACTIVATION half of the swap — is called
    // raw in exactly one place, `document.rs`'s own
    // `private activate`; every other switch site goes through THAT
    // method (never touches `buffer_registry` directly for a take).
    let mut take_hits: std::collections::BTreeMap<String, usize> = Default::default();
    let take_needle = ["self.reg", "istry.take("].concat();
    scan_dir_collapsed(&root, &app_root, &take_needle, &mut take_hits);
    assert_eq!(
        take_hits.keys().collect::<Vec<_>>(),
        vec!["app/document.rs"],
        "raw registry.take() must appear ONLY inside document.rs's private activate; \
         found in: {take_hits:?}"
    );
    assert_eq!(take_hits.get("app/document.rs"), Some(&1));

    // `BufferRegistry::park` — the PARK-OUT half of the swap — is called raw
    // in exactly TWO places inside the private document owner: `document.rs`'s
    // `park_active` parks the buffer being replaced, while
    // `document/session_restore.rs` parks the other never-active survivors
    // read straight from the session file. Any third site is the bypass this
    // law exists to catch.
    let mut park_hits: std::collections::BTreeMap<String, usize> = Default::default();
    let park_needle = ["self.reg", "istry.park("].concat();
    scan_dir_collapsed(&root, &app_root, &park_needle, &mut park_hits);
    assert_eq!(
        park_hits.keys().collect::<Vec<_>>(),
        vec!["app/document.rs", "app/document/session_restore.rs"],
        "raw registry.park() must appear ONLY in the private document owner; \
         found in: {park_hits:?}"
    );
    assert_eq!(park_hits.get("app/document.rs"), Some(&1));
    assert_eq!(park_hits.get("app/document/session_restore.rs"), Some(&1));

    let mut loan_hits: std::collections::BTreeMap<String, usize> = Default::default();
    let loan_needle = ["action_buffer", "_mut("].concat();
    scan_dir_collapsed(&root, &app_root, &loan_needle, &mut loan_hits);
    assert_eq!(
        loan_hits.keys().collect::<Vec<_>>(),
        vec!["app/apply.rs", "app/document.rs"],
        "mutable Buffer loan must be definition + action-core call only: {loan_hits:?}"
    );
    assert_eq!(loan_hits.get("app/apply.rs"), Some(&1));
    assert_eq!(loan_hits.get("app/document.rs"), Some(&1));
}

/// Like [`count_substr_in_dir`], but COLLAPSES all whitespace runs to a
/// single space before matching, per file — so a needle spanning a call's
/// line-wrapped arguments (`self.registry\n    .park(..)`) is found
/// regardless of how the call happens to be formatted, and (unlike a plain
/// substring scan) a future reformat can never silently dodge this law.
/// Records PER-FILE hit counts (relative to `base`), so a failing assertion
/// names exactly where the unexpected occurrence lives.
#[cfg(test)]
pub(super) fn scan_dir_collapsed(
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
            scan_dir_collapsed(base, &path, needle, counts);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        // Strip ALL whitespace (not just collapse runs) so a needle spanning
        // a call's line-wrapped arguments matches regardless of formatting.
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

#[cfg(test)]
fn count_substr_in_dir(dir: &std::path::Path, needle: &str, total: &mut usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            count_substr_in_dir(&path, needle, total);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        *total += text.matches(needle).count();
    }
}

/// The raw winit redraw verb has exactly one source-level owner. Callers use
/// `App::request_frame`; window-assembly paths use that module's narrow
/// window-taking helper. Any restored direct call fails this law by name.
#[test]
fn redraw_requests_have_zero_bypasses_around_the_one_request_door() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let needle = [".", "request_", "redraw", "()"].concat();
    let mut hits: std::collections::BTreeMap<String, usize> = Default::default();
    scan_dir_collapsed(&root, &root, &needle, &mut hits);
    assert_eq!(
        hits,
        std::collections::BTreeMap::from([("app/redraw.rs".to_string(), 1)]),
        "redraw request bypassed the one app/redraw.rs door: {hits:?}"
    );
}

/// The frame poll consumes value snapshots/outcomes from runtime owners. These
/// retired reader names are the loose reach-throughs this seam replaced.
#[test]
fn scheduling_reads_runtime_owners_through_typed_poll_boundaries() {
    let schedule = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/app/schedule.rs"),
    )
    .expect("read scheduler source");
    for retired in [
        "prefix_schedule()",
        "peek_armed_at()",
        "doc_autosave_at()",
        "disarm_doc_autosave()",
    ] {
        assert!(
            !schedule.contains(retired),
            "scheduler reached through a runtime owner via retired `{retired}`"
        );
    }
    assert!(
        !schedule.contains("self.config.ambient_motion_on()"),
        "scheduler must consume ConfigurationRuntime's scheduling snapshot"
    );
    let collapsed: String = schedule.chars().filter(|c| !c.is_whitespace()).collect();
    for required in [
        "self.input.scheduling_snapshot()",
        ".poll_autosave(self.frame.now(),AUTOSAVE_IDLE)",
        "self.config.scheduling_snapshot()",
    ] {
        assert!(
            collapsed.contains(required),
            "scheduler lost typed runtime boundary `{required}`"
        );
    }
}

// ── ITEM 76: the two-desk project-flip command + the old quick-notes-home
// config key are COMPLETELY retired — a grep-forced law, same source-scan
// shape as the guard above. NOTE ON THE NEEDLES: built from concatenated
// fragments AND never spelled out contiguously anywhere in THIS file's own
// comments/strings either (the `app_new_needle` discipline, applied to six
// names instead of one) — otherwise this very law's own source text would
// match itself and inflate its own count.

#[test]
fn retired_item_76_identifiers_leave_no_trace_in_source() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    for needle in retired_item_76_needles() {
        let mut hits = 0usize;
        count_substr_in_dir(&root, &needle, &mut hits);
        // ONE exemption: `capture.rs`'s `SCHEMA_VERSION` history table is
        // DELIBERATELY append-only (never edits a past row, per its own
        // module doc) — a couple of its rows accurately name a field as it
        // was called AT THAT TIME, a legitimate historical record, not a
        // live reference. Nothing else may carry a retired name.
        let config_key_needle = ["notes", "_", "root"].concat();
        if needle == config_key_needle {
            let capture_hits = std::fs::read_to_string(root.join("capture.rs"))
                .map(|text| text.matches(&needle).count())
                .unwrap_or(0);
            hits -= capture_hits;
        }
        assert_eq!(
            hits, 0,
            "retired by item 76 but still present in source outside capture.rs's own \
             append-only schema history: {needle:?} (git log carries the rest)"
        );
    }
}

/// Every identifier item 76 retires, built from concatenated parts (not one
/// contiguous literal) so THIS list can never accidentally match itself.
#[cfg(test)]
fn retired_item_76_needles() -> Vec<String> {
    vec![
        ["Notes", "Flip"].concat(),      // the retired project-flip Action/Effect
        ["notes", "_", "flip"].concat(), // its fn names / [keys] slug
        ["Desk", "Return"].concat(),     // the retired two-desk return-memory type
        ["notes", "_", "return"].concat(), // the App field that held it
        ["notes", "_", "last", "_", "file"].concat(), // its file-side companion field
        ["notes", "_", "root"].concat(), // the retired quick-notes-home config key
    ]
}

/// One page is one rule. The document pager and the History diff's
/// `PageScrollDown`/`PageScrollUp` both step by `App::page_scroll_rows`; a
/// second hand-written copy would let a reader paging a diff and a writer
/// paging a document drift apart by a row.
///
/// This COUNTS rather than merely finding the owner. CLAUDE.md's tripwire is
/// that a needle-locating audit stays green forever while a copy survives
/// beside it — the copy this law replaced lived happily next to its twin. The
/// needle is assembled at runtime so this file's own text cannot match it.
#[test]
fn the_page_scroll_row_rule_has_exactly_one_owner() {
    let needle = ["saturating", "_", "sub(2).max(1)"].concat();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits: Vec<String> = Vec::new();
    fn walk(base: &std::path::Path, dir: &std::path::Path, needle: &str, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, needle, out);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .display()
                .to_string();
            for (n, line) in text.lines().enumerate() {
                if line.contains(needle) {
                    out.push(format!("{rel}:{}", n + 1));
                }
            }
        }
    }
    walk(&root, &root, &needle, &mut hits);
    assert_eq!(
        hits.len(),
        1,
        "the one-page-of-rows rule must live in exactly one place \
         (`App::page_scroll_rows`); found {} sites: {hits:?}",
        hits.len()
    );
}

/// The live root's derived project, index, and workspace all cross from
/// `ConfigurationRuntime` to `ProjectLocation` through one typed policy.
/// `App::resync_project_location` is the sole App-level kernel: a direct
/// `ProjectLocation::resync` elsewhere would recreate the former half-update
/// bug, where a project switch changed the root but left its picker workspace
/// stale.
#[test]
fn resync_project_location_is_the_sole_derivation_of_project_location_fields() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits: std::collections::BTreeMap<String, usize> = Default::default();
    let method = "resync";
    let needle = format!("self.project_location.{method}(");
    scan_dir_collapsed(&root, &root, &needle, &mut hits);
    assert_eq!(
        hits,
        std::collections::BTreeMap::from([("app/files/open.rs".to_string(), 1)]),
        "only App::resync_project_location may cross the typed location-policy \
         boundary; found: {hits:?}"
    );
}

// ── ITEM 183 — THE EVENT-LOOP CENSUS: what stays live-only, exactly ──────
//
// An `&ActiveEventLoop` can only be borrowed from inside a running winit loop
// and cannot be constructed, so ANY `App` transition whose signature demands
// one is, by construction, unreachable from every headless entry point — no
// test, no capture, no sidecar. That made the census below the honest measure
// of the blind region `docs/harness-reach.md` maps, and it is mechanical:
// the parameter position is a fact of the source text, not a judgement call.
//
// The input-dispatch chain takes the narrow `app::Exit` capability. What
// remains in this census genuinely owns a window, surface, or loop control.

/// The census is EXACT, per file, and the input-dispatch chain is EMPTY.
///
/// The second assertion is the load-bearing one: re-typing `&ActiveEventLoop`
/// onto any door in that chain would re-blind the whole live effect surface in
/// one line, and nothing else in the suite would notice.
#[test]
fn the_active_event_loop_census_is_exact_and_the_input_chain_is_free_of_it() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    // Whitespace-collapsed, so a rustfmt line break across the parameter
    // cannot hide a new one. Built at runtime so this file's own prose about
    // the type never counts as a hit.
    let needle = [":&Active", "EventLoop"].concat();
    let mut hits: std::collections::BTreeMap<String, usize> = Default::default();
    scan_dir_collapsed(&root, &root, &needle, &mut hits);

    // Every remaining site, with the window/surface/control-flow capability it
    // genuinely needs — none of them is a pass-through for `exit()`.
    let expected: &[(&str, usize)] = &[
        // `drive_gpu_soak` owns a real window and its control flow.
        ("app.rs", 1),
        // `rebuild_gpu` recreates the window-bound renderer.
        ("app/gpu_recovery.rs", 1),
        // The winit `ApplicationHandler` trait's own six callbacks — their
        // signatures are winit's, not ours: `user_event`, `resumed`,
        // `suspended`, `window_event`, `exiting`, `about_to_wait`. The
        // clock-steppable half of `about_to_wait` already escaped through
        // `Scheduler`/`RecordingScheduler` (`app/schedule.rs`).
        ("app/lifecycle.rs", 6),
        // Surface reconfigure + GPU-fault recovery + redraw: `handle_gpu_fault`,
        // `handle_gpu_frame_outcome`, `on_resized`, `on_redraw_requested`.
        // Each rebuilds a surface or sets `ControlFlow` directly.
        ("app/window.rs", 4),
    ];
    assert_eq!(
        hits,
        expected
            .iter()
            .map(|(f, n)| ((*f).to_string(), *n))
            .collect::<std::collections::BTreeMap<_, _>>(),
        "the event-loop census changed. A NEW site makes that transition \
         unreachable from every headless entry point — take the narrow \
         capability it actually needs (`app::Exit`, `app::Scheduler`) instead, \
         or account for it here AND in docs/harness-reach.md. Found: {hits:?}"
    );

    // THE INPUT-DISPATCH CHAIN, named file by file rather than swept by a
    // wildcard: these are the doors a user's keypress, menu pick, palette
    // command, click, drag or scripted probe chord travels, and they are the
    // reason the live effect-interpretation surface is reachable at all.
    for file in [
        "app/apply.rs",
        "app/input/keys.rs",
        "app/input/mouse.rs",
        "app/input/drags.rs",
        "app/menu.rs",
        "app/probe.rs",
    ] {
        assert_eq!(
            hits.get(file),
            None,
            "{file} must never take an `&ActiveEventLoop` again — one such \
             parameter re-blinds every transition reachable through it"
        );
    }
}
