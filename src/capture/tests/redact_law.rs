//! THE HOME-PATH LAWS — a capture artifact must never carry the builder's home
//! directory, and must still say which directory it listed.
//!
//! WHAT THIS IS FOR. A tracked file carrying a personal-machine path is a rule
//! this repo already states, and it did not reach captures: a lane does not have
//! to WRITE a path to leak one, it only has to CAPTURE. An ordinary
//! `--screenshot` with no `--root` emitted `project.root` = the cwd,
//! `project.workspace` = the effective workspace, and `project.default_folder` =
//! `$HOME/notes` — three absolute home paths in every sidecar ever taken — and a
//! driven picker added a fourth in `overlay.browse_dir`.
//!
//! THREE LAWS, THREE FAILURE SHAPES. The WIRED one lives with the launch-door
//! laws it enrols through (`run::tests::launch_context::
//! a_capture_taken_under_a_non_seeded_root_leaks_no_home_path`, the only module
//! that can reach the private `capture_screenshot`): it drives the real
//! screenshot door with nothing seeded and reads the artifact off disk, so
//! deleting the redaction call from `write_sidecar` fails it. Here,
//! [`every_path_bearing_sidecar_field_is_home_relative`] SWEEPS the fields
//! rather than the one field a bare capture happens to fill, and pins a PRESENCE
//! floor beside the absence floor — a sidecar that dropped its path fields
//! entirely would satisfy "no home path" and be useless. The pure sweep pins the
//! boundary rule the rewrite turns on.

use super::super::redact::{is_redactable, redact_with_home};
use super::super::{BuffersInfo, CaptureOpts, OverlayInfo, ProjectInfo};
use crate::testscratch::ScratchDir;
use std::path::{Path, PathBuf};

/// This host's home, when it is specific enough to strip. `None` means the law
/// cannot enrol here — reported rather than silently passed, because a check
/// that skips quietly reads exactly like a check that ran.
fn redactable_home() -> Option<PathBuf> {
    let home = crate::fs::home_dir()?;
    is_redactable(&home).then_some(home)
}

/// LAW (sweep): EVERY path-bearing sidecar field is home-relative, not just the
/// one a bare capture fills. `project.root` / `.workspace` / `.default_folder`,
/// `overlay.browse_dir` (absolute for the switch-project picker, which lists the
/// workspace) and `buffers.active` are each given a home-anchored value and each
/// asserted to come back `~`-prefixed and readable.
///
/// The values are anchored under `$HOME` by CONSTRUCTION rather than by where
/// this checkout happens to sit, so the enrolment is the same on a developer
/// machine and on a runner whose checkout lives outside its home.
#[test]
fn every_path_bearing_sidecar_field_is_home_relative() {
    if !super::adapter_available() {
        eprintln!("skipping every_path_bearing_sidecar_field_is_home_relative: no wgpu adapter");
        return;
    }
    let Some(home) = redactable_home() else {
        eprintln!(
            "skipping every_path_bearing_sidecar_field_is_home_relative: \
             $HOME is unset or too generic to strip"
        );
        return;
    };
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-402-fields-{}", std::process::id())),
    );
    let png = dir.join("fields.png");

    // Never created on disk: the sidecar reports the paths it was HANDED, and
    // this law is about the serializer, not about the filesystem.
    let root = home.join("Documents").join("awl-402-project");
    let workspace = home.join("Documents");
    let default_folder = home.join("notes");
    let browse_dir = home.join("Documents");
    let active = root.join("note.md");

    let mut opts = CaptureOpts {
        project: Some(ProjectInfo {
            root: root.clone(),
            name: "awl-402-project".into(),
            branch: None,
            dirty: false,
            default_folder: Some(default_folder.clone()),
            workspace: Some(workspace.clone()),
            keymap_flavor: "native",
        }),
        buffers: Some(BuffersInfo {
            open: 1,
            active: Some(active.to_string_lossy().to_string()),
        }),
        ..CaptureOpts::default()
    };
    let mut overlay = OverlayInfo {
        active: true,
        mode: "switch",
        browse_dir: Some(browse_dir.to_string_lossy().to_string()),
        ..blank_overlay()
    };
    overlay.items = vec!["awl-402-project/".to_string()];
    opts.overlay = Some(overlay);

    let buf = crate::buffer::Buffer::from_str("hello\n");
    crate::capture::capture_with(&png, &buf, &opts).expect("capture succeeds");
    let json = std::fs::read_to_string(png.with_extension("json")).expect("sidecar written");
    let home_str = home.to_string_lossy().to_string();
    assert!(
        !json.contains(&home_str),
        "no path-bearing field may carry {home_str:?}"
    );

    let v = super::sidecar(&json);
    let rel = |p: &Path| redact_with_home(&p.to_string_lossy(), &home);
    for (path, got, key) in [
        (&root, &v["project"]["root"], "project.root"),
        (&workspace, &v["project"]["workspace"], "project.workspace"),
        (
            &default_folder,
            &v["project"]["default_folder"],
            "project.default_folder",
        ),
        (
            &browse_dir,
            &v["overlay"]["browse_dir"],
            "overlay.browse_dir",
        ),
        (&active, &v["buffers"]["active"], "buffers.active"),
    ] {
        let got = got
            .as_str()
            .unwrap_or_else(|| panic!("{key} must still be reported, not dropped"));
        assert_eq!(got, rel(path), "{key} must be home-relative and intact");
        assert!(
            got.starts_with("~/"),
            "{key} must be reported under ~/ (got {got:?})"
        );
    }
}

/// LAW (pure): the rewrite's BOUNDARY rule and its refusal to act on a home too
/// generic to recognise. The boundary case that matters is a sibling account
/// whose name extends ours — a naive prefix strip turns `/Users/frankenstein`
/// into `~enstein`, corrupting an unrelated path while claiming to sanitise it.
#[test]
fn the_rewrite_stops_on_a_path_component_boundary() {
    let home = Path::new("/Users/frank");
    for (input, want, why) in [
        ("\"/Users/frank\"", "\"~\"", "the home itself, JSON-quoted"),
        ("\"/Users/frank/a/b\"", "\"~/a/b\"", "a path under home"),
        ("\"/Users/frank/\"", "\"~/\"", "a trailing separator"),
        (
            "\"/Users/frankenstein/x\"",
            "\"/Users/frankenstein/x\"",
            "a sibling account whose name EXTENDS ours is untouched",
        ),
        (
            "cd /Users/frank now",
            "cd ~ now",
            "a home path in running prose still goes",
        ),
        (
            "\"/Users/frank\" and \"/Users/frank/n\"",
            "\"~\" and \"~/n\"",
            "every occurrence, not the first",
        ),
        (
            "\"/opt/Users/frank\"",
            "\"/opt/Users/frank\"",
            "a match must start at a separator, not mid-path",
        ),
        (
            "\"/Users/frank.bak/x\"",
            "\"/Users/frank.bak/x\"",
            "a dot continues a component",
        ),
    ] {
        assert_eq!(redact_with_home(input, home), want, "{why}");
    }

    // A home too generic to tell from an ordinary path is REFUSED, in both
    // halves: the predicate says so, and the rewrite is inert.
    for generic in ["/", "/root", "relative/home", ""] {
        let p = Path::new(generic);
        assert!(
            !is_redactable(p),
            "{generic:?} is too generic to strip safely"
        );
        let text = "\"/root/x\" \"/relative/home/y\"";
        assert_eq!(
            redact_with_home(text, p),
            text,
            "a refused home leaves the artifact byte-identical ({generic:?})"
        );
    }
    assert!(
        is_redactable(Path::new("/home/runner")) && is_redactable(Path::new("/Users/frank")),
        "the real account-home shapes on both platforms DO enrol"
    );
}

/// An `OverlayInfo` with every field at its inert value, so the sweep above can
/// name only the two it cares about. Written out rather than defaulted because
/// `OverlayInfo` has no `Default` — and a new field must be a conscious decision
/// here, exactly as it is at the render seam.
fn blank_overlay() -> OverlayInfo {
    OverlayInfo {
        active: false,
        mode: "switch",
        align: crate::theme::CardAnchor::TopCenter,
        query: String::new(),
        query_caret: 0,
        query_selection: None,
        items: Vec::new(),
        empty: None,
        bindings: Vec::new(),
        ranges: Vec::new(),
        git: Vec::new(),
        selected_index: 0,
        hint: String::new(),
        browse_dir: None,
        spell_target: None,
        table_dims: None,
        context_anchor: None,
        asset_preview: None,
        capture: None,
        notice: String::new(),
        lens: None,
        lens_strip: Vec::new(),
        sections: Vec::new(),
        preview_id: None,
        preview_view: None,
        workspace: false,
        detail_focus: false,
        diff_scroll: 0,
        show_hidden: false,
        return_to: None,
        title: "switch project".to_string(),
    }
}
