//! Files-specific focus and acceptance grammar for the shared overlay gate.

use super::*;
use crate::overlay::FilesFocus;

pub(super) fn intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let card = ctx.journey.card()?;
    if card.kind != OverlayKind::Goto || !card.files_mode {
        return None;
    }
    // An Action can arrive without a key (menu key equivalent, menu click or
    // palette continuation), so New document is claimed at the Action gate.
    if matches!(action, Action::NewDocument) {
        let dest = card.files_destination().unwrap_or_default();
        ctx.journey.dismiss();
        return Some(Effect::NewDocumentAt(dest));
    }
    let prior_focus = card.files_focus;
    match action {
        Action::InsertTab => ctx.journey.card_mut()?.files_focus_step(1),
        Action::Outdent => ctx.journey.card_mut()?.files_focus_step(-1),
        Action::InsertChar(_) => {
            ctx.journey.card_mut()?.files_focus = FilesFocus::Query;
            return None;
        }
        Action::NextLine | Action::PreviousLine => {
            ctx.journey.card_mut()?.files_select_choices();
            if prior_focus == FilesFocus::Choices {
                return None;
            }
        }
        Action::Newline => match ctx.journey.card()?.files_focus {
            FilesFocus::Files => ctx.journey.card_mut()?.focus_facet_id("files"),
            FilesFocus::Recent => ctx.journey.card_mut()?.focus_facet_id("recent"),
            FilesFocus::Up => ascend(ctx),
            _ => return None,
        },
        Action::ForwardChar | Action::BackwardChar => match prior_focus {
            FilesFocus::Query => {
                let card = ctx.journey.card_mut()?;
                if matches!(action, Action::ForwardChar) {
                    card.query_char_right();
                } else {
                    card.query_char_left();
                }
            }
            FilesFocus::Files | FilesFocus::Recent => {
                ctx.journey.card_mut()?.files_focus = if matches!(action, Action::ForwardChar) {
                    FilesFocus::Recent
                } else {
                    FilesFocus::Files
                };
            }
            FilesFocus::Up if matches!(action, Action::BackwardChar) => ascend(ctx),
            FilesFocus::ChangeFolder | FilesFocus::NewDocument | FilesFocus::Up => {}
            FilesFocus::Choices => return None,
        },
        _ => return None,
    }
    Some(Effect::None)
}

fn ascend(ctx: &mut ActionCtx) {
    let card = ctx.journey.card().expect("Files card remains open");
    if let Some(parent) = crate::overlay::ascend_target(card)
        && let Some(next) = (ctx.browse_to)(card.kind, parent)
    {
        ctx.journey.relevel(next);
    }
}

pub(super) fn navigate(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let card = ctx.journey.card()?;
    if card.kind != OverlayKind::Goto || !card.files_mode || !card.query.is_empty() {
        return None;
    }
    match action {
        Action::ForwardChar if card.selected_is_dir() => {
            let path = card.selected_value()?.to_string();
            if let Some(next) = (ctx.browse_to)(card.kind, Some(path)) {
                ctx.journey.relevel(next);
            }
        }
        Action::BackwardChar if card.browse_dir.is_some() => ascend(ctx),
        _ => return None,
    }
    Some(Effect::None)
}

pub(super) fn accept(ctx: &mut ActionCtx) -> Option<Effect> {
    let card = ctx.journey.card()?;
    if card.kind != OverlayKind::Goto {
        return None;
    }
    if !card.files_mode {
        if card.selected_is_goto_folder() {
            let effect = card
                .selected_value()
                .map(|path| Effect::OverlayAccept(OverlayKind::Project, path.to_string()))
                .unwrap_or(Effect::None);
            dispose_after_accept(ctx);
            return Some(effect);
        }
        if selected_meta(card) == Some(crate::overlay::RowMetaTag::FolderChooser) {
            dispose_after_accept(ctx);
            return Some(Effect::Surface(SurfaceEffect::OpenFolderChooser));
        }
        return None;
    }
    if card.selected_is_goto_folder() {
        if let Some(path) = card.selected_value().map(str::to_string)
            && let Some(next) = (ctx.browse_to)(card.kind, Some(path))
        {
            ctx.journey.relevel(next);
        }
        return Some(Effect::None);
    }
    match selected_meta(card) {
        Some(crate::overlay::RowMetaTag::FolderChooser) => {
            Some(Effect::Surface(SurfaceEffect::OpenFolderChooser))
        }
        Some(crate::overlay::RowMetaTag::NewDocument) => {
            let dest = card.files_destination().unwrap_or_default();
            dispose_after_accept(ctx);
            Some(Effect::NewDocumentAt(dest))
        }
        _ => None,
    }
}

fn selected_meta(card: &OverlayState) -> Option<crate::overlay::RowMetaTag> {
    card.selected_corpus_index()
        .and_then(|index| card.rows.get(index))
        .map(|row| row.meta.tag())
}
