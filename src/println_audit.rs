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
    // `--bench-a11y`'s report table — hidden CLI performance harness output,
    // like the `render/*bench.rs` entries below.
    ("app/semantic/bench.rs", 7),
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
    // in `autosave.rs`; the dictionary switch + personal-dictionary append
    // in `dictionary.rs`. Credits opens as a summoned read-only viewer,
    // never a buffer, so it reaches no write path and adds no line here.
    ("app/files/open.rs", 3),
    ("app/files/settings.rs", 2),
    ("app/files/rebind.rs", 2),
    ("app/files/autosave.rs", 2),
    ("app/files/dictionary.rs", 2),
    // Switching the session's spell checker is owned with that checker; its
    // rare load failure remains the same best-effort diagnostic class.
    ("app/document/cache.rs", 1),
    // GPU/render-pipeline errors (`prepare`/`render`) retain a stderr
    // diagnostic while App-owned recovery also paints the calm notice.
    ("app/gpu.rs", 2),
    ("app/window.rs", 1),
    ("app/session.rs", 1),
    // THE LOCAL USAGE LEDGER: ONE `{what} save failed: {e}` stderr line, in the
    // `Dirtying::flush` door both records share (a failed atomic write of
    // `stats.toml` / `streaks.toml` must never disrupt the editor — it warns
    // and moves on). This used to be two identical lines, one per record, in
    // `app/stats.rs` and `app/streaks.rs`; merging the flush merged them.
    ("app/usage.rs", 1),
    ("bench.rs", 4),
    ("buffers.rs", 1),
    // Headless capture harness diagnostics ("spell-check disabled for
    // capture: …") — CLI/test-harness output, not live-app chatter.
    ("capture/policy.rs", 1),
    ("capture/oracle.rs", 1),
    ("config/model.rs", 1),
    ("keymap/state.rs", 4),
    ("main.rs", 2),
    // `--help`'s big usage dump, plus `--list-worlds`: a
    // machine-readable roster dump for `scripts/capture-worlds.sh` and any
    // other script that wants the world list without parsing --help. Plus
    // `--pack-icns`'s two lines: the per-world byte table and the
    // summary the icon export prints as its deliverable receipt. Plus
    // `--export-linux-icon`'s own one-line deliverable receipt, same shape
    // as `--pack-icns`'s summary line. All five are fate (c) — genuine
    // CLI/diagnostic stdout, not app-runtime chatter.
    ("main/args.rs", 5),
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
    // section blocks — two more lines of the same mechanism, same fate.
    ("reference/law/mod.rs", 6),
    // `AWL_FONT` + `AWL_CHROME_FACE_FILE` dev-only env var override
    // diagnostics (the second is the Firetail-showcase round's audition-font
    // loader: a missing/unreadable candidate file prints a note and is
    // skipped — the same advisory class as `AWL_FONT`'s fallback note).
    ("render.rs", 2),
    // `read_forced_knob`'s unrecognized-value warning (moved here with the
    // `AWL_*_FORCE` knobs it serves).
    ("render/overrides/parsers.rs", 1),
    ("render/framebench.rs", 34),
    // The theme-burst profiler's PICKER SWEEP, carved out of `framebench.rs` when
    // that file reached its frozen size baseline. Same fate as its parent: a
    // hidden bench flag printing its own table to stdout.
    ("render/framebench/pickersweep.rs", 9),
    ("render/perfbench.rs", 8),
    ("render/caretbench.rs", 6),
    ("render/benchsuite/mod.rs", 12),
    ("render/benchsuite/report.rs", 9),
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

    let expected: std::collections::BTreeMap<String, usize> =
        EXPECTED.iter().map(|(f, n)| (f.to_string(), *n)).collect();

    assert_eq!(
        counts, expected,
        "a println!/eprintln! call appeared somewhere unaccounted for (a new file, or a \
         changed count in an already-audited one) — give it a fate: route it through the \
         `App::notice` seam (a), silence it (b), or add it to `println_audit::EXPECTED` \
         with a reason (c). See this module's doc comment for the full audit."
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
