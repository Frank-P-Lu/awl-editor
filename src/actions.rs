//! Pure action application shared by the windowed app and headless replay.

use crate::buffer::Buffer;
use crate::keymap::Action;
use crate::overlay::{OverlayKind, OverlayState};
use crate::search::{Direction, SearchState};
// Shared dispatch types stay here; cohesive action families live in submodules.
mod deferred; // filesystem/UI work returned to the live App
mod edit; // the markdown smart-Enter edit (smart_newline + its pure decision)
mod effects; // the closed typed-effect vocabulary + transition decoration
mod flinch; // the caret-feedback triggers (impact_for / recoil_for)
mod format; // the markdown formatting-command toggles (block + inline)
mod link; // LINKS V2 — Cmd-K insert/edit-link (plan + commit, mirrors format.rs)
mod motion; // the oracle-aware caret motions + page scroll + search open
mod overlay_nav; // the modal overlay intercept + browse-path helpers + live preview
pub(crate) mod popover; // the format-popover pure plan (reads format.rs's active-state)
mod rebind; // the game-style rebind-menu key handling
mod workspace_nav; // ITEM 114 — the workspace's two-region keys + the Cmd-P deep link
use deferred::*;
use edit::*;
pub use effects::*;
use flinch::*;
use format::*;
use link::*;
use motion::*;
use overlay_nav::*;
pub(crate) use overlay_nav::{preview_move, preview_overlay};
use rebind::*;

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
        Action::ConvertLineEndings => {
            ctx.buffer.set_eol(ctx.buffer.eol().toggled());
            Effect::None
        }
        Action::ReportProblem => Effect::ReportProblem,
        Action::DownloadFile => Effect::DownloadFile,
        Action::CheckForUpdates => Effect::CheckForUpdates,
        _ => return None,
    };
    Some(effect)
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
        Action::Newline | Action::AcceptAlternate => {
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
        Action::CopyLinkDestination => {
            let byte = ctx.buffer.char_to_byte(ctx.buffer.cursor_char());
            if let Some(url) = crate::markdown::link_at(&ctx.buffer.text(), byte) {
                ctx.buffer.set_kill(&url);
            }
        }
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
    let up = ctx.journey.card().is_some();
    up.then(|| overlay_intercept(ctx, action))
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
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Goto));
        }
        Action::OpenProject => {
            ctx.journey
                .enter((ctx.browse_to)(OverlayKind::Project, None));
        }
        Action::OpenRecentProjects => {
            let mut ov = (ctx.browse_to)(OverlayKind::Project, None);
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
        // (⇧↵) then RESTORES the highlighted version as an undoable edit — item 116c.
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
        // ITEM 116c — see `workspace_nav::open_keep_version`'s own doc: parks
        // whatever is already open rather than `enter`-ing over it.
        Action::KeepVersion => workspace_nav::open_keep_version(ctx),
        // DIFF-AS-PREVIEW ("Compare with version…" from the BUFFER): the palette
        // command REPOINTS to opening the HISTORY picker — whose live preview IS
        // the writer's diff now (arrowing the versions shows each one's marked-up
        // manuscript in the page below the card). ONE behavior, no orphaned second
        // mode: the old read-only takeover view is retired. From an OPEN History
        // picker this action is intercepted earlier (`overlay_nav`'s Tab arm — the
        // focus shift into the diff panel) and never reaches here.
        Action::CompareVersion => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::History));
        }
        Action::OpenBrowse => {
            ctx.journey
                .enter((ctx.browse_to)(OverlayKind::Browse, None));
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
            | Action::AcceptAlternate
            | Action::InsertTab
            | Action::Outdent
            | Action::DeleteBackward
            | Action::DeleteWordBackward
            | Action::DeleteWordForward
            | Action::DeleteToLineStart
            | Action::DeleteForward
            | Action::KillLine
            | Action::YankText
            | Action::InsertImageReference(_)
            | Action::Undo
            | Action::Redo
            | Action::SetMark
            | Action::CopyRegion
            | Action::CopyLinkDestination
            | Action::KillRegion
            | Action::SelectAll => ActionFamily::Buffer,
            Action::ZoomIn
            | Action::ZoomOut
            | Action::ZoomReset
            | Action::PageScrollDown
            | Action::PageScrollUp => ActionFamily::Viewport,
            Action::Yank
            | Action::Save
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
            | Action::KeepTutorial
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
        ActionFamily::Buffer => {
            // The call must NOT sit inside `debug_assert!` — that macro compiles
            // out in release, taking the dispatch with it and leaving the whole
            // family inert in a shipped build. Run it, then assert the result.
            let handled = apply_buffer_action(ctx, action);
            debug_assert!(handled, "buffer family did not handle {action:?}");
        }
        ActionFamily::Viewport => {
            let handled = apply_viewport_action(ctx, action);
            debug_assert!(handled, "viewport family did not handle {action:?}");
        }
        ActionFamily::Session => {
            effect = apply_session_action(ctx, action).expect("session action")
        }
        ActionFamily::View => effect = apply_view_action(ctx, action).expect("view action"),
        ActionFamily::Align => align_table_at_cursor(ctx),
        ActionFamily::Format => {
            let handled = apply_format_action(ctx, action);
            debug_assert!(handled, "format family did not handle {action:?}");
        }
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
        ActionFamily::Overlay => {
            let handled = apply_overlay_open_action(ctx, action);
            debug_assert!(handled, "overlay family did not handle {action:?}");
        }
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
