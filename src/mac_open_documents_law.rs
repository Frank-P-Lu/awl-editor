//! src/mac_open_documents_law.rs — THE ONE-OPEN-PATH LAW.
//!
//! `mac_open_documents.rs`'s whole reason to exist is to give the Finder
//! "Open With" door a way into `App` without inventing a second document-open
//! mechanism beside the daemon's: every Finder-handed URL is posted as the
//! daemon's own open-path event, wrapped in the app's event enum, so
//! `App::handle_daemon_event` never has to know which door it arrived
//! through. This law is the sweep that keeps that true: a THIRD AppKit door
//! built the same way (some other `mac_*` file reaching straight into that
//! machinery by directly constructing the wrapped event, instead of routing
//! through `mac_open_documents`'s own `post`/`handle_open`) fails here until
//! it is named in the allowlist below — the same "grow the count
//! consciously" discipline `app/tests/source_audit.rs` uses for `App::new`.
//!
//! NOTE ON THE NEEDLE, mirroring `source_audit.rs`'s own warning: the exact
//! pattern this scan looks for is built at RUNTIME (`needle`, several
//! literals concatenated) rather than spelled out as one contiguous string
//! anywhere in this file's prose — otherwise this very guard's own source
//! text would match itself and inflate its own count. Keep every comment/
//! message below phrased without writing the wrapping variant and the
//! payload variant directly adjacent to each other.
//!
//! Deliberately NOT gated to macOS (unlike the feature it guards): the check
//! itself is a plain text scan over `src/`, so a Linux CI run keeps sweeping
//! it even though the code it's watching for compiles out there entirely.
#![cfg(test)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The literal shape of a DIRECT construction (as opposed to the daemon's own
/// generic wrap-based plumbing, which never spells the wrapping variant and
/// the payload variant next to each other in one file —
/// `crate::daemon::spawn_accept_thread`'s caller in `app.rs` passes the bare
/// wrapping constructor as a function value, and `daemon.rs` itself builds
/// the payload variant with no wrapping prefix at all). Specific enough that
/// it cannot collide with the plain match arm in `app/lifecycle.rs` either —
/// that arm only ever binds a variable, never spells the payload variant
/// alongside the wrapper.
fn needle() -> String {
    [
        "AwlEvent",
        "::",
        "Daemon",
        "(",
        "DaemonEvent",
        "::",
        "OpenPath",
    ]
    .concat()
}

fn scan(base: &Path, dir: &Path, hits: &mut BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let needle = needle();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan(base, &path, hits);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if text.contains(&needle) {
            hits.insert(
                path.strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

/// THE LAW: exactly one file directly constructs the wrapped open-path event
/// (see `needle`) — `mac_open_documents.rs`'s own `post` function.
/// Everything else that wants to open a path through the daemon's event
/// either goes through `post`/`handle_open` (that same file) or through the
/// daemon's own generic wrap-based plumbing (`daemon.rs`), neither of which
/// trips this specific needle.
#[test]
fn open_path_from_appkit_has_exactly_one_direct_construction_site() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = BTreeSet::new();
    scan(&root, &root, &mut hits);
    let expected: BTreeSet<String> = ["mac_open_documents.rs"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        hits, expected,
        "exactly one file may directly construct the daemon's open-path event \
         wrapped in the app's event enum (see this file's `needle`) — a second \
         AppKit-originated door into App's document-open machinery must route \
         through mac_open_documents's own post()/handle_open() instead of \
         building the event a second way, or be named here consciously if it \
         genuinely needs its own site. Found: {hits:?}"
    );
}
