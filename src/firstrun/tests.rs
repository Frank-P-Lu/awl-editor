//! Laws for the first-run document.
//!
//! The pure branch predicate is swept over its WHOLE axis rather than over the
//! cases the author happened to imagine (CLAUDE.md's law rule): four booleans,
//! sixteen cells, every one asserted.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::*;
use crate::fs::{FileSystem, InMemoryFs};

fn mem() -> Arc<InMemoryFs> {
    Arc::new(InMemoryFs::new())
}

/// Run `body` with `fs` installed as the process backend. `fs::with_fs` takes
/// the one process-wide serialization guard for us (the fs law).
fn on<T>(fs: Arc<InMemoryFs>, body: impl FnOnce() -> T) -> T {
    let dynfs: Arc<dyn FileSystem> = fs;
    crate::fs::with_fs(dynfs, body)
}

fn folder() -> PathBuf {
    PathBuf::from("/home/u/notes")
}

fn some(p: &str) -> Option<PathBuf> {
    Some(PathBuf::from(p))
}

// ── The pure predicate, swept exhaustively ──────────────────────────────────

/// THE BRANCH LAW: a first run is a launch that asked for nothing and had
/// nothing given back. Every one of the sixteen `root × file × remembered ×
/// marked` cells is asserted, so a later edit that widens the predicate by one
/// term fails here rather than shipping a welcome document over somebody's
/// afternoon's work.
#[test]
fn is_first_run_is_true_for_exactly_one_of_the_sixteen_launch_shapes() {
    let remembered_path = PathBuf::from("/home/u/work");
    let mut trues = 0usize;
    for root in [None, some("/r")] {
        for file in [None, some("/f.md")] {
            for remembered in [None, Some(remembered_path.as_path())] {
                for marked in [false, true] {
                    let got = is_first_run(&root, &file, remembered, marked);
                    let want = root.is_none() && file.is_none() && remembered.is_none() && !marked;
                    assert_eq!(
                        got, want,
                        "root={root:?} file={file:?} remembered={remembered:?} marked={marked}"
                    );
                    trues += usize::from(got);
                }
            }
        }
    }
    assert_eq!(trues, 1, "exactly one launch shape is a first run");
}

// ── Seeding ────────────────────────────────────────────────────────────────

#[test]
fn a_first_run_seeds_the_welcome_document_and_opens_it() {
    let fs = mem();
    let opened = on(fs.clone(), || {
        resolve_first_run_document(
            None,
            &None,
            None,
            &folder(),
            Convention::Mac,
            Platform::Native,
        )
    });
    assert_eq!(opened, Some(folder().join(WELCOME_FILE)));
    let text = on(fs.clone(), || {
        crate::fs::active()
            .read_to_string(&folder().join(WELCOME_FILE))
            .expect("the welcome document was written into the active folder")
    });
    assert!(
        !text.is_empty() && !text.contains("{{key:") && !text.contains("{{cmd:"),
        "the seeded bytes are the RENDERED document, not the token source"
    );
    assert!(on(fs, marked), "a successful first run marks the profile");
}

/// The document is seeded with the chords of the machine that will read it —
/// the same reason the web build renders its own seeds at seed time.
#[test]
fn the_seeded_document_carries_this_conventions_own_chords() {
    let mac = mem();
    let mac_text = on(mac, || {
        let p = folder().join(WELCOME_FILE);
        seed(&p, Convention::Mac, Platform::Native).unwrap();
        crate::fs::active().read_to_string(&p).unwrap()
    });
    let linux = mem();
    let linux_text = on(linux, || {
        let p = folder().join(WELCOME_FILE);
        seed(&p, Convention::Linux, Platform::Native).unwrap();
        crate::fs::active().read_to_string(&p).unwrap()
    });
    assert_ne!(
        mac_text, linux_text,
        "the two conventions render differently"
    );
    assert!(mac_text.contains('\u{2318}'), "mac gets ⌘ glyphs");
    assert!(
        !linux_text.contains('\u{2318}'),
        "linux never sees a ⌘ glyph"
    );
    assert!(linux_text.contains("Ctrl+"), "linux gets word-form chords");
}

/// WRITE-IF-ABSENT: the returning user's own bytes win, always. This is the
/// law that makes "edit it, or replace it entirely" safe.
#[test]
fn seeding_never_clobbers_a_welcome_the_user_already_has() {
    let mine = "# my own notes, thanks\n";
    let fs = Arc::new(InMemoryFs::new().with_file("/home/u/notes/welcome.md", mine));
    let opened = on(fs.clone(), || {
        resolve_first_run_document(
            None,
            &None,
            None,
            &folder(),
            Convention::Mac,
            Platform::Native,
        )
    });
    assert_eq!(opened, Some(folder().join(WELCOME_FILE)));
    let text = on(fs, || {
        crate::fs::active()
            .read_to_string(&folder().join(WELCOME_FILE))
            .unwrap()
    });
    assert_eq!(text, mine, "existing bytes are untouched");
}

// ── The one-shot marker: no state leaks into a later session ───────────────

/// THE NO-LEAK LAW: a second launch of the same profile, with the same empty
/// inputs, opens nothing at all — awl hands the user back the ordinary scratch
/// buffer, and the welcome document is not re-seeded over whatever they left in
/// its place. This is the case a bare "is `welcome.md` absent?" test would miss:
/// the user DELETED it on purpose.
#[test]
fn a_second_launch_of_the_same_profile_opens_nothing() {
    let fs = mem();
    let first = on(fs.clone(), || {
        resolve_first_run_document(
            None,
            &None,
            None,
            &folder(),
            Convention::Mac,
            Platform::Native,
        )
    });
    assert!(first.is_some(), "control: the first launch seeded");

    // The user deletes the welcome document and quits.
    on(fs.clone(), || {
        crate::fs::active()
            .remove_file(&folder().join(WELCOME_FILE))
            .unwrap()
    });

    let second = on(fs.clone(), || {
        resolve_first_run_document(
            None,
            &None,
            None,
            &folder(),
            Convention::Mac,
            Platform::Native,
        )
    });
    assert_eq!(second, None, "the welcome never comes back on its own");
    assert!(
        on(fs, || !crate::fs::active()
            .exists(&folder().join(WELCOME_FILE))),
        "and it is not silently re-written either"
    );
}

/// Deleting the marker is the supported way to ask for the welcome again — the
/// marker's own text says so, so the behaviour is law-tested rather than
/// incidental.
#[test]
fn deleting_the_marker_asks_for_the_welcome_again() {
    let fs = mem();
    on(fs.clone(), || {
        resolve_first_run_document(
            None,
            &None,
            None,
            &folder(),
            Convention::Mac,
            Platform::Native,
        )
    });
    on(fs.clone(), || {
        crate::fs::active().remove_file(&marker_path()).unwrap();
        crate::fs::active()
            .remove_file(&folder().join(WELCOME_FILE))
            .unwrap();
    });
    let again = on(fs, || {
        resolve_first_run_document(
            None,
            &None,
            None,
            &folder(),
            Convention::Mac,
            Platform::Native,
        )
    });
    assert_eq!(again, Some(folder().join(WELCOME_FILE)));
}

#[test]
fn the_marker_explains_itself_to_whoever_finds_it() {
    let fs = mem();
    let text = on(fs, || {
        mark();
        crate::fs::active().read_to_string(&marker_path()).unwrap()
    });
    assert!(text.contains("Delete this file"), "{text:?}");
}

// ── Every launch shape that is NOT a first run ─────────────────────────────

/// A returning user resumes their own work: a remembered session folder, an
/// explicit file argument, an explicit `--root`, and an already-marked profile
/// each pass the launch through untouched and seed NOTHING. The assertion is
/// on the filesystem too, not only the return value — the failure that would
/// actually hurt is a stray `welcome.md` appearing in somebody's project.
#[test]
fn no_other_launch_shape_seeds_or_diverts() {
    let remembered = PathBuf::from("/home/u/work");
    /// `(label, file argument, --root, remembered folder, already marked)`.
    type Shape<'a> = (
        &'a str,
        Option<PathBuf>,
        Option<PathBuf>,
        Option<&'a Path>,
        bool,
    );
    let cases: &[Shape] = &[
        (
            "remembered session",
            None,
            None,
            Some(remembered.as_path()),
            false,
        ),
        (
            "file argument",
            some("/home/u/work/draft.md"),
            None,
            None,
            false,
        ),
        ("--root", None, some("/home/u/work"), None, false),
        ("already marked", None, None, None, true),
    ];
    for (label, file, root, remembered, premarked) in cases {
        let fs = mem();
        let got = on(fs.clone(), || {
            if *premarked {
                mark();
            }
            resolve_first_run_document(
                file.clone(),
                root,
                *remembered,
                &folder(),
                Convention::Mac,
                Platform::Native,
            )
        });
        assert_eq!(&got, file, "{label}: the launch's own file is unchanged");
        assert!(
            on(fs, || !crate::fs::active()
                .exists(&folder().join(WELCOME_FILE))),
            "{label}: nothing was seeded"
        );
    }
}

/// Print the EXACT bytes a first launch seeds, for the convention this pass is
/// running under — the same `render_key_tokens` call [`seed`] makes, so a
/// capture of the printed file is a capture of the real first-run document
/// rather than of `samples/welcome.md`'s unrendered token source. Ignored by
/// default; the `print_generated_keys_reference` precedent.
///
/// ```text
/// cargo test --bin awl firstrun::tests::print_seeded_welcome -- --ignored --nocapture
/// ```
#[test]
#[ignore]
fn print_seeded_welcome() {
    print!(
        "{}",
        crate::keytoken::render_key_tokens(
            crate::embedded_docs::WELCOME_MD,
            Convention::current(),
            Platform::Native,
        )
    );
}

// ── Structural: the door has one production call site ──────────────────────

/// THE CAPTURE GATE, structurally. Every headless capture mode resolves its
/// root through the explicit-only `run::resolve_root` and must never reach this
/// module — a `--screenshot` run is not a desktop launch, and its output must
/// not depend on whether the developer running it has ever opened awl. The only
/// way to hold that is to hold the call-site count: one, in the windowed arm.
#[test]
fn the_first_run_door_has_exactly_one_production_call_site() {
    // Built at runtime so this test's own source text cannot match the needle.
    let needle = format!("{}{}", "resolve_first_run_document", "(");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits: Vec<String> = Vec::new();
    let mut stack = vec![src.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src/ is readable").flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            // This module owns the definition and its own laws; both are
            // deliberately outside the count.
            let rel = path
                .strip_prefix(&src)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if rel == "firstrun.rs" || rel.starts_with("firstrun/") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            for _ in 0..text.matches(&needle).count() {
                hits.push(rel.clone());
            }
        }
    }
    hits.sort();
    assert_eq!(
        hits,
        vec!["main/run/location.rs".to_string()],
        "the first-run document has exactly ONE production call site — `launch_windowed`, \
         the windowed launch door, beside the folder half of the same law. A second one \
         (especially on a capture path) breaks the capture gate this module's header states."
    );
}
