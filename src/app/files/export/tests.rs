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
            let got = export_target(doc_path, root, "scratch", format, None);
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
            None,
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
        let target = export_target(
            None,
            std::path::Path::new("/w/proj"),
            "quick-note",
            format,
            None,
        );
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
                None,
            )
            .path;
            let want_bytes = crate::export::to_bytes(
                &app.document.buffer().text(),
                format,
                &crate::export::FsImages {
                    doc_dir: Some(std::path::PathBuf::from("/w/proj")),
                },
            );
            app.export_document(format, None);
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
/// `reveal_path` and this fails by name. It is also the one law that would
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
            !app.reveal_path(&doc),
            "a surface-less App must not reach the platform file viewer",
        );
    });
}

/// TIER 2 (`docs/harness-reach.md`): a copy is a disk-byte snapshot, never a
/// document transition. The source is deliberately dirty CRLF text with a
/// selection, scroll position, undo group and working-set key, so a new buffer,
/// path adoption, save bookkeeping, or even a newline-normalising write fails
/// a concrete source-state assertion rather than a vague "unchanged" claim.
///
/// MUTATION TARGET: replace `save_copy_to` with `load_path(destination)`, call
/// `Buffer::save`, or write `text().as_bytes()` instead of `disk_bytes()`; this
/// fails by name on identity/state or destination bytes respectively.
#[test]
fn save_a_copy_writes_disk_bytes_without_changing_the_source_document() {
    let _g = crate::testlock::serial();
    let source = std::path::PathBuf::from("/w/proj/source.md");
    let destination = std::path::PathBuf::from("/w/proj/copies/snapshot.md");
    let mem = InMemoryFs::new()
        .with_dir("/w/proj/copies")
        .with_file(&source, "one\r\ntwo\r\n");
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut app = App::new(
            Some(source.clone()),
            std::path::PathBuf::from("/w/proj"),
            None,
            None,
            Config {
                session_restore: Some(false),
                reduce_motion: Some(false),
                ..Config::empty()
            },
        );
        app.document
            .action_buffer_mut()
            .expect("source document is active")
            .insert_text("draft");
        app.document
            .action_buffer_mut()
            .expect("source document is active")
            .select_range(1, 4);
        app.document.set_scroll(crate::render::ScrollPos::at_row(2));
        let before_path = app.document.buffer().path().map(|p| p.to_path_buf());
        let before_key = app.document.active_key();
        let before_bytes = app.document.buffer().disk_bytes();
        let before_text = app.document.buffer().text();
        let before_cursor = app.document.buffer().cursor_char();
        let before_selection = app.document.buffer().selection_range();
        let before_scroll = app.document.scroll();
        let before_version = app.document.buffer().version();
        assert!(
            app.document.buffer().can_undo(),
            "PRESENCE: the source has an undo timeline"
        );

        assert!(app.save_copy_to(&destination, false));
        assert_eq!(
            mem.read(&destination).unwrap(),
            before_bytes,
            "destination bytes are the source disk bytes"
        );
        assert_eq!(
            app.document.buffer().path(),
            before_path.as_deref(),
            "copy never adopts its destination"
        );
        assert_eq!(
            app.document.active_key(),
            before_key,
            "copy never swaps the working-set identity"
        );
        assert_eq!(
            app.document.buffer().text(),
            before_text,
            "copy does not edit the source"
        );
        assert_eq!(
            app.document.buffer().cursor_char(),
            before_cursor,
            "copy keeps the caret"
        );
        assert_eq!(
            app.document.buffer().selection_range(),
            before_selection,
            "copy keeps the selection"
        );
        assert_eq!(
            app.document.scroll(),
            before_scroll,
            "copy keeps the scroll position"
        );
        assert_eq!(
            app.document.buffer().version(),
            before_version,
            "copy does not save or alter the autosave baseline"
        );
        app.document.undo();
        assert_eq!(
            app.document.buffer().text(),
            "one\ntwo\n",
            "the pre-existing undo timeline remains intact"
        );
    });
}

/// The modal panel owns cancellation and overwrite confirmation. A hermetic App
/// has no surface and must not reach that modal; the lower write seam therefore
/// refuses an existing destination unless that owner explicitly confirms it.
#[test]
fn save_a_copy_cancellation_and_no_clobber_leave_disk_and_source_intact() {
    let _g = crate::testlock::serial();
    let source = std::path::PathBuf::from(DOC);
    let destination = std::path::PathBuf::from("/w/proj/existing.md");
    let mem = InMemoryFs::new()
        .with_file(&source, BODY)
        .with_file(&destination, "keep this\n");
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut app = App::new(
            Some(source.clone()),
            std::path::PathBuf::from("/w/proj"),
            None,
            None,
            Config {
                session_restore: Some(false),
                reduce_motion: Some(false),
                ..Config::empty()
            },
        );
        let source_before = app.document.buffer().disk_bytes();
        assert!(
            !app.save_copy_via_platform_panel(),
            "a headless App cannot open a blocking save panel"
        );
        assert_eq!(
            mem.read(&destination).unwrap(),
            b"keep this\n",
            "cancellation writes nothing"
        );
        assert!(
            !app.save_copy_to(&destination, false),
            "an unconfirmed existing destination is never clobbered"
        );
        assert_eq!(
            mem.read(&destination).unwrap(),
            b"keep this\n",
            "no-clobber preserves destination bytes"
        );
        assert_eq!(
            app.document.buffer().disk_bytes(),
            source_before,
            "failed/cancelled copies leave the source intact"
        );
    });
}

/// The shared destination route supplies both its chosen folder and the
/// rename-seam filename; neither is silently replaced with the source name.
#[test]
fn save_a_copy_named_uses_the_chosen_filename_and_refuses_a_collision() {
    let _g = crate::testlock::serial();
    let source = std::path::PathBuf::from(DOC);
    let target = std::path::PathBuf::from("/w/proj/copies/final.md");
    let mem = InMemoryFs::new()
        .with_dir("/w/proj/copies")
        .with_file(&source, BODY);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut app = App::new(
            Some(source),
            std::path::PathBuf::from("/w/proj"),
            None,
            None,
            Config {
                session_restore: Some(false),
                reduce_motion: Some(false),
                ..Config::empty()
            },
        );
        app.save_copy_named("copies", "final.md");
        assert_eq!(
            mem.read(&target).unwrap(),
            BODY.as_bytes(),
            "chosen filename reaches the snapshot writer"
        );
        mem.write(&target, b"keep").unwrap();
        app.save_copy_named("copies", "final.md");
        assert_eq!(
            mem.read(&target).unwrap(),
            b"keep",
            "a collision is never silently clobbered"
        );
    });
}

/// A destination can appear after the user chose its name. The no-clobber
/// guarantee is therefore owned by the atomic publish, not an earlier exists
/// check. The scripted filesystem creates the competing file between the temp
/// write and `rename_no_replace`; its bytes must win.
///
/// MUTATION TARGET: replace `durable::write_new` in `save_copy_to` with
/// `durable::write`; this fails because the ordinary rename overwrites the
/// competing creator below.
#[test]
fn save_a_copy_never_clobbers_a_destination_created_after_preflight() {
    let _g = crate::testlock::serial();
    let source = std::path::PathBuf::from(DOC);
    let destination = std::path::PathBuf::from("/w/proj/copies/raced.md");
    let mem = InMemoryFs::new()
        .with_dir("/w/proj/copies")
        .with_file(&source, BODY);
    let scripted = crate::fs::ScriptedFs::new(
        mem.clone(),
        crate::fs::ScriptedFailure {
            operation: crate::fs::ScriptedOperation::Rename,
            ordinal: 99,
            kind: std::io::ErrorKind::Other,
            reason: "not reached",
        },
    )
    .race_create_before_no_replace(destination.clone(), b"racing creator");
    crate::fs::with_fs(Arc::new(scripted), || {
        let mut app = App::new(
            Some(source),
            std::path::PathBuf::from("/w/proj"),
            None,
            None,
            Config {
                session_restore: Some(false),
                reduce_motion: Some(false),
                ..Config::empty()
            },
        );
        assert!(
            !app.save_copy_to(&destination, false),
            "the racing destination rejects an unconfirmed copy"
        );
        assert_eq!(
            mem.read(&destination).unwrap(),
            b"racing creator",
            "the creator that won the destination name is never overwritten"
        );
    });
}

/// A FOLDER CHOSEN IN THE NAVIGATOR wins over both defaults, keeps the
/// document's own stem, and — because it is not the folder the document lives in
/// — is spoken in full. Swept over both sides of the path axis, since the stem
/// rule differs there and a saved-only fixture could not see it.
///
/// MUTATION TARGET: drop the `dest_dir` arm from `export_target` (falling back to
/// the sibling) and this fails by name on the path; make `ExportTarget::at`
/// return a constant and it fails on `show_full`.
#[test]
fn a_chosen_folder_wins_over_both_defaults_and_the_notice_says_where() {
    let root = std::path::Path::new("/w/proj");
    let saved = std::path::PathBuf::from(DOC);
    for format in every_format() {
        let got = export_target(Some(saved.as_path()), root, "ignored", format, Some("out"));
        assert_eq!(
            got.path,
            std::path::PathBuf::from(format!("/w/proj/out/draft.{}", format.ext())),
            "{}: a chosen folder takes the document's own stem",
            format.ext(),
        );
        assert!(
            got.show_full,
            "{}: a folder that is not the document's own must be spoken in full",
            format.ext(),
        );

        let fresh = export_target(None, root, "quick-note", format, Some("out"));
        assert_eq!(
            fresh.path,
            std::path::PathBuf::from(format!("/w/proj/out/quick-note.{}", format.ext())),
            "{}: a path-less buffer keeps its derived stem inside the chosen folder",
            format.ext(),
        );
    }
}

/// THE NOTICE RULE IS A RELATION, NOT A FLAG PER ARM: choosing the folder the
/// document already lives in — which the navigator lets you do, by accepting at
/// the level you started on — must read exactly like the sibling default, bare
/// filename and all. This is the case a per-arm flag got wrong: it is a CHOSEN
/// destination that must NOT be spoken in full.
///
/// MUTATION TARGET: replace `ExportTarget::at`'s comparison with `dest_dir
/// .is_some()` and this fails by name while every other law here stays green.
#[test]
fn choosing_the_documents_own_folder_reads_exactly_like_the_sibling_default() {
    let saved = std::path::PathBuf::from(DOC);
    for format in every_format() {
        let chosen = export_target(
            Some(saved.as_path()),
            std::path::Path::new("/w"),
            "ignored",
            format,
            Some("proj"),
        );
        let default = export_target(
            Some(saved.as_path()),
            std::path::Path::new("/w"),
            "ignored",
            format,
            None,
        );
        assert_eq!(
            (chosen.path.clone(), chosen.show_full),
            (default.path.clone(), default.show_full),
            "{}: the same folder by two routes must produce the same destination \
             and the same notice",
            format.ext(),
        );
        assert!(
            !chosen.show_full,
            "{}: PRESENCE — if both sides were `true` the equality above would \
             hold while the rule was inverted",
            format.ext(),
        );
    }
}

/// A HEADLESS `App` WRITES INTO THE CHOSEN FOLDER, creating it if it does not
/// exist — the navigator accepts a typed name that is not on disk yet, so this is
/// a reachable state and not a defensive one. Tier 2 over the `InMemoryFs` seam.
#[test]
fn a_headless_app_writes_the_export_into_the_chosen_folder_and_names_the_whole_path() {
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
            app.export_document(format, Some("exports/final"));
            let want =
                std::path::PathBuf::from(format!("/w/proj/exports/final/draft.{}", format.ext()));
            assert!(
                mem.read(&want).is_ok(),
                "{}: nothing at {want:?} — the chosen folder was not created or not used",
                format.ext(),
            );
            let spoken = format!("exported {}", want.display());
            assert_eq!(
                app.frame.notice().text(),
                Some(spoken.as_str()),
                "{}: the toast names the whole path of a folder the writer chose",
                format.ext(),
            );
        }
    });
}

/// THE MODAL GATE. `NSSavePanel::runModal` blocks the process main thread until a
/// human closes it, so a surfaceless `App` reaching the panel would hang
/// `cargo test` and `--screenshot-app` forever. Gated on a real surface, exactly
/// as the Finder reveal is — one rule, two live-only doors.
///
/// MUTATION TARGET: drop the `frame.gpu().is_none()` early return in
/// `export_via_platform_panel` and this fails by name. It is also the one law
/// that would notice the regression behaviourally, in the worst possible way: the
/// suite would stop producing output instead of failing.
#[test]
fn a_headless_app_never_opens_the_platform_save_panel() {
    let _g = crate::testlock::serial();
    let doc = std::path::PathBuf::from(DOC);
    let fake = Arc::new(InMemoryFs::new().with_file(doc.clone(), BODY));
    crate::fs::with_fs(fake, || {
        let mut app = App::new_hermetic(
            Some(doc.clone()),
            std::path::PathBuf::from("/w/proj"),
            Config::empty(),
        );
        assert!(
            app.frame.gpu().is_none(),
            "PRESENCE: a hermetic App has no surface — without this the gate below \
             is untested rather than satisfied",
        );
        for format in every_format() {
            assert!(
                !app.export_via_platform_panel(format),
                "{}: a surface-less App must not reach a main-thread modal",
                format.ext(),
            );
        }
    });
}
