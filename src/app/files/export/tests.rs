//! src/app/files/export/tests.rs — the export DESTINATION laws.
//!
//! Three things are pinned here, in the order the value goes down:
//!
//! 1. **The destination is unchanged by the extraction** — a differential
//!    oracle. The arithmetic `export_target` now owns used to sit inline in the
//!    live effect interpreter; this file writes the OLD expressions out
//!    literally and demands they agree with the new owner over the whole
//!    `Format` roster on both sides of "does the document have a path". A
//!    single-expression refactor asserted to be identical is a refactor nobody
//!    checked; asked as a differential over the axis the caller collapses, it is
//!    a checked one.
//! 2. **A headless `App` writes exactly there, with exactly those bytes** —
//!    tier 2 over the `InMemoryFs` seam (`docs/harness-reach.md`), because the
//!    write is live-`App` work no `--keys` replay performs (`export` classifies
//!    Intercepted).
//! 3. **A headless `App` never reveals** — the live-only Finder handoff is
//!    gated on a real surface, so the hermetic tiers stay hermetic and the
//!    write path is bit-for-bit the live one minus that handoff.

use super::*;
use crate::fs::{FileSystem, InMemoryFs};
use std::sync::Arc;

const DOC: &str = "/w/proj/draft.md";
const BODY: &str = "# Title\n\nSome prose.\n";

/// The full [`crate::export::Format`] roster, enrolled by a NO-WILDCARD match
/// so a fourth export format cannot ride in unswept — adding a variant fails to
/// compile HERE, before it can pass any law below by omission. Derived from the
/// enum itself rather than pinned to the three names a reader happens to know.
fn every_format() -> Vec<crate::export::Format> {
    use crate::export::Format;
    let all = vec![Format::Docx, Format::Html, Format::Pdf];
    for format in &all {
        match format {
            Format::Docx | Format::Html | Format::Pdf => {}
        }
    }
    all
}

/// The destination arithmetic as it read BEFORE it had an owner — copied out of
/// the live effect interpreter it used to live in, not re-derived from
/// [`export_target`]. This is the whole point of the law below: two independent
/// spellings of the same rule, required to agree.
fn destination_the_old_way(
    doc_path: Option<&std::path::Path>,
    root: &std::path::Path,
    stem: &str,
    format: crate::export::Format,
) -> (std::path::PathBuf, bool) {
    match doc_path {
        Some(p) => (p.with_extension(format.ext()), false),
        None => (root.join(format!("{stem}.{}", format.ext())), true),
    }
}

/// MUTATION TARGET: change either side of [`export_target`] — the extension, the
/// folder it joins onto, or which arm sets `show_full` — and this fails by name,
/// naming the format and which side of the path axis disagreed.
#[test]
fn the_export_destination_is_unchanged_by_its_extraction_on_both_sides_of_the_path_axis() {
    let root = std::path::Path::new("/w/proj");
    let saved = std::path::PathBuf::from(DOC);
    // BOTH sides of the condition the caller collapses: a document with a path
    // of its own, and one with none. Asking only the saved side would leave the
    // arm that invents a folder — the one a writer cannot predict — unswept.
    let cases: [(Option<&std::path::Path>, &str); 2] =
        [(Some(saved.as_path()), "saved"), (None, "path-less")];
    for format in every_format() {
        for (doc_path, label) in cases {
            let want = destination_the_old_way(doc_path, root, "scratch", format);
            let got = export_target(doc_path, root, "scratch", format);
            assert_eq!(
                (got.path.clone(), got.show_full),
                want,
                "{label} buffer, format {}: the destination owner disagrees with the \
                 arithmetic it replaced",
                format.ext(),
            );
        }
    }
}

/// A saved document exports as its own SIBLING, and the notice names the bare
/// filename because the folder is one the writer already chose.
#[test]
fn a_saved_document_exports_beside_itself_and_the_notice_needs_no_folder() {
    let saved = std::path::PathBuf::from(DOC);
    for format in every_format() {
        let target = export_target(
            Some(saved.as_path()),
            std::path::Path::new("/w/other"),
            "ignored",
            format,
        );
        assert_eq!(
            target.path,
            std::path::PathBuf::from(format!("/w/proj/draft.{}", format.ext())),
            "the sibling never moves to the active folder",
        );
        assert!(
            !target.show_full,
            "{}: a sibling's folder is already on screen",
            format.ext()
        );
    }
}

/// A buffer with NO path lands in the active folder under the name it already
/// calls itself — and the notice says the whole path, because nothing else
/// tells the writer where it went. That second half is the anti-surprise
/// mechanism, so it is asserted rather than assumed.
#[test]
fn a_path_less_buffer_exports_into_the_active_folder_and_the_notice_says_where() {
    for format in every_format() {
        let target = export_target(None, std::path::Path::new("/w/proj"), "quick-note", format);
        assert_eq!(
            target.path,
            std::path::PathBuf::from(format!("/w/proj/quick-note.{}", format.ext())),
        );
        assert!(
            target.show_full,
            "{}: a destination the writer never named must be spoken in full",
            format.ext()
        );
    }
}

/// TIER 2 (`docs/harness-reach.md`): a real headless `App` drives the real
/// effect body. Asserts the file appears at the destination the pure owner
/// names, carrying exactly the bytes the pure emitter produces — so the write
/// path adds no transformation of its own — and that the toast names it.
///
/// MUTATION TARGET: point `write_export` at any other path, or hand
/// `write_atomic` anything but the emitted bytes, and this fails by name.
#[test]
fn a_headless_app_writes_the_export_at_the_destination_with_the_emitted_bytes() {
    let _g = crate::testlock::serial();
    let doc = std::path::PathBuf::from(DOC);
    let mem = InMemoryFs::new().with_file(doc.clone(), BODY);
    let fake = Arc::new(mem.clone());
    crate::fs::with_fs(fake, || {
        for format in every_format() {
            let mut app = App::new_hermetic(
                Some(doc.clone()),
                std::path::PathBuf::from("/w/proj"),
                Config::empty(),
            );
            let want_path = export_target(
                Some(doc.as_path()),
                std::path::Path::new("/w/proj"),
                "draft",
                format,
            )
            .path;
            let want_bytes = crate::export::to_bytes(
                &app.document.buffer().text(),
                format,
                &crate::export::FsImages {
                    doc_dir: Some(std::path::PathBuf::from("/w/proj")),
                },
            );
            app.export_document(format);
            let written = mem
                .read(&want_path)
                .unwrap_or_else(|e| panic!("{}: nothing at {want_path:?}: {e}", format.ext()));
            assert_eq!(
                written,
                want_bytes,
                "{}: the write path must add no transformation of its own",
                format.ext()
            );
            assert!(
                !want_bytes.is_empty(),
                "{}: PRESENCE — an emitter returning nothing would satisfy the \
                 equality above while exporting an empty file",
                format.ext()
            );
            assert_eq!(
                app.frame.notice().text(),
                Some(format!("exported draft.{}", format.ext()).as_str()),
                "{}: the toast names the file",
                format.ext()
            );
        }
    });
}

/// The live-only Finder handoff is gated on a real SURFACE, not on `cfg`, so
/// every hermetic tier — `--screenshot-app`, every test in this suite — takes
/// the identical write path and reveals nothing.
///
/// MUTATION TARGET: drop the `frame.gpu().is_none()` early return in
/// `reveal_export` and this fails by name. It is also the one law that would
/// notice the regression behaviourally: without the gate, running the suite on
/// macOS opens a Finder window per export test.
#[test]
fn a_headless_app_never_reveals_the_export_in_the_platform_file_viewer() {
    let _g = crate::testlock::serial();
    let doc = std::path::PathBuf::from(DOC);
    let fake = Arc::new(InMemoryFs::new().with_file(doc.clone(), BODY));
    crate::fs::with_fs(fake, || {
        let app = App::new_hermetic(
            Some(doc.clone()),
            std::path::PathBuf::from("/w/proj"),
            Config::empty(),
        );
        assert!(
            app.frame.gpu().is_none(),
            "PRESENCE: a hermetic App has no surface — without this the gate below \
             is untested rather than satisfied",
        );
        assert!(
            !app.reveal_export(&doc),
            "a surface-less App must not reach the platform file viewer",
        );
    });
}
