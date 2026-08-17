use super::*;
use crate::caret::RecoilDir;

fn preference(effect: crate::actions::PreferenceEffect) -> Effect {
    Effect::Persistence(crate::actions::PersistenceEffect::Preference(effect))
}

/// One sample instance of EVERY `Effect` variant (the compile-time
/// exhaustiveness law lives in `classify_for`'s own no-wildcard match; this
/// roster makes each variant's BUCKET explicit and reviewed).
fn roster() -> Vec<Effect> {
    vec![
        Effect::None,
        Effect::Quit,
        Effect::Persistence(crate::actions::PersistenceEffect::Save(
            crate::actions::SaveKind::Manual,
        )),
        Effect::Persistence(crate::actions::PersistenceEffect::Save(
            crate::actions::SaveKind::Finish,
        )),
        Effect::Persistence(crate::actions::PersistenceEffect::ResolveExternalChange(
            crate::actions::Resolution::KeepMine,
        )),
        Effect::Persistence(crate::actions::PersistenceEffect::ResolveExternalChange(
            crate::actions::Resolution::TakeTheirs,
        )),
        Effect::Persistence(crate::actions::PersistenceEffect::ReviewExternalChange),
        preference(crate::actions::PreferenceEffect::CaretMode),
        preference(crate::actions::PreferenceEffect::PageMode),
        preference(crate::actions::PreferenceEffect::PageWidth),
        preference(crate::actions::PreferenceEffect::PageReset),
        preference(crate::actions::PreferenceEffect::Outline),
        preference(crate::actions::PreferenceEffect::MenuBar),
        preference(crate::actions::PreferenceEffect::Typewriter),
        preference(crate::actions::PreferenceEffect::Spellcheck),
        preference(crate::actions::PreferenceEffect::WritingNits),
        preference(crate::actions::PreferenceEffect::WritingStreaks),
        Effect::Clipboard(crate::actions::ClipboardEffect::WriteKillRing),
        Effect::Clipboard(crate::actions::ClipboardEffect::PasteImage),
        Effect::Buffer(crate::actions::BufferEffect::Previous { finished: false }),
        Effect::Buffer(crate::actions::BufferEffect::Previous { finished: true }),
        Effect::Buffer(crate::actions::BufferEffect::NewDocument),
        Effect::Buffer(crate::actions::BufferEffect::OpenSettings),
        Effect::Buffer(crate::actions::BufferEffect::OpenCredits),
        Effect::Buffer(crate::actions::BufferEffect::OpenGuide),
        Effect::Buffer(crate::actions::BufferEffect::OpenReference),
        Effect::Daemon(crate::actions::DaemonEffect::NotifyFinished),
        Effect::Surface(crate::actions::SurfaceEffect::ShowAbout),
        Effect::Surface(crate::actions::SurfaceEffect::OpenFileChooser),
        Effect::Surface(crate::actions::SurfaceEffect::OpenFolderChooser),
        Effect::Notice(crate::actions::NoticeEffect::Toast("saved".into())),
        Effect::Notice(crate::actions::NoticeEffect::Sticky("failed".into())),
        Effect::Notice(crate::actions::NoticeEffect::Clear),
        Effect::Render(crate::actions::RenderEffect::SyncView { follow: true }),
        Effect::Render(crate::actions::RenderEffect::Reshape),
        Effect::Render(crate::actions::RenderEffect::ZoomChanged),
        Effect::Render(crate::actions::RenderEffect::Redraw),
        Effect::Render(crate::actions::RenderEffect::EditStreak),
        Effect::RunAction(Action::Save),
        Effect::OverlayAccept(OverlayKind::Goto, "a.md".into()),
        Effect::JumpToLine(3),
        Effect::RebindCommit {
            slug: "save".into(),
            binding: "C-t".into(),
            confirmed: false,
        },
        Effect::RebindReset {
            slug: "save".into(),
        },
        Effect::Recoil(RecoilDir::Left),
        Effect::TypeImpact,
        Effect::DeleteSquash,
        Effect::Gulp,
        Effect::LineLand,
        Effect::AddToDictionary("awlword".into()),
        Effect::KeepVersion {
            name: Some("draft A".into()),
        },
        Effect::FollowLink("https://example.com".into()),
        Effect::ReportProblem,
        Effect::DownloadFile,
        Effect::Export(crate::export::Format::Docx, None),
        Effect::CheckForUpdates,
        Effect::CopyPulse,
        Effect::SettingToggle {
            key: "wysiwyg".into(),
        },
        Effect::SettingValueCommit {
            key: "page_width_prose".into(),
            value: "66".into(),
        },
        Effect::SettingPathPick {
            key: "default_folder".into(),
            path: "/tmp/n".into(),
        },
        Effect::SettingRangeStep { key: "zoom".into() },
        Effect::TrashAsset {
            rel: "assets/orphan.png".into(),
        },
        Effect::RenameNoteCommit {
            new_name: "new.md".into(),
        },
        Effect::DuplicateNote,
        Effect::InsertDate,
    ]
}

#[test]
fn every_effect_lands_in_its_documented_bucket() {
    // The bucket each variant belongs to, pinned by NAME (the classify
    // match is the compile-time sweep; this is the reviewed membership).
    let applied = [
        "none",
        "new_document",
        "open_settings",
        "open_credits",
        "open_guide",
        "open_reference",
        "show_about",
        "run_action",
        "overlay_accept",
        "jump_to_line",
        "persist_caret_mode",
        "persist_page_mode",
        "persist_page_width",
        "persist_page_reset",
        "persist_outline",
        "persist_menu_bar",
        "persist_typewriter",
        "persist_spellcheck",
        "persist_writing_nits",
        "flush_writing_streaks",
        "notice_toast",
        "notice_sticky",
        "notice_clear",
        "sync_view",
        "reshape",
        "zoom_changed",
        "redraw",
        "edit_streak",
        "recoil",
        "type_impact",
        "delete_squash",
        "gulp",
        "line_land",
        "copy_pulse",
        "insert_date",
        // A range STEP applies in the shared core (unlike its
        // Toggle/Value siblings, which are Unsupported below) — see its arm.
        "setting_range_step",
    ];
    let intercepted = [
        "follow_link",
        "report_problem",
        "download_file",
        "export",
        "check_for_updates",
        "trash_asset",
        "clipboard_write",
        "clipboard_paste_image",
        "daemon_notify_finished",
    ];
    let unsupported = [
        "quit",
        "last_buffer",
        "finish_buffer",
        "save",
        "finish_save",
        "keep_version",
        "add_to_dictionary",
        "rebind_commit",
        "rebind_reset",
        "setting_toggle",
        "setting_value_commit",
        "setting_path_pick",
        "rename_note_commit",
        "duplicate_note",
        "resolve_keep_mine",
        "resolve_take_theirs",
        "review_external_change",
        "open_file_chooser",
        "open_folder_chooser",
    ];
    for e in roster() {
        let c = classify(&e);
        let expected: &[&str] = match c.class {
            EffectClass::Applied => &applied,
            EffectClass::Intercepted { .. } => &intercepted,
            EffectClass::Unsupported { .. } => &unsupported,
        };
        assert!(
            expected.contains(&c.name),
            "`{}` classified off its documented bucket",
            c.name
        );
    }
    // The three buckets partition the roster exactly (no name missing/extra).
    assert_eq!(
        roster().len(),
        applied.len() + intercepted.len() + unsupported.len()
    );
}

#[test]
fn effect_names_are_unique_and_stable() {
    let mut names: Vec<&'static str> = roster().iter().map(|e| classify(e).name).collect();
    let total = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), total, "duplicate effect name in classify");
}

/// The settings trio joins `Save`/`Finish` as effects an Isolated
/// filesystem promotes: `SettingToggle` (every key except `"keymap"`),
/// `SettingValueCommit` and `SettingPathPick` all go Unsupported → Applied
/// the identical way as `save`/`finish_save`. Everything else in
/// the roster — including a `SettingToggle{key:"keymap"}` sample, which
/// stays Unsupported even WITH the capability (a live keymap rebuild, not a
/// filesystem write) — must be untouched by the capability, so this is also
/// the law that would catch a future grant leaking into unrelated routing.
#[test]
fn isolated_filesystem_authority_promotes_only_save_and_setting_requests() {
    for kind in [
        crate::actions::SaveKind::Manual,
        crate::actions::SaveKind::Finish,
    ] {
        let effect = Effect::Persistence(crate::actions::PersistenceEffect::Save(kind));
        assert!(matches!(
            classify_for(&effect, FilesystemCapability::None).class,
            EffectClass::Unsupported { .. }
        ));
        assert_eq!(
            classify_for(&effect, FilesystemCapability::Isolated).class,
            EffectClass::Applied
        );
    }
    let promoted = [
        Effect::SettingToggle {
            key: "wysiwyg".into(),
        },
        Effect::SettingValueCommit {
            key: "page_width_prose".into(),
            value: "66".into(),
        },
        Effect::SettingPathPick {
            key: "default_folder".into(),
            path: "/tmp/n".into(),
        },
    ];
    for effect in &promoted {
        assert!(
            matches!(
                classify_for(effect, FilesystemCapability::None).class,
                EffectClass::Unsupported { .. }
            ),
            "{effect:?}: ordinary replay (no capability) must still be Unsupported"
        );
        assert_eq!(
            classify_for(effect, FilesystemCapability::Isolated).class,
            EffectClass::Applied,
            "{effect:?}: Isolated must promote it, the same shape as Save"
        );
    }
    // The one deliberate exception: keymap-flavor toggle needs a LIVE keymap
    // rebuild, not a filesystem write, so Isolated does not promote it.
    let keymap_toggle = Effect::SettingToggle {
        key: "keymap".into(),
    };
    for filesystem in [FilesystemCapability::None, FilesystemCapability::Isolated] {
        assert!(
            matches!(
                classify_for(&keymap_toggle, filesystem).class,
                EffectClass::Unsupported { .. }
            ),
            "{keymap_toggle:?} under {filesystem:?}: keymap-flavor toggle stays \
             Unsupported regardless of filesystem authority"
        );
    }
    for effect in roster() {
        let is_save = matches!(
            effect,
            Effect::Persistence(crate::actions::PersistenceEffect::Save(_))
        );
        let is_promoted_setting = matches!(
            &effect,
            Effect::SettingToggle { key } if key != "keymap"
        ) || matches!(
            effect,
            Effect::SettingValueCommit { .. } | Effect::SettingPathPick { .. }
        );
        if !is_save && !is_promoted_setting {
            assert_eq!(
                classify_for(&effect, FilesystemCapability::None).class,
                classify_for(&effect, FilesystemCapability::Isolated).class,
                "filesystem authority must not change routing for {effect:?}"
            );
        }
    }
}

/// Closes the direction `the_harness_reach_map_matches_the_
/// production_classifier` (below) leaves unpinned: THIS classifier is
/// derived from the doc, but nothing pinned the classifier BACK to the
/// interpreter that actually has to perform its promise.
/// `classify_settings` used to say `Applied` for every `toggle_key` except
/// `"keymap"`, while `ReplaySession::interpret_setting_toggle` handled a
/// hand-copied roster behind its own `_ => return` — so a key added to
/// `settings::toggle_key` without a matching interpreter arm would work
/// live and silently no-op through replay while the sidecar still reported
/// `Applied`. A no-wildcard match over EVERY [`crate::settings::SettingId`]
/// variant — no `_`/catch-all binding anywhere, the same shape
/// `settings::value_for`'s own no-wildcard readout match already uses, so a
/// 32nd variant fails to COMPILE here until it is placed in one of the three
/// buckets below, and only THEN can the sweep fail by name: non-toggle rows
/// resolve no key at all (checked against `toggle_key` directly, the same
/// claim `settings::tests::every_toggle_has_a_config_key_and_nothing_else_
/// does` makes — restated here as a sanity guard, not duplicated as a second
/// authority); every other `Toggle` row must be a key the shared toggle core
/// ([`crate::settings::flip_toggle_global`], `App::setting_toggle`'s and this
/// classifier's ONE shared owner) recognizes, and an Isolated replay must
/// promise it `Applied`.
///
/// `Keymap` used to be a NAMED, deliberate exclusion in this sweep (a live
/// keymap rebuild, not a boolean flip, so `SettingToggle{"keymap"}` stayed
/// Unsupported even Isolated) — it is a `SettingKind::Picker` now, so
/// `toggle_key` names nothing for it and it folds into the plain non-toggle
/// bucket below. The equivalent claim for its NEW door
/// (`Effect::OverlayAccept(OverlayKind::Keymap, _)`) lives in
/// `accept_class`'s own `Keymap` arm, asserted by
/// `the_harness_reach_map_matches_the_production_classifier` below.
#[test]
fn the_settings_toggle_core_handles_every_key_toggle_key_names() {
    use crate::settings::SettingId;
    for row in crate::settings::SETTINGS {
        match row.id {
            // Non-toggle rows: `toggle_key` must name nothing for them, and
            // there is nothing further for this sweep to check.
            SettingId::CaretStyle
            | SettingId::PageWidthProse
            | SettingId::PageWidthCode
            | SettingId::Zoom
            | SettingId::ScrollSensitivity
            | SettingId::DateFormat
            | SettingId::Theme
            | SettingId::Dictionary
            | SettingId::CjkReadsAs
            | SettingId::DefaultFolder
            | SettingId::ProjectsFolder
            | SettingId::ProjectRoot
            | SettingId::Keymap
            | SettingId::Keybindings
            | SettingId::ReportProblem
            | SettingId::EditConfigAsText => {
                assert!(
                    crate::settings::toggle_key(row.id).is_none(),
                    "{:?}: not a Toggle row, yet toggle_key resolved one",
                    row.name
                );
            }
            // The shared toggle core's whole domain: a key the classifier
            // promises Applied must be one `flip_toggle_global` handles.
            SettingId::PageMode
            | SettingId::TypewriterScroll
            | SettingId::ReduceMotion
            | SettingId::Wysiwyg
            | SettingId::FormatPopover
            | SettingId::InlineImages
            | SettingId::CodeLigatures
            | SettingId::Outline
            | SettingId::MenuBar
            | SettingId::Spellcheck
            | SettingId::WritingNits
            | SettingId::FileVisibility
            | SettingId::Autosave
            | SettingId::LocalHistory
            | SettingId::SessionRestore => {
                let key = crate::settings::toggle_key(row.id)
                    .unwrap_or_else(|| panic!("{:?}: expected a Toggle row's key", row.name));
                assert!(
                    crate::settings::is_core_toggle_key(key),
                    "{:?} ({key:?}): `toggle_key` names this key but the shared toggle \
                     core (`settings::flip_toggle_global`) does not recognize it — the \
                     classifier would promise `Applied` for a key the interpreter \
                     silently drops through its own `_ => return`",
                    row.name
                );
                let effect = Effect::SettingToggle {
                    key: key.to_string(),
                };
                assert_eq!(
                    classify_for(&effect, FilesystemCapability::Isolated).class,
                    EffectClass::Applied,
                    "{:?} ({key:?}): an Isolated replay must promise Applied for a \
                     core-handled key",
                    row.name
                );
                assert!(
                    matches!(
                        classify_for(&effect, FilesystemCapability::None).class,
                        EffectClass::Unsupported { .. }
                    ),
                    "{:?} ({key:?}): ordinary replay (no capability) must stay Unsupported",
                    row.name
                );
            }
        }
    }
}

#[test]
fn intercepted_effects_carry_their_payload_as_detail() {
    let follow = classify(&Effect::FollowLink("https://awl.example/g".into()));
    assert_eq!(
        follow.class,
        EffectClass::Intercepted {
            detail: "https://awl.example/g".into()
        }
    );
    let trash = classify(&Effect::TrashAsset {
        rel: "assets/o.png".into(),
    });
    assert_eq!(
        trash.class,
        EffectClass::Intercepted {
            detail: "assets/o.png".into()
        }
    );
    // Payload-free handoffs record an empty detail, not a placeholder.
    let report = classify(&Effect::ReportProblem);
    assert_eq!(
        report.class,
        EffectClass::Intercepted {
            detail: String::new()
        }
    );
}

#[test]
fn overlay_accepts_are_classified_per_kind() {
    // The headlessly-real accepts stay Applied…
    for kind in [
        OverlayKind::Goto,
        OverlayKind::Project,
        OverlayKind::History,
        OverlayKind::Theme,
        OverlayKind::Caret,
        OverlayKind::Dictionary,
        OverlayKind::CjkLang,
    ] {
        let c = classify(&Effect::OverlayAccept(kind, "v".into()));
        assert_eq!(
            c.class,
            EffectClass::Applied,
            "{kind:?} accept should be Applied"
        );
    }
    // …the live-only note move is Unsupported…
    let mv = classify(&Effect::OverlayAccept(
        OverlayKind::MoveDest,
        "inbox".into(),
    ));
    assert!(matches!(mv.class, EffectClass::Unsupported { .. }));
    // …and a kind that never emits an accept fails safe (Unsupported), so a
    // new emission aborts a strict run until consciously classified.
    let odd = classify(&Effect::OverlayAccept(OverlayKind::Spell, "word".into()));
    assert!(matches!(odd.class, EffectClass::Unsupported { .. }));
}

#[test]
fn strict_error_and_warn_line_name_the_exact_action_and_effect() {
    let c = classify(&Effect::Quit);
    let err = strict_error(&Action::Quit, &c).to_string();
    assert!(err.contains("`quit`"), "effect named: {err}");
    assert!(err.contains("Quit"), "action named: {err}");
    assert!(err.starts_with("strict replay:"), "strict prefix: {err}");

    let warn = warn_line(&Action::Quit, &c).expect("unsupported warns");
    assert!(
        warn.contains("`quit`") && warn.contains("Quit"),
        "warn names both: {warn}"
    );
    assert!(
        warn.starts_with("--keys replay:"),
        "permissive prefix: {warn}"
    );

    // Intercepted warning carries the payload; Applied warns about nothing.
    let f = classify(&Effect::FollowLink("https://x.y/z".into()));
    let warn = warn_line(&Action::FollowLink, &f).expect("intercepted warns");
    assert!(
        warn.contains("`follow_link`") && warn.contains("https://x.y/z"),
        "{warn}"
    );
    assert_eq!(warn_line(&Action::Save, &classify(&Effect::None)), None);
}

#[test]
fn missing_oracle_error_names_the_fallback_it_refuses() {
    let msg = missing_oracle_error().to_string();
    assert!(msg.starts_with("strict replay:"), "{msg}");
    assert!(msg.contains("layout oracle"), "{msg}");
    assert!(msg.contains("logical lines"), "{msg}");
}

// ── The harness-reach map is DERIVED, never hand-copied ─────────────────
//
// `docs/harness-reach.md` publishes, for a brief author, exactly what a
// `--keys` capture can and cannot witness. The half of that map covering the
// effect boundary is already owned in production by [`classify_for`]
// (and [`accept_class`] for the per-picker accepts), so the doc must be a VIEW
// of that owner rather than a second list beside it. A hand-copied table that
// drifts from the function it describes is the defect this law exists to
// fix elsewhere; the map's whole value is that a brief author can trust it.

/// The `(name, bucket)` pairs the production classifier actually produces —
/// the effect roster above, with `overlay_accept` expanded over the whole
/// `OverlayKind` roster (a no-wildcard `ALL`, so a new picker joins this map
/// automatically).
fn reach_rows() -> std::collections::BTreeMap<String, &'static str> {
    fn bucket(class: &EffectClass) -> &'static str {
        match class {
            EffectClass::Applied => "Applied",
            EffectClass::Intercepted { .. } => "Intercepted",
            EffectClass::Unsupported { .. } => "Unsupported",
        }
    }
    let mut rows: std::collections::BTreeMap<String, &'static str> = Default::default();
    for effect in roster() {
        if matches!(effect, Effect::OverlayAccept(..)) {
            continue; // expanded per picker below
        }
        let c = classify(&effect);
        rows.insert(c.name.to_string(), bucket(&c.class));
    }
    for kind in OverlayKind::ALL {
        let c = classify(&Effect::OverlayAccept(kind, String::new()));
        rows.insert(format!("{}:{kind:?}", c.name), bucket(&c.class));
    }
    rows
}

/// The map's effect table equals what production classifies, row for row.
#[test]
fn the_harness_reach_map_matches_the_production_classifier() {
    let doc = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs/harness-reach.md"),
    )
    .expect("docs/harness-reach.md is part of the deliverable");
    let mut in_table = false;
    let mut found: std::collections::BTreeMap<String, &'static str> = Default::default();
    for line in doc.lines() {
        if line.starts_with("<!-- reach-table:begin -->") {
            in_table = true;
            continue;
        }
        if line.starts_with("<!-- reach-table:end -->") {
            in_table = false;
            continue;
        }
        if !in_table || !line.starts_with("| `") {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        let name = cells[0].trim_matches('`').to_string();
        let bucket = match cells[1] {
            "Applied" => "Applied",
            "Intercepted" => "Intercepted",
            "Unsupported" => "Unsupported",
            other => panic!("unknown bucket {other:?} in the reach table row {line:?}"),
        };
        found.insert(name, bucket);
    }
    let expected = reach_rows();
    if found != expected {
        let rendered: String = expected
            .iter()
            .map(|(n, b)| format!("| `{n}` | {b} |\n"))
            .collect();
        panic!(
            "docs/harness-reach.md's effect table has drifted from \
             `replay::classify_for`/`accept_class`. Replace the rows between the \
             reach-table markers with exactly:\n{rendered}"
        );
    }
    assert!(
        found.len() >= 60,
        "the map must cover the whole effect + picker roster, not a sample; got {}",
        found.len()
    );
}
