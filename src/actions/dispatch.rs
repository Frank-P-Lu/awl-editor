//! Exhaustive action-family classification and routing.

use super::*;

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
            | Action::ForwardSentence
            | Action::BackwardSentence
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
            | Action::DeleteSentenceForward
            | Action::DeleteSentenceBackward
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
            | Action::CopyFilePath
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
            | Action::CheckForUpdates
            | Action::OpenBrowse
            | Action::OpenFolder => ActionFamily::View,
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
            | Action::Strikethrough
            | Action::InsertFootnote
            | Action::TagDocumentLanguage => ActionFamily::Format,
            Action::ExportWord
            | Action::ExportHtml
            | Action::ExportPdf
            | Action::InsertLink
            | Action::InsertTable
            | Action::InsertDate => ActionFamily::Export,
            Action::OpenGoto
            | Action::OpenProject
            | Action::OpenRecentProjects
            | Action::OpenThemeMenu
            | Action::OpenCaretMenu
            | Action::OpenDictionaryMenu
            | Action::OpenKeymapMenu
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
            | Action::OpenCredits => ActionFamily::Overlay,
            Action::LastBuffer
            | Action::NewDocument
            | Action::OpenScratch
            | Action::KeepTutorial
            | Action::MoveFile
            | Action::OpenRenameNote
            | Action::DuplicateNote
            | Action::SaveCopy
            | Action::TrashFile
            | Action::OpenSettings
            | Action::OpenSettingsMenu
            | Action::FinishBuffer
            | Action::ReviewChange
            | Action::ResolveKeepMine
            | Action::ResolveTakeTheirs
            | Action::FollowLink
            | Action::RevealInFileManager
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
            effect = apply_format_action(ctx, action).expect("format action");
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

pub(super) fn dispatch_action(ctx: &mut ActionCtx, action: &Action) -> Effect {
    match action_family(action) {
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
