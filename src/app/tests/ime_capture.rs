//! Real App events reach the same transient field snapshot in captures and live frames.

use super::summoned_field_actions::{app, doc_state, seeded, summon};
use super::*;
use crate::textbox::TextField;
use std::sync::Arc;

#[test]
fn ime_app_capture_reports_and_paints_transient_text_at_each_field() {
    let _g = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping App IME capture: no wgpu adapter");
        return;
    }
    let supplied = std::env::var_os("AWL_IME_CAPTURE_DIR").map(PathBuf::from);
    let scratch = supplied.is_none().then(|| {
        crate::testscratch::ScratchDir::new(
            std::env::temp_dir().join(format!("awl-ime-fields-{}", std::process::id())),
        )
    });
    let out = supplied.unwrap_or_else(|| scratch.as_ref().unwrap().to_path_buf());
    std::fs::create_dir_all(&out).unwrap();
    let worlds = ["Paperbark", "Bowerbird", "Gumtree", "Saltpan", "Wagtail"];
    for (index, field) in TextField::ALL.into_iter().enumerate() {
        let mut app = app();
        let _world = crate::theme::WorldPin::world(worlds[index % worlds.len()]).unwrap();
        if field == TextField::PickerQuery {
            app.press_spec_headless(super::ime_fields::files_chord())
                .unwrap();
        } else {
            summon(&mut app, field);
        }
        let before = doc_state(&app);
        super::ime_fields::select_field(&mut app, field);
        let text = if field == TextField::SettingsValue {
            "125"
        } else {
            "にほん"
        };
        app.on_ime(Ime::Preedit(text.into(), Some((text.len(), text.len()))));
        let mut opts = app.capture_opts();
        opts.canvas = Some((1200, 800));
        let marked = out.join(format!("{index}-{field:?}.png"));
        crate::capture::capture_with(&marked, app.document.buffer(), &opts).unwrap();
        let sidecar: serde_json::Value =
            serde_json::from_slice(&std::fs::read(marked.with_extension("json")).unwrap()).unwrap();
        assert_eq!(sidecar["focused_field"]["text"], text);
        assert_eq!(sidecar["focused_field"]["caret"], text.chars().count());
        assert_eq!(
            sidecar["focused_field"]["preedit"],
            serde_json::json!([0, text.chars().count()])
        );
        assert!(
            sidecar["focused_field"]["candidate_rect"].is_array(),
            "{field:?}"
        );
        assert_eq!(sidecar["text"], before.0);
        app.on_ime(Ime::Preedit(String::new(), None));
        let mut opts = app.capture_opts();
        opts.canvas = Some((1200, 800));
        let unmarked = out.join(format!("{index}-{field:?}-unmarked.png"));
        crate::capture::capture_with(&unmarked, app.document.buffer(), &opts).unwrap();
        let a = image::open(&marked).unwrap().to_rgba8();
        let b = image::open(&unmarked).unwrap().to_rgba8();
        let changed = a.pixels().zip(b.pixels()).filter(|(a, b)| a != b).count();
        assert!(
            changed > 20,
            "{field:?}: transient text did not change the frame"
        );
        assert_eq!(doc_state(&app), before);
        eprintln!(
            "IME capture {field:?}: {changed} changed pixels, {}",
            marked.display()
        );
    }
}
