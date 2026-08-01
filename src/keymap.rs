use std::collections::HashMap;

use winit::event::Modifiers;
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

use crate::convention::Convention;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ForwardChar,
    BackwardChar,
    NextLine,
    PreviousLine,
    LineStart,
    LineEnd,
    ForwardWord,
    BackwardWord,
    BufferStart,
    BufferEnd,
    InsertChar(char),
    Newline,
    /// SHIFT-HELD ACCEPT (`⇧↵`, item 116c) — the deliberate, footer-taught
    /// "yes, really" (restores a History row; bare `Enter` no longer does).
    /// Resolved in [`KeymapState::resolve_named`], never a catalog chord. IN
    /// THE EDITOR it rides the exact same arm as [`Action::Newline`] — see
    /// `actions::tests::alternate_accept_item116c`.
    AcceptAlternate,
    InsertTab,
    Outdent,
    DeleteBackward,
    DeleteWordBackward,
    DeleteWordForward,
    DeleteToLineStart,
    DeleteForward,
    KillLine,
    Yank,
    YankText,
    InsertImageReference(String),
    /// Undo the last edit group (Cmd+Z / C-/).
    Undo,
    Redo,
    SetMark,
    CopyRegion,
    KillRegion,
    SelectAll,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    PageScrollDown,
    PageScrollUp,
    Save,
    Quit,
    SearchForward,
    SearchBackward,
    /// Cmd-R (headline) / Cmd-Option-F (legacy): summon the find-and-replace panel
    /// — the SAME panel as isearch, with the labeled REPLACE row revealed (a MODE of
    /// the one panel, no separate chrome). A fresh open shows the replace row but
    /// keeps focus on the FIND field; pressing Cmd-R again while the panel is open
    /// jumps focus into the replacement (consumed by the shared search-key seam,
    /// `crate::search::keys::intercept`, on both drivers). Tab switches fields.
    OpenReplace,
    Cancel,
    OpenThemeMenu,
    OpenCommandPalette,
    OpenOutline,
    OpenSpellSuggest,
    ToggleCaretMode,
    OpenCaretMenu,
    OpenDictionaryMenu,
    /// Cmd-P → "Toggle spellcheck": flip the GLOBAL spell-check on/off (default
    /// ON — the escape hatch for no-squiggles-ever people). OFF silences EVERY
    /// squiggle (prose comments and code strings alike, per `spell.rs`'s ONE
    /// owner gate) and turns `Cmd-;` / a right-click into a calm no-op. A real
    /// `Action` (not the `writing_nits` sentinel hack) so it round-trips through
    /// `RunAction` unambiguously; render-only (no buffer change), sticky
    /// (persisted like `writing_nits`). No default chord (palette-summoned);
    /// rebindable via `[keys]`. See `spell.rs`.
    ToggleSpellcheck,
    TogglePageMode,
    PageWider,
    PageNarrower,
    /// RESET PAGE WIDTH — snap the measure back to the ACTIVE buffer's OWN built-in
    /// default (see [`crate::page::PageClass::default_measure`] — 70 prose / 100
    /// code) and CLEAR the sticky `page_width_prose`/`page_width_code` config
    /// override matching that SAME class entirely (back to `None`, which already
    /// means "use the built-in default"), so a future default change flows through
    /// instead of pinning a stale value. The "there's no easy way back" fix for
    /// [`PageWider`]/[`PageNarrower`]. No default chord — reachable via the palette
    /// ("Reset page width") and a DOUBLE-CLICK on the draggable page edge
    /// (pointing-not-buttons); rebindable via `[keys]`. Render-only (re-wraps).
    PageReset,
    ToggleDebug,
    ToggleOutline,
    /// "Fold section" (Cmd-Shift-E / C-c C-f): TOGGLE the collapse of the markdown SECTION
    /// enclosing the caret — the heading plus its body + nested subheadings hide to a
    /// quiet one-line summary; toggling again re-expands. VIEW state only (the rope is
    /// untouched, never on the undo timeline); a no-op off a heading / on a
    /// non-markdown buffer. See `fold.rs` + `buffer::toggle_fold_at_cursor`.
    ToggleFold,
    CollapseOtherSections,
    ToggleMenuBar,
    ToggleTypewriter,
    ToggleWritingNits,
    ShowStatsHud,
    About,
    LifetimeStats,
    WritingStreaks,
    /// Palette "Line endings…": TOGGLE the active buffer's line-ending
    /// discipline (`LF`↔`CRLF`, [`crate::buffer::Eol`]) — the rope is byte-identical
    /// either way (always pure `\n`); only the ON-DISK encoding a save restores
    /// differs. Document-level metadata, NOT an undoable edit (Cmd-Z does not
    /// restore it, mirroring VS Code); it marks the buffer dirty + bumps `version`
    /// so autosave rewrites with the new ending. No default chord — the palette IS
    /// its entry point, like Settings/About. See `buffer.rs`'s `set_eol`.
    ConvertLineEndings,
    /// Palette "Align table": RE-PAD the GFM markdown table under the caret so its
    /// `|` line up (Prettier-style monospace alignment of the SOURCE — awl never
    /// draws a grid). Finds the table block around the caret, replaces it via
    /// [`crate::markdown::align_table`] as ONE undoable edit (Cmd-Z restores the
    /// pre-align source); a calm no-op when the caret is not in a table. No default
    /// chord — the palette IS its entry point (like Settings/About); a real
    /// `Action`, independently rebindable via `[keys]`. See `markdown/`.
    AlignTable,
    /// Palette "Report a Problem": compose a `mailto:` link to the maintainer —
    /// subject `"awl problem report (v…)"`, a calm what-happened template body,
    /// and (if a local crash log exists) that log's PATH with a "please attach
    /// this file" line (`mailto:` cannot attach a file; the body never inlines
    /// the log's own content — see the crash-visibility privacy law). The pure
    /// core can't reach the crash-log directory or the OS mail client, so it
    /// signals [`crate::actions::Effect::ReportProblem`] for the live App to
    /// compose (`crashlog::report_problem_mailto`) and open through the SAME
    /// OS-handoff seam `Action::FollowLink` uses (`App::follow_link`). No
    /// document content is ever touched. No default chord (palette-only, like
    /// Settings/About); `native_only: false` — available on the web build too.
    /// Headless replay never opens anything (live-App-only). See `crashlog.rs`.
    ReportProblem,
    /// Palette "Download file" (WEB-ONLY — `web_only: true`, the inverse of
    /// `native_only`): export the ACTIVE buffer's text as a browser download —
    /// `Blob` + object URL + a synthetic `<a download>` click (`web_export.rs`).
    /// A native user already has a real file on real disk (this command is hidden
    /// there entirely — see `commands.rs`'s `web_only` field); on the web build it
    /// is the escape hatch for the no-real-filesystem/no-OS-clipboard sandbox (see
    /// WEB.md). The pure core can't touch `web_sys` (no DOM handoff seam in
    /// `ActionCtx`), so it signals a bare request
    /// ([`crate::actions::Effect::DownloadFile`]) for the live App to perform.
    /// Escapes the SCRATCH buffer too (its virtual `display_name()`). No default
    /// chord (palette-only, like Settings/About); rebindable via `[keys]`.
    /// LIVE-APP-ONLY: headless `--keys` replay never touches the DOM, so it is a
    /// no-op there — a settled capture stays byte-identical. See `web_export.rs`.
    DownloadFile,
    /// Palette "Check for Updates": the app NEVER phones home — this composes
    /// ONE static URL (the site's `/check` page, carrying `CARGO_PKG_VERSION`
    /// as a `?v=` query param — [`crate::updates::check_url`]) and hands it to
    /// the OS browser through the SAME OS-handoff seam `Action::FollowLink` /
    /// `Action::ReportProblem` use (`App::follow_link`); the actual
    /// version-comparison happens in the browser, against a static
    /// `version.json` the site regenerates at deploy — never a fetch from this
    /// binary. The pure core can't reach the fs/OS-handoff, so it signals
    /// [`crate::actions::Effect::CheckForUpdates`] for the live App to (a)
    /// record a LOCAL "last checked" marker (best-effort,
    /// `updates::record_checked`) and (b) open the browser. No document
    /// content is ever touched. No default chord (palette-only, like Report a
    /// Problem/Settings/About); `native_only: true` — the web build updates by
    /// deploy, so the command is meaningless there. Headless replay never
    /// writes the marker or opens anything (live-App-only, mirroring
    /// `ReportProblem`/`FollowLink`). See `updates.rs`.
    CheckForUpdates,
    ToggleBlockquote,
    ToggleBulletList,
    ToggleNumberedList,
    ToggleTaskList,
    ToggleHeading,
    HeadingCycle,
    ToggleCodeBlock,
    Bold,
    Italic,
    InlineCode,
    Highlight,
    Strikethrough,
    ExportWord,
    ExportHtml,
    ExportPdf,
    OpenGoto,
    OpenProject,
    OpenRecentProjects,
    OpenBrowse,
    LastBuffer,
    NewDocument,
    KeepTutorial,
    MoveFile,
    OpenRenameNote,
    DuplicateNote,
    #[allow(dead_code)] // next-phase: fired by the settings menu's "Edit config as text" row.
    OpenSettings,
    OpenSettingsMenu,
    OpenKeybindings,
    OpenCredits,
    OpenGuide,
    OpenHistory,
    /// THE WRITER'S DIFF (palette "Compare with version…", markdown buffers only):
    /// open the READ-ONLY prose-diff view comparing the CURRENT buffer against a
    /// past version — the marked-up manuscript (`crate::prosediff`: struck deletions,
    /// washed insertions, moves, folds). From the BUFFER (no overlay) it compares
    /// against the most-recent version (a loose file's newest history snapshot, or a
    /// git-managed file's HEAD via `git show`); from the open HISTORY picker it
    /// compares against the HIGHLIGHTED row. Esc returns to the live document exactly
    /// (the buffer is never touched). No default chord (a palette command like
    /// Version history / Settings), rebindable via `[keys] compare_with_version`.
    CompareVersion,
    /// Clean unused assets (summon by name, Cmd-P): open the ASSET CLEANER — a
    /// summoned, transient picker listing the ORPHAN image files under the active
    /// project (an image under an `assets/` directory that no document references, per
    /// [`crate::assets::scan`]). Enter on a row moves that file to the macOS TRASH
    /// (recoverable — never `rm`; the row leaves the list, the picker stays open). No
    /// default chord (a palette command, "Clean unused assets…", like Settings/About),
    /// rebindable via `[keys] clean_unused_assets`. See `overlay/`
    /// (`OverlayKind::Assets`) + `assets.rs`.
    OpenAssetClean,
    KeepVersion,
    /// FINISH the active buffer (the emacsclient "server-edit" convention; the emacs
    /// `C-x #` default is retired, so it is palette-only now): save it, notify any
    /// daemon `--wait` client waiting on it, and
    /// switch to the previously-open buffer (the same swap [`Action::LastBuffer`]
    /// performs). The core only does the SAVE (identically to [`Action::Save`], so
    /// history/mtime bookkeeping stays on one door); the daemon-notify + buffer-swap
    /// are caller-level (the pure core can't reach the daemon, and headless replay has
    /// none to notify). Also a palette command ("Finish file"), rebindable via
    /// `[keys]`. See `crate::daemon`.
    FinishBuffer,
    /// Cmd-click a markdown link (the advertised mouse affordance), or the emacs
    /// `C-c C-o` chord (the org-mode "open link at point" convention, kept): if the
    /// caret sits inside a markdown link, open that link's URL in the default browser.
    /// The pure core extracts the URL ([`crate::markdown::link_at`]) and signals it
    /// back as [`crate::actions::Effect::FollowLink`]; the live App performs the OS
    /// browser handoff (a user-initiated launch, not an app network fetch — the
    /// zero-network invariant holds). A caret outside every link is a calm no-op.
    /// Headless replay never opens a browser (the effect is live-App-only). Also a
    /// palette command ("Follow link"), rebindable via `[keys]`.
    FollowLink,
    InsertLink,
    InsertDate,
    BeginPrefix,
    Ignore,
}
impl Action {
    pub fn is_motion(&self) -> bool {
        matches!(
            self,
            Action::ForwardChar
                | Action::BackwardChar
                | Action::NextLine
                | Action::PreviousLine
                | Action::LineStart
                | Action::LineEnd
                | Action::ForwardWord
                | Action::BackwardWord
                | Action::BufferStart
                | Action::BufferEnd
        )
    }

    /// True when this action mutates buffer content and records undo history.
    pub fn is_edit(&self) -> bool {
        matches!(
            self,
            Action::InsertChar(_)
                | Action::Newline
                | Action::AcceptAlternate
                | Action::InsertTab
                | Action::Outdent
                | Action::DeleteBackward
                | Action::DeleteWordBackward
                | Action::DeleteWordForward
                | Action::DeleteToLineStart
                | Action::DeleteForward
                | Action::KillLine
                | Action::Yank
                | Action::YankText
                | Action::InsertImageReference(_)
                | Action::KillRegion
                | Action::AlignTable
                | Action::ToggleBlockquote
                | Action::ToggleBulletList
                | Action::ToggleNumberedList
                | Action::ToggleTaskList
                | Action::ToggleHeading
                | Action::HeadingCycle
                | Action::ToggleCodeBlock
                | Action::Bold
                | Action::Italic
                | Action::InlineCode
                | Action::Highlight
                | Action::Strikethrough
        )
    }
}

pub enum Chord {
    Single(Key, ModifiersState),
    Cx(Key, ModifiersState),
    Cc(Key, ModifiersState),
}

pub struct KeymapState {
    convention: Convention,
    in_c_x: bool,
    in_c_c: bool,
    default_single: HashMap<(Key, ModifiersState), Action>,
    default_c_x: HashMap<(Key, ModifiersState), Action>,
    default_c_c: HashMap<(Key, ModifiersState), Action>,
    override_single: HashMap<(Key, ModifiersState), Action>,
    override_c_x: HashMap<(Key, ModifiersState), Action>,
    override_c_c: HashMap<(Key, ModifiersState), Action>,
    /// THE EMACS-HANDS-ON-LINUX ROUND — the config `linux_keep_emacs` list, parsed
    /// into concrete `(key, mods)` chords: on [`Convention::Linux`], a chord in
    /// this set does NOT participate in the native-wins collision (see
    /// [`Self::linux_keeps`]) — its bare-control emacs meaning fires instead. Built
    /// by [`Self::apply_linux_keep`], consulted ONLY when `convention ==
    /// Convention::Linux` (so a Mac keymap can carry a non-empty set — e.g. a test
    /// exercising `apply_linux_keep` before switching convention — and it is still
    /// STRUCTURALLY inert there, matching "Mac convention ignores the key
    /// entirely"). NEVER truly empty by construction — [`Self::apply_linux_keep`]
    /// always seeds `linux_builtin_keep()` first (the insert-link-yields-to-
    /// kill-line floor), so an absent config keeps today's dispatch PLUS that one
    /// unconditional floor chord; every OTHER letter still needs an explicit
    /// `linux_keep_emacs`/`keymap = "emacs"` opt-in, unchanged.
    linux_keep: std::collections::HashSet<(Key, ModifiersState)>,
}

impl Default for KeymapState {
    fn default() -> Self {
        let mut km = Self {
            convention: Convention::current(),
            in_c_x: false,
            in_c_c: false,
            default_single: HashMap::new(),
            default_c_x: HashMap::new(),
            default_c_c: HashMap::new(),
            override_single: HashMap::new(),
            override_c_x: HashMap::new(),
            override_c_c: HashMap::new(),
            linux_keep: std::collections::HashSet::new(),
        };
        // Seed the unconditional built-in keep floor (see `apply_linux_keep`'s
        // doc) — so even a `KeymapState` that never has `apply_linux_keep`
        // called on it (a bare `new`/`new_with_convention`, the shape most of
        // this module's own unit tests use) still carries the floor.
        km.apply_linux_keep(&[]);
        km
    }
}

impl KeymapState {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn new_with_convention(convention: Convention) -> Self {
        let mut km = Self {
            convention,
            ..Self::default()
        };
        km.seed_defaults();
        km
    }

    pub fn with_overrides(keys: &[(String, Vec<String>)]) -> Self {
        let mut km = Self::new();
        km.apply_overrides(keys);
        km
    }

    #[cfg(test)]
    pub fn with_overrides_and_convention(
        keys: &[(String, Vec<String>)],
        convention: Convention,
    ) -> Self {
        let mut km = Self::new_with_convention(convention);
        km.apply_overrides(keys);
        km
    }

    /// [`Self::with_overrides`], ALSO applying the config `linux_keep_emacs` list
    /// (see [`Self::apply_linux_keep`]) — the real production door every live/
    /// headless call site should use once it has a [`crate::config::Config`] in
    /// hand (`App::new`, the `--keys` replay keymap built in `main/args.rs`);
    /// `with_overrides` alone
    /// stays as the simpler door for the many call sites (mostly tests) that
    /// never touch the keep-list.
    pub fn with_overrides_and_keep(keys: &[(String, Vec<String>)], keep: &[String]) -> Self {
        let mut km = Self::with_overrides(keys);
        km.apply_linux_keep(keep);
        km
    }

    /// True when this convention's NATIVE modifier alone is held (never together
    /// with the OTHER convention's own physical modifier, so the two never
    /// double-fire): [`Convention::Mac`] wants Super without Control;
    /// [`Convention::Linux`] wants Control without Super. THE ONE GATE every
    /// native policy arm below reads. Catalog default collision precedence is
    /// applied while seeding the maps; this helper remains for uncatalogued
    /// native aliases such as Cmd-P and Cmd-G.
    fn native_down(&self, state: ModifiersState) -> bool {
        match self.convention {
            Convention::Mac => {
                state.contains(ModifiersState::SUPER) && !state.contains(ModifiersState::CONTROL)
            }
            Convention::Linux => {
                state.contains(ModifiersState::CONTROL) && !state.contains(ModifiersState::SUPER)
            }
        }
    }

    /// Rebuild the catalog-default dispatch layer from the same resolved command
    /// slots every label surface reads. Platform collision policy stays here: on
    /// Linux a kept emacs chord suppresses its native claimant, while a displaced
    /// emacs chord is omitted. Duplicate effective defaults are an embedded-data
    /// bug and panic unless both rows intentionally resolve to the same action.
    fn seed_defaults(&mut self) {
        self.default_single.clear();
        self.default_c_x.clear();
        self.default_c_c.clear();

        for command in crate::commands::COMMANDS.iter() {
            let native = crate::commands::resolved_native(command, self.convention);
            let native_suppressed = self.convention == Convention::Linux
                && linux_keeps_chord_raw(&self.linux_keep, &native);
            let emacs_displaced = self.convention == Convention::Linux
                && linux_displaces_emacs_default_raw(command.emacs, &self.linux_keep);

            if !emacs_displaced {
                self.insert_default(command.emacs, command.action.clone(), command.name);
            }
            if !native_suppressed {
                self.insert_default(&native, command.action.clone(), command.name);
            }
        }

        let shifted: Vec<_> = self
            .default_single
            .iter()
            .filter(|((_, mods), _)| !mods.contains(ModifiersState::SHIFT))
            .map(|((key, mods), action)| {
                ((key.clone(), *mods | ModifiersState::SHIFT), action.clone())
            })
            .collect();
        for (chord, action) in shifted {
            self.default_single.entry(chord).or_insert(action);
        }

        for command in crate::commands::COMMANDS.iter() {
            self.insert_control_super_variants(command.emacs, command.action.clone(), command.name);
            if command.native.starts_with("C-") {
                self.insert_control_super_variants(
                    command.native,
                    command.action.clone(),
                    command.name,
                );
            }
        }
    }

    fn insert_default(&mut self, spec: &str, action: Action, name: &str) {
        if spec.trim().is_empty() {
            return;
        }
        let chord = parse_binding(spec).unwrap_or_else(|e| {
            panic!("assets/keymap-defaults.toml: {name:?} has invalid chord {spec:?}: {e}")
        });
        match chord {
            Chord::Single(k, m) => {
                insert_default_entry(
                    &mut self.default_single,
                    (k.clone(), m),
                    action.clone(),
                    name,
                    spec,
                );
            }
            Chord::Cx(k, m) => {
                insert_default_entry(&mut self.default_c_x, (k, m), action, name, spec);
            }
            Chord::Cc(k, m) => {
                insert_default_entry(&mut self.default_c_c, (k, m), action, name, spec);
            }
        }
    }

    fn insert_control_super_variants(&mut self, spec: &str, action: Action, name: &str) {
        if spec.trim().is_empty() {
            return;
        }
        let chord = parse_binding(spec).unwrap_or_else(|e| {
            panic!("assets/keymap-defaults.toml: {name:?} has invalid chord {spec:?}: {e}")
        });
        let add_super = |mods: ModifiersState| {
            mods.contains(ModifiersState::CONTROL)
                .then_some(mods | ModifiersState::SUPER)
        };
        match chord {
            Chord::Single(k, m) => {
                if let Some(m) = add_super(m) {
                    self.default_single
                        .entry((k.clone(), m))
                        .or_insert(action.clone());
                    self.default_single
                        .entry((k, m | ModifiersState::SHIFT))
                        .or_insert(action);
                }
            }
            Chord::Cx(k, m) => {
                if let Some(m) = add_super(m) {
                    self.default_c_x
                        .entry((k.clone(), m))
                        .or_insert(action.clone());
                    self.default_c_x
                        .entry((k, m | ModifiersState::SHIFT))
                        .or_insert(action);
                }
            }
            Chord::Cc(k, m) => {
                if let Some(m) = add_super(m) {
                    self.default_c_c
                        .entry((k.clone(), m))
                        .or_insert(action.clone());
                    self.default_c_c
                        .entry((k, m | ModifiersState::SHIFT))
                        .or_insert(action);
                }
            }
        }
    }

    /// Apply (or RE-apply, on a live config reload) the `[keys]` rebinds. Each entry
    /// maps an action NAME (the command-palette name, slugified) to a LIST of up to 2
    /// chords (slot 1 = native, slot 2 = emacs); each valid chord OVERRIDES that
    /// action's binding (additively — both the configured chords AND the default
    /// still fire). An unknown action or a bad chord is reported to stderr and
    /// SKIPPED, keeping the default — never a crash. Only the FIRST TWO chords of a
    /// list are honoured (the model is capped at 2). Clears any prior overrides first
    /// so a reload reflects exactly the current file.
    pub fn apply_overrides(&mut self, keys: &[(String, Vec<String>)]) {
        self.override_single.clear();
        self.override_c_x.clear();
        self.override_c_c.clear();
        for (name, chords) in keys {
            let Some(action) = crate::commands::action_for_name(name) else {
                eprintln!("config [keys]: unknown action {name:?}; ignored");
                continue;
            };
            for chord in chords.iter().take(2) {
                match parse_binding(chord) {
                    Ok(Chord::Single(k, m)) => {
                        self.override_single.insert((k, m), action.clone());
                    }
                    Ok(Chord::Cx(k, m)) => {
                        self.override_c_x.insert((k, m), action.clone());
                    }
                    Ok(Chord::Cc(k, m)) => {
                        self.override_c_c.insert((k, m), action.clone());
                    }
                    Err(e) => {
                        eprintln!("config [keys]: {name} = {chord:?}: {e}; keeping default");
                    }
                }
            }
        }
    }

    /// Apply (or RE-apply, on a live config reload) the `linux_keep_emacs` list —
    /// THE PER-CHORD DOOR the emacs-hands-on-Linux round adds: under
    /// [`Convention::Linux`], every chord named here is EXEMPTED from the
    /// native-wins collision (`native_down`'s displacement), so its bare-control
    /// emacs meaning keeps firing instead of the native chord that would
    /// otherwise claim that letter (see the module's collision-table doc). A
    /// chord is a plain SINGLE spec (`"C-f"`, no `C-x`/`C-c` prefix — the
    /// collision only ever touches single Ctrl-letter chords); a bad/unparseable
    /// entry, or one that isn't a single chord, is reported to stderr and
    /// SKIPPED (never a crash), mirroring [`Self::apply_overrides`]'s leniency.
    /// On [`Convention::Mac`] the list is parsed but the set stays
    /// consultable-yet-inert — [`Self::linux_keeps`] gates on convention too, so
    /// even a stray non-empty set can never fire there (belt + suspenders with
    /// the convention check at the call site in [`Self::resolve`]).
    ///
    /// THE INSERT-LINK-YIELDS-TO-KILL-LINE ROUND: clears any prior keep-set
    /// first (so a reload reflects exactly the current file), then ALWAYS
    /// re-seeds `linux_builtin_keep()` before layering `keep` on top — the
    /// built-in floor is UNREMOVABLE by this function, whether called with the
    /// full `Config::effective_linux_keep()` composition, a hand-rolled test
    /// list, or an empty one. This is what makes the floor real even for a
    /// caller (a bare unit test, `linux_emacs_preset_keep()` applied on its
    /// own) that never threads it through `Config` at all.
    pub fn apply_linux_keep(&mut self, keep: &[String]) {
        self.linux_keep.clear();
        for chord in linux_builtin_keep()
            .iter()
            .copied()
            .chain(keep.iter().map(String::as_str))
        {
            match parse_binding(chord) {
                Ok(Chord::Single(k, m)) => {
                    self.linux_keep.insert((k, m));
                }
                Ok(_) => {
                    eprintln!(
                        "config linux_keep_emacs: {chord:?}: only a single chord (no C-x/C-c prefix) is supported; ignored"
                    );
                }
                Err(e) => {
                    eprintln!("config linux_keep_emacs: {chord:?}: {e}; ignored");
                }
            }
        }
        self.seed_defaults();
    }

    fn linux_keeps(&self, key: &Key, state: ModifiersState) -> bool {
        self.convention == Convention::Linux && self.linux_keep.contains(&(canon_key(key), state))
    }

    pub fn in_prefix(&self) -> bool {
        self.in_c_x || self.in_c_c
    }

    /// True when `key` — interpreted as the UN-COMPOSED logical key while Alt/Meta is
    /// held — would resolve to a real Meta (Option) chord rather than self-insert.
    ///
    /// This exists for the LIVE macOS Option dead-key fix (`app.rs`): Option composes
    /// a letter into a glyph (Option-f -> 'ƒ'), so `event.logical_key` is the composed
    /// char and a Meta chord would never match. The app asks this of the key WITHOUT
    /// Option composition (`key_without_modifiers`): if it IS a Meta chord, the app
    /// feeds the un-composed key to [`resolve`]; otherwise it keeps the composed char
    /// so Option-accent text INPUT (Option-e -> é) still types.
    ///
    /// Since the identity round RETIRED the built-in Option-letter layer (macOS owns
    /// those keys for typing), there are NO default Meta chords left — a key is a Meta
    /// chord ONLY when a config `[keys]` rebind reclaims it with Meta (Alt). So an
    /// unbound Option-letter always keeps its composed glyph and self-inserts, while a
    /// user-configured Option chord is still un-composed to match. Keyed by the
    /// canonical key. The headless `--keys` path already sends the un-composed key +
    /// ALT, so this predicate is only consulted live.
    pub fn is_meta_chord(&self, key: &Key) -> bool {
        let k = canon_key(key);
        self.override_single
            .keys()
            .any(|(mk, ms)| *mk == k && ms.contains(ModifiersState::ALT))
    }

    pub fn resolve(&mut self, logical: &Key, mods: &Modifiers) -> Action {
        let state = mods.state();
        if !self.in_c_x && !self.in_c_c {
            let chord = (canon_key(logical), state);
            if let Some(a) = self.override_single.get(&chord) {
                return a.clone();
            }
            if let Some(a) = self.default_single.get(&chord) {
                return a.clone();
            }
        }
        let ctrl = state.contains(ModifiersState::CONTROL);
        let alt = state.contains(ModifiersState::ALT);
        let sup = state.contains(ModifiersState::SUPER);
        let shift = state.contains(ModifiersState::SHIFT);
        let native = self.native_down(state) && !self.linux_keeps(logical, state);

        // MID-PREFIX (C-x ...): interpret this key as the SECOND key BEFORE the
        // global Super shortcuts below. Otherwise a Cmd combo pressed mid-prefix
        // (Cmd+C/V/Z/P/zoom) would fire its global shortcut AND leave the prefix
        // armed (the early `return` never clears `in_c_x`), so the NEXT key is
        // wrongly swallowed as a C-x second key — a stuck-prefix bug. With the
        // check here, an undefined `C-x <combo>` cancels and clears the prefix.
        //
        // THE C-x DEFAULTS ARE RETIRED (identity round): the static second-key
        // arms are gone, so C-x is now a bare, defaultless prefix — the MACHINERY
        // (prefix state + the `c_x` config-override map + the which-key panel) is
        // KEPT so a `[keys]` "C-x <key>" line reclaims any chord, but WITHOUT a
        // config binding a C-x sequence just cancels quietly.
        if self.in_c_x {
            self.in_c_x = false;
            let chord = (canon_key(logical), state);
            if let Some(a) = self.override_c_x.get(&chord) {
                return a.clone();
            }
            if let Some(a) = self.default_c_x.get(&chord) {
                return a.clone();
            }
            return Action::Cancel;
        }

        if self.in_c_c {
            self.in_c_c = false;
            let chord = (canon_key(logical), state);
            if let Some(a) = self.override_c_c.get(&chord) {
                return a.clone();
            }
            if let Some(a) = self.default_c_c.get(&chord) {
                return a.clone();
            }
            return Action::Cancel;
        }

        if native {
            match logical {
                Key::Character(s) if s.as_str() == "+" => return Action::ZoomIn,
                Key::Character(s) if s.as_str() == "_" => return Action::ZoomOut,
                _ => {}
            }
        }

        // Cmd-P (Super+P): summon the COMMAND PALETTE. This is its OWN dedicated
        // key — NOT a C-x chord — so it never disturbs the prefix bindings. 'p' is
        // free under Super (undo=z, zoom ==/+/-/0, clipboard=c/x/v), so no
        // collision. Plain (no Shift) — Cmd-Shift-P is Switch project, above.
        if native
            && let Key::Character(s) = logical
            && matches!(s.chars().next(), Some('p') | Some('P'))
        {
            return Action::OpenCommandPalette;
        }

        if native
            && !shift
            && let Key::Character(s) = logical
            && s.starts_with('.')
        {
            return Action::Cancel;
        }

        if native
            && alt
            && let Key::Character(s) = logical
            && matches!(s.chars().next(), Some('i') | Some('I'))
        {
            return Action::ShowStatsHud;
        }

        if native
            && alt
            && let Key::Character(s) = logical
            && matches!(s.chars().next(), Some('f') | Some('F'))
        {
            return Action::OpenReplace;
        }

        if native
            && !alt
            && let Key::Character(s) = logical
            && matches!(s.chars().next(), Some('g') | Some('G'))
        {
            return if shift {
                Action::SearchBackward
            } else {
                Action::SearchForward
            };
        }

        match logical {
            Key::Named(named) => self.resolve_named(*named, ctrl, alt, state),
            Key::Character(s) => self.resolve_char(s, ctrl, alt, sup),
            _ => Action::Ignore,
        }
    }

    fn resolve_named(
        &mut self,
        named: NamedKey,
        ctrl: bool,
        alt: bool,
        state: ModifiersState,
    ) -> Action {
        if let NamedKey::Space = named
            && ctrl
        {
            return Action::SetMark;
        }
        let sup = state.contains(ModifiersState::SUPER);
        match named {
            NamedKey::ArrowLeft => {
                if state.contains(ModifiersState::CONTROL) {
                    Action::BackwardWord
                } else {
                    Action::BackwardChar
                }
            }
            NamedKey::ArrowRight => {
                if state.contains(ModifiersState::CONTROL) {
                    Action::ForwardWord
                } else {
                    Action::ForwardChar
                }
            }
            NamedKey::ArrowUp => Action::PreviousLine,
            NamedKey::ArrowDown => Action::NextLine,
            // THE LINUX-NATIVE override for "Document start"/"Document end"
            // (`commands::LINUX_NATIVE_OVERRIDE`): Ctrl-Home/Ctrl-End is the
            // gedit/VS Code/GTK convention for buffer start/end — NOT the naive
            // Cmd→Ctrl translation of Cmd-Up/Down (which would land on Ctrl-Up/Down,
            // an unclaimed but non-idiomatic chord). Convention-gated (never fires
            // on Mac, where Cmd-Up/Down already owns this) and CHECKED BEFORE the
            // unconditional Home/End arms below, so plain Home/End keep meaning
            // line start/end on every convention — only the CTRL-held combination
            // differs by convention.
            NamedKey::Home if self.convention == Convention::Linux && ctrl => Action::BufferStart,
            NamedKey::End if self.convention == Convention::Linux && ctrl => Action::BufferEnd,
            NamedKey::Home => Action::LineStart,
            NamedKey::End => Action::LineEnd,
            NamedKey::PageUp => Action::PageScrollUp,
            NamedKey::PageDown => Action::PageScrollDown,
            NamedKey::Enter if state.contains(ModifiersState::SHIFT) => Action::AcceptAlternate,
            NamedKey::Enter => Action::Newline,
            // Ctrl-Tab: switch to the LAST (previously-open) buffer — the native
            // slot-1 door (the emacs `C-x b` default is retired). Checked before the
            // indent arms so it never inserts a tab. Native-only in practice: a
            // browser grabs Ctrl-Tab on the web build, where the palette is the door.
            // Shift-Tab OUTDENTS a list level (Tab indents); on a plain line it strips
            // up to two leading spaces (a no-op with none).
            NamedKey::Tab if state.contains(ModifiersState::SHIFT) => Action::Outdent,
            NamedKey::Tab => Action::InsertTab,
            NamedKey::Backspace if sup => Action::DeleteToLineStart,
            NamedKey::Backspace if alt || state.contains(ModifiersState::CONTROL) => {
                Action::DeleteWordBackward
            }
            NamedKey::Backspace => Action::DeleteBackward,
            NamedKey::Delete if alt || state.contains(ModifiersState::CONTROL) => {
                Action::DeleteWordForward
            }
            NamedKey::Delete => Action::DeleteForward,
            NamedKey::Space if !alt => Action::InsertChar(' '),
            NamedKey::Space => Action::Ignore,
            NamedKey::Escape => Action::Cancel,
            _ => Action::Ignore,
        }
    }

    fn resolve_char(&mut self, s: &str, ctrl: bool, alt: bool, sup: bool) -> Action {
        let Some(c) = s.chars().next() else {
            return Action::Ignore;
        };
        let lower = c.to_ascii_lowercase();

        if ctrl && !alt {
            return match lower {
                'd' => Action::DeleteForward,
                'k' => Action::KillLine,
                'v' => Action::PageScrollDown,
                'g' => Action::Cancel,
                'x' => {
                    self.in_c_x = true;
                    Action::BeginPrefix
                }
                'c' => {
                    self.in_c_c = true;
                    Action::BeginPrefix
                }
                _ => Action::Ignore,
            };
        }

        // THE UNBOUND-SUPER SWALLOW GUARD (keybinding audit, 2026-07): every bound
        // Cmd-<x> chord already returned earlier in `resolve` (Cmd-Z, Cmd-S, zoom,
        // Cmd-P, Cmd-B/I/E, …) or via a `[keys]` override (consulted before dispatch
        // ever reaches here). Reaching here WITH Super held means the chord truly
        // has no meaning — mac convention is that an unhandled Cmd combo is inert
        // (at most a beep), never text, so ⌘H/⌘K/⌘D/… must NOT type their letter
        // into the document. This intentionally also swallows Cmd+Option combos
        // (Option's dead-key composition doesn't apply once Cmd is held — a
        // Cmd-chord reads as a shortcut attempt, not typing) and Cmd+Control
        // combos with no ctrl arm above. A bare Control chord (no Super) is NOT
        // affected — it already fell through the `ctrl && !alt` match above with
        // its own `Ignore` default.
        //
        // ⌘K WAS RESERVED here (unbound, falling into this guard) since the
        // keybinding-idiom audit's W1 — Bear/Craft/Notion/Things/Ulysses/Slack all
        // spend Cmd-K on insert/edit-link, the single strongest writer-cluster
        // chord awl didn't yet claim. LINKS V2 spent it: Cmd-K now resolves to
        // `Action::InsertLink` in the native-doors block above, so it no longer
        // reaches this guard.
        if sup {
            return Action::Ignore;
        }

        if !c.is_control() {
            Action::InsertChar(c)
        } else {
            Action::Ignore
        }
    }
}

fn insert_default_entry(
    map: &mut HashMap<(Key, ModifiersState), Action>,
    chord: (Key, ModifiersState),
    action: Action,
    name: &str,
    spec: &str,
) {
    if let Some(existing) = map.get(&chord) {
        assert_eq!(
            existing, &action,
            "assets/keymap-defaults.toml: conflicting effective default {spec:?} for {name:?}: {existing:?} versus {action:?}"
        );
        return;
    }
    map.insert(chord, action);
}

fn linux_keeps_chord_raw(
    keep: &std::collections::HashSet<(Key, ModifiersState)>,
    chord_spec: &str,
) -> bool {
    match parse_binding(chord_spec) {
        Ok(Chord::Single(k, m)) => keep.contains(&(k, m)),
        _ => false,
    }
}

fn linux_displaces_emacs_default_raw(
    emacs: &str,
    keep: &std::collections::HashSet<(Key, ModifiersState)>,
) -> bool {
    let Some(first) = emacs.split_whitespace().next() else {
        return false;
    };
    let Ok((key, mods)) = crate::keyspec::parse_chord(first) else {
        return false;
    };
    if mods.state() != ModifiersState::CONTROL {
        return false;
    }
    let Key::Character(s) = &key else {
        return false;
    };
    s.chars().next().is_some_and(|c| {
        LINUX_DISPLACED_LETTERS.contains(&c.to_ascii_lowercase())
            && !keep.contains(&(canon_key(&key), mods.state()))
    })
}

/// The LETTERS the table above displaces (every `Ctrl-<letter>` whose native
/// meaning wins on [`Convention::Linux`]) — the ONE data owner both
/// `tests::linux_collision_table_matches_the_documented_displaced_list` (which
/// still separately pins EACH letter's resolved `Action`) and
/// [`linux_displaces_emacs_default`] (the LABEL-TRUTH half — is an emacs
/// default worth SHOWING under this convention) read, so the dispatch table and
/// the label truth can never silently drift apart. `k` is deliberately NOT
/// here — see `linux_builtin_keep()`'s doc for why Insert link's Ctrl-K is a
/// third, unconditionally-kept case rather than an ordinary displaced letter.
pub(crate) const LINUX_DISPLACED_LETTERS: &[char] = &[
    's', 'p', 'n', 'w', 'f', 'e', 'a', 'g', 'r', 'b', 'c', 'x', 'v',
];

/// THE INSERT-LINK-YIELDS-TO-KILL-LINE ROUND (settled — the user's own call:
/// "kill-line is too load-bearing for emacs hands to lose by default") — chords
/// that keep their EMACS meaning on [`Convention::Linux`] UNCONDITIONALLY,
/// independent of `linux_keep_emacs`/the `keymap` flavor preset. Currently just
/// `C-k` (Kill line survives Links v2's Cmd-K spend): unlike every letter in
/// [`LINUX_DISPLACED_LETTERS`] (which a user must opt BACK into via
/// `linux_keep_emacs`/`keymap = "emacs"` to keep), `C-k` never displaces at
/// all out of the box, on EITHER keymap flavor — the native Insert-link chord
/// simply has NO effective Linux binding by default (still one `[keys]
/// insert_link = "C-k"` line away for a Linux hand who explicitly wants the
/// trade — a `[keys]` override is consulted before this floor, same as every
/// other override).
///
/// Consumed from TWO structurally separate places that must agree (mirroring
/// how [`LINUX_DISPLACED_LETTERS`] itself already feeds both the dispatch
/// table and the label-truth functions): [`KeymapState::apply_linux_keep`]
/// seeds it UNCONDITIONALLY on every call (the dispatch half — a reload can
/// never clear it away) and [`crate::config::Config::effective_linux_keep`]
/// seeds it into the composed keep-list it returns (the label half —
/// `commands::join_slots_truthful` never touches `KeymapState` directly, so
/// it needs its own copy of the same guarantee). `Convention::Mac` never
/// consults `linux_keep` at all, so this is structurally inert there — Cmd-K
/// stays Insert link on Mac, unconditionally.
///
/// THE KEYMAP-DEFAULTS-AS-DATA ROUND: this is now a thin accessor over
/// [`crate::keymap_defaults::linux_builtin_keep`] (itself parsed once from
/// the embedded `assets/keymap-defaults.toml`'s `linux_builtin_keep` array)
/// rather than a literal `const` — the value (`["C-k"]`) is unchanged, only
/// where it lives moved, so every call site needed only `()` added.
pub(crate) fn linux_builtin_keep() -> &'static [&'static str] {
    crate::keymap_defaults::linux_builtin_keep()
}

/// THE WEB CHORD SANITY ROUND, Tier 3 — is `emacs` (a command's static slot-2
/// text, e.g. `"C-s"` or the `"C-c C-o"` prefix sequence) quietly DISPLACED under
/// [`Convention::Linux`]? Checks only the emacs default's FIRST key: a bare
/// (no Shift/Alt/Super) `Ctrl-<letter>` whose letter appears in
/// [`LINUX_DISPLACED_LETTERS`] is displaced — this covers both a single-chord
/// default (`"C-s"`) and a prefix sequence whose FIRST key is itself claimed
/// (`"C-c C-o"`: Ctrl-C now resolves straight to Copy, so the whole sequence
/// never arms). `false` for an empty/unparsable emacs slot, or a modified chord
/// (`"C-/"`, `"C-y"`) outside the displaced-letter set.
///
/// `keep` is the config `linux_keep_emacs` list (THE EMACS-HANDS-ON-LINUX
/// per-chord door) — a chord named there is NEVER displaced, regardless of
/// whether its letter is in [`LINUX_DISPLACED_LETTERS`] (checked via
/// [`linux_keeps_chord`], the SAME canonical-compare helper the label owner's
/// native-suppression half uses, so the two directions of this round's fix can
/// never disagree about what "kept" means). Pure — the label-truth owner
/// (`commands::join_slots_truthful`) is the only caller; mirrors the dispatch
/// collision table structurally, never re-derives it.
pub(crate) fn linux_displaces_emacs_default(emacs: &str, keep: &[String]) -> bool {
    let Some(first) = emacs.split_whitespace().next() else {
        return false;
    };
    let Ok((key, mods)) = crate::keyspec::parse_chord(first) else {
        return false;
    };
    if mods.state() != ModifiersState::CONTROL {
        return false; // must be a BARE Ctrl chord — no Shift/Alt/Super riders.
    }
    let Key::Character(s) = &key else {
        return false;
    };
    let letter_displaced = s
        .chars()
        .next()
        .is_some_and(|c| LINUX_DISPLACED_LETTERS.contains(&c.to_ascii_lowercase()));
    letter_displaced && !linux_keeps_chord(keep, first)
}

/// Is `chord_spec` (a raw chord string, e.g. `"C-f"` or a command's resolved
/// native chord like `"Ctrl-F"`) present in the LINUX KEEP-LIST `keep`, compared
/// CANONICALLY ([`crate::keyspec::canonical_binding`], so `"C-f"` == `"Ctrl-f"`
/// == `"Control-F"`)? `false` for an empty/unparsable `chord_spec` on EITHER
/// side. The ONE comparison both halves of the emacs-hands-on-Linux label fix
/// share: [`linux_displaces_emacs_default`] (does a kept chord stop displacing
/// the emacs default?) and `commands::join_slots_truthful`'s native-suppression
/// check (does a kept chord stop the NATIVE command from advertising it?) — so
/// the two directions can never quietly disagree about what "kept" means.
pub(crate) fn linux_keeps_chord(keep: &[String], chord_spec: &str) -> bool {
    let Some(want) = crate::keyspec::canonical_binding(chord_spec) else {
        return false;
    };
    keep.iter()
        .any(|k| crate::keyspec::canonical_binding(k).as_deref() == Some(want.as_str()))
}

/// THE KEYMAP FLAVOR ROUND — a config `keymap = "native" | "emacs"` PRESET,
/// orthogonal to [`Convention`] (which decides whether slot 1 SPEAKS ⌘-chords
/// or Ctrl-chords). `Native` (the default) is today's behavior byte-identical.
/// `Emacs` widens the emacs-hands-on-Linux `linux_keep_emacs` PER-CHORD door
/// (see [`KeymapState::apply_linux_keep`]/[`linux_keeps_chord`] above) into a
/// whole-catalog PRESET: every chord [`LINUX_DISPLACED_LETTERS`] names keeps
/// its emacs meaning, unioned with the user's own explicit `linux_keep_emacs`
/// entries — see `crate::config::Config::effective_linux_keep`, THE ONE
/// COMPOSITION OWNER (this module stays unaware of the config field entirely;
/// it only ever sees the already-composed `keep` list `with_overrides_and_keep`/
/// `apply_linux_keep` take). Inert on [`Convention::Mac`] structurally, same as
/// `linux_keep_emacs` itself — no collisions exist there to keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeymapFlavor {
    #[default]
    Native,
    Emacs,
}

impl KeymapFlavor {
    pub fn parse(s: &str) -> Option<KeymapFlavor> {
        match s.trim().to_ascii_lowercase().as_str() {
            "native" => Some(KeymapFlavor::Native),
            "emacs" => Some(KeymapFlavor::Emacs),
            _ => None,
        }
    }

    pub fn config_name(self) -> &'static str {
        match self {
            KeymapFlavor::Native => "native",
            KeymapFlavor::Emacs => "emacs",
        }
    }
}

/// The `Emacs` flavor's PRESET keep-list: every `Ctrl-<letter>` chord
/// [`LINUX_DISPLACED_LETTERS`] names, formatted as a plain single-chord spec
/// (`"C-f"`) ready for [`KeymapState::apply_linux_keep`]/[`linux_keeps_chord`].
/// Derived FROM the displaced-letters table itself — NEVER hand-copied — so a
/// future change to the collision table flows into the preset automatically
/// (the no-drift law this round's tests pin: the preset always equals the
/// displaced set, letter for letter). Deliberately does NOT include `C-k` —
/// `linux_builtin_keep()` covers it unconditionally, on EITHER flavor, so it
/// has no business in a flavor-gated preset; `Config::effective_linux_keep`
/// unions both in regardless of which flavor is active.
pub fn linux_emacs_preset_keep() -> Vec<String> {
    LINUX_DISPLACED_LETTERS
        .iter()
        .map(|c| format!("C-{c}"))
        .collect()
}

fn canon_key(key: &Key) -> Key {
    match key {
        Key::Character(s) => Key::Character(SmolStr::new(s.to_lowercase())),
        other => other.clone(),
    }
}

fn key_is_char(key: &Key, c: char) -> bool {
    matches!(key, Key::Character(s) if s.eq_ignore_ascii_case(&c.to_string()))
}

/// Parse a config CHORD STRING into a [`Chord`] keyed for the override maps. Reuses
/// the headless [`crate::keyspec::parse_chord`] so config chords and `--keys` chords
/// share one grammar. Two shapes are accepted (matching the keymap's prefix model):
/// a single chord (`"C-t"`, `"M-g"`), or a `C-x`/`C-c` prefix plus one key (`"C-x g"`,
/// `"C-c C-o"`). Anything else (an unsupported prefix, 3+ chords, an empty/garbled
/// token) is an `Err(String)` the caller reports while keeping the default — never a panic.
pub fn parse_binding(spec: &str) -> Result<Chord, String> {
    let toks: Vec<&str> = spec.split_whitespace().collect();
    match toks.as_slice() {
        [one] => {
            let (k, m) = crate::keyspec::parse_chord(one).map_err(|e| e.to_string())?;
            Ok(Chord::Single(canon_key(&k), m.state()))
        }
        [a, b] => {
            let (ka, ma) = crate::keyspec::parse_chord(a).map_err(|e| e.to_string())?;
            let is_cx = ma.state() == ModifiersState::CONTROL && key_is_char(&ka, 'x');
            let is_cc = ma.state() == ModifiersState::CONTROL && key_is_char(&ka, 'c');
            if !is_cx && !is_cc {
                return Err(format!(
                    "only the C-x / C-c prefixes are supported for two-chord bindings, got {a:?}"
                ));
            }
            let (kb, mb) = crate::keyspec::parse_chord(b).map_err(|e| e.to_string())?;
            if is_cx {
                Ok(Chord::Cx(canon_key(&kb), mb.state()))
            } else {
                Ok(Chord::Cc(canon_key(&kb), mb.state()))
            }
        }
        [] => Err("empty binding".to_string()),
        _ => Err(format!(
            "expected one chord or 'C-x <key>', got {} chords",
            toks.len()
        )),
    }
}

#[cfg(test)]
mod tests;
