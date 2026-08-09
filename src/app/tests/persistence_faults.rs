use super::*;
use crate::fs::{FileSystem, InMemoryFs, ScriptedFailure, ScriptedFs, ScriptedOperation};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Copy)]
struct FailurePhase {
    name: &'static str,
    operation: ScriptedOperation,
    kind: io::ErrorKind,
    reason: &'static str,
}

const FAILURE_PHASES: &[FailurePhase] = &[
    FailurePhase {
        name: "tmp-write/permission",
        operation: ScriptedOperation::Write,
        kind: io::ErrorKind::PermissionDenied,
        reason: "permission denied while writing temporary sibling",
    },
    FailurePhase {
        name: "tmp-write/disk-full",
        operation: ScriptedOperation::Write,
        kind: io::ErrorKind::StorageFull,
        reason: "disk full while writing temporary sibling",
    },
    FailurePhase {
        name: "tmp-write/parent-removed",
        operation: ScriptedOperation::Write,
        kind: io::ErrorKind::NotFound,
        reason: "parent removed before temporary write",
    },
    FailurePhase {
        name: "final-rename/permission",
        operation: ScriptedOperation::Rename,
        kind: io::ErrorKind::PermissionDenied,
        reason: "permission denied replacing destination",
    },
    FailurePhase {
        name: "final-rename/disk-full",
        operation: ScriptedOperation::Rename,
        kind: io::ErrorKind::StorageFull,
        reason: "disk full replacing destination",
    },
    FailurePhase {
        name: "final-rename/parent-renamed",
        operation: ScriptedOperation::Rename,
        kind: io::ErrorKind::NotFound,
        reason: "parent renamed before replacement",
    },
];

const SIBLING: &str = "/matrix/metadata.sibling";
const SIBLING_BYTES: &[u8] = b"unrelated metadata remains byte-identical\n";

fn editable_buffer_stays_dirty() {
    let mut buffer = crate::buffer::Buffer::from_str("before\n");
    buffer.set_text("still editable after metadata failure\n");
    assert!(buffer.is_dirty(), "metadata failure never blocks editing");
}

fn seed_owner(inner: &InMemoryFs, owner: crate::durable::Owner) -> (PathBuf, Vec<u8>) {
    inner.write(Path::new(SIBLING), SIBLING_BYTES).unwrap();
    let target = match owner {
        crate::durable::Owner::ManualSave | crate::durable::Owner::Autosave => {
            let path = PathBuf::from("/matrix/document.md");
            inner.write(&path, b"old complete document\n").unwrap();
            path
        }
        crate::durable::Owner::Scratch => {
            let path = crate::fs::scratch_stash_path();
            inner.write(&path, b"old complete scratch\n").unwrap();
            path
        }
        crate::durable::Owner::Recovery => {
            let record = crate::recovery::Record {
                path: PathBuf::from("/matrix/conflicted.md"),
                text: "old complete recovery\n".to_string(),
            };
            let _baseline = crate::fs::FsGuard::install(Arc::new(inner.clone()));
            assert!(crate::recovery::write(&record));
            crate::recovery::record_path()
        }
        crate::durable::Owner::History => {
            let source = PathBuf::from("/matrix/history-source.md");
            let _baseline = crate::fs::FsGuard::install(Arc::new(inner.clone()));
            crate::history::record_at(
                &source,
                "old complete history\n",
                &Config::empty(),
                1,
                false,
                None,
            );
            crate::history::log_path(&source)
        }
        crate::durable::Owner::Config => {
            let path = PathBuf::from("/matrix/config.toml");
            let _baseline = crate::fs::FsGuard::install(Arc::new(inner.clone()));
            Config::write_pref(&path, "theme", "\"Tawny\"").unwrap();
            path
        }
        crate::durable::Owner::Session => {
            let path = PathBuf::from("/matrix/session.toml");
            let state = crate::session::SessionState {
                root: Some(PathBuf::from("/old-root")),
                ..Default::default()
            };
            let _baseline = crate::fs::FsGuard::install(Arc::new(inner.clone()));
            crate::session::save(&path, &state).unwrap();
            path
        }
        crate::durable::Owner::Export => {
            let source = PathBuf::from("/matrix/export-source.md");
            let target = source.with_extension("html");
            inner.write(&source, b"source before export\n").unwrap();
            inner.write(&target, b"old complete export\n").unwrap();
            target
        }
    };
    let old = inner.read(&target).expect("owner baseline exists");
    (target, old)
}

fn invoke_owner(owner: crate::durable::Owner, target: &Path) {
    match owner {
        crate::durable::Owner::ManualSave => {
            let mut app =
                super::common::app_on(Some(target.to_path_buf()), "/matrix", Config::empty());
            app.document.set_text("new dirty manual bytes\n");
            app.manual_save();
            assert!(app.is_document_dirty(), "failed manual save remains dirty");
            assert!(
                app.frame
                    .notice()
                    .text()
                    .is_some_and(|text| text.starts_with("save failed:")),
                "explicit failure is a calm durable notice"
            );
        }
        crate::durable::Owner::Autosave => {
            let mut app =
                super::common::app_on(Some(target.to_path_buf()), "/matrix", Config::empty());
            app.document.set_text("new dirty autosave bytes\n");
            app.autosave_flush();
            assert!(app.is_document_dirty(), "failed autosave remains dirty");
            assert!(
                app.frame
                    .notice()
                    .text()
                    .is_some_and(|text| text.starts_with("autosave held:")),
                "background failure is calm, durable, and visible"
            );
        }
        crate::durable::Owner::Scratch => {
            let mut app = super::common::app_on(None, "/matrix", Config::empty());
            app.document.set_text("new dirty scratch bytes\n");
            app.autosave_flush();
            assert!(
                app.is_document_dirty(),
                "failed scratch stash remains dirty"
            );
            assert!(
                app.frame
                    .notice()
                    .text()
                    .is_some_and(|text| text.starts_with("scratch save held:")),
                "scratch failure is calm, durable, and visible"
            );
        }
        crate::durable::Owner::Recovery => {
            let record = crate::recovery::Record {
                path: PathBuf::from("/matrix/conflicted.md"),
                text: "new dirty recovery\n".to_string(),
            };
            assert!(
                !crate::recovery::write(&record),
                "best-effort failure is reported"
            );
            editable_buffer_stays_dirty();
        }
        crate::durable::Owner::History => {
            crate::history::record_at(
                Path::new("/matrix/history-source.md"),
                "new dirty history\n",
                &Config::empty(),
                2,
                false,
                None,
            );
            editable_buffer_stays_dirty();
        }
        crate::durable::Owner::Config => {
            assert!(
                Config::write_pref(target, "theme", "\"Quokka\"").is_err(),
                "config failure returns without a panic"
            );
            editable_buffer_stays_dirty();
        }
        crate::durable::Owner::Session => {
            let state = crate::session::SessionState {
                root: Some(PathBuf::from("/new-root")),
                active: Some(PathBuf::from("/matrix/document.md")),
                ..Default::default()
            };
            assert!(
                crate::session::save(target, &state).is_err(),
                "session failure returns without a panic"
            );
            editable_buffer_stays_dirty();
        }
        crate::durable::Owner::Export => {
            let source = PathBuf::from("/matrix/export-source.md");
            let mut app = super::common::app_on(Some(source), "/matrix", Config::empty());
            app.document.set_text("# new dirty export\n\nbody\n");
            app.export_document(crate::export::Format::Html, None);
            assert!(
                app.is_document_dirty(),
                "export never launders source dirtiness"
            );
            assert!(
                app.frame
                    .notice()
                    .text()
                    .is_some_and(|text| text.starts_with("export failed:")),
                "export failure is calm, durable, and visible"
            );
        }
    }
}

#[test]
fn every_durable_owner_keeps_old_complete_bytes_and_recoverable_edit_state_at_every_failure_phase()
{
    let _serial = crate::testlock::serial();
    let mut report = Vec::new();
    for &owner in crate::durable::OWNERS {
        for phase in FAILURE_PHASES {
            let inner = InMemoryFs::new();
            let (target, old) = seed_owner(&inner, owner);
            let scripted = Arc::new(ScriptedFs::new(
                inner.clone(),
                ScriptedFailure {
                    operation: phase.operation,
                    ordinal: 1,
                    kind: phase.kind,
                    reason: phase.reason,
                },
            ));
            let _fs = crate::fs::FsGuard::install(scripted.clone());

            invoke_owner(owner, &target);

            assert_eq!(
                inner.read(&target).unwrap(),
                old,
                "{} × {} replaced prior complete bytes",
                owner.name(),
                phase.name
            );
            assert_eq!(
                inner.read(Path::new(SIBLING)).unwrap(),
                SIBLING_BYTES,
                "{} × {} touched an unrelated metadata sibling",
                owner.name(),
                phase.name
            );
            let needle = format!("{}#1", phase.operation.name());
            let trace = scripted.trace();
            assert!(
                trace.iter().any(|line| line.starts_with(&needle)),
                "{} × {} never reached its named fault {needle}: {trace:?}",
                owner.name(),
                phase.name
            );
            report.push(format!(
                "owner={} phase={} fault={} result=old-complete+dirty-recoverable+sibling-isolated",
                owner.name(),
                phase.name,
                needle
            ));
        }
    }

    assert_eq!(
        report.len(),
        crate::durable::OWNERS.len() * FAILURE_PHASES.len(),
        "the report is the full production roster cross product"
    );
    eprintln!("persistence-fault-matrix rows={}", report.len());
    for row in report {
        eprintln!("{row}");
    }
    eprintln!(
        "excluded=create-only first-run/open/dictionary/image writes; path-only rename/move; \
         best-effort telemetry/recents/stats/update markers; crashlog's mid-panic primitive. \
         None replaces an existing manuscript or belongs to the brief's durable-owner roster."
    );
}

#[test]
fn the_failure_matrix_enrols_from_the_exact_production_owner_roster() {
    let names: Vec<&str> = crate::durable::OWNERS
        .iter()
        .copied()
        .map(crate::durable::Owner::name)
        .collect();
    assert_eq!(
        names,
        [
            "manual-save",
            "autosave",
            "scratch",
            "recovery",
            "history",
            "config",
            "session",
            "export",
        ]
    );
}
