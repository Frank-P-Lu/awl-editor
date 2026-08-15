use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::buffer::Buffer;
use crate::config::Config;
use crate::fs::{FileSystem, InMemoryFs};

use super::App;

#[test]
fn pdf_export_writes_saved_sibling_and_scratch_active_folder_without_other_formats() {
    let saved_fs = InMemoryFs::new().with_dir("/docs");
    let mut saved = App::new_hermetic(None, PathBuf::from("/docs"), Config::empty());
    saved
        .document
        .replace_buffer(Buffer::from_str("# Saved PDF\n\nSibling export body.\n"));
    saved.document.set_path(PathBuf::from("/docs/draft.md"));
    crate::fs::with_fs(Arc::new(saved_fs.clone()), || {
        saved.export_document(crate::export::Format::Pdf, None);
        let pdf = saved_fs.read(Path::new("/docs/draft.pdf")).unwrap();
        assert!(pdf.starts_with(b"%PDF-1.7\n"));
        assert!(!saved_fs.exists(Path::new("/docs/draft.docx")));
        assert!(!saved_fs.exists(Path::new("/docs/draft.html")));
        assert_eq!(saved.frame.notice().text(), Some("exported draft.pdf"));
    });

    let scratch_fs = InMemoryFs::new().with_dir("/notes");
    let mut scratch = App::new_hermetic(None, PathBuf::from("/notes"), Config::empty());
    scratch.document.replace_buffer(Buffer::from_str(
        "# Scratch PDF\n\nActive-folder export body.\n",
    ));
    crate::fs::with_fs(Arc::new(scratch_fs.clone()), || {
        scratch.export_document(crate::export::Format::Pdf, None);
        let target = Path::new("/notes/scratch-pdf.pdf");
        let pdf = scratch_fs.read(target).unwrap();
        assert!(pdf.starts_with(b"%PDF-1.7\n"));
        assert_eq!(
            scratch.frame.notice().text(),
            Some("exported /notes/scratch-pdf.pdf")
        );
    });
}

#[test]
fn every_theme_preview_input_door_uses_the_one_latest_wins_policy() {
    let apply = include_str!("apply.rs");
    let mouse = include_str!("input/mouse.rs");
    let window = include_str!("window.rs");
    let schedule = include_str!("schedule.rs");

    assert_eq!(
        apply.matches("retint_theme_preview(").count(),
        2,
        "apply.rs must contain exactly the policy owner and the shared keyboard transition call"
    );
    assert_eq!(
        mouse.matches("retint_theme_preview(").count(),
        3,
        "hover, wheel, and faceted-pointer navigation must all call the shared policy"
    );
    assert_eq!(
        apply.matches("ShapeReach::Presentable").count(),
        1,
        "only retint_theme_preview may start a supersedable shape"
    );
    assert_eq!(
        apply
            .matches("arm_settle(frame::SettleKind::Crossing")
            .count(),
        1,
        "the preview owner must re-stamp exactly one shared quiet settle"
    );
    assert_eq!(
        window.matches("finish_shape_tail(").count(),
        0,
        "a presented frame must not finish an intermediate preview tail"
    );
    assert_eq!(
        schedule.matches("self.finish_shape_tail();").count(),
        1,
        "only the shared quiet-settle interpreter may finish a preview tail"
    );
}
