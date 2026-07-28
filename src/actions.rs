//! Pure action application shared by the windowed app and headless replay.

use crate::buffer::Buffer;
use crate::keymap::Action;
use crate::overlay::OverlayState;
use crate::search::{Direction, SearchState};

// Shared dispatch types stay here; cohesive action families live in submodules.
mod edit; // the markdown smart-Enter edit (smart_newline + its pure decision)
mod flinch; // the caret-feedback triggers (impact_for / recoil_for)
mod format; // the markdown formatting-command toggles (block + inline)
mod link; // LINKS V2 — Cmd-K insert/edit-link (plan + commit, mirrors format.rs)
mod motion; // the oracle-aware caret motions + page scroll + search open
mod overlay_nav; // the modal overlay intercept + browse-path helpers + live preview
pub(crate) mod popover; // the format-popover pure plan (reads format.rs's active-state)
mod rebind; // the game-style rebind-menu key handling
use edit::*;
use flinch::*;
use format::*;
use link::*;
use motion::*;
use overlay_nav::*;
use rebind::*;

pub(crate) use overlay_nav::{preview_move, preview_overlay};

// Shared by live and replay re-dispatch so overlay pops restore the palette.
pub(crate) use overlay_nav::stamp_return_to;

/// Renderer-owned visual-row geometry for shared motion. Columns are chars and x
/// values are pixels relative to text left; affinity resolves soft-wrap boundaries.
pub trait LayoutOracle {
    fn visual_x_of(&self, line: usize, col: usize, affinity: crate::caret::Affinity) -> f32;
    fn visual_line_up(
        &self,
        line: usize,
        col: usize,
        goal_x: f32,
        affinity: crate::caret::Affinity,
    ) -> (usize, usize);
    fn visual_line_down(
        &self,
        line: usize,
        col: usize,
        goal_x: f32,
        affinity: crate::caret::Affinity,
    ) -> (usize, usize);
    fn visual_line_start(
        &self,
        line: usize,
        col: usize,
        affinity: crate::caret::Affinity,
    ) -> (usize, usize);
    fn visual_line_end(
        &self,
        line: usize,
        col: usize,
        affinity: crate::caret::Affinity,
    ) -> (usize, usize);
}

pub struct ActionCtx<'a> {
    pub buffer: &'a mut Buffer,
    pub shift_selecting: &'a mut bool,
    pub zoom: &'a mut f32,
    pub search: &'a mut Option<SearchState>,
    /// How many logical lines one PageScrollDown/PageScrollUp moves. The windowed
    /// app passes a screenful computed from the live viewport; headless passes a
    /// fixed value (no GPU to measure), keeping replay deterministic.
    pub scroll_page_lines: usize,
    /// The SUMMONED navigation overlay. `None` = editing normally; `Some` = the
    /// go-to / switch-project overlay is open, and while it is, typed chars edit
    /// the overlay query (NOT the buffer), Up/Down move the selection, Enter
    /// accepts, Esc/C-g cancels. Putting this in the shared core (not just `App`)
    /// is what makes the overlay drivable from the headless `--keys` replay.
    pub overlay: &'a mut Option<OverlayState>,
    /// The active project context the overlay needs when it OPENS: a builder that
    /// produces a fresh `OverlayState` for a given kind. The core can't read the
    /// filesystem itself (and headless replay must stay deterministic), so the
    /// caller injects this; `OpenGoto`/`OpenProject` invoke it.
    pub make_overlay: &'a mut dyn FnMut(crate::overlay::OverlayKind) -> Option<OverlayState>,
    pub browse_to:
        &'a mut dyn FnMut(crate::overlay::OverlayKind, Option<String>) -> Option<OverlayState>,
    /// The visual-line motion LAYOUT ORACLE (the SHAPED text's wrap geometry),
    /// supplied by the live GPU pipeline (`app.rs`) and the headless offscreen
    /// pipeline (`capture.rs`) so the two flows can't drift. `None` in the pure
    /// `apply_core` unit tests (no pipeline), where motion falls back to LOGICAL
    /// lines. Consulted by the vertical (C-n/C-p, Up/Down), line-edge (C-a/C-e,
    /// Home/End) and kill-line (C-k) motions, which follow the SHAPED visual rows
    /// whenever it is present (the flat default).
    pub oracle: Option<&'a dyn LayoutOracle>,
}

/// The single deferred side effect an `apply_core` call signals back to its
/// caller. The pure core can't touch the filesystem, GPU, window, or the caller's
/// buffer history, so rather than PERFORM those effects it RETURNS one of these
/// for the caller to carry out. The signalling paths are mutually exclusive — at
/// most one effect fires per call — so the caller matches ONCE and leans on
/// exhaustiveness. This replaces the former cluster of `&mut` out-params.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    None,
    Quit,
    LastBuffer,
    /// Cmd-N: swap in a fresh, unnamed document buffer IN THE ACTIVE FOLDER
    /// (item 76 retired the old separate-notes-root jump). The buffer-swap is
    /// caller-level (the core never touches the filesystem/window).
    NewDocument,
    OpenSettings,
    OpenCredits,
    OpenGuide,
    RunAction(Action),
    /// An overlay ACCEPTED (Enter on a selected item, or a Theme cancel-revert):
    /// the chosen value — a root-relative path for Goto/Browse, an absolute dir for
    /// Project, a notes-root-relative folder for MoveDest, or a world name for
    /// Theme — for the caller to act on (load the file / switch the root / move the
    /// note / re-tint). The core never touches the filesystem, GPU, or window.
    OverlayAccept(crate::overlay::OverlayKind, String),
    /// Go-to's HEADINGS lens accepted (Enter on a heading row): JUMP the cursor to
    /// document line `.0` (0-based). The fold that retired the standalone Outline
    /// picker — a heading row's accept is a cursor move, not a file open, so it rides
    /// its own effect rather than `OverlayAccept(Goto, …)` (which opens a path). The
    /// caller moves the cursor (live App + headless replay both); the core never
    /// touches the buffer here.
    JumpToLine(usize),
    /// SPELL picker "Add '<word>' to dictionary" accepted (Enter on the add row):
    /// add `.0` to the USER (personal) dictionary — the live App
    /// ([`crate::app::App::add_to_dictionary`]) both silences the word in the live
    /// checker AND appends it to the on-disk word list beside `config.toml`, then
    /// rescans so the squiggle clears THIS frame. The pure core can't reach the fs
    /// / the checker, so it signals this. LIVE-APP-ONLY: the headless `--keys`
    /// replay no-ops it (a capture must never write the dictionary file — the same
    /// determinism gate `KeepVersion`/`Export` sit behind); the load/persist itself
    /// is unit-tested at the App seam, and the picker's own add-row select/accept
    /// flow IS core-driven and fully `--keys`-drivable up to this signal. ZERO-
    /// NETWORK: a file append, never a fetch.
    AddToDictionary(String),
    /// REBIND MENU committed a capture: write `binding` into the `[keys]` SLOT of the
    /// command `slug` (the caller persists to config + live-reloads). `confirmed` is
    /// true when the user already accepted a CONFLICT warning (Confirm stage), so the
    /// caller must NOT re-gate on the clash. The core leaves the overlay open (the
    /// menu stays up); the caller refreshes its bindings + notice after the reload.
    RebindCommit {
        slug: String,
        binding: String,
        confirmed: bool,
    },
    RebindReset {
        slug: String,
    },
    /// A discrete action was REQUESTED but could NOT PROCEED (a motion into a wall,
    /// a page that can't page further, an exhausted undo/redo, a delete with nothing
    /// to remove). The caller bumps the VISUAL caret in `dir` — away from the wall —
    /// via [`crate::caret::CaretAnim::recoil`]; the spring self-settles it back to
    /// rest. The buffer/cursor are UNCHANGED (that's the whole point — it's a
    /// blocked action), so a settled capture stays byte-identical; the headless
    /// replay simply ignores it (no clock/animation). Mutually exclusive with the
    /// real effects: a recoil only arms when the action produced no other effect.
    Recoil(crate::caret::RecoilDir),
    TypeImpact,
    DeleteSquash,
    Gulp,
    /// PHASE 3 — ENTER JUICE / LINE LANDING: Enter SUCCESSFULLY inserted a newline
    /// (including the markdown smart-Enter continue/end-block edits). The caller
    /// gives the VISUAL caret a caret-level "touchdown" squash via
    /// [`crate::caret::CaretAnim::line_land`] — CARET-LEVEL ONLY (no content
    /// reflow / row animation; rows never dance). Live-only, byte-identical
    /// settled; the headless replay ignores it (no clock).
    LineLand,
    /// C-x #: the core already SAVED the buffer (identically to [`Action::Save`]).
    /// The caller notifies any daemon `--wait` client waiting on this buffer (a
    /// live-App-only concern — the pure core can't reach the socket) and switches
    /// to the previously-open buffer (the same swap `Effect::LastBuffer` performs).
    /// Headless replay treats this exactly like `LastBuffer` — a no-op (no daemon,
    /// no 2-deep history in a one-shot replay).
    FinishBuffer,
    /// THE CONSCIOUS MARK ("Keep version…"): the naming minibuffer COMMITTED —
    /// record the current buffer as a PINNED, prune-EXEMPT local-history snapshot,
    /// optionally NAMED (`Some("draft A")` when the user typed a name, `None` for
    /// a blank Enter — the plain, zero-friction keep). The pure core can't reach
    /// the store (no fs / config / buffer path), so it signals this for the live
    /// App to perform ([`crate::app::App::keep_version`] →
    /// [`crate::history::record_pinned`]). LIVE-APP-ONLY: the headless `--keys`
    /// replay no-ops it (the history determinism gate — a capture never touches
    /// the store), so a settled frame stays byte-identical; the naming
    /// minibuffer's open/type/cancel flow itself IS core-driven and fully
    /// `--keys`-drivable (see `overlay_nav`'s `keep_edit` block).
    KeepVersion {
        name: Option<String>,
    },
    /// C-c C-o (follow-link-at-point): the caret sat inside a markdown link, whose
    /// destination URL is carried here for the caller to open in the OS default
    /// browser (a user-initiated handoff — the app never fetches it, so the
    /// zero-network invariant holds). LIVE-APP-ONLY: `App::follow_link` performs the
    /// `open`/`xdg-open`/`window.open` launch; the headless `--keys` replay no-ops it
    /// (a capture must never spawn a browser), so a settled frame stays byte-identical.
    /// A caret OUTSIDE every link never produces this effect (`Effect::None`, the calm
    /// no-op) — `Action::FollowLink` only arms it when `markdown::link_at` is `Some`.
    FollowLink(String),
    /// "Report a Problem": the pure core can't reach the crash-log directory or
    /// the OS mail client (no fs / no OS handoff seam in `ActionCtx`), so it
    /// signals a bare request for the live App to compose the `mailto:` URL
    /// (`crashlog::report_problem_mailto`, pulling in the newest crash log's
    /// PATH if one exists — native only) and open it through the SAME seam
    /// [`Effect::FollowLink`] uses (`App::follow_link`). LIVE-APP-ONLY: headless
    /// `--keys` replay no-ops it (never composes a URL, never spawns anything),
    /// so a settled capture stays byte-identical. See `crashlog.rs`.
    ReportProblem,
    /// "Download file" (WEB-ONLY): the pure core can't touch `web_sys` (no DOM
    /// handoff seam in `ActionCtx`), so it signals a bare request for the live App
    /// to build a Blob + object URL from the active buffer's text and click a
    /// synthetic `<a download>` (`App::download_file`, `web_export.rs`). Gated off
    /// on native entirely (`commands::action_available`, `web_only: true` — see
    /// `commands.rs`), so this variant can only ever be produced on the web
    /// build. LIVE-APP-ONLY: headless `--keys` replay no-ops it (never touches the
    /// DOM), so a settled capture stays byte-identical. See `web_export.rs`.
    DownloadFile,
    /// EXPORT (`Action::ExportWord` / `Action::ExportHtml` / `Action::ExportPdf`):
    /// render the ACTIVE markdown buffer to a `.docx`, standalone `.html`, or
    /// native `.pdf` document. The pure core can't reach the filesystem
    /// (sibling-file write) or the DOM (web download),
    /// and image embedding reads the doc's `assets/` off disk — all caller-level
    /// concerns — so it signals the requested [`crate::export::Format`] for the
    /// live App to perform (`App::export_document`, `export/`). Only produced for
    /// a markdown buffer (the export action arms gate on
    /// `Buffer::is_markdown`; a `.rs`/`.txt` buffer is a calm no-op, mirroring the
    /// format toggles). LIVE-APP-ONLY: headless `--keys` replay never writes a
    /// file or touches the DOM, so a settled capture stays byte-identical.
    Export(crate::export::Format),
    /// "Check for Updates": the pure core can't reach the fs marker or the OS
    /// browser handoff, so it signals a bare request for the live App to (a)
    /// record a LOCAL "last checked" marker (`updates::record_checked`,
    /// best-effort, mirroring `crashlog::acknowledge`) and (b) compose
    /// [`crate::updates::check_url`] and open it through the SAME seam
    /// [`Effect::FollowLink`]/[`Effect::ReportProblem`] use
    /// (`App::follow_link`) — never a fetch FROM this binary; the actual
    /// version comparison happens in the browser. LIVE-APP-ONLY: headless
    /// `--keys` replay no-ops it (never writes the marker, never spawns
    /// anything), so a settled capture stays byte-identical. See `updates.rs`.
    CheckForUpdates,
    /// COPY PULSE: M-w / Cmd-C successfully copied a NON-EMPTY selection into the
    /// kill ring — copy's one common but otherwise INVISIBLE result finally gets an
    /// in-world confirmation. The caller plays a gentle caret kick
    /// ([`crate::caret::CaretAnim::copy_pulse`], distinct from every edit flinch —
    /// nothing was edited) AND brightens the selection quad's own tint, decaying
    /// back over the live clock (`TextPipeline::copy_pulse`) — "obvious and
    /// understated", never amber. Unlike the edit flinches this never touches the
    /// buffer content, so it can't ride `impact_for`'s version-changed gate; see
    /// `copy_pulse_for`. Live-only, byte-identical settled: the headless replay
    /// ignores it (no clock), and `has_selection() == false` (an empty-selection
    /// copy) never arms it — that stays the documented no-op.
    ///
    /// DESIGN CALL, logged: `DESIGN.md` §3 states "the caret is the only thing
    /// allowed juice… selection, errors: Calm, geometric, precise. No juice." This
    /// round is a deliberate, user-approved, NARROW exception — the selection
    /// brightens only as a direct, one-shot REACTION to the caret's own copy
    /// action (never ambient, never idle chrome), and decays back to the exact
    /// same calm rendering within `COPY_PULSE_MS`. Flagged here rather than
    /// silently widening the law; a future pass may want to fold this into an
    /// explicit `DESIGN.md` amendment (mirroring the WYSIWYG conceal-on-cursor
    /// round's own "settled 2026-07" `PHILOSOPHY.md` amendment) rather than
    /// leaving it as an unstated one-off.
    CopyPulse,
    /// SETTINGS MENU: Enter on a TOGGLE row (page mode / wysiwyg / spellcheck / …).
    /// The core can't flip a process-global-and-persist (no config path / GPU), so it
    /// signals the sticky `key` back for the caller to (a) flip the live global +
    /// re-render this frame, (b) `persist_pref` the negated value into `config.toml`,
    /// and (c) refresh the STILL-OPEN menu's value cell (`App::setting_toggle`). The
    /// core leaves the overlay open (the menu stays up); the `key` is the config key
    /// from [`crate::settings::toggle_key`]. Headless replay reflects nothing (the
    /// capture path has no live global setter / config write) — a no-op there.
    SettingToggle {
        key: String,
    },
    /// SETTINGS MENU: Enter COMMITTED an inline VALUE edit (page widths / zoom). The
    /// core built + committed the typed `value` for config `key`; it can't parse-clamp-
    /// apply-persist (no config path / GPU / zoom owner), so it signals the raw typed
    /// string back for the caller to parse + clamp (`settings::clamp_page_width` /
    /// `settings::parse_zoom`), apply LIVE (`page::set_measure` via `sync_page_measure`
    /// / `set_zoom`), persist the NAMED key, and refresh the still-open menu's cell
    /// (`App::setting_value_commit`). The core already cleared the value-edit sub-state
    /// (the menu stays open). Headless replay reflects nothing (no live setter / config).
    SettingValueCommit {
        key: String,
        value: String,
    },
    SettingPathPick {
        key: String,
        path: String,
    },
    SettingRangeStep {
        key: String,
    },
    /// ASSET CLEANER: Enter on an orphan row REQUESTED that its file (root-relative
    /// `rel`) be moved to the OS Trash. The pure core can't reach the Trash / the
    /// filesystem (no root, no [`crate::assets::TrashCan`]), so it signals `rel` back
    /// for the live App to (a) trash `self.root.join(rel)` via the trash seam and (b),
    /// on success, REMOVE that row from the still-open picker
    /// ([`crate::overlay::OverlayState::remove_asset_row`]) — the picker stays open. The
    /// core leaves the overlay OPEN and does NOT remove the row (the determinism gate:
    /// a headless `--keys` replay no-ops this effect, so its orphan list stays whole
    /// and the sidecar never claims a file was trashed that wasn't). A trash FAILURE
    /// leaves the row + shows a calm notice. LIVE-APP-ONLY; a default `--screenshot`
    /// never reaches it (the command is summon-by-name).
    TrashAsset {
        rel: String,
    },
    /// SAVE-FEEDBACK round: manual save (`Action::Save`) on the TRUE scratch
    /// surface — no path, never named as a note — cannot be performed by the
    /// core alone (converting it into a real document needs the ACTIVE
    /// folder, which `ActionCtx` doesn't carry — a project-level concern, not
    /// a buffer one). The caller (live App / headless `--keys` replay) calls
    /// [`crate::buffer::Buffer::save_into_folder`] with its own active
    /// folder — reusing the SAME auto-name machinery `App::ensure_note_named_before_paste`
    /// already established for the paste-image door — then finishes the
    /// bookkeeping a normal save would have (title, go-to index, sticky page
    /// measure, a "saved"/"save failed: …" notice). A buffer that is ALREADY a
    /// note (even unnamed) or already pathed never produces this — see the
    /// `Action::Save` arm's own gate. USER-FLIPPABLE (logged): a future
    /// preference could make this notice-only instead ("nothing to save yet —
    /// start a note first") rather than silently promoting scratch to a note.
    ConvertScratchAndSave,
    SaveDone {
        ok: bool,
        message: String,
    },
    /// NOTES VERBS round: the RENAME minibuffer committed (Enter while the Rename
    /// overlay's `rename_edit` sub-state was active) — the core already CLOSED the
    /// overlay; `new_name` is the typed filename for the caller to act on. The pure
    /// core can't touch the filesystem, so the caller ([`crate::app::App::rename_current_file`])
    /// performs the actual disk rename + the ONE-owner path-keyed bookkeeping (buffer
    /// path, history log, file index) — refusing calmly (a notice, no write) on a
    /// git-managed file or a name collision, never clobbering. An UNCHANGED or blank
    /// typed name is a quiet no-op (the caller's own gate). LIVE-APP-ONLY: the
    /// headless `--keys` replay treats this like `MoveDest`'s own accept (reflected in
    /// the sidecar via the overlay's live prompt while typing; the actual disk rename
    /// is live-App-only, mirroring `move_current_file`'s own precedent), so a settled
    /// capture never mutates the filesystem.
    RenameNoteCommit {
        new_name: String,
    },
    /// NOTES VERBS round: DUPLICATE the current file (`Action::DuplicateNote`) — the
    /// pure core can't reach the filesystem, so it signals the request for the caller
    /// to copy the CURRENT buffer content to an auto-named sibling (the same
    /// no-clobber dedup [`crate::buffer::unique_path`] uses) and open the copy as the
    /// active buffer (parking the original first, so its own live edits are never
    /// lost — a fresh history timeline, since the copy is a genuinely new file). A
    /// pathless buffer (scratch / an unnamed note) is a calm no-op — there is nothing
    /// to duplicate yet. See [`crate::app::App::duplicate_current_file`].
    DuplicateNote,
    InsertDate,
}

/// Apply one resolved `action` to the editor core. `shift` is whether Shift was
/// held (so a motion extends the selection, Shift+Arrow style). Returns the one
/// deferred [`Effect`] the action signals back to the caller (`Effect::None` for
/// the common case) — the caller carries out the filesystem/window/quit work the
/// pure core can't. Mutates only what `ActionCtx` exposes; no GPU, window, or
/// clipboard.
fn apply_view_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    match action {
        Action::ToggleCaretMode => {
            crate::caret::toggle_mode();
        }
        Action::TogglePageMode => {
            crate::page::toggle();
        }
        Action::PageWider => {
            crate::page::widen();
        }
        Action::PageNarrower => {
            crate::page::narrow();
        }
        Action::PageReset => {
            crate::page::set_measure(ctx.buffer.page_class().default_measure());
        }
        Action::ToggleDebug => {
            crate::debug::toggle();
        }
        Action::ToggleOutline => {
            crate::outline::toggle();
        }
        Action::ToggleFold => {
            ctx.buffer.toggle_fold_at_cursor();
        }
        Action::CollapseOtherSections => {
            ctx.buffer.collapse_other_sections();
        }
        Action::ToggleMenuBar => {
            crate::menubar::toggle();
        }
        Action::ToggleTypewriter => {
            crate::typewriter::toggle();
        }
        Action::ShowStatsHud => {
            crate::hud::set_held(true);
        }
        Action::About => {
            crate::about::set_open(true);
        }
        Action::LifetimeStats => {
            crate::lifetime::set_open(true);
        }
        Action::WritingStreaks => {
            crate::streaks::set_open(true);
        }
        Action::ConvertLineEndings => {
            ctx.buffer.set_eol(ctx.buffer.eol().toggled());
        }
        Action::ReportProblem => return Some(Effect::ReportProblem),
        Action::DownloadFile => return Some(Effect::DownloadFile),
        Action::CheckForUpdates => return Some(Effect::CheckForUpdates),
        _ => return None,
    }
    Some(Effect::None)
}

fn apply_format_action(ctx: &mut ActionCtx, action: &Action) -> bool {
    match action {
        Action::ToggleBlockquote => apply_block_format(ctx, format::BlockKind::Blockquote),
        Action::ToggleBulletList => apply_block_format(ctx, format::BlockKind::Bullet),
        Action::ToggleNumberedList => apply_block_format(ctx, format::BlockKind::Numbered),
        Action::ToggleTaskList => apply_block_format(ctx, format::BlockKind::Task),
        Action::ToggleHeading => apply_block_format(ctx, format::BlockKind::Heading),
        Action::HeadingCycle => format::apply_heading_cycle(ctx),
        Action::ToggleCodeBlock => apply_block_format(ctx, format::BlockKind::CodeBlock),
        Action::Bold => apply_inline_format(ctx, format::InlineKind::Bold),
        Action::Italic => apply_inline_format(ctx, format::InlineKind::Italic),
        Action::InlineCode => apply_inline_format(ctx, format::InlineKind::InlineCode),
        Action::Highlight => apply_inline_format(ctx, format::InlineKind::Highlight),
        Action::Strikethrough => apply_inline_format(ctx, format::InlineKind::Strikethrough),
        _ => return false,
    }
    true
}

fn apply_buffer_action(ctx: &mut ActionCtx, action: &Action) -> bool {
    match action {
        Action::ForwardChar => ctx.buffer.forward_char(),
        Action::BackwardChar => ctx.buffer.backward_char(),
        Action::NextLine => vertical_motion(ctx, true),
        Action::PreviousLine => vertical_motion(ctx, false),
        Action::LineStart => line_edge_motion(ctx, false),
        Action::LineEnd => line_edge_motion(ctx, true),
        Action::ForwardWord => ctx.buffer.forward_word(),
        Action::BackwardWord => ctx.buffer.backward_word(),
        Action::BufferStart => ctx.buffer.buffer_start(),
        Action::BufferEnd => ctx.buffer.buffer_end(),
        Action::InsertChar(c) => ctx.buffer.insert_char(*c),
        Action::Newline => {
            if !smart_newline(ctx) {
                ctx.buffer.insert_newline();
            }
        }
        Action::InsertTab => list_tab(ctx),
        Action::Outdent => list_outdent(ctx),
        Action::DeleteBackward => ctx.buffer.delete_backward(),
        Action::DeleteWordBackward => ctx.buffer.delete_word_backward(),
        Action::DeleteWordForward => ctx.buffer.delete_word_forward(),
        Action::DeleteToLineStart => ctx.buffer.delete_to_line_start(),
        Action::DeleteForward => ctx.buffer.delete_forward(),
        Action::KillLine => kill_line_motion(ctx),
        Action::Yank => ctx.buffer.yank(),
        Action::Undo => {
            ctx.buffer.undo();
            *ctx.shift_selecting = false;
        }
        Action::Redo => {
            ctx.buffer.redo();
            *ctx.shift_selecting = false;
        }
        Action::SetMark => {
            ctx.buffer.set_mark();
            *ctx.shift_selecting = false;
        }
        Action::CopyRegion => ctx.buffer.copy_region(),
        Action::KillRegion => ctx.buffer.kill_region(),
        Action::SelectAll => {
            ctx.buffer.select_all();
            *ctx.shift_selecting = false;
        }
        _ => return false,
    }
    true
}

fn apply_viewport_action(ctx: &mut ActionCtx, action: &Action) -> bool {
    match action {
        Action::ZoomIn => *ctx.zoom = crate::range::ZOOM.stepped(*ctx.zoom, 1),
        Action::ZoomOut => *ctx.zoom = crate::range::ZOOM.stepped(*ctx.zoom, -1),
        Action::ZoomReset => *ctx.zoom = crate::range::ZOOM.default,
        Action::PageScrollDown => scroll_page(ctx.buffer, ctx.scroll_page_lines, true),
        Action::PageScrollUp => scroll_page(ctx.buffer, ctx.scroll_page_lines, false),
        _ => return false,
    }
    true
}

fn apply_session_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let effect = match action {
        Action::Save => {
            if ctx.buffer.path().is_none() && !ctx.buffer.is_unnamed_fresh() {
                Effect::ConvertScratchAndSave
            } else {
                match ctx.buffer.save() {
                    Ok(()) => Effect::SaveDone {
                        ok: true,
                        message: "saved".to_string(),
                    },
                    Err(e) => Effect::SaveDone {
                        ok: false,
                        message: format!("save failed: {e}"),
                    },
                }
            }
        }
        Action::Quit => Effect::Quit,
        Action::Cancel => {
            ctx.buffer.clear_mark();
            *ctx.shift_selecting = false;
            Effect::None
        }
        Action::SearchForward => {
            start_search(ctx, Direction::Forward);
            Effect::None
        }
        Action::SearchBackward => {
            start_search(ctx, Direction::Backward);
            Effect::None
        }
        Action::OpenReplace => {
            start_search(ctx, Direction::Forward);
            if let Some(st) = ctx.search.as_mut() {
                st.reveal_replace();
            }
            Effect::None
        }
        _ => return None,
    };
    Some(effect)
}

fn apply_deferred_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let effect = match action {
        Action::LastBuffer => Effect::LastBuffer,
        Action::NewDocument => Effect::NewDocument,
        Action::MoveFile => {
            *ctx.overlay = (ctx.browse_to)(crate::overlay::OverlayKind::MoveDest, None);
            Effect::None
        }
        Action::OpenRenameNote => {
            if let Some(path) = ctx.buffer.path() {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                *ctx.overlay = Some(OverlayState::new_rename(name));
            }
            Effect::None
        }
        Action::DuplicateNote => Effect::DuplicateNote,
        Action::OpenSettings => Effect::OpenSettings,
        Action::OpenCredits => Effect::OpenCredits,
        Action::OpenGuide => Effect::OpenGuide,
        Action::OpenSettingsMenu => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Settings);
            Effect::None
        }
        Action::FinishBuffer => {
            let _ = ctx.buffer.save();
            Effect::FinishBuffer
        }
        Action::FollowLink => {
            crate::markdown::link_at(&ctx.buffer.text(), ctx.buffer.cursor_byte())
                .map(Effect::FollowLink)
                .unwrap_or(Effect::None)
        }
        Action::BeginPrefix | Action::Ignore => Effect::None,
        _ => return None,
    };
    Some(effect)
}

struct ActionSnapshot {
    cursor_before: usize,
    version_before: u64,
    could_undo: bool,
    could_redo: bool,
    had_selection_before: bool,
}

impl ActionSnapshot {
    fn capture(ctx: &ActionCtx) -> Self {
        Self {
            cursor_before: ctx.buffer.cursor_char(),
            version_before: ctx.buffer.version(),
            could_undo: ctx.buffer.can_undo(),
            could_redo: ctx.buffer.can_redo(),
            had_selection_before: ctx.buffer.has_selection(),
        }
    }
}

fn finish_action(
    ctx: &mut ActionCtx,
    action: &Action,
    snapshot: ActionSnapshot,
    mut effect: Effect,
) -> Effect {
    if !action.is_edit() && !matches!(action, Action::Undo | Action::Redo) {
        ctx.buffer.seal_undo_group();
    }
    if !ctx.buffer.has_selection() {
        *ctx.shift_selecting = false;
    }
    ctx.buffer.reveal_placement();
    if effect == Effect::None
        && let Some(dir) = recoil_for(
            action,
            ctx,
            snapshot.cursor_before,
            snapshot.version_before,
            snapshot.could_undo,
            snapshot.could_redo,
        )
    {
        effect = Effect::Recoil(dir);
    }
    if effect == Effect::None
        && let Some(impact) = impact_for(action, snapshot.version_before, ctx)
    {
        effect = impact;
    }
    if effect == Effect::None
        && let Some(copy_pulse) = copy_pulse_for(action, snapshot.had_selection_before)
    {
        effect = copy_pulse;
    }
    effect
}

fn intercept_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    if !crate::commands::action_available(action, crate::commands::Platform::current()) {
        return Some(Effect::None);
    }
    if crate::streaks::streaks_open()
        && matches!(action, Action::ForwardChar | Action::BackwardChar)
    {
        crate::streaks::toggle_view();
        return Some(Effect::None);
    }
    if crate::card::dismiss_summoned_card() {
        return Some(Effect::None);
    }
    ctx.overlay
        .is_some()
        .then(|| overlay_intercept(ctx, action))
}

fn apply_export_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let effect = match action {
        Action::ExportWord if ctx.buffer.is_markdown() => {
            Effect::Export(crate::export::Format::Docx)
        }
        Action::ExportHtml if ctx.buffer.is_markdown() => {
            Effect::Export(crate::export::Format::Html)
        }
        Action::ExportWord | Action::ExportHtml => Effect::None,
        Action::ExportPdf => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if ctx.buffer.is_markdown() {
                    Effect::Export(crate::export::Format::Pdf)
                } else {
                    Effect::None
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                Effect::None
            }
        }
        Action::InsertLink => {
            open_insert_link(ctx);
            Effect::None
        }
        Action::InsertDate => Effect::InsertDate,
        _ => return None,
    };
    Some(effect)
}

fn apply_overlay_open_action(ctx: &mut ActionCtx, action: &Action) -> bool {
    match action {
        Action::OpenGoto => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Goto);
        }
        Action::OpenProject => {
            *ctx.overlay = (ctx.browse_to)(crate::overlay::OverlayKind::Project, None);
        }
        Action::OpenRecentProjects => {
            let mut ov = (ctx.browse_to)(crate::overlay::OverlayKind::Project, None);
            if let Some(o) = ov.as_mut() {
                o.focus_facet_id("recent");
            }
            *ctx.overlay = ov;
        }
        Action::OpenThemeMenu => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Theme);
        }
        Action::OpenCaretMenu => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Caret);
        }
        Action::OpenDictionaryMenu => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Dictionary);
        }
        // Toggling spellcheck is a pure render/detection concern (no buffer change).
        // The process-global flip lives HERE on the shared seam (like the page/caret
        // toggles); `App::apply` persists the sticky pref + forces an immediate
        // rescan as a post-`apply_core` side effect the core can't reach. A
        // `--keys "..."` capture renders (and records in its sidecar) the toggled
        // state — every `misspellings_for`/`suggest_at` call already reads the
        // global fresh, so the flip is visible with no extra plumbing headlessly.
        Action::ToggleSpellcheck => {
            crate::spell::toggle();
        }
        Action::ToggleWritingNits => {
            crate::nits::toggle();
        }
        Action::OpenCommandPalette => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Command);
        }
        Action::OpenKeybindings => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Keybindings);
        }
        // "Go to heading…" (palette): open GO-TO pre-lensed onto its HEADINGS lens —
        // the fold that retired the standalone Outline picker. `make_overlay` builds
        // the Go-to overlay with the doc's headings already folded in (its Headings
        // lens's corpus); focusing the `headings` lens opens it showing them. Over a
        // buffer with no headings the lens reads "no headings yet" (never a no-op —
        // the file list is still there behind the other lenses; also reachable via
        // ⌘O → ←/→).
        Action::OpenOutline => {
            let mut ov = (ctx.make_overlay)(crate::overlay::OverlayKind::Goto);
            if let Some(o) = ov.as_mut() {
                o.focus_facet_id("headings");
            }
            *ctx.overlay = ov;
        }
        Action::OpenSpellSuggest => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Spell);
        }
        // Cmd-Shift-H: summon the HISTORY TIMELINE picker for the current file. The
        // caller's `make_overlay` gathers the file's versions (via
        // `history::timeline_rows`); an empty history still opens (the calm "no
        // history yet" row), so this is never a silent no-op. Enter then RESTORES the
        // highlighted version into the buffer as an undoable edit.
        Action::OpenHistory => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::History);
        }
        // Cmd-P → "Clean unused assets…": summon the ASSET CLEANER. The caller's
        // `make_overlay` builds it from the scanned orphan list (`assets::scan`,
        // threaded via `BuildCtx::assets`); an empty list still opens (the calm "no
        // unused assets" row), so this is never a silent no-op. Enter then requests the
        // highlighted orphan be trashed (`Effect::TrashAsset`), keeping the picker open.
        Action::OpenAssetClean => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Assets);
        }
        Action::KeepVersion => {
            *ctx.overlay = Some(OverlayState::new_keep_name());
        }
        // DIFF-AS-PREVIEW ("Compare with version…" from the BUFFER): the palette
        // command REPOINTS to opening the HISTORY picker — whose live preview IS
        // the writer's diff now (arrowing the versions shows each one's marked-up
        // manuscript in the page below the card). ONE behavior, no orphaned second
        // mode: the old read-only takeover view is retired. From an OPEN History
        // picker this action is intercepted earlier (`overlay_nav`'s Tab arm — the
        // focus shift into the diff panel) and never reaches here.
        Action::CompareVersion => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::History);
        }
        Action::OpenBrowse => {
            *ctx.overlay = (ctx.browse_to)(crate::overlay::OverlayKind::Browse, None);
        }
        _ => return false,
    }
    true
}

enum ActionFamily {
    Buffer,
    Viewport,
    Session,
    View,
    Align,
    Format,
    Export,
    Overlay,
    Deferred,
}

macro_rules! classify_action_family {
    ($action:expr) => {
        match $action {
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
            | Action::InsertChar(_)
            | Action::Newline
            | Action::InsertTab
            | Action::Outdent
            | Action::DeleteBackward
            | Action::DeleteWordBackward
            | Action::DeleteWordForward
            | Action::DeleteToLineStart
            | Action::DeleteForward
            | Action::KillLine
            | Action::Yank
            | Action::Undo
            | Action::Redo
            | Action::SetMark
            | Action::CopyRegion
            | Action::KillRegion
            | Action::SelectAll => ActionFamily::Buffer,
            Action::ZoomIn
            | Action::ZoomOut
            | Action::ZoomReset
            | Action::PageScrollDown
            | Action::PageScrollUp => ActionFamily::Viewport,
            Action::Save
            | Action::Quit
            | Action::Cancel
            | Action::SearchForward
            | Action::SearchBackward
            | Action::OpenReplace => ActionFamily::Session,
            Action::ToggleCaretMode
            | Action::TogglePageMode
            | Action::PageWider
            | Action::PageNarrower
            | Action::PageReset
            | Action::ToggleDebug
            | Action::ToggleOutline
            | Action::ToggleFold
            | Action::CollapseOtherSections
            | Action::ToggleMenuBar
            | Action::ToggleTypewriter
            | Action::ShowStatsHud
            | Action::About
            | Action::LifetimeStats
            | Action::WritingStreaks
            | Action::ConvertLineEndings
            | Action::ReportProblem
            | Action::DownloadFile
            | Action::CheckForUpdates => ActionFamily::View,
            Action::AlignTable => ActionFamily::Align,
            Action::ToggleBlockquote
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
            | Action::Strikethrough => ActionFamily::Format,
            Action::ExportWord
            | Action::ExportHtml
            | Action::ExportPdf
            | Action::InsertLink
            | Action::InsertDate => ActionFamily::Export,
            Action::OpenGoto
            | Action::OpenProject
            | Action::OpenRecentProjects
            | Action::OpenThemeMenu
            | Action::OpenCaretMenu
            | Action::OpenDictionaryMenu
            | Action::ToggleSpellcheck
            | Action::ToggleWritingNits
            | Action::OpenCommandPalette
            | Action::OpenKeybindings
            | Action::OpenOutline
            | Action::OpenSpellSuggest
            | Action::OpenHistory
            | Action::OpenAssetClean
            | Action::KeepVersion
            | Action::CompareVersion
            | Action::OpenBrowse => ActionFamily::Overlay,
            Action::LastBuffer
            | Action::NewDocument
            | Action::MoveFile
            | Action::OpenRenameNote
            | Action::DuplicateNote
            | Action::OpenSettings
            | Action::OpenCredits
            | Action::OpenGuide
            | Action::OpenSettingsMenu
            | Action::FinishBuffer
            | Action::FollowLink
            | Action::BeginPrefix
            | Action::Ignore => ActionFamily::Deferred,
        }
    };
}

fn action_family(action: &Action) -> ActionFamily {
    classify_action_family!(action)
}

fn dispatch_editor_action(ctx: &mut ActionCtx, action: &Action, family: ActionFamily) -> Effect {
    let mut effect = Effect::None;
    match family {
        ActionFamily::Buffer => debug_assert!(apply_buffer_action(ctx, action)),
        ActionFamily::Viewport => debug_assert!(apply_viewport_action(ctx, action)),
        ActionFamily::Session => {
            effect = apply_session_action(ctx, action).expect("session action")
        }
        ActionFamily::View => effect = apply_view_action(ctx, action).expect("view action"),
        ActionFamily::Align => align_table_at_cursor(ctx),
        ActionFamily::Format => debug_assert!(apply_format_action(ctx, action)),
        ActionFamily::Export => effect = apply_export_action(ctx, action).expect("export action"),
        ActionFamily::Overlay | ActionFamily::Deferred => {
            unreachable!("command family routed to editor dispatcher")
        }
    }
    effect
}

fn dispatch_command_action(ctx: &mut ActionCtx, action: &Action, family: ActionFamily) -> Effect {
    let mut effect = Effect::None;
    match family {
        ActionFamily::Overlay => debug_assert!(apply_overlay_open_action(ctx, action)),
        ActionFamily::Deferred => {
            effect = apply_deferred_action(ctx, action).expect("deferred action");
        }
        ActionFamily::Buffer
        | ActionFamily::Viewport
        | ActionFamily::Session
        | ActionFamily::View
        | ActionFamily::Align
        | ActionFamily::Format
        | ActionFamily::Export => unreachable!("editor family routed to command dispatcher"),
    }
    effect
}

fn dispatch_action(ctx: &mut ActionCtx, action: &Action) -> Effect {
    let family = action_family(action);
    match family {
        family @ (ActionFamily::Buffer
        | ActionFamily::Viewport
        | ActionFamily::Session
        | ActionFamily::View
        | ActionFamily::Align
        | ActionFamily::Format
        | ActionFamily::Export) => dispatch_editor_action(ctx, action, family),
        family @ (ActionFamily::Overlay | ActionFamily::Deferred) => {
            dispatch_command_action(ctx, action, family)
        }
    }
}

pub fn apply_core(ctx: &mut ActionCtx, action: &Action, shift: bool) -> Effect {
    // Serializes this whole action against any other thread's global-touching
    // test, under test only (see [`crate::testlock`]): `about_open()` /
    // `lifetime_open()` are read unconditionally just below, for every action, so
    // a concurrently-running test that flips one (only `Action::About` /
    // `Action::LifetimeStats` ever do) could otherwise leak its state into a
    // totally unrelated test's action, changing its returned `Effect`. It is the
    // ONE reentrant guard, so a test that already holds it around its own drive
    // nests here for free, and there is no lock ORDER left to ABBA (the page
    // writers acquire the SAME guard, reentrantly). This is the `product` door:
    // an action may intentionally leave a theme preview active. If a TEST already
    // owns `serial`, this nests and the outer test still verifies/cleans its own
    // world on exit. Held for the whole function; zero cost outside `cfg(test)`.
    #[cfg(test)]
    let _test_guard = crate::testlock::product();

    // WRITING-STREAKS VIEW TOGGLE. While the streaks card is open, ←/→ FLIP it
    // between its two pages (per-day heatmap ⇄ cumulative running total —
    // `streaks::toggle_view`, a pure view flip over the same records) instead of
    // dismissing — the overlay's Right/Left lens precedent, applied to the one
    // summoned card with a second page. Consumed entirely (the caret never
    // moves, the card stays open); every OTHER key still falls through to the
    // modal dismiss just below, so the arrows are that door's ONE exception,
    // and — sitting here in the shared core — the flip is `--keys "Left"`-
    // drivable headlessly like everything else.
    // MODAL CARD DISMISSAL (About / Lifetime stats / Writing streaks). While a
    // summoned card is open it OWNS the very next key — ANY key closes it and is
    // otherwise consumed (no other effect; the streaks card's ←/→ page flip
    // above is the one carve-out), mirroring the "any key/click dismisses" spec
    // rather than the navigation overlay's narrower Esc/Enter contract (a card
    // has nothing to navigate). ONE owner of the check+close
    // (`card::dismiss_summoned_card`), shared verbatim with the live App's
    // mouse-press handler. Checked BEFORE the overlay intercept: the cards are
    // never open at once, nor with an overlay (each opens via
    // `Effect::RunAction` after the palette that summoned it has already
    // closed).
    // OVERLAY INTERCEPT. When the summoned navigation overlay is open it OWNS
    // every key (printable chars filter the query, Up/Down move the selection,
    // Right/Left descend/ascend the explorers, Enter accepts, Esc/C-g cancels);
    // routing it through the shared core is what makes the overlay `--keys`-
    // drivable. The modal dispatch lives in [`overlay_nav::overlay_intercept`].
    if let Some(effect) = intercept_action(ctx, action) {
        return effect;
    }

    // NOTE — there is deliberately NO search intercept here. While the isearch
    // panel is open, EVERY key is consumed BEFORE keymap resolution by the ONE
    // shared interception seam (`crate::search::keys::intercept`) — the live
    // window's search guard (`app/input/keys.rs`) and the headless replay's
    // guard (`main/run.rs::replay_keys_mode`) are the same code — so no key
    // path can reach `apply_core` with `ctx.search` still `Some`. The old
    // Action-level Tab/OpenReplace intercept that lived here (the partial
    // headless mirror from before the seam existed) was retired with it:
    // same behavior must be same code, not an aligned copy.

    if action.is_motion() {
        if shift {
            if ctx.buffer.anchor_char().is_none() {
                ctx.buffer.set_mark();
            }
            *ctx.shift_selecting = true;
        } else if *ctx.shift_selecting {
            ctx.buffer.clear_mark();
            *ctx.shift_selecting = false;
        }
    }

    // RECOIL PRIMITIVE — snapshot the pre-action state so we can detect a BLOCKED
    // action (one that couldn't proceed) AFTER the match and bump the caret away
    // from the wall. Cheap scalars: the cursor char index (a motion that hit a wall
    // leaves it unchanged), the content version (a no-op delete never bumps it), and
    // whether undo/redo had anything to do. See `recoil_for`.
    let snapshot = ActionSnapshot::capture(ctx);

    let effect = dispatch_action(ctx, action);

    finish_action(ctx, action, snapshot, effect)
}

#[cfg(test)]
mod tests;
