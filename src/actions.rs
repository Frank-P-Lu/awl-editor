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

    if !crate::commands::action_available(action, crate::commands::Platform::current()) {
        return Effect::None;
    }

    // WRITING-STREAKS VIEW TOGGLE. While the streaks card is open, ←/→ FLIP it
    // between its two pages (per-day heatmap ⇄ cumulative running total —
    // `streaks::toggle_view`, a pure view flip over the same records) instead of
    // dismissing — the overlay's Right/Left lens precedent, applied to the one
    // summoned card with a second page. Consumed entirely (the caret never
    // moves, the card stays open); every OTHER key still falls through to the
    // modal dismiss just below, so the arrows are that door's ONE exception,
    // and — sitting here in the shared core — the flip is `--keys "Left"`-
    // drivable headlessly like everything else.
    if crate::streaks::streaks_open()
        && matches!(action, Action::ForwardChar | Action::BackwardChar)
    {
        crate::streaks::toggle_view();
        return Effect::None;
    }

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
    if crate::card::dismiss_summoned_card() {
        return Effect::None;
    }

    // OVERLAY INTERCEPT. When the summoned navigation overlay is open it OWNS
    // every key (printable chars filter the query, Up/Down move the selection,
    // Right/Left descend/ascend the explorers, Enter accepts, Esc/C-g cancels);
    // routing it through the shared core is what makes the overlay `--keys`-
    // drivable. The modal dispatch lives in [`overlay_nav::overlay_intercept`].
    if ctx.overlay.is_some() {
        return overlay_intercept(ctx, action);
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
    let cursor_before = ctx.buffer.cursor_char();
    let version_before = ctx.buffer.version();
    let could_undo = ctx.buffer.can_undo();
    let could_redo = ctx.buffer.can_redo();
    let had_selection_before = ctx.buffer.has_selection();

    let mut effect = Effect::None;
    match action {
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
        | Action::SelectAll => {
            debug_assert!(apply_buffer_action(ctx, action));
        }
        // ITEM 94: ⌘= / ⌘- move exactly ONE AUTHORED STEP of the zoom range spec —
        // the same `stepped` owner Right/Left on the Settings rail row use, so the
        // two keyboards can never disagree about what one zoom increment is.
        Action::ZoomIn => *ctx.zoom = crate::range::ZOOM.stepped(*ctx.zoom, 1),
        Action::ZoomOut => *ctx.zoom = crate::range::ZOOM.stepped(*ctx.zoom, -1),
        Action::ZoomReset => *ctx.zoom = crate::range::ZOOM.default,
        Action::PageScrollDown => scroll_page(ctx.buffer, ctx.scroll_page_lines, true),
        Action::PageScrollUp => scroll_page(ctx.buffer, ctx.scroll_page_lines, false),
        Action::Save => {
            if ctx.buffer.path().is_none() && !ctx.buffer.is_unnamed_fresh() {
                // A TRUE scratch buffer (no path, never named as a note):
                // convert it into a real document — the caller has the
                // active folder, the core doesn't. See `Effect::ConvertScratchAndSave`.
                effect = Effect::ConvertScratchAndSave;
            } else {
                effect = match ctx.buffer.save() {
                    Ok(()) => Effect::SaveDone {
                        ok: true,
                        message: "saved".to_string(),
                    },
                    Err(e) => Effect::SaveDone {
                        ok: false,
                        message: format!("save failed: {e}"),
                    },
                };
            }
        }
        Action::Quit => effect = Effect::Quit,
        // C-g / Escape / Cmd-. : cancel clears any active selection. A live
        // search can never still be open here — the shared search-key seam
        // (`crate::search::keys::intercept`) consumes Escape/C-g on BOTH the
        // live and replay paths and closes the panel itself (restoring the
        // origin cursor + remembering the query), so this arm no longer
        // carries a search-close copy of that rule.
        Action::Cancel => {
            ctx.buffer.clear_mark();
            *ctx.shift_selecting = false;
        }
        // C-s / C-r: open an incremental search anchored at the cursor. While a
        // search is already live neither driver reaches this arm — the shared
        // search guard consumes C-s/C-r as STEP next/previous first — so this
        // only ever models the OPEN.
        Action::SearchForward => start_search(ctx, Direction::Forward),
        Action::SearchBackward => start_search(ctx, Direction::Backward),
        Action::OpenReplace => {
            start_search(ctx, Direction::Forward);
            if let Some(st) = ctx.search.as_mut() {
                st.reveal_replace();
            }
        }
        action @ (Action::ToggleCaretMode
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
        | Action::CheckForUpdates) => effect = apply_view_action(ctx, action).expect("view action"),
        Action::AlignTable => align_table_at_cursor(ctx),
        action @ (Action::ToggleBlockquote
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
        | Action::Strikethrough) => {
            debug_assert!(apply_format_action(ctx, action));
        }
        Action::ExportWord => {
            if ctx.buffer.is_markdown() {
                effect = Effect::Export(crate::export::Format::Docx);
            }
        }
        Action::ExportHtml => {
            if ctx.buffer.is_markdown() {
                effect = Effect::Export(crate::export::Format::Html);
            }
        }
        Action::ExportPdf =>
        {
            #[cfg(not(target_arch = "wasm32"))]
            if ctx.buffer.is_markdown() {
                effect = Effect::Export(crate::export::Format::Pdf);
            }
        }
        Action::InsertLink => open_insert_link(ctx),
        Action::InsertDate => {
            effect = Effect::InsertDate;
        }
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
        Action::LastBuffer => {
            effect = Effect::LastBuffer;
        }
        Action::NewDocument => {
            effect = Effect::NewDocument;
        }
        Action::MoveFile => {
            *ctx.overlay = (ctx.browse_to)(crate::overlay::OverlayKind::MoveDest, None);
        }
        Action::OpenRenameNote => {
            if let Some(path) = ctx.buffer.path() {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                *ctx.overlay = Some(OverlayState::new_rename(name));
            }
        }
        Action::DuplicateNote => {
            effect = Effect::DuplicateNote;
        }
        Action::OpenSettings => {
            effect = Effect::OpenSettings;
        }
        Action::OpenCredits => {
            effect = Effect::OpenCredits;
        }
        Action::OpenGuide => {
            effect = Effect::OpenGuide;
        }
        Action::OpenSettingsMenu => {
            *ctx.overlay = (ctx.make_overlay)(crate::overlay::OverlayKind::Settings);
        }
        // C-x #: SAVE the buffer (the SAME `Buffer::save` call `Action::Save` makes)
        // then signal the caller to notify daemon waiters + switch to the
        // previously-open buffer. The caller (`App::finish_buffer`) mirrors
        // `Action::Save`'s history-snapshot + mtime bookkeeping itself, BEFORE the
        // buffer swap — `post_apply_effects` runs after this effect and would
        // otherwise stamp the wrong (just-switched-to) buffer. The core can't reach
        // the daemon socket or the 2-deep buffer history itself.
        Action::FinishBuffer => {
            // SAVE-FEEDBACK round: no terminal echo (matches `Action::Save`'s
            // own fix) — a failure here is a narrower gap than plain Save's
            // (C-x # only ever targets an already-pathed, daemon-served
            // buffer, never the scratch surface), logged rather than fully
            // routed to a notice: `finish_buffer` immediately switches away
            // to the previous buffer right after, so a notice would flash
            // and vanish before it could be read. Banked as a fast-follow if
            // that ever proves confusing in practice.
            let _ = ctx.buffer.save();
            effect = Effect::FinishBuffer;
        }
        // C-c C-o: FOLLOW the markdown link under the caret. Extract its URL from
        // the parsed spans ([`crate::markdown::link_at`], a pure function of the
        // text + caret BYTE offset); a link → signal the URL back for the caller to
        // open in the browser, a caret outside every link → a calm no-op
        // (`Effect::None`). The core never opens anything itself (no window/process
        // reach) — the live App performs the OS handoff, the headless replay no-ops.
        Action::FollowLink => {
            if let Some(url) =
                crate::markdown::link_at(&ctx.buffer.text(), ctx.buffer.cursor_byte())
            {
                effect = Effect::FollowLink(url);
            }
        }
        Action::BeginPrefix | Action::Ignore => {}
    }

    // Seal the undo group after any NON-edit command so the next edit starts a
    // fresh group. Undo/Redo manage history themselves and must not seal.
    if !action.is_edit() && !matches!(action, Action::Undo | Action::Redo) {
        ctx.buffer.seal_undo_group();
    }
    if !ctx.buffer.has_selection() {
        *ctx.shift_selecting = false;
    }

    // AUTO-EXPAND (folds): any edit / motion that lands the caret INSIDE a collapsed
    // section reveals it, and a selection never spans a fold invisibly. This is the
    // action-motion ingress into the ONE revealed-placement owner
    // (`Buffer::reveal_placement`); the mouse click/drag, search step, and heading/
    // line jump ingresses call the same owner at their own seams (the search guard +
    // overlay accept return before reaching here). A cheap no-op when nothing is
    // folded (the common case), so this runs after every action without measurable
    // cost. The two fold GESTURES above leave the caret on the still-visible heading
    // line, so they are unaffected.
    ctx.buffer.reveal_placement();

    // RECOIL PRIMITIVE — if the action produced no other effect, see whether it was
    // BLOCKED (couldn't proceed) and, if so, arm a caret bump away from the wall.
    // Mutually exclusive with the real effects (a blocked action never sets one), so
    // we only test when `effect` is still `None`.
    if effect == Effect::None
        && let Some(dir) = recoil_for(
            action,
            ctx,
            cursor_before,
            version_before,
            could_undo,
            could_redo,
        )
    {
        effect = Effect::Recoil(dir);
    }
    if effect == Effect::None
        && let Some(imp) = impact_for(action, version_before, ctx)
    {
        effect = imp;
    }
    // COPY PULSE — a successful M-w/Cmd-C copy of a NON-EMPTY selection: arm the
    // caret kick + selection-tint brighten/decay. Never touches buffer content, so
    // it can't ride `impact_for`'s version-changed gate above; a separate check
    // against the PRE-action selection snapshot (`copy_region` always clears the
    // mark, even on a no-op). Mutually exclusive with the other effects by
    // construction (`Action::CopyRegion` never recoils or flinches), so gating on
    // `effect == Effect::None` here is a formality that keeps the same shape as
    // the recoil/impact cascade above.
    if effect == Effect::None
        && let Some(e) = copy_pulse_for(action, had_selection_before)
    {
        effect = e;
    }
    effect
}

#[cfg(test)]
mod tests;
