//! Pure action application shared by the windowed app and headless replay.

use crate::buffer::Buffer;
use crate::keymap::Action;
use crate::overlay::{OverlayKind, OverlayState};
use crate::search::{Direction, SearchState};
// Shared dispatch types stay here; cohesive action families live in submodules.
mod deferred; // filesystem/UI work returned to the live App
mod dispatch; // exhaustive Action-family classification + routing
mod edit; // the markdown smart-Enter edit (smart_newline + its pure decision)
mod effects; // the closed typed-effect vocabulary + transition decoration
mod flinch; // the caret-feedback triggers (impact_for / recoil_for)
mod format; // the markdown formatting-command toggles (block + inline)
pub(crate) mod link; // LINKS V2 — Cmd-K insert/edit-link (plan + commit, mirrors format.rs)
mod motion; // the oracle-aware caret motions + page scroll + search open
mod overlay_nav; // the modal overlay intercept + browse-path helpers + live preview
pub(crate) mod popover; // the format-popover pure plan (reads format.rs's active-state)
mod rebind; // the game-style rebind-menu key handling
pub(crate) mod table; // Insert-table -- open the dimension picker + build its FormatResult
mod workspace_nav; // the workspace's two-region keys + the Cmd-P deep link
use deferred::*;
use dispatch::dispatch_action;
use edit::*;
pub use effects::*;
use flinch::*;
use format::*;
use link::*;
use motion::*;
use overlay_nav::*;
pub(crate) use overlay_nav::{preview_move, preview_overlay};
use rebind::*;
use table::*;

/// Renderer-owned visual-row geometry for shared motion.
pub trait LayoutOracle {
    fn visual_row_of(&self, line: usize, col: usize) -> usize;
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
    /// Measured page rows live; a fixed deterministic page headlessly.
    pub scroll_page_lines: usize,
    /// THE SUMMONED-UI JOURNEY (`overlay::Journey`): which surface is up, what
    /// is parked beneath it, and the one table saying where every Esc/Back/
    /// accept lands. While a card is up, typed chars edit its query (NOT the
    /// buffer), Up/Down move the selection, Enter accepts, Esc/C-g cancels.
    /// Keeping the lifecycle in the shared core keeps it `--keys`-drivable.
    pub journey: &'a mut crate::overlay::Journey,
    /// The active project context the overlay needs when it OPENS: a builder that
    /// produces a fresh `OverlayState` for a given kind. The core can't read the
    /// filesystem itself (and headless replay must stay deterministic), so the
    /// caller injects this; `OpenGoto`/`OpenProject` invoke it.
    pub make_overlay: &'a mut dyn FnMut(OverlayKind) -> Option<OverlayState>,
    pub browse_to: &'a mut dyn FnMut(OverlayKind, Option<String>) -> Option<OverlayState>,
    /// The visual-line motion LAYOUT ORACLE (the SHAPED text's wrap geometry),
    /// supplied by the live GPU pipeline (`app.rs`) and the headless offscreen
    /// pipeline (`capture.rs`) so the two flows can't drift. `None` in the pure
    /// `apply_transition` unit tests (no pipeline), where motion falls back to LOGICAL
    /// lines. Consulted by the vertical (C-n/C-p, Up/Down), line-edge (C-a/C-e,
    /// Home/End) and kill-line (C-k) motions, which follow the SHAPED visual rows
    /// whenever it is present (the flat default).
    pub oracle: Option<&'a dyn LayoutOracle>,
}

/// Apply one resolved `action` to the editor core. `shift` is whether Shift was
/// held (so a motion extends the selection, Shift+Arrow style). Returns the one
/// deferred [`Effect`] the action signals back to the caller (`Effect::None` for
/// the common case) — the caller carries out the filesystem/window/quit work the
/// pure core can't. Mutates only what `ActionCtx` exposes; no GPU, window, or
/// clipboard.
fn apply_view_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let effect = match action {
        Action::ToggleCaretMode => {
            crate::caret::toggle_mode();
            Effect::Persistence(PersistenceEffect::Preference(PreferenceEffect::CaretMode))
        }
        Action::TogglePageMode => {
            crate::page::toggle();
            Effect::Persistence(PersistenceEffect::Preference(PreferenceEffect::PageMode))
        }
        Action::PageWider => {
            crate::page::widen();
            Effect::Persistence(PersistenceEffect::Preference(PreferenceEffect::PageWidth))
        }
        Action::PageNarrower => {
            crate::page::narrow();
            Effect::Persistence(PersistenceEffect::Preference(PreferenceEffect::PageWidth))
        }
        Action::PageReset => {
            crate::page::set_measure(ctx.buffer.page_class().default_measure());
            Effect::Persistence(PersistenceEffect::Preference(PreferenceEffect::PageReset))
        }
        Action::ToggleDebug => {
            crate::debug::toggle();
            Effect::None
        }
        Action::ToggleOutline => {
            crate::outline::toggle();
            Effect::Persistence(PersistenceEffect::Preference(PreferenceEffect::Outline))
        }
        Action::ToggleFold => {
            ctx.buffer.toggle_fold_at_cursor();
            Effect::None
        }
        Action::CollapseOtherSections => {
            ctx.buffer.collapse_other_sections();
            Effect::None
        }
        Action::ToggleMenuBar => {
            crate::menubar::toggle();
            Effect::Persistence(PersistenceEffect::Preference(PreferenceEffect::MenuBar))
        }
        Action::ToggleTypewriter => {
            crate::typewriter::toggle();
            Effect::Persistence(PersistenceEffect::Preference(PreferenceEffect::Typewriter))
        }
        Action::ShowStatsHud => {
            crate::hud::set_held(true);
            Effect::None
        }
        Action::About => Effect::Surface(SurfaceEffect::ShowAbout),
        Action::OpenBrowse => Effect::Surface(SurfaceEffect::OpenFileChooser),
        Action::OpenFolder => Effect::Surface(SurfaceEffect::OpenFolderChooser),
        Action::LifetimeStats => {
            crate::lifetime::set_open(true);
            Effect::None
        }
        Action::WritingStreaks => {
            crate::streaks::set_open(true);
            Effect::Persistence(PersistenceEffect::Preference(
                PreferenceEffect::WritingStreaks,
            ))
        }
        // LINE ENDINGS: the one verb in the catalog whose entire result was
        // invisible. It changes what the NEXT save writes to disk, it is
        // deliberately off the undo timeline (EOL is document metadata, not an
        // edit — see `docs/platform.md`), and it is a TOGGLE, so without a notice
        // a double invocation was indistinguishable from none. The notice names
        // the convention now in effect rather than reporting that a toggle
        // happened, because "which one am I on" is the question the user has.
        Action::ConvertLineEndings => {
            let now = ctx.buffer.eol().toggled();
            ctx.buffer.set_eol(now);
            Effect::Notice(NoticeEffect::Toast(format!(
                "line endings: {}",
                now.label()
            )))
        }
        Action::ReportProblem => Effect::ReportProblem,
        Action::DownloadFile => Effect::DownloadFile,
        Action::CheckForUpdates => Effect::CheckForUpdates,
        _ => return None,
    };
    Some(effect)
}

fn apply_format_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
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
        Action::InsertFootnote => format::apply_insert_footnote(ctx),
        Action::TagDocumentLanguage => return Some(tag_document_language(ctx)),
        _ => return None,
    };
    Some(Effect::None)
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
        Action::ForwardSentence => ctx.buffer.forward_sentence(),
        Action::BackwardSentence => ctx.buffer.backward_sentence(),
        Action::BufferStart => ctx.buffer.buffer_start(),
        Action::BufferEnd => ctx.buffer.buffer_end(),
        Action::InsertChar(c) => ctx.buffer.insert_char(*c),
        Action::Newline => {
            if !table_newline(ctx) && !smart_newline(ctx) {
                ctx.buffer.insert_newline();
            }
        }
        // Shift-Enter is the deliberate literal line-split escape hatch in a
        // table; it keeps the ordinary smart-Enter semantics everywhere else.
        Action::AcceptAlternate => {
            if !smart_newline(ctx) {
                ctx.buffer.insert_newline();
            }
        }
        Action::InsertTab => {
            if !table_tab(ctx, true) {
                list_tab(ctx)
            }
        }
        Action::Outdent => {
            if !table_tab(ctx, false) {
                list_outdent(ctx)
            }
        }
        Action::MoveLineUp => ctx.buffer.move_line_up(),
        Action::MoveLineDown => ctx.buffer.move_line_down(),
        Action::DeleteBackward => ctx.buffer.delete_backward(),
        Action::DeleteWordBackward => ctx.buffer.delete_word_backward(),
        Action::DeleteWordForward => ctx.buffer.delete_word_forward(),
        Action::DeleteSentenceForward => ctx.buffer.delete_sentence_forward(),
        Action::DeleteSentenceBackward => ctx.buffer.delete_sentence_backward(),
        Action::DeleteToLineStart => ctx.buffer.delete_to_line_start(),
        Action::DeleteForward => ctx.buffer.delete_forward(),
        Action::KillLine => kill_line_motion(ctx),
        Action::YankText => ctx.buffer.yank(),
        Action::InsertImageReference(reference) => {
            let (_, col) = ctx.buffer.cursor_line_col();
            let text = image_reference_text(col == 0, reference);
            let at = ctx.buffer.cursor_char();
            ctx.buffer.replace_char_range(at, at, &text);
        }
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
        Action::CopyLinkDestination => crate::context_menu::copy_link_destination(ctx.buffer),
        Action::CopyFilePath => crate::context_menu::copy_file_path(ctx.buffer),
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
        Action::PageScrollDown => scroll_page(ctx, true),
        Action::PageScrollUp => scroll_page(ctx, false),
        _ => return false,
    }
    true
}

fn apply_session_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let effect = match action {
        Action::Yank => Effect::Clipboard(ClipboardEffect::PasteImage),
        Action::Save => Effect::Persistence(PersistenceEffect::Save(SaveKind::Manual)),
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

struct ActionSnapshot {
    cursor_before: usize,
    version_before: u64,
    could_undo: bool,
    could_redo: bool,
    had_selection_before: bool,
    /// The caret's LOGICAL LINE before this action dispatches — cheap (a rope
    /// line lookup, not a document scan), unlike the table-block detection
    /// itself, which `finish_action` only pays for once this differs from the
    /// post-dispatch line (see its own doc for why that gate matters).
    row_before: usize,
}

impl ActionSnapshot {
    fn capture(ctx: &ActionCtx) -> Self {
        Self {
            cursor_before: ctx.buffer.cursor_char(),
            version_before: ctx.buffer.version(),
            could_undo: ctx.buffer.can_undo(),
            could_redo: ctx.buffer.can_redo(),
            had_selection_before: ctx.buffer.has_selection(),
            row_before: ctx.buffer.cursor_line_col().0,
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
    // AUTO-ALIGN ON ROW-LEAVE: the caret's logical line just changed from
    // `row_before` — cheap to notice on every action, unlike the table-block
    // detection `auto_align_table_on_row_leave` itself does, which this gate
    // is what keeps off the O(document) common case of "still on the same
    // line" (the overwhelming majority of keystrokes). Skipped for Undo/Redo
    // (an auto-edit firing right after either would clear the redo stack and
    // silently rewrite what the user just time-traveled to), for `AlignTable`
    // itself (already this exact re-pad, manually, and already idempotent
    // otherwise) and the six structural row/column verbs (each already
    // re-emits its whole block through that same padder, as one sealed undo
    // group), and for the table's OWN structural row actions — Enter/
    // Shift-Enter and Tab/Shift-Tab (`table_newline`/`table_tab`) — which
    // already emit a correctly-columned scaffold row through their own
    // dedicated logic and carry their own atomic-undo contract (e.g. Tab
    // appending a fresh row is ONE undo step); re-aligning immediately after
    // would edit their freshly-scaffolded row a second time as a SEPARATE
    // undo step, splitting what the user experiences as one keystroke into
    // two undos. A row genuinely left via caret motion (arrows, Goto, mouse)
    // is unaffected: none of those are edit actions, so none are excluded
    // here. See `edit::auto_align_table_on_row_leave`'s own doc for the
    // trigger's full design and the undo/caret guarantees.
    if snapshot.row_before != ctx.buffer.cursor_line_col().0
        && !matches!(
            action,
            Action::Undo
                | Action::Redo
                | Action::AlignTable
                | Action::TableInsertRowAbove
                | Action::TableInsertRowBelow
                | Action::TableInsertColumnLeft
                | Action::TableInsertColumnRight
                | Action::TableDeleteRow
                | Action::TableDeleteColumn
                | Action::Newline
                | Action::AcceptAlternate
                | Action::InsertTab
                | Action::Outdent
        )
    {
        auto_align_table_on_row_leave(ctx, snapshot.row_before);
    }
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
    let up = ctx.journey.card().is_some();
    up.then(|| overlay_intercept(ctx, action))
}

/// The File-menu Export rows stay enabled on every buffer (unlike hiding a row
/// with genuinely nothing to say — see `commands::row_hidden`'s doc — there IS
/// something to say: which document kind the command needs), so a non-Markdown
/// buffer earns this explicit sticky notice instead of the silent no-op an
/// enabled-but-inert row would be. Never a promise of a path that doesn't
/// exist (the "reopen for theirs" defect `app/files/external.rs` retired) —
/// only a fact about the current buffer.
const EXPORT_REQUIRES_MARKDOWN: &str = "can't export a non-Markdown file";

fn export_requires_markdown_notice() -> Effect {
    Effect::Notice(NoticeEffect::Sticky(EXPORT_REQUIRES_MARKDOWN.to_string()))
}

/// DOES AN EXPORT ASK WHERE BEFORE IT WRITES? A native build writes a real file
/// into a folder the writer chooses, so the destination navigator
/// ([`crate::overlay::OverlayKind::ExportDest`]) opens first and the File-menu
/// label carries the ellipsis that promises it. The web build hands the bytes to
/// the browser's own download, which owns where they land — there is nothing to
/// choose, so no surface opens.
///
/// PLATFORM-PARAMETERISED rather than `cfg!`-shaped, on
/// `commands::Platform::current`'s own pattern: both answers are then assertable
/// from one native test run, so the arm this host does not compile is still
/// swept. ⚠️ The label is ONE static string for both platforms
/// (`menu::FILE_ITEMS`), so the web build's ellipsis over-promises — see
/// `menu::ellipsis_law` for the decision and what it costs.
pub fn export_picks_destination(platform: crate::commands::Platform) -> bool {
    match platform {
        crate::commands::Platform::Native => true,
        crate::commands::Platform::Web => false,
    }
}

/// Begin one export: on a platform that picks a destination, summon the
/// folders-only navigator with `format` riding the card; otherwise write
/// immediately at the destination the pure owner derives.
///
/// A navigator the level supplier declines to build leaves the editor and
/// exports nothing, exactly as `Action::MoveFile` does — the two destination
/// verbs share one failure shape.
fn begin_export(ctx: &mut ActionCtx, format: crate::export::Format) -> Effect {
    if !export_picks_destination(crate::commands::Platform::current()) {
        return Effect::Export(format, None);
    }
    let card = (ctx.browse_to)(crate::overlay::OverlayKind::ExportDest, None).map(|mut card| {
        card.export_format = Some(format);
        card
    });
    ctx.journey.enter(card);
    Effect::None
}

fn apply_export_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let effect = match action {
        Action::ExportWord if ctx.buffer.is_markdown() => {
            begin_export(ctx, crate::export::Format::Docx)
        }
        Action::ExportHtml if ctx.buffer.is_markdown() => {
            begin_export(ctx, crate::export::Format::Html)
        }
        Action::ExportWord | Action::ExportHtml => export_requires_markdown_notice(),
        Action::ExportPdf => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if ctx.buffer.is_markdown() {
                    begin_export(ctx, crate::export::Format::Pdf)
                } else {
                    export_requires_markdown_notice()
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
        Action::InsertTable => {
            open_insert_table(ctx);
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
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Goto));
        }
        // THE TWO DOORS ONTO THE FLAT SWITCH-PROJECT PICKER. Both attach its own
        // door row (`OverlayState::attach_browse_door` — the reach past the
        // direct workspace children the flat roster deliberately stops at); the
        // Settings folder-VALUE picker, which shares this kind's card shape and
        // already walks the whole tree, deliberately does not.
        Action::OpenProject => {
            let mut ov = (ctx.make_overlay)(OverlayKind::Goto);
            if let Some(o) = ov.as_mut() {
                o.focus_facet_id("folders");
            }
            ctx.journey.enter(ov);
        }
        Action::OpenRecentProjects => {
            let mut ov = (ctx.make_overlay)(OverlayKind::Goto);
            if let Some(o) = ov.as_mut() {
                o.focus_facet_id("recent");
            }
            ctx.journey.enter(ov);
        }
        Action::OpenThemeMenu => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Theme));
        }
        Action::OpenCaretMenu => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Caret));
        }
        Action::OpenDictionaryMenu => {
            ctx.journey
                .enter((ctx.make_overlay)(OverlayKind::Dictionary));
        }
        Action::OpenKeymapMenu => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Keymap));
        }
        // Toggling spellcheck is a pure render/detection concern (no buffer change).
        // The process-global flip lives HERE on the shared seam (like the page/caret
        // toggles); `App::apply` persists the sticky pref + forces an immediate
        // rescan as a post-`apply_transition` side effect the core can't reach. A
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
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Command));
        }
        Action::OpenKeybindings => {
            ctx.journey
                .enter((ctx.make_overlay)(OverlayKind::Keybindings));
        }
        // "Go to heading…" (palette): open GO-TO pre-lensed onto its HEADINGS lens —
        // the fold that retired the standalone Outline picker. `make_overlay` builds
        // the Go-to overlay with the doc's headings already folded in (its Headings
        // lens's corpus); focusing the `headings` lens opens it showing them. Over a
        // buffer with no headings the lens reads "no headings yet" (never a no-op —
        // the file list is still there behind the other lenses; also reachable via
        // ⌘O → ←/→).
        Action::OpenOutline => {
            let mut ov = (ctx.make_overlay)(OverlayKind::Goto);
            if let Some(o) = ov.as_mut() {
                o.focus_facet_id("headings");
            }
            ctx.journey.enter(ov);
        }
        Action::OpenSpellSuggest => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Spell));
        }
        // Cmd-Shift-H: summon the HISTORY TIMELINE picker for the current file. The
        // caller's `make_overlay` gathers the file's versions (via
        // `history::timeline_rows`); an empty history still opens (the calm "no
        // history yet" row), so this is never a silent no-op. `AcceptAlternate`
        // (⇧↵) then RESTORES the highlighted version as an undoable edit.
        Action::OpenHistory => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::History));
        }
        // Cmd-P → "Clean unused assets…": summon the ASSET CLEANER. The caller's
        // `make_overlay` builds it from the scanned orphan list (`assets::scan`,
        // threaded via `BuildCtx::assets`); an empty list still opens (the calm "no
        // unused assets" row), so this is never a silent no-op. Enter then requests the
        // highlighted orphan be trashed (`Effect::TrashAsset`), keeping the picker open.
        Action::OpenAssetClean => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Assets));
        }
        // Cmd-P → "Personal dictionary…": summon the picker over the words the
        // user has added to spell-check. The caller's `make_overlay` builds it
        // from the gathered word list (`BuildCtx::user_words`); an empty list
        // still opens (the calm row naming where words come from), so this is
        // never a silent no-op. Enter then requests the highlighted word be
        // forgotten (`Effect::ForgetUserWord`), keeping the picker open.
        Action::OpenUserWords => {
            ctx.journey
                .enter((ctx.make_overlay)(OverlayKind::UserWords));
        }
        // Cmd-P → "Search in folder…": summon the FULL-TEXT SEARCH picker over
        // the active folder. The caller's `make_overlay` builds it from the
        // already-loaded corpus (`BuildCtx::search_corpus`); an empty query is
        // the summon state (the calm "no matches" row), so this is never a
        // silent no-op. Enter then opens the highlighted match's file at its
        // line/col through `Effect::OpenPathAtLine`
        // (`actions::overlay_nav::accept_value_overlay`).
        Action::OpenSearchFolder => {
            ctx.journey
                .enter((ctx.make_overlay)(OverlayKind::SearchFolder));
        }
        // Cmd-P → "Credits": summon the read-only CREDITS VIEWER — a summoned
        // workspace, never a buffer swap, so the active document's path and
        // version are untouched by opening, scrolling or dismissing it
        // (`OverlayKind::Credits`'s own module doc). `make_overlay` needs no
        // caller-gathered context (the corpus is a compiled-in constant), so
        // this always summons, exactly like History/Assets above. Then the
        // SAME deep link `CompareVersion` uses below: transfer focus to the
        // content stage through the lifecycle, never by constructing the
        // card with `detail_focus` already set — there is no row to choose,
        // so the first keypress must scroll rather than step the one-row
        // rail.
        Action::OpenCredits => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Credits));
            if ctx
                .journey
                .card()
                .is_some_and(|o| o.comparison_request().is_some())
            {
                ctx.journey.toggle_detail();
            }
        }
        // See `workspace_nav::open_keep_version`'s own doc: this parks
        // whatever is already open rather than `enter`-ing over it.
        Action::KeepVersion => workspace_nav::open_keep_version(ctx),
        // "Compare with version…" from the BUFFER: the SAME workspace
        // `OpenHistory` enters — one surface, no orphaned second mode — at a
        // DIFFERENT FOCUS, which is the whole of what makes it a distinct deep
        // link. "Version history…" asks WHICH version and lands on the timeline;
        // this asks WHAT CHANGED and lands in the comparison. The transfer goes
        // through the LIFECYCLE, never by writing `detail_focus`, and declines
        // with nothing to compare, so an empty history degrades to the timeline
        // rather than opening a blank region. From an OPEN History workspace the
        // workspace intercept owns this action as the same toggle.
        Action::CompareVersion => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::History));
            if ctx
                .journey
                .card()
                .is_some_and(|o| o.comparison_request().is_some())
            {
                ctx.journey.toggle_detail();
            }
        }
        _ => return false,
    }
    true
}

fn apply_transition_primary(ctx: &mut ActionCtx, action: &Action, shift: bool) -> Effect {
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
    // path can reach `apply_transition` with `ctx.search` still `Some`. The old
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

/// Apply the one shared editor transition and return every typed request.
pub fn apply_transition(ctx: &mut ActionCtx, action: &Action, shift: bool) -> Transition {
    let primary = apply_transition_primary(ctx, action, shift);
    effects::complete(primary, action)
}

#[cfg(test)]
mod tests;
