//! The native audition keeps document, preferences and artifact inputs sealed.
use super::*;
use crate::fs::FileSystem;
use crate::semantic::SemanticRequest;

const TEXT: &str = concat!(
    "# Synthetic letter\n\nDear reader,\n\n",
    "This sample contains **emphasis** and an ordinary list.\n\n",
    "- First item\n- Second item\n\nRegards,\nThe sample writer\n",
);

fn seeded() -> crate::fs::InMemoryFs {
    crate::fs::InMemoryFs::new()
        .with_file("/synthetic/letter.md", TEXT)
        .with_file(
            "/synthetic/config.toml",
            "# keep this comment\n[ keys ]\nsave = \"Cmd-S\"\n",
        )
}

fn app() -> App {
    let mut config = Config::empty();
    config.path = "/synthetic/config.toml".into();
    config.autosave = Some(false);
    config.session_restore = Some(false);
    App::new_hermetic(
        Some("/synthetic/letter.md".into()),
        "/synthetic".into(),
        config,
    )
}

fn state(app: &App) -> (String, u64, usize, Option<(usize, usize)>) {
    let b = app.document.buffer();
    (b.text(), b.version(), b.cursor_char(), b.selection_range())
}

#[test]
fn pdf_modal_blocks_document_actions_semantic_selection_ime_and_drop_then_cancel_restores() {
    let _serial = crate::testlock::serial();
    let _restore = crate::testlock::TogglesRestore::capture();
    let fs = seeded();
    let _fs = crate::fs::FsGuard::install(Arc::new(fs.clone()));
    let mut app = app();
    app.document.set_anchor(4);
    app.document.set_cursor(16);
    let before = state(&app);
    let config = fs
        .read_to_string(Path::new("/synthetic/config.toml"))
        .unwrap();
    app.open_pdf_export();
    assert_eq!(app.workspace_state.layer(), workspace::Layer::PdfExport);
    let exit = schedule::RecordingExit::default();
    for action in [
        Action::Undo,
        Action::Save,
        Action::SelectAll,
        Action::InsertText("hidden".into()),
    ] {
        app.apply(action, false, &exit, crate::stats::Door::Menu);
        assert_eq!(state(&app), before);
    }
    for request in [
        SemanticRequest::SetTextSelection {
            id: "document".into(),
            anchor: 0,
            focus: 1,
        },
        SemanticRequest::SetValue {
            id: "document".into(),
            value: "hidden".into(),
        },
        SemanticRequest::ReplaceSelectedText {
            id: "document".into(),
            value: "hidden".into(),
        },
    ] {
        assert!(!app.apply_semantic_request(request));
    }
    app.on_ime(winit::event::Ime::Commit("隠れた文字".into()));
    app.on_dropped_file(&exit, "/synthetic/config.toml".into());
    assert_eq!(state(&app), before);
    app.apply_pdf_export_action(&Action::Cancel);
    assert!(app.workspace_state.pdf_export().is_none());
    assert_eq!(state(&app), before);
    assert_eq!(
        fs.read_to_string(Path::new("/synthetic/config.toml"))
            .unwrap(),
        config
    );
    assert!(!fs.exists(Path::new("/synthetic/letter.pdf")));
}

#[test]
fn pdf_options_snapshot_theme_and_images_including_missing_images() {
    let _serial = crate::testlock::serial();
    let _restore = crate::testlock::TogglesRestore::capture();
    let fs = seeded();
    let _fs = crate::fs::FsGuard::install(Arc::new(fs.clone()));
    let before_theme = crate::theme::active_index();
    let mut dialog = Dialog::fixture();
    let saved = dialog.theme_index;
    crate::theme::set_active((saved + 1) % crate::theme::THEMES.len());
    dialog.change(1).unwrap();
    assert_eq!(
        dialog.options.appearance,
        PdfAppearance::CurrentTheme(saved)
    );
    crate::theme::set_active(before_theme);
    let source = FrozenImages {
        source: crate::export::FsImages {
            doc_dir: Some("/synthetic".into()),
        },
        cache: Default::default(),
    };
    assert!(source.resolve("later.png").is_none());
    let png = crate::paste_image::encode_rgba_png(1, 1, &[10, 20, 30, 255]).unwrap();
    fs.write(Path::new("/synthetic/later.png"), &png).unwrap();
    assert!(
        source.resolve("later.png").is_none(),
        "a missing summon-time image stays missing"
    );
    let resolved = source.resolve("fresh.png");
    assert!(resolved.is_none());
    let source = FrozenImages {
        source: crate::export::FsImages {
            doc_dir: Some("/synthetic".into()),
        },
        cache: Default::default(),
    };
    let captured = source.resolve("later.png").unwrap();
    fs.remove_file(Path::new("/synthetic/later.png")).unwrap();
    assert_eq!(source.resolve("later.png").unwrap(), captured);
}

#[test]
fn pdf_export_repeats_the_frozen_origin_and_remembers_only_a_successful_commit() {
    let _serial = crate::testlock::serial();
    let _restore = crate::testlock::TogglesRestore::capture();
    let fs = seeded();
    let _fs = crate::fs::FsGuard::install(Arc::new(fs.clone()));
    let mut app = app();
    let before = state(&app);
    app.open_pdf_export();
    let mut dialog = app.workspace_state.take_pdf_export().unwrap();
    dialog.options = PdfOptions {
        paper: PdfPaper::Letter,
        appearance: PdfAppearance::Plain,
    };
    dialog.refresh(dialog.options, 0).unwrap();
    let bytes = dialog.artifact.bytes().unwrap();
    app.document.set_path("/synthetic/changed-name.md".into());
    app.finish_pdf_export(dialog);
    assert_eq!(fs.read(Path::new("/synthetic/letter.pdf")).unwrap(), bytes);
    assert!(!fs.exists(Path::new("/synthetic/changed-name.pdf")));
    assert_eq!(app.config.pdf_paper.as_deref(), Some("letter"));
    assert_eq!(app.config.pdf_appearance.as_deref(), Some("plain"));
    assert_eq!(state(&app), before);
    let config = fs
        .read_to_string(Path::new("/synthetic/config.toml"))
        .unwrap();
    assert!(config.contains("# keep this comment"));
    assert!(config.contains("save = \"Cmd-S\""));
    app.document.set_path("/synthetic/letter.md".into());
    app.open_pdf_export();
    let repeat = app.workspace_state.take_pdf_export().unwrap();
    assert_eq!(repeat.options.paper, PdfPaper::Letter);
    assert_eq!(repeat.artifact.bytes().unwrap(), bytes);
    app.finish_pdf_export(repeat);
    assert_eq!(fs.read(Path::new("/synthetic/letter.pdf")).unwrap(), bytes);
}

#[test]
fn pdf_page_change_reuses_layout_and_bounds_both_directions() {
    let _serial = crate::testlock::serial();
    let _restore = crate::testlock::TogglesRestore::capture();
    let mut dialog = Dialog::build(
        TEXT.repeat(40),
        crate::export::FsImages { doc_dir: None },
        PdfOptions::default(),
        Origin {
            path: None,
            root: "/synthetic".into(),
            stem: "sample".into(),
        },
    )
    .unwrap();
    assert!(dialog.artifact.page_count() > 2);
    let before = dialog.artifact.bytes().unwrap();
    dialog.selected = 2;
    dialog.change(-1).unwrap();
    assert_eq!(dialog.page, 0);
    dialog.change(1).unwrap();
    assert_eq!(dialog.page, 1);
    dialog.change(-1).unwrap();
    assert_eq!(dialog.page, 0);
    dialog.change(isize::MAX).unwrap();
    assert_eq!(dialog.page, dialog.artifact.page_count() - 1);
    assert_eq!(dialog.artifact.bytes().unwrap(), before);
}

#[test]
fn pdf_write_failure_and_destination_cancel_preserve_document_defaults_and_audition() {
    let _serial = crate::testlock::serial();
    let _restore = crate::testlock::TogglesRestore::capture();
    let fs = seeded();
    let _fs = crate::fs::FsGuard::install(Arc::new(fs.clone()));
    let mut app = app();
    let before = state(&app);
    let settings = fs
        .read_to_string(Path::new("/synthetic/config.toml"))
        .unwrap();
    app.open_pdf_export();
    let dialog = app.workspace_state.take_pdf_export().unwrap();
    let bytes = dialog.artifact.bytes().unwrap();
    app.commit_pdf_destination(dialog, bytes, None);
    assert!(app.workspace_state.pdf_export().is_some());
    assert_eq!(state(&app), before);
    assert!(!fs.exists(Path::new("/synthetic/letter.pdf")));
    let dialog = app.workspace_state.take_pdf_export().unwrap();
    {
        let _failed = crate::fs::FsGuard::install(Arc::new(crate::fs::UnwritableFs));
        app.finish_pdf_export(dialog);
    }
    let view = app.pdf_export_view().unwrap();
    assert!(view.error.as_deref().unwrap().contains("unwritable"));
    assert_eq!(state(&app), before);
    assert!(app.config.pdf_paper.is_none() && app.config.pdf_appearance.is_none());
    assert_eq!(
        fs.read_to_string(Path::new("/synthetic/config.toml"))
            .unwrap(),
        settings
    );
    assert!(!fs.exists(Path::new("/synthetic/letter.pdf")));
}
