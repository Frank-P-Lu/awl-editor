use std::path::PathBuf;

use anyhow::Result;

use crate::args::Mode;
use crate::buffer::Buffer;
use crate::capture::{self, CaptureOpts};
use crate::config::Config;
use crate::keymap::Action;
use crate::replay_report::ReplayResult;
use crate::{actions, bench};

#[path = "run/capture_fold.rs"]
mod capture_fold;
#[path = "run/effect_interpreter.rs"]
mod effect_interpreter;
/// The live-`App` capture mode (`--screenshot-app`).
#[cfg(not(target_arch = "wasm32"))]
#[path = "run/live_app.rs"]
mod live_app;
/// WHERE AM I WORKING — the launch/project-location resolvers, one owner each.
#[path = "run/location.rs"]
mod location;
mod replay_effects;
#[path = "run/settings_effects.rs"]
mod settings_effects;
/// The driver's seam — `App` implements it only on native, where the
/// `--screenshot-app` mode that reads it exists.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use capture_fold::CaptureSubject;
#[cfg(test)]
use capture_fold::comparison_preview_for;
pub(crate) use capture_fold::{fold_capture_state, overlay_capture_info};
pub(crate) use location::{launch_windowed, project_info, resolve_root, resolve_workspace};
// The launch-precedence law's own unit tests (`main/tests/launch_context.rs`) drive it directly;
// production reaches it only through `launch_windowed`, in its own module.
#[cfg(test)]
pub(crate) use location::resolve_launch_context;

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
/// THROUGH the shared `actions::apply_transition` seam, so headless replay is
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
        crate::replay::FilesystemCapability::None,
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
    filesystem: crate::replay::FilesystemCapability,
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
    km: &mut crate::keymap::KeymapState,
) -> Result<ReplayResult> {
    let policy = ReplayPolicy { mode, filesystem };
    let mut session =
        ReplaySession::new(policy, buffer, corpus, root, workspace, config, oracle, km);
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
    filesystem: crate::replay::FilesystemCapability,
    buffer: &'a mut Buffer,
    // RE-SCOPED PROJECT LOCATION — `corpus`/`root`/`workspace`
    // are OWNED, not borrowed, so [`Self::resync_project_location`] can rebuild
    // them in place the moment a Switch-project accept lands. Before this,
    // these three were fixed `&'a` borrows for the session's whole lifetime:
    // the sidecar's *accepted* location was re-derived correctly,
    // `run::project_info`), but a chord applied AFTER the accept — a Cmd-O
    // opening Goto, a Browse summon — still read the LAUNCH root's file index
    // and workspace. `docs/harness-reach.md` named the residue; this struct
    // closes it. `resync_project_location` is the ONE place any of the three
    // is ever reassigned after construction — mirrors `App::
    // resync_project_location` (`app/files/open.rs`), the live analogue.
    corpus: Vec<String>,
    root: std::path::PathBuf,
    // THE RAW `--workspace` flag (already folded over config by the caller —
    // see `project_info`'s doc), kept alongside the RESOLVED `workspace` below
    // so a re-scope can re-run `location::resolve_workspace` against the NEW
    // root: an EXPLICIT flag stays pinned across a project switch; an UNSET
    // one re-derives the new root's parent — the exact distinction `App`'s
    // `cli_workspace` vs `workspace_root` makes, so a same-parent switch and a
    // filesystem-root (no-parent) switch both re-derive honestly rather than
    // by coincidence.
    workspace_flag: Option<std::path::PathBuf>,
    workspace: std::path::PathBuf,
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
    journey: crate::overlay::Journey,
    accept: Option<(crate::overlay::OverlayKind, String)>,
    // MULTI-BUFFER REGISTRY: the same `crate::buffers::BufferRegistry` the live
    // App uses, so a `--keys` spec that Goes-to file A, edits, Goes-to file B,
    // edits, then Goes back to A sees A's PRESERVED cursor/edits/undo — the
    // v1 multi-buffer win, headlessly drivable. Carries no extra payload
    // (`()`): headless replay tracks nothing per-buffer beyond the `Buffer`
    // itself (no scroll/spell/autosave state to preserve here).
    registry: crate::buffers::BufferRegistry<()>,
    cursor_px: (f32, f32),
    /// THE CALM NOTICE this replay has raised, with its kind — the same one slot
    /// the live `App`'s frame holds, so an ordinary `--keys` capture of a
    /// notice-raising action photographs what the live editor would show. `None`
    /// until an effect raises one; a `Clear` puts it back. There is no expiry
    /// here because there is no clock (see the `CaptureSubject` impl).
    notice: Option<(String, crate::actions::NoticeKind)>,
}

pub(crate) struct ReplayPolicy {
    mode: crate::replay::Mode,
    filesystem: crate::replay::FilesystemCapability,
}

impl ReplayPolicy {
    #[cfg(test)]
    pub(crate) fn ordinary() -> Self {
        Self {
            mode: crate::replay::Mode::Permissive,
            filesystem: crate::replay::FilesystemCapability::None,
        }
    }
}

impl<'a> ReplaySession<'a> {
    #[allow(clippy::too_many_arguments)] // mirrors replay_keys_mode's own surface
    pub(crate) fn new(
        policy: ReplayPolicy,
        buffer: &'a mut Buffer,
        corpus: &[String],
        root: &std::path::Path,
        // THE RAW flag, not a pre-resolved value — see the struct's
        // `workspace_flag` doc. `project_info`'s own derivation (used for the
        // sidecar) takes the identical raw flag, so the two never disagree on
        // what "unset" means.
        workspace_flag: Option<&std::path::Path>,
        config: &'a Config,
        oracle: Option<&'a mut capture::OraclePipeline>,
        km: &'a mut crate::keymap::KeymapState,
    ) -> Self {
        let ReplayPolicy { mode, filesystem } = policy;
        let resolver = crate::keyspec::ChordResolver::new(km, mode == crate::replay::Mode::Strict);
        let workspace_flag = workspace_flag.map(|p| p.to_path_buf());
        let workspace = location::resolve_workspace(&workspace_flag, root);
        Self {
            mode,
            filesystem,
            buffer,
            corpus: corpus.to_vec(),
            root: root.to_path_buf(),
            workspace_flag,
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
            journey: crate::overlay::Journey::default(),
            accept: None,
            registry: crate::buffers::BufferRegistry::default(),
            cursor_px: (0.0, 0.0),
            notice: None,
        }
    }

    /// THE POINTER-REPLAY STEP: move the replay's pointer to
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
            && let Some(ov) = self.journey.card_mut()
        {
            pipeline.resolve_overlay_hover(ov, px, py);
        }
    }

    #[allow(dead_code)]
    fn sync_oracle_overlay(&mut self) {
        if let Some(ov) = self.journey.card()
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
        let mut work = actions::EffectWorklist::root(resolved);
        let mut pending_return_to: Option<crate::overlay::OverlayKind> = None;
        while let Some(item) = work.next() {
            let actions::EffectWorkItem::Action(action) = item else {
                let actions::EffectWorkItem::Effect { owner, effect } = item else {
                    unreachable!()
                };
                self.interpret_effect(&owner, chord, effect, &mut work, &mut pending_return_to)?;
                continue;
            };
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
                crate::assets::scan(&self.root, &self.corpus)
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
                    &self.root,
                    self.zoom,
                    crate::dateformat::CAPTURE_PLACEHOLDER_YMD,
                ),
                assets,
                // The headless capture/replay path is structurally daemon-free (never
                // imports `crate::daemon` — the daemon capture gate, docs/platform.md):
                // there is no waiter to have, so "Finish file" is deterministically
                // hidden from every `--keys`/`--screenshot` palette.
                row_gates: Default::default(),
            };
            let mut make_overlay =
                |kind: crate::overlay::OverlayKind| crate::overlay::build(kind, &build_ctx);
            let (root, workspace) = (self.root.as_path(), Some(self.workspace.as_path()));
            let mut browse_to = |kind: crate::overlay::OverlayKind, rel: Option<String>| {
                // Shared one-level builder: Project navigates the workspace by absolute
                // path; Browse and the two DESTINATION navigators (MoveDest, ExportDest)
                // walk the SAME active root — destinations list folders only.
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
                journey: &mut self.journey,
                make_overlay: &mut make_overlay,
                browse_to: &mut browse_to,
                oracle: self.oracle.as_deref().map(|op| op.as_oracle()),
            };
            let transition = actions::apply_transition(&mut ctx, &action, shift);
            let _ = ctx;
            let primary = transition.primary();
            let classified = crate::replay::classify_for(&primary, self.filesystem);
            self.records.push(replay_effects::chord_trace(
                &chord.spec,
                &action,
                &classified,
            ));
            // Apply a palette breadcrumb requested by the PREVIOUS action
            // after this action's core transition has had the chance to open
            // its child overlay, but before interpreting this action's effects.
            self.journey.attribute_launch(pending_return_to.take());
            work.expand(action, transition);
        }
        // The headless twin of `App::apply`'s own stamp: re-anchor the
        // hover movement-slop gate to the replay's CURRENT pointer position after
        // this whole chord (including any chained palette re-dispatch) has
        // applied, so a scripted keyboard nav step never leaves a stale (or
        // `None`) hover baseline for a LATER `move` step to read as unconditional
        // real motion. See `OverlayState::arm_hover_baseline`'s doc.
        if let Some(ov) = self.journey.card_mut() {
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
            journey: self.journey,
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
        self.journey.card()
    }

    /// The whole JOURNEY, for the sidecar fold: a parked parent is lifecycle
    /// state, not card content, so `overlay()` cannot answer it.
    pub(crate) fn journey(&self) -> &crate::overlay::Journey {
        &self.journey
    }

    #[cfg(test)]
    pub(crate) fn oracle(&self) -> Option<&capture::OraclePipeline> {
        self.oracle.as_deref()
    }

    pub(crate) fn buffers_open(&self) -> usize {
        self.registry.len() + 1
    }

    /// The calm notice this replay is showing, with its kind. Read by the sidecar
    /// fold through `CaptureSubject`.
    pub(crate) fn notice(&self) -> Option<(String, crate::actions::NoticeKind)> {
        self.notice.clone()
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
    let corpus = crate::index::build_index(&active_root);
    opts.project = Some(project_info(
        &active_root,
        &workspace,
        Some(default_folder.as_path()),
        &config,
    ));

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
    // Replay first so its state is what capture reflects, without clobbering
    // explicit verification hooks. The workspace already defaults to the
    // active root's parent, making project siblings available to Cmd-Shift-P.
    // With keys, shape an offscreen oracle like the upcoming capture so
    // visual-line motion reads real wrap geometry. Empty specs skip it and
    // GPU-less permissive captures retain the logical fallback.
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
    // `workspace` here is the RAW, already-config-folded flag (see
    // `project_info`'s doc) — `ReplaySession` resolves it itself, both now and
    // again on any Switch-project accept (`resync_project_location`), so the
    // two derivations can never disagree on what "unset" means.
    let res = replay_effects::capture_replay(
        strict,
        &mut buffer,
        &keys,
        &corpus,
        &active_root,
        workspace.as_deref(),
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
            // SWITCH-PROJECT: re-derive the WHOLE sidecar location from the
            // accepted root through the one builder, never a subset of it —
            // see [`project_info`] for the half-derivation this replaced. The
            // replay session ITSELF re-scoped `root`/`workspace`/`corpus` the
            // moment the accept fired (`ReplaySession::resync_project_location`,
            // `docs/harness-reach.md` names this as the live-only residue, so a
            // chord applied AFTER the
            // accept reads the new tree exactly like live.
            crate::overlay::OverlayKind::Project => {
                opts.project = Some(project_info(
                    std::path::Path::new(val),
                    &workspace,
                    Some(default_folder.as_path()),
                    &config,
                ));
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
    if let Some((info, preview_text, diff)) = overlay_capture_info(&res.journey, &buffer) {
        opts.overlay = Some(info);
        opts.preview_text = preview_text;
        if opts.diff.is_none() {
            opts.diff = diff;
        }
        if opts.scroll.is_none() && opts.preview_text.is_some() {
            let row = res.journey.card().map(|o| o.diff_scroll).unwrap_or(0);
            opts.scroll = Some(crate::render::ScrollPos::at_row(row));
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
        Mode::ScreenshotApp { out, spec } => live_app::capture_live_app(out, spec),
        #[cfg(not(target_arch = "wasm32"))]
        Mode::SemanticJson(spec) => live_app::print_semantic_json(spec),
        #[cfg(not(target_arch = "wasm32"))]
        Mode::ScreenshotFrames {
            out,
            file,
            frames,
            step_ms,
            canvas,
            dpi,
        } => {
            let buffer = load_buffer(&file);
            let opts = CaptureOpts {
                canvas,
                dpi,
                ..CaptureOpts::default()
            };
            capture::capture_frames(&out, &buffer, frames, step_ms, &opts)?;
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
        #[cfg(not(target_arch = "wasm32"))]
        Mode::BenchA11y => crate::app::semantic::bench::run(),
        Mode::BenchZoomBurst => crate::render::framebench::run_zoom_burst(),
        Mode::BenchFrost => crate::render::framebench::run_frost(),
        Mode::BenchCaret => crate::render::caretbench::run(),
        Mode::BenchSuite { baseline } => crate::render::benchsuite::run(baseline),
        #[cfg(not(target_arch = "wasm32"))]
        Mode::SoakGpu(config) => {
            let root = std::env::temp_dir().join(format!("awl-soak-gpu-{}", std::process::id()));
            std::fs::create_dir_all(&root)?;
            let result = crate::app::run(
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
        } => launch_windowed(file, root, workspace, default_folder, config, wait, live),
    }
}

#[cfg(test)]
mod tests;
