use std::path::PathBuf;

use anyhow::Result;

use crate::args::Mode;
use crate::buffer::Buffer;
use crate::capture::{self, CaptureOpts};
use crate::config::Config;
use crate::keymap::Action;
use crate::replay_report::ReplayResult;
use crate::{actions, bench};

#[path = "run/buffers.rs"]
mod buffers;
#[path = "run/capture_fold.rs"]
mod capture_fold;
#[path = "run/chord.rs"]
mod chord;
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
#[path = "run/trace.rs"]
mod trace;
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
            zoom: crate::range::ZOOM.default,
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

    fn finish(self) -> ReplayResult {
        let buffers_open = self.registry.len() + 1;
        let set_wants_outline_rail = self.registry.backgrounded_wants_rail();
        let zoom_out = if self.zoom != crate::range::ZOOM.default {
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
            notice: self.notice,
            buffers_open,
            set_wants_outline_rail,
            #[cfg(test)]
            background_buffers: self.registry.text_snapshots(),
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

    /// Does any BACKGROUNDED buffer want the margin outline's rail? The working
    /// set's half of the rail reservation, read off the shared registry's own
    /// per-slot stamps — the same fact the live `App` reports, from the same
    /// type. Read by the sidecar fold through `CaptureSubject`.
    pub(crate) fn set_wants_outline_rail(&self) -> bool {
        self.registry.backgrounded_wants_rail()
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
    // Resolve the active project + index before replay so Go-to is scoped. Capture
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
    // visual-line motion reads real wrap geometry; empty specs skip it.
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
    capture_fold::apply_replay_accept(
        res.accept.as_ref(),
        &mut buffer,
        &mut opts,
        &workspace,
        &default_folder,
        &config,
    );
    if let Some((info, preview_text, diff)) = overlay_capture_info(&res.journey, &buffer) {
        opts.overlay = Some(info);
        opts.overlay_hug_roster = res
            .journey
            .card()
            .and_then(crate::overlay::OverlayState::hug_roster);
        opts.preview_text = preview_text;
        if opts.diff.is_none() {
            opts.diff = diff;
        }
        if opts.scroll.is_none() && opts.preview_text.is_some() {
            let row = res.journey.card().map(|o| o.diff_scroll).unwrap_or(0);
            opts.scroll = Some(crate::render::ScrollPos::at_row(row));
        }
    }
    opts.notice = res.notice;
    if keys.is_empty()
        && let Some((_, (l1, c1))) = opts.selection
    {
        let end = buffer.line_col_to_char(l1, c1);
        buffer.set_cursor(end);
    }
    if crate::whichkey::force_shown() {
        opts.whichkey = Some(
            crate::whichkey::continuations_cx(
                &config.keys,
                crate::convention::Convention::current(),
                config.keymap_flavor(),
            )
            .into_iter()
            .map(|c| (c.key, c.name))
            .collect(),
        );
    }
    capture_fold::apply_replay_tail(
        &mut opts,
        res.buffers_open,
        res.set_wants_outline_rail,
        &buffer,
        res.replay_skips,
    );
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
