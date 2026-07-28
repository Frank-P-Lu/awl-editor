use std::path::PathBuf;

use anyhow::Result;

use crate::args::Mode;
use crate::buffer::Buffer;
use crate::capture::{self, CaptureOpts};
use crate::config::Config;
use crate::keymap::Action;
use crate::{actions, app, bench};

/// Build the editor buffer. Refused files become unbound scratch buffers so a
/// replayed save can never overwrite them.
pub(crate) fn load_buffer(file: &Option<PathBuf>) -> Buffer {
    match file {
        Some(p) => match crate::openable::classify(p).refusal_message() {
            Some(msg) => {
                eprintln!("{msg} — opening a scratch buffer instead");
                Buffer::scratch()
            }
            None => Buffer::from_file(p),
        },
        None => Buffer::scratch(),
    }
}

pub(crate) fn resolve_root(root: &Option<PathBuf>, file: &Option<PathBuf>) -> PathBuf {
    if let Some(r) = root {
        return r.clone();
    }
    if let Some(f) = file {
        if crate::fs::active().is_dir(f) {
            return f.clone();
        }
        if let Some(p) = f.parent()
            && !p.as_os_str().is_empty()
        {
            return p.to_path_buf();
        }
    }
    crate::fs::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// THE ONE launch-precedence law (item 76), for the WINDOWED launch door
/// only (`Mode::Windowed` in [`run`]):
///
/// 1. **EXPLICIT TARGET WINS** — `--root`, or a file/dir argument (`awl .`
///    included) — delegates to [`resolve_root`], unaffected by anything
///    remembered.
/// 2. **ARGUMENT-FREE LAUNCH RESTORES** — bare `awl`: the remembered active
///    folder (`remembered`, from `crate::session::remembered_root`, gated by
///    the caller on `Config::session_restore_on()`) wins if there is one.
/// 3. **FIRST RUN** — bare launch, nothing remembered (a fresh install, or
///    the session kill-switch is off): `default_folder` (the resolved
///    `--default-folder`/config/`~/notes` value).
///
/// The DOCUMENT half of a bare launch (which file becomes active, the rest
/// parked behind it) is owned by `App::apply_session_restore`, reading the
/// SAME underlying session state — see `app/session.rs`'s module doc for why
/// the two halves can never disagree.
pub(crate) fn resolve_launch_context(
    root: &Option<PathBuf>,
    file: &Option<PathBuf>,
    remembered: Option<&std::path::Path>,
    default_folder: &std::path::Path,
) -> PathBuf {
    if root.is_some() || file.is_some() {
        return resolve_root(root, file);
    }
    match remembered {
        Some(p) => p.to_path_buf(),
        None => default_folder.to_path_buf(),
    }
}

pub(crate) fn resolve_workspace(workspace: &Option<PathBuf>, root: &std::path::Path) -> PathBuf {
    if let Some(w) = workspace {
        return w.clone();
    }
    match root.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => root.to_path_buf(),
    }
}

struct ReplayResult {
    zoom: Option<f32>,
    selection: Option<((usize, usize), (usize, usize))>,
    search_query: Option<String>,
    search_case: bool,
    /// Whether the replay left the search panel in REPLACE mode (Cmd-R / Tab /
    /// Cmd-Option-F — all drivable through the shared search-key seam).
    replace_active: bool,
    replacement: String,
    editing_replacement: bool,
    overlay: Option<crate::overlay::OverlayState>,
    accept: Option<(crate::overlay::OverlayKind, String)>,
    /// How many buffers are open at the end of the replay (the active `buffer`
    /// + everything the MULTI-BUFFER REGISTRY still has backgrounded) — feeds
    ///   the sidecar `buffers.open` count. Stays `1` for any replay that never
    ///   drives a Goto accept, so a plain `--screenshot` (no `--keys`, or keys
    ///   that never open a second file) is unaffected.
    buffers_open: usize,
    #[allow(dead_code)]
    intercepts: Vec<crate::replay::Intercept>,
    replay_skips: Vec<crate::replay::SkippedEffect>,
    #[allow(dead_code)]
    warnings: Vec<String>,
}

fn park_active(buffer: &mut Buffer, registry: &mut crate::buffers::BufferRegistry<()>) {
    if let Some(key) = crate::buffers::BufferKey::of(buffer) {
        let old = std::mem::replace(buffer, Buffer::scratch());
        registry.park(
            key,
            crate::buffers::Entry {
                buffer: old,
                extra: (),
            },
        );
    }
}

/// Replay a parsed `--keys` CHORD stream against `buffer` — each chord either
/// consumed by the SEARCH GUARD (the shared `crate::search::keys::intercept`
/// seam, while the isearch panel is open) or resolved through `km` and applied
/// THROUGH the shared `actions::apply_core` seam, so headless replay is
/// byte-for-byte identical to live editing. `corpus` is the active project's
/// file index (Goto), `root` scopes the Browse navigator, and `workspace`
/// supplies the switch-project children — so a replayed Cmd-O / Cmd-Shift-P /
/// Browse summons a real overlay the rest of the key-spec can filter / move /
/// descend / accept. Returns the post-replay App-level state.
// Replay inputs mirror the capture CLI fields so the one-shot and storyboard paths share this seam.
#[allow(clippy::too_many_arguments)]
fn replay_keys(
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
    km: &mut crate::keymap::KeymapState,
) -> ReplayResult {
    match replay_keys_mode(
        crate::replay::Mode::Permissive,
        buffer,
        keys,
        corpus,
        root,
        workspace,
        config,
        oracle,
        km,
    ) {
        Ok(res) => res,
        Err(e) => unreachable!("permissive replay never aborts: {e}"),
    }
}

/// The mode-aware core both doors share — a thin loop over [`ReplaySession`]
/// (construct, apply every chord, finish). PERMISSIVE never errors (it warns on
/// stderr + records); STRICT returns the exact offender the moment an
/// Unsupported effect fires ([`crate::replay::strict_error`]) — the scenario
/// runner's truthfulness contract (see [`crate::replay`]'s module doc).
// Replay mode adds the explicit mode selector to the same capture CLI field set.
#[allow(clippy::too_many_arguments)]
fn replay_keys_mode(
    mode: crate::replay::Mode,
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
    km: &mut crate::keymap::KeymapState,
) -> Result<ReplayResult> {
    let mut session = ReplaySession::new(mode, buffer, corpus, root, workspace, config, oracle, km);
    for chord in keys {
        session.apply_chord(chord)?;
    }
    Ok(session.finish())
}

/// ONE headless replay in progress: the whole `--keys` engine's state (search
/// guard, overlay, multi-buffer registry, zoom, chord resolver) held as a
/// STRUCT so a caller can interleave key application with other work — the
/// storyboard runner (`crate::story`) applies a step's chords, then renders a
/// film frame from the CURRENT state, then applies the next step's. The one-shot
/// doors ([`replay_keys`] / [`replay_keys_mode`]) are thin loops over this same
/// session, so a storyboard step and a `--keys` replay can never disagree on
/// what a chord does.
pub(crate) struct ReplaySession<'a> {
    mode: crate::replay::Mode,
    buffer: &'a mut Buffer,
    corpus: &'a [String],
    root: &'a std::path::Path,
    workspace: Option<&'a std::path::Path>,
    config: &'a Config,
    // The visual-line motion LAYOUT ORACLE (an offscreen-shaped pipeline), so the
    // headless replay sees the SAME wrap geometry the live window does. Held
    // MUTABLY because the loop RE-SHAPES it from the current buffer / zoom /
    // page-measure state before EVERY action (`OraclePipeline::refresh` — the one
    // freshness seam), mirroring the live window's between-keystrokes re-sync so
    // an edit / zoom / Goto switch can never leave a later motion on stale wrap
    // geometry. `None` in the unit tests / GPU-less paths, where motion falls
    // back to LOGICAL lines.
    oracle: Option<&'a mut capture::OraclePipeline>,
    resolver: crate::keyspec::ChordResolver<'a>,
    spell: Option<crate::spell::SpellChecker>,
    intercepts: Vec<crate::replay::Intercept>,
    replay_skips: Vec<crate::replay::SkippedEffect>,
    warnings: Vec<String>,
    /// The storyboard trace's per-chord record ([`crate::storyboard::ChordTrace`]):
    /// what each chord resolved to and how its effect was classified. Recorded
    /// under both modes (cheap; replay is never hot), drained per-step by the
    /// storyboard runner via [`Self::drain_records`].
    records: Vec<crate::storyboard::ChordTrace>,
    shift_selecting: bool,
    zoom: f32,
    search: Option<crate::search::SearchState>,
    overlay: Option<crate::overlay::OverlayState>,
    accept: Option<(crate::overlay::OverlayKind, String)>,
    // MULTI-BUFFER REGISTRY: the same `crate::buffers::BufferRegistry` the live
    // App uses, so a `--keys` spec that Goes-to file A, edits, Goes-to file B,
    // edits, then Goes back to A sees A's PRESERVED cursor/edits/undo — the
    // v1 multi-buffer win, headlessly drivable. Carries no extra payload
    // (`()`): headless replay tracks nothing per-buffer beyond the `Buffer`
    // itself (no scroll/spell/autosave state to preserve here).
    registry: crate::buffers::BufferRegistry<()>,
    cursor_px: (f32, f32),
}

impl<'a> ReplaySession<'a> {
    #[allow(clippy::too_many_arguments)] // mirrors replay_keys_mode's own surface
    pub(crate) fn new(
        mode: crate::replay::Mode,
        buffer: &'a mut Buffer,
        corpus: &'a [String],
        root: &'a std::path::Path,
        workspace: Option<&'a std::path::Path>,
        config: &'a Config,
        oracle: Option<&'a mut capture::OraclePipeline>,
        km: &'a mut crate::keymap::KeymapState,
    ) -> Self {
        let resolver = crate::keyspec::ChordResolver::new(km, mode == crate::replay::Mode::Strict);
        Self {
            mode,
            buffer,
            corpus,
            root,
            workspace,
            config,
            oracle,
            resolver,
            spell: crate::spell::SpellChecker::new(crate::spell::active_variant()).ok(),
            intercepts: Vec::new(),
            replay_skips: Vec::new(),
            warnings: Vec::new(),
            records: Vec::new(),
            shift_selecting: false,
            zoom: 1.0,
            search: None,
            overlay: None,
            accept: None,
            registry: crate::buffers::BufferRegistry::default(),
            cursor_px: (0.0, 0.0),
        }
    }

    /// ITEM 106 — THE POINTER-REPLAY STEP: move the replay's pointer to
    /// PHYSICAL `(px, py)` and, if an overlay is open, run it through the SAME
    /// hover resolution `App::overlay_hover` uses live (via `OraclePipeline::
    /// resolve_overlay_hover`, which wraps the identical `render::TextPipeline`
    /// hit-test — never a second, hand-rolled implementation for the headless
    /// path). A no-op on `selected` when the movement-slop gate refuses (no
    /// oracle — no wgpu adapter, matching every other oracle-less fallback —
    /// or no overlay open) still records the new position, mirroring the live
    /// `App::cursor_px` write that happens unconditionally on every
    /// `CursorMoved`.
    // Reserved for the oracle pointer-replay seam, which is compiled in every capture build.
    #[allow(dead_code)]
    pub(crate) fn apply_move(&mut self, px: f32, py: f32) {
        self.cursor_px = (px, py);
        // The oracle's own view is otherwise buffer-only (its job is
        // wrap-motion, not overlay hit-testing); sync the CURRENT overlay
        // geometry onto it first, so the hit-test below reads the real,
        // scrolled candidate window a keyboard nav step just produced,
        // exactly like the live GPU pipeline's `sync_view` does every frame.
        self.sync_oracle_overlay();
        if let Some(pipeline) = self.oracle.as_deref()
            && let Some(ov) = self.overlay.as_mut()
        {
            pipeline.resolve_overlay_hover(ov, px, py);
        }
    }

    #[allow(dead_code)]
    fn sync_oracle_overlay(&mut self) {
        if let Some(ov) = self.overlay.as_ref()
            && let Some(op) = self.oracle.as_deref_mut()
        {
            op.sync_overlay(self.buffer, self.zoom, ov);
        }
    }

    pub(crate) fn apply_chord(&mut self, chord: &crate::keyspec::Chord) -> Result<()> {
        // SEARCH GUARD — the live `App::on_keyboard_input` guard's exact position,
        // now the exact same code: while the isearch panel is open EVERY chord is
        // consumed by the ONE interception seam (`crate::search::keys::intercept`)
        // and never reaches the keymap — query/replacement typing, Backspace,
        // C-s/C-r/arrow steps, M-c case toggle, Tab/Cmd-R field moves, Enter
        // accept / replace-one, Cmd-Enter replace-all, Esc/C-g abort. The returned
        // recoil is a LIVE-only caret flourish, dropped here exactly like
        // `Effect::Recoil` (no clock, settled frame unchanged). Strict never
        // judges a consumed chord "unbound" — the panel owning it IS its binding.
        if self.search.is_some() {
            let _ = crate::search::keys::intercept(
                &mut self.search,
                self.buffer,
                &chord.key,
                chord.mods.state(),
            );
            self.records.push(crate::storyboard::ChordTrace {
                chord: chord.spec.clone(),
                action: None,
                effect: "search_input".to_string(),
                class: "applied",
                detail: String::new(),
            });
            return Ok(());
        }
        let Some(resolved) = self.resolver.resolve(chord)? else {
            self.records.push(crate::storyboard::ChordTrace {
                chord: chord.spec.clone(),
                action: None,
                effect: "prefix".to_string(),
                class: "applied",
                detail: String::new(),
            });
            return Ok(());
        };
        // SHIFT = SELECT-INTENT, the live dispatch's exact derivation
        // (`app/input/keys.rs::on_keyboard_input`): the chord's `S-` modifier
        // extends a selection across a motion, routed through the ONE owner
        // `crate::app::motion_honors_shift_select` — keyed on the pressed chord's
        // KEY, not the Action alone, so `M-<` / `M->` (a `Key::Character` whose
        // Shift is incidental to typing the glyph) stay pure motion while
        // `S-s-Up`/`S-s-Down` and `S-C-Home`/`S-C-End` (named nav keys reaching
        // the same actions) extend, exactly like live. Derived ONCE per pressed
        // chord from the FIRST resolved action and carried into a palette-chained
        // re-dispatch unchanged — mirroring the live `Effect::RunAction` arm,
        // which re-applies with the same `shift` bool. (This retired the old
        // "replay is unshifted" hole: `--keys "S-Right"` silently ran the motion
        // unshifted and left `selection: null`.)
        let shift = chord
            .mods
            .state()
            .contains(winit::keyboard::ModifiersState::SHIFT)
            && crate::app::motion_honors_shift_select(&resolved, &chord.key);
        let mut current: Option<Action> = Some(resolved);
        let mut pending_return_to: Option<crate::overlay::OverlayKind> = None;
        while let Some(action) = current.take() {
            // FRESH LAYOUT ORACLE PER ACTION: re-shape the oracle from the CURRENT
            // buffer / zoom / page-measure state BEFORE the action consults it —
            // the live window's pipeline re-syncs between keystrokes, so the
            // headless twin must too, or an edit that re-wraps a line (or a zoom
            // change, or the Goto arm's buffer + measure switch below) leaves the
            // NEXT motion reading stale wrap geometry. One seam, unconditional by
            // design (both underlying calls no-op cheaply when nothing changed).
            if let Some(op) = self.oracle.as_deref_mut() {
                op.refresh(self.buffer, self.zoom);
            }
            let goto_headings: Vec<(String, usize)> =
                if matches!(action, Action::OpenGoto | Action::OpenOutline)
                    && self.buffer.is_markdown()
                {
                    crate::markdown::headings(&self.buffer.text())
                        .into_iter()
                        .map(|h| (h.label(), h.line))
                        .collect()
                } else {
                    Vec::new()
                };
            #[allow(clippy::type_complexity)]
            let spell_target: Option<(Vec<String>, (usize, usize, usize), String)> =
                if matches!(action, Action::OpenSpellSuggest) {
                    self.spell.as_ref().and_then(|sc| {
                        let (line, col) = self.buffer.cursor_line_col();
                        sc.suggest_at(&self.buffer.text(), line, col, self.buffer.syntax_lang())
                            .map(|t| {
                                (
                                    t.suggestions,
                                    (
                                        t.misspelling.line,
                                        t.misspelling.start_col,
                                        t.misspelling.end_col,
                                    ),
                                    t.word,
                                )
                            })
                    })
                } else {
                    None
                };
            // HISTORY TIMELINE rows for the current file (newest-first), each answering
            // WHEN + WHICH with a "+N −M" changed-count vs the current buffer. Read from
            // the store ONLY when the History binding fired (so a `--keys "Cmd-S-h"`
            // capture shows the real versions of the seeded file); the history key comes
            // from the ONE shared derivation (`history::source_path`: buffer path, else
            // the scratch stash — the replay has no App-level `file`), matching the live
            // gather. `now` stamps the relative labels. History is an explicitly-summoned
            // overlay, so this never runs in a default capture.
            let history_entries: Vec<crate::history::TimelineRow> =
                if matches!(action, Action::OpenHistory | Action::CompareVersion) {
                    match crate::history::source_path(
                        self.buffer.path(),
                        self.buffer.is_unnamed_fresh(),
                    ) {
                        Some(path) => crate::history::timeline_rows(
                            &path,
                            &self.buffer.text(),
                            crate::history::now_millis(),
                        ),
                        None => Vec::new(),
                    }
                } else {
                    Vec::new()
                };
            let assets: Vec<crate::assets::Orphan> = if matches!(action, Action::OpenAssetClean) {
                crate::assets::scan(self.root, self.corpus)
            } else {
                Vec::new()
            };
            let effective_keep = self.config.effective_linux_keep();
            let build_ctx = crate::overlay::BuildCtx {
                goto_corpus: self.corpus.to_vec(),
                goto_open: Vec::new(),
                goto_recent: Vec::new(),
                goto_times: Vec::new(),
                config_keys: &self.config.keys,
                config_linux_keep: &effective_keep,
                goto_headings,
                spell_target,
                history_entries,
                history_now: None,
                history_session_start: None,
                settings_values: crate::settings::SettingsValues::gather(
                    self.config,
                    self.root,
                    self.zoom,
                    crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
                ),
                assets,
                // The headless capture/replay path is structurally daemon-free (never
                // imports `crate::daemon` — the daemon capture gate, docs/platform.md):
                // there is no waiter to have, so "Finish file" is deterministically
                // hidden from every `--keys`/`--screenshot` palette.
                has_waiter: false,
            };
            let mut make_overlay =
                |kind: crate::overlay::OverlayKind| crate::overlay::build(kind, &build_ctx);
            let (root, workspace) = (self.root, self.workspace);
            let mut browse_to = |kind: crate::overlay::OverlayKind, rel: Option<String>| {
                // Shared one-level builder: Project navigates the workspace by absolute
                // path, MoveDest and Browse both walk the SAME active root (item 76 —
                // MoveDest folders only, Browse files + folders).
                // The recent-PROJECTS MRU is live-only persisted state; the headless
                // replay passes an empty list (the determinism gate), so the Project
                // navigator's Recent lens is inert in a capture — byte-stable.
                crate::overlay::browse_level(kind, rel, root, workspace, &[])
            };
            let mut ctx = actions::ActionCtx {
                buffer: &mut *self.buffer,
                shift_selecting: &mut self.shift_selecting,
                zoom: &mut self.zoom,
                search: &mut self.search,
                scroll_page_lines: 20,
                overlay: &mut self.overlay,
                make_overlay: &mut make_overlay,
                browse_to: &mut browse_to,
                oracle: self.oracle.as_deref().map(|op| op.as_oracle()),
            };
            let effect = actions::apply_core(&mut ctx, &action, shift);
            let _ = ctx;
            let classified = crate::replay::classify(&effect);
            self.records.push(crate::storyboard::ChordTrace {
                chord: chord.spec.clone(),
                action: Some(format!("{action:?}")),
                effect: classified.name.to_string(),
                class: match &classified.class {
                    crate::replay::EffectClass::Applied => "applied",
                    crate::replay::EffectClass::Intercepted { .. } => "intercepted",
                    crate::replay::EffectClass::Unsupported { .. } => "unsupported",
                },
                detail: match &classified.class {
                    crate::replay::EffectClass::Intercepted { detail } => detail.clone(),
                    _ => String::new(),
                },
            });
            if let crate::replay::EffectClass::Intercepted { detail } = &classified.class {
                self.intercepts.push(crate::replay::Intercept {
                    effect: classified.name,
                    detail: detail.clone(),
                });
            }
            if self.mode == crate::replay::Mode::Strict
                && let crate::replay::EffectClass::Unsupported { .. } = classified.class
            {
                return Err(crate::replay::strict_error(&action, &classified));
            }
            if self.mode == crate::replay::Mode::Permissive
                && let Some(skip) = crate::replay::permissive_skip(&action, &classified)
            {
                self.replay_skips.push(skip);
            }
            if self.mode == crate::replay::Mode::Permissive
                && let Some(w) = crate::replay::warn_line(&action, &classified)
            {
                eprintln!("{w}");
                self.warnings.push(w);
            }
            crate::actions::stamp_return_to(&mut self.overlay, pending_return_to.take());
            match effect {
            actions::Effect::NewDocument => {
                park_active(self.buffer, &mut self.registry);
                self.buffer.start_fresh_doc(self.root.to_path_buf());
            }
            actions::Effect::OpenSettings => {
                if !self.config.path.as_os_str().is_empty() {
                    if !crate::fs::active().exists(&self.config.path) {
                        let _ = Config::write_default(&self.config.path);
                    }
                    *self.buffer = Buffer::from_file(&self.config.path);
                }
            }
            // Credits: load the embedded CREDITS.md text directly into the buffer
            // — no filesystem write at all (the headless capture path stays
            // side-effect-light, mirroring OpenSettings' spirit without needing a
            // disk round trip, since the text is compiled in rather than
            // user-owned). No park needed here either: `replay_keys` never stashes
            // scratch (structurally autosave-free), so there is nothing to protect.
            actions::Effect::OpenCredits => {
                *self.buffer = Buffer::from_str(crate::credits::CREDITS_MD);
            }
            actions::Effect::OpenGuide => {
                *self.buffer = Buffer::from_str(&crate::guide::render(
                    crate::convention::Convention::current(),
                    crate::commands::Platform::current(),
                ));
            }
            actions::Effect::InsertDate => {
                let (y, m, d) = crate::dateformat::CAPTURE_PLACEHOLDER_YMD;
                let text = crate::dateformat::active_format().format(y, m, d);
                self.buffer.insert_text(&text);
            }
            // An overlay accepted (Goto file / Project / MoveDest / Theme): remember
            // the chosen value for the caller to load before capturing. Persists
            // across keys like the old out-param (later accepts overwrite).
            //
            // A Goto accept ALSO drives the real MULTI-BUFFER switch right here,
            // inline in the replay loop (not deferred to the caller, which only
            // ever sees the FINAL accepted value): opening a path already
            // resident in `registry` (a previous Goto in this same `--keys` run)
            // restores its live buffer — cursor, edits, undo intact — instead of
            // re-reading disk, mirroring `App::load_path` exactly. This is what
            // makes an A -> B -> A round trip verifiable from one `--keys` spec.
            actions::Effect::OverlayAccept(kind, val) => {
                if kind == crate::overlay::OverlayKind::Goto {
                    let path = crate::index::resolve(self.root, &val);
                    // Compared via the normalized registry identity, not raw
                    // path equality — mirrors `App::load_path`'s "already
                    // active" check (see `BufferKey::path`'s doc: a launch
                    // file argument that stayed relative and this ALWAYS
                    // root-joined Goto path must be recognized as the same
                    // file, or the switch below re-reads it fresh from disk
                    // and orphans the relative spelling's live edit).
                    let new_key = crate::buffers::BufferKey::path(&path);
                    if crate::buffers::BufferKey::of(self.buffer).as_ref() != Some(&new_key) {
                        park_active(self.buffer, &mut self.registry);
                        *self.buffer = match self.registry.take(&new_key) {
                            Some(entry) => entry.buffer,
                            None => Buffer::from_file(&path),
                        };
                        // STICKY PAGE WIDTH: re-apply the measure for the ARRIVING
                        // buffer's own kind, mirroring `App::load_path`'s post-switch
                        // resync (`App::sync_page_measure`) — a `--keys` Goto from a
                        // `.md` to a `.rs` fixture (or back) picks up that file's own
                        // configured/default measure, exactly like the live app.
                        // (This made every Goto-replay TEST a page-global writer;
                        // `set_measure` self-serializes under cfg(test) — see
                        // `page::test_lock()` — so those tests need no lock of
                        // their own and can never stomp a locked reader again.)
                        crate::page::set_measure(self.config.measure_for(self.buffer.page_class()));
                    }
                }
                self.accept = Some((kind, val));
            }
            actions::Effect::ConvertScratchAndSave => {
                let _ = self.buffer.save_into_folder(self.root);
            }
            actions::Effect::JumpToLine(line) => {
                let idx = self.buffer.line_col_to_char(line, 0);
                self.buffer.set_cursor(idx);
                // REVEALED PLACEMENT (folds): the headless twin of `App::jump_to_line`
                // — route through the ONE placement owner so a heading jump onto a
                // hidden line reveals it here exactly as it does live, keeping the
                // sidecar's cursor + filtered mapping honest. A cheap no-op unless
                // a section is folded.
                self.buffer.reveal_placement();
            }
            actions::Effect::RunAction(a) => {
                pending_return_to = Some(crate::overlay::OverlayKind::Command);
                current = Some(a);
            }
            actions::Effect::RebindCommit { slug, binding, .. } => {
                if let Some(ov) = self.overlay.as_mut() {
                    ov.notice = format!("bound {slug} -> {binding}");
                    ov.capture_abort();
                }
            }
            actions::Effect::RebindReset { slug } => {
                if let Some(ov) = self.overlay.as_mut()
                    && ov.notice.is_empty() {
                        ov.notice = format!("reset {slug}");
                    }
            }
            // Quit / LastBuffer have nothing to do in the headless capture path.
            // Recoil and the edit flinches (TypeImpact / DeleteSquash / Gulp /
            // LineLand / CopyPulse) are LIVE-ONLY caret flourishes (a squash-pop /
            // velocity kick / selection-tint brighten that self-settles) — the
            // headless capture has no clock and renders the SETTLED caret + selection,
            // so they are no-ops here and the frame stays byte-identical (CopyPulse
            // never touches the buffer either way — the copy itself already ran).
            // FinishBuffer (Finish file): the core already ran the SAME `buffer.save()` a
            // headless `Action::Save` replay always has (writes through the active
            // `fs` backend); the daemon-notify + buffer-swap are live-App-only (no
            // daemon, no 2-deep buffer history, in a one-shot replay) — a no-op here,
            // exactly like `LastBuffer`.
            actions::Effect::LastBuffer
            | actions::Effect::Quit
            | actions::Effect::Recoil(_)
            | actions::Effect::TypeImpact
            | actions::Effect::DeleteSquash
            | actions::Effect::Gulp
            | actions::Effect::LineLand
            | actions::Effect::CopyPulse
            | actions::Effect::SettingToggle { .. }
            // SETTINGS MENU inline VALUE commit / PATH pick: parse-clamp-apply-persist
            // and folder-key writes are the live App's job (`App::setting_value_commit`
            // / `setting_path_pick`) — the capture path has no live global setter it
            // should mutate nor a config file to write, so both reflect nothing here
            // (the value-edit round-trip is unit-tested at the apply seam instead). The
            // pure inline-edit sub-state itself IS driven by the shared core, so the
            // still-open menu's cell reflects the typed value; only the commit is inert.
            | actions::Effect::SettingValueCommit { .. }
            | actions::Effect::SettingPathPick { .. }
            // ITEM 94 — a RANGE row's step: the value change ALREADY happened in the
            // shared core (see `Effect::SettingRangeStep`'s doc), so the capture's
            // still-open menu genuinely shows the stepped value + thumb; only the
            // live tail (reflow + the sticky config write) is skipped here.
            | actions::Effect::SettingRangeStep { .. }
            | actions::Effect::FinishBuffer
            | actions::Effect::KeepVersion { .. }
            // FollowLink (C-c C-o): opening the OS browser is a live-App-only
            // handoff (`App::follow_link`) — a capture must never spawn a browser,
            // so it is a no-op here (the URL extraction itself is unit-tested pure).
            | actions::Effect::FollowLink(_)
            // REPORT A PROBLEM: composing the mailto: URL (which needs the
            // crash-log directory) and opening it are both live-App-only
            // concerns (`App::report_problem`) — a capture must never spawn a
            // mail client, so this is a no-op here; the composition itself is
            // unit-tested pure (`crashlog::report_problem_mailto`).
            | actions::Effect::ReportProblem
            // DOWNLOAD FILE (web-only): building a Blob/object-URL and clicking a
            // synthetic download anchor is a live-App-only DOM handoff
            // (`App::download_file`) — a capture must never touch the DOM, so this
            // is a no-op here; the filename derivation itself is unit-tested pure
            // (`web_export::filename_for`). Also gated off entirely on native by
            // `commands::action_available` before this effect can even be signaled.
            | actions::Effect::DownloadFile
            // EXPORT: rendering the document + writing the `.docx`/`.html` sibling
            // (or a web download) is a live-App-only concern (`App::export_document`)
            // — a capture must never write an export file, so this is a documented
            // no-op here; the exporter core itself is unit-tested pure (`export/`).
            | actions::Effect::Export(_)
            // CHECK FOR UPDATES: recording the local "last checked" marker and
            // opening the site's `/check?v=…` page are both live-App-only
            // concerns (`App::check_for_updates`) — a capture must never touch
            // the marker file or spawn a browser, so this is a documented no-op
            // here; the URL composition + marker round-trip are unit-tested pure
            // (`updates.rs`). Matches `ReportProblem`'s own headless behavior
            // exactly — see `updates.rs`'s module doc.
            | actions::Effect::CheckForUpdates
            // TRASH ASSET: moving an orphan to the OS Trash is a live-App-only
            // concern (`App::trash_asset`) — a capture must never touch the real Trash,
            // so this is a documented no-op here. The picker's orphan list therefore
            // stays WHOLE in a `--keys` replay (the sidecar never claims a file was
            // trashed that wasn't); the trash + row-removal wiring is unit-tested at
            // the apply seam with a fake trash instead.
            | actions::Effect::TrashAsset { .. }
            | actions::Effect::SaveDone { .. }
            // NOTES VERBS round: both the actual disk RENAME (`App::rename_current_file`
            // — git-managed gate, no-clobber refusal, the one-owner path-keyed
            // bookkeeping) and the DUPLICATE copy+swap (`App::duplicate_current_file`)
            // are live-App-only, mirroring `MoveDest`'s own real-move precedent (its
            // ACCEPT is reflected below via `accept`, but the actual `fs::rename` is
            // live-only too) — a no-op here. The RENAME MINIBUFFER's typing/open/
            // cancel flow IS driven by the shared core (`overlay_intercept`'s
            // `rename_edit` block), so it stays fully `--keys`-drivable and sidecar-
            // reflected via `overlay.hint` (`OverlayState::foot_hint`) up to the
            // moment of commit; only the disk write itself is deferred here.
            | actions::Effect::RenameNoteCommit { .. }
            | actions::Effect::DuplicateNote
            // ADD TO DICTIONARY (Cmd-`;` add row): silencing the word + APPENDING it
            // to the on-disk personal dictionary is a live-App-only concern
            // (`App::add_to_dictionary`) — a capture must never write the dictionary
            // file (the same determinism gate `KeepVersion`/`Export` sit behind), so
            // this is a documented no-op here; the load/persist itself is unit-tested
            // at the App seam, and the picker's add-row select/accept flow IS
            // core-driven and fully `--keys`-drivable up to this signal.
            | actions::Effect::AddToDictionary(_)
            | actions::Effect::None => {}
        }
        }
        // ITEM 106 — the headless twin of `App::apply`'s own stamp: re-anchor the
        // hover movement-slop gate to the replay's CURRENT pointer position after
        // this whole chord (including any chained palette re-dispatch) has
        // applied, so a scripted keyboard nav step never leaves a stale (or
        // `None`) hover baseline for a LATER `move` step to read as unconditional
        // real motion. See `OverlayState::arm_hover_baseline`'s doc.
        if let Some(ov) = self.overlay.as_mut() {
            ov.arm_hover_baseline(self.cursor_px.0, self.cursor_px.1);
        }
        Ok(())
    }

    fn finish(self) -> ReplayResult {
        let buffers_open = self.registry.len() + 1;
        let zoom_out = if self.zoom != 1.0 {
            Some(self.zoom)
        } else {
            None
        };
        let sel = self.buffer.selection_line_col();
        let search_query = self.search.as_ref().map(|s| s.query().to_string());
        let search_case = self
            .search
            .as_ref()
            .map(|s| s.is_case_sensitive())
            .unwrap_or(false);
        let replace_active = self
            .search
            .as_ref()
            .map(|s| s.is_replace_active())
            .unwrap_or(false);
        let replacement = self
            .search
            .as_ref()
            .map(|s| s.replacement().to_string())
            .unwrap_or_default();
        let editing_replacement = self
            .search
            .as_ref()
            .map(|s| s.is_editing_replacement())
            .unwrap_or(false);
        ReplayResult {
            zoom: zoom_out,
            selection: sel,
            search_query,
            search_case,
            replace_active,
            replacement,
            editing_replacement,
            overlay: self.overlay,
            accept: self.accept,
            buffers_open,
            intercepts: self.intercepts,
            replay_skips: self.replay_skips,
            warnings: self.warnings,
        }
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        self.buffer
    }

    pub(crate) fn zoom(&self) -> f32 {
        self.zoom
    }

    pub(crate) fn search(&self) -> Option<&crate::search::SearchState> {
        self.search.as_ref()
    }

    pub(crate) fn overlay(&self) -> Option<&crate::overlay::OverlayState> {
        self.overlay.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn oracle(&self) -> Option<&capture::OraclePipeline> {
        self.oracle.as_deref()
    }

    pub(crate) fn buffers_open(&self) -> usize {
        self.registry.len() + 1
    }

    pub(crate) fn drain_records(&mut self) -> Vec<crate::storyboard::ChordTrace> {
        std::mem::take(&mut self.records)
    }
}

#[allow(clippy::too_many_arguments)]
fn capture_screenshot(
    out: PathBuf,
    file: Option<PathBuf>,
    mut opts: CaptureOpts,
    keys: Vec<crate::keyspec::Chord>,
    mut km: crate::keymap::KeymapState,
    root: Option<PathBuf>,
    workspace: Option<PathBuf>,
    default_folder: PathBuf,
    config: Config,
    strict: bool,
) -> Result<()> {
    // Resolve the active project + its file index BEFORE the replay so a
    // `Cmd-O` in the key-spec summons a real, scoped go-to overlay. Capture
    // is structurally free of remembered session state (the capture-gate
    // law) — `resolve_root` only ever consults the EXPLICIT `--root`/file,
    // never a "first run" default either (that's a windowed-launch concern).
    let active_root = resolve_root(&root, &file);
    let proj = crate::project::Project::resolve(&active_root);
    let corpus = crate::index::build_index(&active_root);
    let effective_workspace = resolve_workspace(&workspace, &active_root);
    opts.project = Some(capture::ProjectInfo {
        root: active_root.clone(),
        name: proj.name.clone(),
        branch: proj.branch.clone(),
        dirty: proj.dirty,
        default_folder: Some(default_folder.clone()),
        workspace: Some(effective_workspace.clone()),
        keymap_flavor: config.keymap_flavor().config_name(),
    });

    let mut buffer = load_buffer(&file);
    if let Some((md, counts, label)) = crate::prosediff::env_capture_render() {
        buffer = crate::buffer::Buffer::from_str(&md);
        // Park the caret on the blank line 1 (between the title and the first
        // diff block) so NO line's WYSIWYG conceal reveals — the reveal is
        // caret-line-scoped and line 1 carries no markup, so the title's `#`
        // and every `==`/`>`/strike marker below stay concealed: the clean
        // marked-up manuscript, never a revealed-raw line. Mirrors the live
        // History-preview fold (`sync_view` parks the caret the same way —
        // the ONE reveal-suppression rule, shared, so live == capture).
        buffer.set_cursor(buffer.line_col_to_char(1, 0));
        opts.diff = Some(capture::DiffInfo {
            active: true,
            label,
            struck: counts.struck,
            washed: counts.washed,
            modified: counts.modified,
            moved: counts.moved,
            folds: counts.folds,
        });
    }
    // Replay `--keys` FIRST so the cursor/selection/search the spec
    // produces are what the capture reflects. Fold the App-level state
    // (zoom / selection / search) the replay produced into the capture
    // opts — but never clobber an explicit verification hook.
    // Default the switch-project workspace to the active root's PARENT
    // when no explicit `--workspace` was given, so a replayed `Cmd-Shift-P`
    // summons the picker listing the root's SIBLING projects (rather than
    // silently doing nothing). An explicit `--workspace` still overrides.
    //
    // Visual-line motion ORACLE: when the spec has keys, build an offscreen
    // pipeline shaped like the upcoming capture so headless motion reads the
    // SAME wrap geometry the live window does (and is re-shaped from the
    // current replay state before every action — `OraclePipeline::refresh`,
    // called inside the replay loop). Skipped for an empty spec (no motion
    // to resolve) and absent on GPU-less hosts (logical fallback).
    let mut oracle = if keys.is_empty() {
        None
    } else {
        capture::build_oracle(&buffer, &opts)
    };
    // STRICT REPLAY: a spec with keys MUST ride the real wrap geometry —
    // a missing oracle (no GPU adapter) means visual-line motion would
    // silently fall back to logical lines, so strict refuses up front.
    if strict && !keys.is_empty() && oracle.is_none() {
        return Err(crate::replay::missing_oracle_error());
    }
    let mode = if strict {
        crate::replay::Mode::Strict
    } else {
        crate::replay::Mode::Permissive
    };
    let res = replay_keys_mode(
        mode,
        &mut buffer,
        &keys,
        &corpus,
        &active_root,
        Some(effective_workspace.as_path()),
        &config,
        oracle.as_mut(),
        &mut km,
    )?;
    if opts.zoom.is_none() {
        opts.zoom = res.zoom;
    }
    if opts.selection.is_none() {
        opts.selection = res.selection;
    }
    if opts.search.is_none() {
        opts.search = res.search_query;
        opts.search_case_sensitive = opts.search_case_sensitive || res.search_case;
        // REPLACE mode the replay opened (Cmd-R / Tab / Cmd-Option-F) —
        // surfaced so the panel's replace row renders + the sidecar
        // reports it, along with the replayed replacement TEXT and which
        // field currently has focus (all typed/moved through the shared
        // search-key seam).
        opts.search_replace_active = res.replace_active;
        opts.search_replacement = res.replacement;
        opts.search_editing_replacement = res.editing_replacement;
    }
    if let Some((kind, val)) = &res.accept {
        match kind {
            crate::overlay::OverlayKind::Goto => {}
            crate::overlay::OverlayKind::Project => {
                let new_root = std::path::PathBuf::from(val);
                let proj = crate::project::Project::resolve(&new_root);
                opts.project = Some(capture::ProjectInfo {
                    root: new_root,
                    name: proj.name.clone(),
                    branch: proj.branch.clone(),
                    dirty: proj.dirty,
                    default_folder: Some(default_folder.clone()),
                    workspace: Some(effective_workspace.clone()),
                    keymap_flavor: config.keymap_flavor().config_name(),
                });
            }
            // History: RESTORE the accepted version into the buffer (an undoable
            // edit), so a `--keys "Cmd-S-h <down> <enter>"` capture reflects the
            // restored text — the same `history::load` + `set_text` the App runs,
            // keyed by the same shared `source_path` derivation.
            crate::overlay::OverlayKind::History => {
                if let Some(path) =
                    crate::history::source_path(buffer.path(), buffer.is_unnamed_fresh())
                    && let Some(content) = crate::history::load(&path, val)
                {
                    buffer.set_text(&content);
                }
            }
            _ => {}
        }
    }
    if let Some(ov) = &res.overlay {
        let (info, preview_text, diff) = overlay_capture_info(ov, &buffer);
        opts.overlay = Some(info);
        opts.preview_text = preview_text;
        if opts.diff.is_none() {
            opts.diff = diff;
        }
        if opts.scroll.is_none() && opts.preview_text.is_some() {
            opts.scroll = Some(crate::render::ScrollPos::at_row(ov.diff_scroll));
        }
    }
    if keys.is_empty()
        && let Some((_, (l1, c1))) = opts.selection
    {
        let end = buffer.line_col_to_char(l1, c1);
        buffer.set_cursor(end);
    }
    if crate::whichkey::force_shown() {
        opts.whichkey = Some(
            crate::whichkey::continuations_cx(&config.keys)
                .into_iter()
                .map(|c| (c.key, c.name))
                .collect(),
        );
    }
    opts.buffers = Some(capture::BuffersInfo {
        open: res.buffers_open,
        active: match buffer.path() {
            Some(p) => p.display().to_string(),
            None => "scratch".to_string(),
        },
    });
    opts.replay_skips = res.replay_skips;
    capture::capture_with(&out, &buffer, &opts)?;
    println!("wrote {} (+ sidecar .json)", out.display());
    Ok(())
}

/// Fold ONE still-open overlay into its sidecar [`capture::OverlayInfo`] block
/// plus the History live-preview TEXT (if that overlay is the History timeline
/// — see [`history_preview_for`]). Extracted from [`capture_screenshot`]
/// VERBATIM so the storyboard runner's per-step render (`crate::story`) and the
/// one-shot `--keys` capture share ONE owner of "what does an open overlay
/// report" — the two can never drift.
pub(crate) fn overlay_capture_info(
    ov: &crate::overlay::OverlayState,
    buffer: &Buffer,
) -> (
    capture::OverlayInfo,
    Option<String>,
    Option<capture::DiffInfo>,
) {
    let preview = history_preview_for(ov, buffer);
    let preview_text = preview
        .as_ref()
        .map(|(_, transcript, _)| transcript.clone());
    let diff = preview.as_ref().map(|(_, _, c)| capture::DiffInfo {
        active: true,
        label: ov
            .selected_value()
            .unwrap_or("an earlier version")
            .to_string(),
        struck: c.struck,
        washed: c.washed,
        modified: c.modified,
        moved: c.moved,
        folds: c.folds,
    });
    let info = capture::OverlayInfo {
        active: true,
        mode: ov.kind.as_str(),
        align: ov.align,
        query: ov.query.text().to_string(),
        items: ov.item_strings(),
        empty: ov.empty_notice(),
        bindings: ov.item_bindings(),
        ranges: ov.item_range_fracs(),
        git: ov.item_git_tags(),
        selected_index: ov.selected,
        hint: ov.foot_hint(),
        browse_dir: ov.browse_dir.clone(),
        return_to: ov.return_to.map(|k| k.as_str()),
        spell_target: ov.spell_target,
        preview_id: preview.map(|(id, _, _)| id),
        diff_focus: ov.diff_focus,
        diff_scroll: ov.diff_scroll,
        show_hidden: ov.kind.hides_dotfiles() && crate::file_visibility::all_on(),
        capture: ov.capture.as_ref().map(|c| capture::CaptureInfo {
            command: c.cmd_name.clone(),
            stage: match c.stage {
                crate::overlay::CaptureStage::ChooseMode => "choose",
                crate::overlay::CaptureStage::Recording => "recording",
                crate::overlay::CaptureStage::Confirm => "confirm",
            },
            chord_mode: c.chord_mode,
            captured: c.captured.clone(),
            prompt: c.prompt(),
        }),
        notice: ov.notice.clone(),
        lens: ov.active_facet_id(),
        lens_strip: ov.lens_strip(),
        sections: ov.item_sections(),
        title: ov.kind.title(),
    };
    (info, preview_text, diff)
}

/// The HISTORY timeline's headless live preview: when the replay left the History
/// overlay OPEN, resolve its highlighted row's restore id to that version's
/// `(id, content)` via [`crate::history::load`] — keyed by the same shared
/// [`crate::history::source_path`] derivation the live App uses — so the capture
/// shows THAT VERSION in the document itself and the sidecar reports which.
/// `None` for every other overlay kind, the empty-state row, or an unresolvable
/// id (the capture then just shows the buffer — the live degrade). Pure over the
/// store, so it is unit-testable with a seeded log.
fn history_preview_for(
    ov: &crate::overlay::OverlayState,
    buffer: &Buffer,
) -> Option<(String, String, crate::prosediff::DiffCounts)> {
    // DIFF-AS-PREVIEW: the preview IS the writer's diff of the current buffer vs
    // the highlighted version — built by the SAME one owner the live App renders
    // through (`history::diff_preview`), synchronously (the live debounce is a
    // wall-clock concern the deterministic capture never has).
    crate::history::diff_preview(ov, buffer.path(), buffer.is_unnamed_fresh(), &buffer.text())
}

pub(crate) fn run(mode: Mode) -> Result<()> {
    match mode {
        Mode::Screenshot {
            out,
            file,
            opts,
            keys,
            km,
            root,
            workspace,
            default_folder,
            config,
            strict,
        } => capture_screenshot(
            out,
            file,
            opts,
            keys,
            km,
            root,
            workspace,
            default_folder,
            config,
            strict,
        ),
        Mode::ScreenshotMotion {
            out,
            file,
            keys,
            mut km,
        } => {
            let mut buffer = load_buffer(&file);
            let root = resolve_root(&None, &file);
            replay_keys(
                &mut buffer,
                &keys,
                &[],
                &root,
                None,
                &Config::empty(),
                None,
                &mut km,
            );
            capture::capture_motion(&out, &buffer)?;
            println!("wrote {} (mid-glide, + sidecar .json)", out.display());
            Ok(())
        }
        Mode::ScreenshotMotionVertical {
            out,
            file,
            keys,
            mut km,
        } => {
            let mut buffer = load_buffer(&file);
            let root = resolve_root(&None, &file);
            replay_keys(
                &mut buffer,
                &keys,
                &[],
                &root,
                None,
                &Config::empty(),
                None,
                &mut km,
            );
            capture::capture_motion_vertical(&out, &buffer)?;
            println!(
                "wrote {} (mid-glide vertical, + sidecar .json)",
                out.display()
            );
            Ok(())
        }
        Mode::ScreenshotMotionDiagonal {
            out,
            file,
            keys,
            mut km,
        } => {
            let mut buffer = load_buffer(&file);
            let root = resolve_root(&None, &file);
            replay_keys(
                &mut buffer,
                &keys,
                &[],
                &root,
                None,
                &Config::empty(),
                None,
                &mut km,
            );
            capture::capture_motion_diagonal(&out, &buffer)?;
            println!(
                "wrote {} (mid-glide diagonal, + sidecar .json)",
                out.display()
            );
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        Mode::ScreenshotFrames {
            out,
            file,
            frames,
            step_ms,
        } => {
            let buffer = load_buffer(&file);
            capture::capture_frames(&out, &buffer, frames, step_ms, &CaptureOpts::default())?;
            let stem = out
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "capture".to_string());
            println!(
                "wrote {frames} frame(s) to {stem}.fNNN.png (step {step_ms}ms, + per-frame sidecars, + {stem}.frames.json)"
            );
            Ok(())
        }
        Mode::CaptureTimeline {
            out,
            file,
            keys,
            mut km,
            steps,
            root,
            canvas,
            dpi,
        } => {
            let active_root = resolve_root(&root, &file);
            let proj = crate::project::Project::resolve(&active_root);
            let corpus = crate::index::build_index(&active_root);
            let opts = CaptureOpts {
                project: Some(capture::ProjectInfo {
                    root: active_root.clone(),
                    name: proj.name.clone(),
                    branch: proj.branch.clone(),
                    dirty: proj.dirty,
                    default_folder: None,
                    workspace: None,
                    keymap_flavor: "native",
                }),
                canvas,
                dpi,
                ..CaptureOpts::default()
            };

            let mut buffer = load_buffer(&file);
            let (last, init) = match keys.split_last() {
                Some((last, init)) => (Some(last.clone()), init.to_vec()),
                None => (None, Vec::new()),
            };
            if !init.is_empty() {
                replay_keys(
                    &mut buffer,
                    &init,
                    &corpus,
                    &active_root,
                    None,
                    &Config::empty(),
                    None,
                    &mut km,
                );
            }
            let origin = buffer.cursor_line_col();
            if let Some(last) = last {
                replay_keys(
                    &mut buffer,
                    std::slice::from_ref(&last),
                    &corpus,
                    &active_root,
                    None,
                    &Config::empty(),
                    None,
                    &mut km,
                );
            }
            capture::capture_timeline(&out, &buffer, origin, &steps, &opts)?;
            println!(
                "wrote {} timeline frames for {} (+ per-step sidecars)",
                steps.len(),
                out.display()
            );
            Ok(())
        }
        Mode::CaptureHeld {
            out,
            file,
            keys,
            mut km,
            dir,
            steps,
            root,
            canvas,
            dpi,
        } => {
            let active_root = resolve_root(&root, &file);
            let proj = crate::project::Project::resolve(&active_root);
            let corpus = crate::index::build_index(&active_root);
            let opts = CaptureOpts {
                project: Some(capture::ProjectInfo {
                    root: active_root.clone(),
                    name: proj.name.clone(),
                    branch: proj.branch.clone(),
                    dirty: proj.dirty,
                    default_folder: None,
                    workspace: None,
                    keymap_flavor: "native",
                }),
                canvas,
                dpi,
                ..CaptureOpts::default()
            };

            let mut buffer = load_buffer(&file);
            if !keys.is_empty() {
                replay_keys(
                    &mut buffer,
                    &keys,
                    &corpus,
                    &active_root,
                    None,
                    &Config::empty(),
                    None,
                    &mut km,
                );
            }
            let origin = buffer.cursor_line_col();
            capture::capture_held(&out, &buffer, origin, dir, &steps, &opts)?;
            println!(
                "wrote {} held frames for {} (+ per-step sidecars)",
                steps.len(),
                out.display()
            );
            Ok(())
        }
        Mode::Storyboard {
            board,
            file,
            out_dir,
            root,
            workspace,
            default_folder,
            config,
            km,
        } => crate::story::run_storyboard(
            board,
            file,
            out_dir,
            root,
            workspace,
            default_folder,
            config,
            km,
        ),
        Mode::BenchTyping => bench::run(),
        Mode::BenchPerf => crate::render::perfbench::run(),
        Mode::BenchFrame => crate::render::framebench::run(),
        Mode::BenchThemeBurst => crate::render::framebench::run_theme_burst(),
        Mode::BenchZoomBurst => crate::render::framebench::run_zoom_burst(),
        Mode::BenchFrost => crate::render::framebench::run_frost(),
        Mode::BenchCaret => crate::render::caretbench::run(),
        Mode::BenchSuite { baseline } => crate::render::benchsuite::run(baseline),
        #[cfg(not(target_arch = "wasm32"))]
        Mode::SoakGpu(config) => {
            let root = std::env::temp_dir().join(format!("awl-soak-gpu-{}", std::process::id()));
            std::fs::create_dir_all(&root)?;
            let result = app::run(
                None,
                root.clone(),
                None,
                None,
                Config::empty(),
                false,
                Some(config),
                None,
            );
            let _ = std::fs::remove_dir(&root);
            result
        }
        Mode::Windowed {
            file,
            root,
            workspace,
            default_folder,
            config,
            wait,
            live,
        } => {
            // THE ONE LAUNCH-PRECEDENCE LAW (item 76): explicit --root/file wins;
            // else a bare launch restores the remembered active folder (the
            // session's one owner, native + kill-switch gated); else (first run,
            // or the switch is off) the configured default folder.
            #[cfg(not(target_arch = "wasm32"))]
            let remembered = if config.session_restore_on() {
                crate::session::remembered_root()
            } else {
                None
            };
            #[cfg(target_arch = "wasm32")]
            let remembered: Option<PathBuf> = None; // session is native-only
            let default_folder_resolved = crate::args::resolve_default_folder(
                &default_folder
                    .clone()
                    .or_else(|| config.default_folder.clone()),
            );
            let active_root = resolve_launch_context(
                &root,
                &file,
                remembered.as_deref(),
                &default_folder_resolved,
            );
            // Pass the RAW flags + config; `App::new` folds them (flag > config >
            // default) and re-folds on a live config reload. `wait` (native-only,
            // the single-instance daemon's `--wait`) rides straight through, as
            // does `live` (the `--live-script` probe — see `crate::probe`).
            #[cfg(not(target_arch = "wasm32"))]
            {
                app::run(
                    file,
                    active_root,
                    workspace,
                    default_folder,
                    config,
                    wait,
                    None,
                    live,
                )
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = live; // native-live-only; parsed as None on wasm
                app::run(file, active_root, workspace, default_folder, config, wait)
            }
        }
    }
}

#[cfg(test)]
mod tests;
