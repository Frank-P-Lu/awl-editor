//! THE NO-DOCUMENT SCREEN NAMES ITS OPEN FOLDER.
//!
//! The honest empty surface (`render/chrome/start.rs`) names the open
//! folder (its last path segment — `ProjectInfo::name`, already just that
//! one component) on a THIRD dim line above the two start actions, with a
//! convention-truthful Go-to chord read through `keytoken` rather than a
//! literal glyph — otherwise the folder having opened is invisible on
//! screen.
//!
//! These laws drive the ordinary (non-App) capture door directly through
//! `CaptureOpts`/`capture_with` — `document_absent` + `project` are exactly
//! the two facts `CaptureOpts::fold_gutter` needs, with no live `App`
//! required — and a SEEDED root (never the ambient cwd), per this repo's
//! public-repo capture discipline (`redact_law.rs`'s own doc).

use super::super::*;
use super::{adapter_available, sidecar};
use crate::buffer::Buffer;
use crate::testscratch::ScratchDir;

fn seeded_project(name: &str) -> ProjectInfo {
    ProjectInfo {
        root: std::path::PathBuf::from("/seeded").join(name),
        name: name.to_string(),
        branch: None,
        dirty: false,
        default_folder: None,
        workspace: None,
        keymap_flavor: "native",
    }
}

/// LAW: with a root set (`project: Some(..)`) and no document, the sidecar
/// names the folder and the exact Go-to chord the line drew — the SAME
/// derivation (`crate::render::chrome::goto_chord_label`) the pixel actually
/// came from, not a second recomputation.
#[test]
fn start_folder_is_named_in_the_sidecar_when_a_root_is_set() {
    if !adapter_available() {
        eprintln!(
            "skipping start_folder_is_named_in_the_sidecar_when_a_root_is_set: no wgpu adapter"
        );
        return;
    }
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-start-folder-named-{}", std::process::id())),
    );
    let buffer = Buffer::scratch();
    let opts = CaptureOpts {
        document_absent: true,
        project: Some(seeded_project("seeded-notes")),
        ..CaptureOpts::default()
    };
    let png = dir.join("named.png");
    capture_with(&png, &buffer, &opts).expect("zero-document capture with a seeded root");
    let json = std::fs::read_to_string(png.with_extension("json")).unwrap();
    let value = sidecar(&json);
    assert_eq!(
        value["document"]["start_folder"].as_str(),
        Some("seeded-notes"),
        "sidecar: {value}"
    );
    let expected_chord = crate::render::chrome::goto_chord_label();
    assert_eq!(
        value["document"]["start_goto_chord"].as_str(),
        Some(expected_chord.as_str()),
        "sidecar must report the SAME chord the surface drew, not a re-derivation"
    );
    assert!(
        !expected_chord.is_empty(),
        "go_to must resolve to a real chord"
    );
}

/// LAW: with NO root (`project: None`) the empty state names nothing —
/// `start_folder` reports JSON `null`, never an invented or stale name.
#[test]
fn start_folder_is_absent_in_the_sidecar_with_no_root() {
    if !adapter_available() {
        eprintln!("skipping start_folder_is_absent_in_the_sidecar_with_no_root: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-start-folder-absent-{}", std::process::id())),
    );
    let buffer = Buffer::scratch();
    let opts = CaptureOpts {
        document_absent: true,
        project: None,
        ..CaptureOpts::default()
    };
    let png = dir.join("no-root.png");
    capture_with(&png, &buffer, &opts).expect("zero-document capture with no root");
    let json = std::fs::read_to_string(png.with_extension("json")).unwrap();
    let value = sidecar(&json);
    assert!(
        value["document"]["start_folder"].is_null(),
        "no root was set, so start_folder must be null; got {value}"
    );
}

/// LAW: an ACTIVE document never grows this line — `start_folder` stays
/// `null` even when a root is set, because the field means "the empty
/// state's folder", not "the ambient project". Nothing changes when a
/// document is open.
#[test]
fn start_folder_is_absent_while_a_document_is_active_even_with_a_root_set() {
    if !adapter_available() {
        eprintln!(
            "skipping start_folder_is_absent_while_a_document_is_active_even_with_a_root_set: \
             no wgpu adapter"
        );
        return;
    }
    let _g = crate::testlock::serial();
    let dir = ScratchDir::new(std::env::temp_dir().join(format!(
        "awl-start-folder-active-doc-{}",
        std::process::id()
    )));
    let buffer = Buffer::from_str("hello\n");
    let opts = CaptureOpts {
        document_absent: false,
        project: Some(seeded_project("seeded-notes")),
        ..CaptureOpts::default()
    };
    let png = dir.join("active.png");
    capture_with(&png, &buffer, &opts).expect("ordinary document capture");
    let json = std::fs::read_to_string(png.with_extension("json")).unwrap();
    let value = sidecar(&json);
    assert!(
        value["document"]["start_folder"].is_null(),
        "a document is active, so start_folder must stay null; got {value}"
    );
}
