const EXPECTED: &[(&str, usize)] = &[
    // Startup / rare live-only failure paths, largely before or around a
    // usable window (spell dictionary / clipboard / render-state init, the
    // daemon socket) — none has a `self.notice` seam to route through this
    // early, and each is a one-time, non-recurring condition.
    ("app.rs", 3),
    // The document owner constructs the shared spell checker before an App
    // notice seam exists, so dictionary-load failure remains a startup note.
    ("app/document.rs", 1),
    ("app/gpu_recovery.rs", 1),
    ("app/lifecycle.rs", 1),
    // NOTE: benchmark-module rows used to live inline here, one per file, each
    // holding an exact print count. See `BENCHMARK_MODULE_PATHS` below — a
    // module living under one of those paths needs no row in this table at
    // all, and the audit test enforces that the two mechanisms never overlap.
    // LIVE PROBE harness protocol/diagnostic lines (fate (c), CLI harness
    // output): the driver's ready-timeout warning + the ONE `PROBE-TRACE …`
    // owner (`probe.rs::trace`, the single stderr print site every present /
    // crossing / move trace routes through, so the scattered call sites in
    // `app/apply.rs` / `app/gpu.rs` / `app/window.rs` carry no print macro of
    // their own), the per-shot `LIVE-PROBE shot …` protocol lines the wrapping
    // script asserts on (`app/probe.rs`), and `app.rs`'s shots-dir creation
    // failure (counted in the row above). The third print site is the FLIGHT
    // RECORDER's `AWL_FLIGHT_RECORDER` open-failure warning (`init_flight`) — a
    // one-time startup-before-notice diagnostic (fate (c)), like the config/GPU
    // init failures above; when the recorder can't open its file it says so and
    // stays off rather than failing the launch.
    ("probe.rs", 3),
    // TWO entries cover the `LIVE-PROBE latency …` protocol line's ok/none arms
    // (`ProbeEvent::Latency`) — the movement-latency distribution report,
    // mirroring the existing per-shot line's fate (c) exactly.
    ("app/probe.rs", 7),
    // `--bench-a11y`'s report table is hidden CLI performance-harness output,
    // enrolled structurally under `BENCHMARK_MODULE_PATHS` below rather than
    // as a row here.
    // The hidden persistence fault probe's autosave/export completion markers
    // and large-save bytes/time/RSS receipt are CLI test protocol output. The
    // probe is native-only and cannot reach the interactive App.
    ("app/persistence/fault_probe.rs", 4),
    ("app/apply.rs", 1),
    // Best-effort background bookkeeping failures (config write, a
    // sticky-pref/rebind write, the recent-files/projects MRU save, a
    // dictionary switch, the autosave/scratch-stash engine, the
    // personal-dictionary FILE-append) — all rare, all non-fatal by design
    // ("never disrupt the edit/save"). Flagged as future notice-routing
    // candidates. `app/files/` is a split of the former `app/files.rs`
    // monolith, the same best-effort-write sites redistributed by which
    // submodule now owns each verb: config open + the recent MRUs in
    // `open.rs`; sticky-pref + page-width-reset in `settings.rs`; the
    // rebind-menu writes in `rebind.rs`; the autosave/scratch-stash engine
    // in `autosave.rs`; the dictionary switch + the
    // personal-dictionary append and its mirror-image forget in
    // `dictionary.rs`. Credits opens as a summoned read-only viewer,
    // never a buffer, so it reaches no write path and adds no line here.
    ("app/files/open.rs", 3),
    ("app/files/settings.rs", 2),
    ("app/files/rebind.rs", 2),
    ("app/files/autosave.rs", 2),
    ("app/files/dictionary.rs", 3),
    // Switching the session's spell checker is owned with that checker; its
    // rare load failure remains the same best-effort diagnostic class.
    ("app/document/cache.rs", 1),
    // GPU/render-pipeline errors (`prepare`/`render`) retain a stderr
    // diagnostic while App-owned recovery also paints the calm notice. The
    // presentation half lives with the extracted present transaction.
    ("app/gpu.rs", 1),
    ("app/gpu/present.rs", 1),
    ("app/window.rs", 1),
    ("app/session.rs", 1),
    // THE LOCAL USAGE LEDGER: ONE `{what} save failed: {e}` stderr line, in the
    // `Dirtying::flush` door both records share (a failed atomic write of
    // `stats.toml` / `streaks.toml` must never disrupt the editor — it warns
    // and moves on).
    ("app/usage.rs", 1),
    ("buffers.rs", 1),
    // Headless capture harness diagnostics ("spell-check disabled for
    // capture: …") — CLI/test-harness output, not live-app chatter.
    ("capture/policy.rs", 1),
    ("capture/oracle.rs", 1),
    ("config/model.rs", 1),
    // A `[keys] follow = "…"` entry the pointer-chord grammar cannot spell:
    // the line is named, the platform default is kept, and the reader is told
    // — the same shape a bad KEY chord already gets, on the same config-load
    // path as `config/model.rs`'s own note above. Startup config parsing, not
    // live-app chatter, so it does not belong on the notice seam.
    ("keymap/follow.rs", 1),
    ("keymap/state.rs", 4),
    ("main.rs", 2),
    // `--help`'s big usage dump, plus `--list-worlds`: a
    // machine-readable roster dump for `scripts/capture-worlds.sh` and any
    // other script that wants the world list without parsing --help. Plus
    // `--pack-icns`'s two lines: the per-world byte table and the
    // summary the icon export prints as its deliverable receipt. Plus
    // `--export-linux-icon`'s own one-line deliverable receipt, same shape
    // as `--pack-icns`'s summary line. All five are fate (c) — genuine
    // CLI/diagnostic stdout, not app-runtime chatter. Lives in the
    // argument-token loop, one phase of `parse_args`'s own decomposition
    // (`args/parse.rs`'s module doc) — the former `main/args.rs` row.
    ("main/args/parse/loop_flags.rs", 5),
    // `--screenshot`/`--screenshot-motion*`/`--screenshot-frames`/`--capture-*`'s
    // "wrote …" deliverable output — this IS the CLI's product, read by
    // scripts/agents — plus the permissive `--keys` replay's ONE stderr warning
    // seam (the strict-replay round: `replay::warn_line` fires when a replay crosses
    // an Unsupported/Intercepted effect; CLI diagnostic output by design, and the
    // same string is recorded in the replay result so tests pin it). (The 8th is the
    // virtual-clock frame-loop capture's own "wrote N frame(s)…" deliverable line.)
    // `load_buffer`'s own refusal line is the
    // headless capture door's analog of `App::new`'s sticky notice (there is
    // no live App/notice seam here to route through), reported the same way
    // every other CLI-only diagnostic in this file is: a stderr line naming
    // what happened before the capture proceeds on a scratch buffer instead.
    // The permissive replay warning lives in the typed effect interpreter;
    // the total count spans both files.
    ("main/run.rs", 8),
    ("main/run/trace.rs", 1),
    // The live-`App` capture mode's one "wrote OUT.png (+ sidecar
    // .json)" deliverable line, worded identically to `capture_screenshot`'s in
    // `main/run.rs` above. CLI product output, not a diagnostic — a capture
    // mode's whole job is to say where it put the artifact.
    ("main/run/live_app.rs", 2),
    // `--storyboard`'s deliverable output (the run summary + "wrote film…"),
    // plus the BEST-EFFORT film-encode notes ("no ffmpeg on PATH", a nonzero
    // ffmpeg exit, a non-UTF-8 output path) — CLI product + diagnostics by
    // design; the raw frames are always retained, so each note is advisory.
    ("main/story.rs", 5),
    ("menu.rs", 1),
    // The reference's REGENERATION TOOL: an `#[ignore]`d test that prints each
    // generated section fenced by a delimiter `scripts/regen-reference.sh`
    // splices on. Stdout is the whole mechanism (the repo's regeneration
    // convention — a test prints, a human-run script writes; no test ever
    // writes a repo file), and the module is `cfg(test)`, so none of these can
    // reach a shipped binary. The site page's generated sidebar nav adds a
    // second fenced block (BEGIN/END) printed the same way, ahead of the five
    // section blocks. The supported-Markdown page adds its Markdown and HTML
    // fenced blocks through that same ignored generator, with the same fate.
    ("reference/law/mod.rs", 11),
    // `AWL_FONT` + `AWL_CHROME_FACE_FILE` dev-only env var override
    // diagnostics (the second is the Firetail-showcase round's audition-font
    // loader: a missing/unreadable candidate file prints a note and is
    // skipped — the same advisory class as `AWL_FONT`'s fallback note).
    ("render.rs", 2),
    // `read_forced_knob`'s unrecognized-value warning (moved here with the
    // `AWL_*_FORCE` knobs it serves).
    ("render/overrides/parsers.rs", 1),
    // The frame/picker/caret/theme-burst profilers (`render/framebench.rs` and
    // its `pickersweep` submodule, `render/perfbench.rs`, `render/caretbench.rs`,
    // `render/benchsuite/**`) are hidden CLI performance-harness output —
    // enrolled structurally under `BENCHMARK_MODULE_PATHS` below, not as rows
    // here. A file living there, or a println! added to one, needs no edit
    // above: the recurring cost this replaced was re-counting an existing
    // bench file on every edit and moving lines between two rows on every
    // carve-out (a themed-burst count bump, a picker sweep split from its
    // parent, a new typing-benchmark stage — three separate table edits for
    // output that was never anything but hidden bench protocol).
    // `--soak-gpu`'s bounded native-probe report is CLI product: result,
    // counters (incl. the per-cause `skipped_by_kind` breakdown), memory
    // summaries, recovery timings, and explicit defects. All print sites live
    // in the report submodule; `soak_gpu/mod.rs` (the schedule/observe half)
    // prints nothing, so it does not appear here.
    ("soak_gpu/report.rs", 8),
    // The shared test device's allocation trace, printed only when
    // `AWL_GPU_ALLOC_TRACE` is set in the environment. It is a diagnostic for
    // the render suite's own GPU accounting, not product output: `main.rs`
    // declares `mod test_gpu` under `#[cfg(test)]`, so this line cannot exist in
    // a shipping binary, and with the variable unset it never runs. stdout on
    // purpose — it has to interleave in order with libtest's own `test … ok`
    // lines to be readable at all.
    ("test_gpu.rs", 1),
];

/// Benchmark modules: hidden CLI performance-harness output (`--bench-*`,
/// `--bench-theme-burst`, the picker/typing scenario sweeps) that may print
/// freely with no row in `EXPECTED` above. A module ENROLLS by living under
/// one of these paths — structurally, not by name — so a brand-new file here,
/// or a new `println!`/`eprintln!` in an existing one, needs no table edit.
/// This is the type/module-boundary check `docs/verification.md` asks for in
/// place of an exact count: the exact-count shape made every bench edit or
/// split a table edit too (see `no_stray_println_outside_the_audited_table`'s
/// own git history — a theme-burst count bump, a picker-sweep carve-out that
/// moved lines between two rows, a new typing-benchmark stage), all churn
/// with nothing to do with the rule itself (benchmark diagnostics stay in
/// benchmark modules). A trailing `/` matches the whole subtree; otherwise
/// the path must match exactly, so this cannot accidentally swallow a
/// same-prefixed sibling file that is not itself a bench module.
const BENCHMARK_MODULE_PATHS: &[&str] = &[
    "bench.rs",
    "app/semantic/bench.rs",
    "render/framebench.rs",
    "render/framebench/",
    "render/perfbench.rs",
    "render/caretbench.rs",
    "render/benchsuite/",
];

/// Whether `path` (already `/`-normalized and relative to `src/`) lives under
/// an enrolled benchmark module. See `BENCHMARK_MODULE_PATHS`.
fn is_benchmark_module(path: &str) -> bool {
    BENCHMARK_MODULE_PATHS
        .iter()
        .any(|entry| match entry.strip_suffix('/') {
            Some(dir) => path == dir || path.starts_with(entry),
            None => path == *entry,
        })
}

/// Drops every benchmark-module entry from a scanned-or-synthetic count map,
/// leaving only the files this module's exact-count table is still
/// responsible for. Shared by the real audit and its own law tests below, so
/// a law test exercises the identical filter the production check runs.
fn strip_benchmark_modules(
    counts: std::collections::BTreeMap<String, usize>,
) -> std::collections::BTreeMap<String, usize> {
    counts
        .into_iter()
        .filter(|(path, _)| !is_benchmark_module(path))
        .collect()
}

/// The pure per-line needle counter: matches `println!(` / `eprintln!(` as a
/// whole macro-call token — trying `eprintln!(` FIRST at each position, so
/// its trailing `println!(` suffix is consumed as part of THAT one match
/// rather than counted a second, phantom time (the naive "just count both
/// substrings separately" trap: `"eprintln!(".contains("println!(")`).
/// Advances by one whole `char` on a non-match, so a non-ASCII line (the
/// `"こんにちは"` test fixture in `app.rs`) never panics on a bad byte offset.
fn needle_count(line: &str) -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < line.len() {
        let rest = &line[i..];
        if rest.starts_with("eprintln!(") {
            n += 1;
            i += "eprintln!(".len();
        } else if rest.starts_with("println!(") {
            n += 1;
            i += "println!(".len();
        } else {
            i += rest.chars().next().map(char::len_utf8).unwrap_or(1);
        }
    }
    n
}

fn scan_file(text: &str) -> usize {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Normal,
        AfterCfgTest,
        InSkippedBlock(i32),
    }
    let mut state = State::Normal;
    let mut n = 0usize;
    for line in text.lines() {
        state = match state {
            State::Normal => {
                let t = line.trim_start();
                if t.starts_with("#[cfg(test)") || t.starts_with("#[cfg(all(test") {
                    State::AfterCfgTest
                } else {
                    n += needle_count(line);
                    State::Normal
                }
            }
            State::AfterCfgTest => {
                let t = line.trim_start();
                if t.starts_with("#[") {
                    State::AfterCfgTest // a stacked attribute; keep waiting
                } else if line.contains('{') {
                    let d = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                    if d <= 0 {
                        State::Normal
                    } else {
                        State::InSkippedBlock(d)
                    }
                } else if line.trim_end().ends_with(';') {
                    State::Normal // a bare `mod tests;` declaration
                } else {
                    State::AfterCfgTest // a multi-line signature; keep waiting
                }
            }
            State::InSkippedBlock(depth) => {
                let d = depth + line.matches('{').count() as i32 - line.matches('}').count() as i32;
                if d <= 0 {
                    State::Normal
                } else {
                    State::InSkippedBlock(d)
                }
            }
        };
    }
    n
}

fn scan_dir(
    base: &std::path::Path,
    dir: &std::path::Path,
    counts: &mut std::collections::BTreeMap<String, usize>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // A `tests/` directory is entirely test fixtures/harness code —
            // its own `eprintln!("skipping …: no wgpu adapter")` guards are
            // never runtime-reachable.
            if path.file_name().and_then(|n| n.to_str()) == Some("tests") {
                continue;
            }
            scan_dir(base, &path, counts);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("println_audit.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let n = scan_file(&text);
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

#[test]
fn no_stray_println_outside_the_audited_table() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    scan_dir(&root, &root, &mut counts);
    // Benchmark modules enroll structurally (BENCHMARK_MODULE_PATHS) and carry
    // no row in EXPECTED — see `structural_benchmark_check_*` below for the
    // mutation proof that this filter neither hides a non-benchmark forbidden
    // case nor demands a table edit for a legitimate new bench file.
    let counts = strip_benchmark_modules(counts);

    let expected: std::collections::BTreeMap<String, usize> =
        EXPECTED.iter().map(|(f, n)| (f.to_string(), *n)).collect();

    assert_eq!(
        counts, expected,
        "a println!/eprintln! call appeared somewhere unaccounted for (a new file, or a \
         changed count in an already-audited one) — give it a fate: route it through the \
         `App::notice` seam (a), silence it (b), enroll it structurally under \
         `BENCHMARK_MODULE_PATHS` if it is genuinely hidden benchmark-harness output (c), \
         or add it to `println_audit::EXPECTED` with a reason (d). See this module's doc \
         comment for the full audit."
    );
}

#[test]
fn expected_table_holds_no_benchmark_module_rows() {
    // The two enrollment mechanisms (the exact-count table and the structural
    // benchmark check) must never overlap: a row here for a path the
    // structural check already owns is dead weight that could silently drift
    // from the real count with nothing to catch it.
    for (path, _) in EXPECTED {
        assert!(
            !is_benchmark_module(path),
            "{path} is enrolled under BENCHMARK_MODULE_PATHS; drop its EXPECTED row instead \
             of keeping both"
        );
    }
}

#[test]
fn is_benchmark_module_sweeps_known_and_hypothetical_paths() {
    for known in [
        "bench.rs",
        "app/semantic/bench.rs",
        "render/framebench.rs",
        "render/framebench/pickersweep.rs",
        "render/perfbench.rs",
        "render/caretbench.rs",
        "render/benchsuite/mod.rs",
        "render/benchsuite/report.rs",
        "render/benchsuite/scenarios/typing.rs",
        // Hypothetical new files under an already-enrolled module: the whole
        // point of a structural check is that these need no registration.
        "render/framebench/newarm.rs",
        "render/benchsuite/scenarios/madeup.rs",
        "render/benchsuite/newfile.rs",
    ] {
        assert!(
            is_benchmark_module(known),
            "{known} should structurally enroll as a benchmark module"
        );
    }
    for not_bench in [
        "app.rs",
        "render.rs",
        "render/rects.rs",
        // A near-miss that shares a prefix but is not itself the named file
        // or under the named directory must NOT match.
        "render/framebenchmark.rs",
        "app/semantic/benchmarks.rs",
        "render/benchsuited/mod.rs",
        // A hypothetical genuinely new, non-benchmark file.
        "render/newfeature.rs",
    ] {
        assert!(
            !is_benchmark_module(not_bench),
            "{not_bench} should NOT enroll as a benchmark module"
        );
    }
}

#[test]
fn structural_benchmark_check_still_catches_forbidden_application_output() {
    // Mutation proof for docs/verification.md's condition: replacing an
    // exact-count row with a structural check must not let a forbidden case
    // — a println!/eprintln! landing OUTSIDE both the audited table and any
    // enrolled benchmark module — through undetected.
    let mut counts: std::collections::BTreeMap<String, usize> =
        EXPECTED.iter().map(|(f, n)| (f.to_string(), *n)).collect();
    counts.insert("app/some_new_unaudited_file.rs".to_string(), 1);

    let stripped = strip_benchmark_modules(counts);
    let expected: std::collections::BTreeMap<String, usize> =
        EXPECTED.iter().map(|(f, n)| (f.to_string(), *n)).collect();

    assert_ne!(
        stripped, expected,
        "a forbidden println!/eprintln! outside every benchmark module and the audited \
         table must still make the real audit fail"
    );
}

#[test]
fn legitimate_benchmark_modules_enroll_without_a_table_edit() {
    // Mutation proof for the companion half of the same condition: a
    // legitimate new file under an already-enrolled benchmark module — or a
    // new println! in an existing one — needs no EXPECTED edit at all.
    let mut counts: std::collections::BTreeMap<String, usize> =
        EXPECTED.iter().map(|(f, n)| (f.to_string(), *n)).collect();
    counts.insert("render/benchsuite/scenarios/newly_added.rs".to_string(), 3);
    counts.insert("render/framebench/newarm.rs".to_string(), 7);
    // Any count, however large, in an already-enrolled benchmark file also
    // needs no tracking — the structural check does not read the number.
    counts.insert("render/benchsuite/mod.rs".to_string(), 999);

    let stripped = strip_benchmark_modules(counts);
    let expected: std::collections::BTreeMap<String, usize> =
        EXPECTED.iter().map(|(f, n)| (f.to_string(), *n)).collect();

    assert_eq!(
        stripped, expected,
        "a new file or a growing count under an enrolled benchmark module must not require \
         an EXPECTED table edit"
    );
}

#[test]
fn needle_count_never_double_counts_eprintln_as_two_hits() {
    assert_eq!(needle_count(r#"eprintln!("x: {e}");"#), 1);
    assert_eq!(needle_count(r#"println!("x");"#), 1);
    assert_eq!(needle_count("no macro here at all"), 0);
    assert_eq!(
        needle_count(r#"println!("a"); eprintln!("b");"#),
        2,
        "one of each on the same line counts as two, not three"
    );
}
