//! THE ACTIVE-WORLD GLOBAL'S HYGIENE LAW (item 94, third repair pass).
//!
//! The active theme is a process-GLOBAL index (`theme::ACTIVE`). `cargo test`
//! runs thousands of tests in one process, so a test that swaps worlds and never
//! swaps back does not merely dirty itself — it hands its world to whatever runs
//! NEXT. That is not a hypothetical: `capture::tests::pickers_faceted` ended on a
//! swap to Tawny and never restored, and roughly twenty other tests left the
//! global wherever their last swap put it. The visible symptom was a render law
//! (`render::tests::range_rail`'s thumb law) that passed or failed purely on test
//! ORDER — green under the usual parallel shuffle, deterministically RED when the
//! faceted-picker test happened to run just before it.
//!
//! The cure is STRUCTURAL, not per-test discipline: `crate::testlock::serial`'s
//! outermost guard now holds a [`crate::theme::WorldPin`], which snapshots the
//! active index on acquire and stores it back on drop (before the mutex is
//! released, so no other thread can see the dirt). Every test already takes that
//! guard — the standing test-global locking rule — so every test is world-clean
//! on exit whether or not its author thought about the world at all.
//!
//! That leaves exactly ONE way to dodge the restore: mutate the global from test
//! code that never takes the guard. [`every_test_side_world_swap_happens_under_the_serial_guard`]
//! is the sweep that closes it. It walks `src/` from the filesystem — there is NO
//! roster to add a new file to and NO audited-count table to bless an exception
//! with, so a test file written tomorrow is swept the moment it exists — finds
//! every function in TEST code (a `tests/` directory, a `tests.rs`, or a
//! `#[cfg(test)]`-gated item) that moves the world global, and requires that same
//! function to acquire `crate::testlock::serial()`. The guard is REENTRANT, so a
//! helper the test already called under the guard may take it again for free;
//! there is no cost to obeying the law and no legal way around it.
//!
//! `src/` is the whole territory that needs sweeping: awl is a BIN crate, so the
//! `tests/` integration binaries cannot reach `theme::ACTIVE` (or `testlock`) at
//! all — each is its own process with its own copy of nothing.
//!
//! The scanner is deliberately the same shape as [`crate::println_audit`]'s (the
//! precedent for "walk `src/`, brace-balance the `#[cfg(test)]` regions") with the
//! cfg(test) polarity INVERTED — that module audits runtime code and skips tests;
//! this one audits tests and skips runtime code (a live `--theme NAME` pin, the
//! theme picker's own preview, the bench harnesses' world sweeps are all
//! deliberate, guardless, and correct).

/// The world-global MUTATORS as they are spelled OUTSIDE `src/theme/`: qualified,
/// because the crate has a second, unrelated `set_active` (the fs BACKEND's) and
/// plenty of unrelated `cycle`s. `set_active_by_name` is the world's alone, so it
/// counts bare anywhere. Assembled from fragments so that this very file — which
/// the sweep walks like any other — cannot match its own needles (`durable.rs`'s
/// precedent for a self-scanning module).
const MUTATORS: [&str; 3] = [
    concat!("theme::", "set_active("),
    concat!("set_", "active_by_name("),
    concat!("theme::", "cycle("),
];

/// The extra, UNQUALIFIED spellings that count inside `src/theme/` itself, where
/// `use super::*` puts the owner's own functions in scope bare.
const MUTATORS_IN_THEME: [&str; 2] = [concat!("set_", "active("), concat!("cycle", "(")];

/// The acquisition the sweep demands beside a mutator — bare, so both
/// `crate::testlock::serial()` at a call site and the plain `serial()` that
/// `testlock`'s own tests use satisfy it. There is exactly one `serial()` in the
/// crate. Fragmented for the same reason as [`MUTATORS`].
const ACQUIRE: &str = concat!("serial", "()");

/// Strip a line down to CODE: no `//` comment tail, no double-quoted string
/// contents. Without this, a law that merely NAMES a mutator in a doc comment or
/// asserts on one inside a string literal (`theme_caps_law`'s own scanner test
/// does exactly that) would read as a call site.
fn code_only(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_str = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if in_str => {
                chars.next(); // the escaped char is never a quote/terminator
            }
            '"' => in_str = !in_str,
            '/' if !in_str && chars.peek() == Some(&'/') => break,
            _ if !in_str => out.push(c),
            _ => {}
        }
    }
    out
}

/// Does this code text call a world MUTATOR? A `fn set_active…` DECLARATION does
/// not count (the one owner declares them; the sweep is about callers).
/// `in_theme_module` adds the unqualified spellings — see [`MUTATORS_IN_THEME`].
fn mutates_world(code: &str, in_theme_module: bool) -> bool {
    let needles = MUTATORS
        .iter()
        .chain(MUTATORS_IN_THEME.iter().take(if in_theme_module { 2 } else { 0 }));
    needles.into_iter().any(|needle| {
        code.match_indices(needle).any(|(i, _)| !code[..i].trim_end().ends_with("fn"))
    })
}

/// Per-line "is this TEST code?" flags. A file that IS test code (under a
/// `tests/` directory, or literally named `tests.rs`) is test code throughout;
/// anywhere else, only `#[cfg(test)]`-gated items count — brace-balanced,
/// mirroring `println_audit::scan_file`'s state machine with the polarity
/// inverted.
fn test_lines(text: &str, whole_file_is_test: bool) -> Vec<bool> {
    if whole_file_is_test {
        return text.lines().map(|_| true).collect();
    }
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Normal,
        AfterCfgTest,
        InTestBlock(i32),
    }
    let mut state = State::Normal;
    let mut flags = Vec::with_capacity(text.lines().count());
    for line in text.lines() {
        let (flag, next) = match state {
            State::Normal => {
                let t = line.trim_start();
                if t.starts_with("#[cfg(test)") || t.starts_with("#[cfg(all(test") {
                    (false, State::AfterCfgTest)
                } else {
                    (false, State::Normal)
                }
            }
            State::AfterCfgTest => {
                let t = line.trim_start();
                if t.starts_with("#[") {
                    (false, State::AfterCfgTest) // a stacked attribute; keep waiting
                } else if line.contains('{') {
                    let d = line.matches('{').count() as i32 - line.matches('}').count() as i32;
                    if d <= 0 {
                        (true, State::Normal)
                    } else {
                        (true, State::InTestBlock(d))
                    }
                } else if line.trim_end().ends_with(';') {
                    (false, State::Normal) // a bare `mod tests;` declaration
                } else {
                    (false, State::AfterCfgTest) // a multi-line signature
                }
            }
            State::InTestBlock(depth) => {
                let d = depth + line.matches('{').count() as i32 - line.matches('}').count() as i32;
                if d <= 0 {
                    (true, State::Normal)
                } else {
                    (true, State::InTestBlock(d))
                }
            }
        };
        flags.push(flag);
        state = next;
    }
    flags
}

/// Every function in `text` as `(name, 1-based line of its body's opening brace,
/// body text)` — brace-matched, so a nested fn or a closure's braces never end
/// the enclosing body early.
fn fn_bodies(text: &str) -> Vec<(String, usize, String)> {
    let bytes: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    while i < bytes.len() {
        if bytes[i] == '\n' {
            line += 1;
        }
        let is_fn_kw = bytes[i] == 'f'
            && bytes.get(i + 1) == Some(&'n')
            && bytes.get(i + 2).is_some_and(|c| c.is_whitespace())
            && (i == 0 || !bytes[i - 1].is_alphanumeric() && bytes[i - 1] != '_');
        if !is_fn_kw {
            i += 1;
            continue;
        }
        let mut j = i + 3;
        while j < bytes.len() && bytes[j].is_whitespace() {
            j += 1;
        }
        let name_start = j;
        while j < bytes.len() && (bytes[j].is_alphanumeric() || bytes[j] == '_') {
            j += 1;
        }
        let name: String = bytes[name_start..j].iter().collect();
        // The body's opening brace: the first `{` after the signature. A trait
        // method declaration (`fn f(&self);`) has none before its `;`.
        let mut k = j;
        let mut newlines = 0usize;
        let mut open = None;
        while k < bytes.len() {
            match bytes[k] {
                '\n' => newlines += 1,
                '{' => {
                    open = Some(k);
                    break;
                }
                ';' => break,
                _ => {}
            }
            k += 1;
        }
        let Some(open) = open else {
            i += 1;
            continue;
        };
        let mut depth = 0i32;
        let mut end = open;
        while end < bytes.len() {
            match bytes[end] {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            end += 1;
        }
        let body: String = bytes[open..=end.min(bytes.len() - 1)].iter().collect();
        out.push((name, line + newlines, body));
        i += 1;
    }
    out
}

/// Walk `dir`, collecting `(relative path, fn name, line)` for every test-code
/// function that moves the world global without acquiring the serial guard.
fn scan_dir(
    base: &std::path::Path,
    dir: &std::path::Path,
    out: &mut Vec<(String, String, usize)>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<std::path::PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            scan_dir(base, &path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        let whole_file_is_test = rel.contains("tests/") || rel.ends_with("tests.rs");
        let in_theme_module = rel.starts_with("theme/");
        let flags = test_lines(&text, whole_file_is_test);
        for (name, line, body) in fn_bodies(&text) {
            if !flags.get(line - 1).copied().unwrap_or(false) {
                continue; // runtime code: a deliberate, guardless world pin
            }
            let code: String =
                body.lines().map(code_only).collect::<Vec<_>>().join("\n");
            if mutates_world(&code, in_theme_module) && !code.contains(ACQUIRE) {
                out.push((rel.clone(), name, line));
            }
        }
    }
}

/// THE SWEEP: no test-code function may move the world global outside the one
/// restore owner's window. Enumerated from the FILESYSTEM — no roster, no
/// exception table — so a test file that does not exist yet is already covered.
#[test]
fn every_test_side_world_swap_happens_under_the_serial_guard() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut viol = Vec::new();
    scan_dir(&root, &root, &mut viol);
    assert!(
        viol.is_empty(),
        "these test-side functions move the ACTIVE-WORLD global without holding \
         `crate::testlock::serial()`, so nothing restores it and the world leaks into \
         whatever test runs next (item 94's order-dependent `range_rail` failure). \
         Take the guard — it is reentrant, so a helper called from a test that already \
         holds it costs nothing: {viol:#?}"
    );
}

/// The sweep must be able to SEE a violation — proved on synthetic sources rather
/// than by breaking the crate, and on both shapes of test code (a `tests/` file
/// and a `#[cfg(test)]` island in a runtime file).
#[test]
fn the_sweep_catches_an_unguarded_swap_and_clears_a_guarded_one() {
    let dirty = "fn helper(world: &str) {\n    crate::theme::set_active_by_name(world);\n}\n";
    let clean = "fn helper(world: &str) {\n    let _g = crate::testlock::serial();\n    \
                 crate::theme::set_active_by_name(world);\n}\n";
    for whole_file_is_test in [true, false] {
        let wrap = |src: &str| {
            if whole_file_is_test {
                src.to_string()
            } else {
                format!("#[cfg(test)]\nmod tests {{\n{src}}}\n")
            }
        };
        let judge = |src: &str| -> bool {
            let text = wrap(src);
            let flags = test_lines(&text, whole_file_is_test);
            fn_bodies(&text).into_iter().any(|(_, line, body)| {
                let code: String = body.lines().map(code_only).collect::<Vec<_>>().join("\n");
                flags.get(line - 1).copied().unwrap_or(false)
                    && mutates_world(&code, false)
                    && !code.contains(ACQUIRE)
            })
        };
        assert!(judge(dirty), "an unguarded swap must be caught (test file: {whole_file_is_test})");
        assert!(!judge(clean), "a guarded swap must pass (test file: {whole_file_is_test})");
    }
    // RUNTIME code is not swept: the same unguarded swap outside a test region is
    // a deliberate live pin (`--theme NAME`, the picker preview, the benches).
    let flags = test_lines(dirty, false);
    assert!(flags.iter().all(|f| !f), "a runtime file's lines are not test code");
}

/// The line reducer: prose and string literals that merely NAME a mutator are not
/// call sites, and a real call keeps its needle.
#[test]
fn code_only_drops_comments_and_string_contents_but_keeps_calls() {
    assert!(!mutates_world(
        &code_only(
            r#"    assert!(line_violates("theme::set_active_by_name(\"Wagtail\")").is_some());"#
        ),
        false
    ));
    assert!(!mutates_world(&code_only("    // theme::set_active(0) used to live here"), false));
    assert!(mutates_world(&code_only("    crate::theme::set_active(0); // back to Tawny"), false));
    assert!(mutates_world(&code_only("    set_active_by_name(\"Bilby\").unwrap();"), false));
    assert!(mutates_world(&code_only("    crate::theme::cycle(1);"), false));
    // A DECLARATION of the owner is not a call site.
    assert!(!mutates_world(&code_only("pub fn set_active(index: usize) -> Theme {"), true));
    // THE FS BACKEND'S OWN `set_active` is a different global with its own guard:
    // bare spellings count only inside `src/theme/`, where `use super::*` makes
    // the world's own setters reachable unqualified.
    assert!(!mutates_world(&code_only("    set_active(self.prev.clone());"), false));
    assert!(mutates_world(&code_only("    set_active(DEFAULT_THEME);"), true));
}

/// The body finder: nested braces (a closure, a nested fn, a match) must not end
/// a body early, or a mutation late in a long test would be scanned as if it sat
/// outside any function.
#[test]
fn fn_bodies_are_brace_matched_through_nested_blocks() {
    let src = "fn outer() {\n    let f = || { 1 };\n    if true { let _ = 2; }\n    \
               crate::theme::set_active(3);\n}\nfn after() {\n}\n";
    let bodies = fn_bodies(src);
    let outer = bodies.iter().find(|(n, _, _)| n == "outer").expect("outer is found");
    assert!(mutates_world(&outer.2, false), "the body reaches past the nested blocks");
    assert!(!outer.2.contains("fn after"), "and stops at its own closing brace");
    assert_eq!(bodies.len(), 2, "both top-level fns are found: {bodies:?}");
}
