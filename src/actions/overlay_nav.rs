//! Modal navigation-overlay actions, shared by live input and `--keys` replay.

use super::*;
const OVERLAY_PAGE: isize = 12;

/// **THE ONE REBUILD A RESUME USES.** [`crate::overlay::Journey`] rebuilds a
/// parked parent FRESH (so its value cells show what the child just committed)
/// and then re-aims it — but the core cannot read a directory, so which injected
/// builder can answer depends on the kind: an explorer's LEVEL comes from
/// `browse_to` ([`crate::overlay::OverlayKind::needs_dir_level`]), everything
/// else from `make_overlay`, which returns `None` for those kinds by
/// construction. Handed `make_overlay` alone, a parked explorer rebuilds as
/// `None` and the whole journey falls to the editor — which is what Esc out of
/// the switch-project door did before this existed.
///
/// It also re-attaches the flat picker's door
/// ([`crate::overlay::OverlayState::attach_browse_door`], a no-op for every
/// other kind), because a rebuilt level is a bare directory listing and the door
/// belongs to the FEATURE, not to the disk.
///
/// Every lifecycle call that can land on `Resume` goes through the three
/// wrappers below, so which door you came back through cannot decide whether you
/// come back.
fn resume_rebuild<'c>(
    make_overlay: &'c mut dyn FnMut(crate::overlay::OverlayKind) -> Option<OverlayState>,
    browse_to: &'c mut dyn FnMut(
        crate::overlay::OverlayKind,
        Option<String>,
    ) -> Option<OverlayState>,
) -> impl FnMut(crate::overlay::OverlayKind) -> Option<OverlayState> + 'c {
    move |kind| match kind.needs_dir_level() {
        true => browse_to(kind, None).map(|mut card| {
            card.attach_browse_door();
            card
        }),
        false => make_overlay(kind),
    }
}

/// Esc / Back through the lifecycle's one cancel door, with [`resume_rebuild`].
fn journey_cancel(ctx: &mut ActionCtx) {
    let ActionCtx {
        journey,
        make_overlay,
        browse_to,
        ..
    } = ctx;
    journey.cancel(&mut resume_rebuild(&mut **make_overlay, &mut **browse_to));
}

/// An accept dispatched by its declared disposition, with [`resume_rebuild`].
fn journey_accept(ctx: &mut ActionCtx, disposition: crate::overlay::AcceptDisposition) {
    let ActionCtx {
        journey,
        make_overlay,
        browse_to,
        ..
    } = ctx;
    journey.accept(
        disposition,
        &mut resume_rebuild(&mut **make_overlay, &mut **browse_to),
    );
}

/// A row flipped in place, with [`resume_rebuild`].
fn journey_toggled(ctx: &mut ActionCtx) {
    let ActionCtx {
        journey,
        make_overlay,
        browse_to,
        ..
    } = ctx;
    journey.toggled(&mut resume_rebuild(&mut **make_overlay, &mut **browse_to));
}
fn rename_edit_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    ctx.journey.card().unwrap().rename_edit.as_ref()?;
    let overlay = ctx.journey.card_mut().unwrap();
    match action {
        Action::InsertChar(c) => overlay.rename_edit_push(*c),
        Action::DeleteBackward => overlay.rename_edit_pop(),
        Action::DeleteWordBackward => overlay.rename_edit_pop_word(),
        Action::ForwardChar => overlay.rename_edit_char_right(),
        Action::BackwardChar => overlay.rename_edit_char_left(),
        Action::ForwardWord => overlay.rename_edit_word_right(),
        Action::BackwardWord => overlay.rename_edit_word_left(),
        Action::DeleteWordForward => overlay.rename_edit_delete_word_forward(),
        Action::Newline => {
            let target = overlay.rename_edit_target();
            ctx.journey.dismiss();
            return Some(
                target
                    .map(|new_name| Effect::RenameNoteCommit { new_name })
                    .unwrap_or(Effect::None),
            );
        }
        Action::Cancel => {
            ctx.journey.dismiss();
        }
        _ => {}
    }
    Some(Effect::None)
}
fn link_edit_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    ctx.journey.card().unwrap().link_edit.as_ref()?;
    let overlay = ctx.journey.card_mut().unwrap();
    match action {
        Action::InsertChar(c) => overlay.link_edit_push(*c),
        Action::DeleteBackward => overlay.link_edit_pop(),
        Action::DeleteWordBackward => overlay.link_edit_pop_word(),
        Action::ForwardChar => overlay.link_edit_char_right(),
        Action::BackwardChar => overlay.link_edit_char_left(),
        Action::ForwardWord => overlay.link_edit_word_right(),
        Action::BackwardWord => overlay.link_edit_word_left(),
        Action::DeleteWordForward => overlay.link_edit_delete_word_forward(),
        Action::Newline => {
            let target = overlay.link_edit_target();
            ctx.journey.dismiss();
            if let Some((url, mode)) = target {
                let text = ctx.buffer.text();
                let result = crate::actions::link::commit(&text, &mode, &url);
                ctx.buffer
                    .apply_format(&result.text, result.anchor, result.cursor);
            }
        }
        Action::Cancel => {
            ctx.journey.dismiss();
        }
        _ => {}
    }
    Some(Effect::None)
}
fn keep_edit_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    ctx.journey.card().unwrap().keep_edit.as_ref()?;
    let overlay = ctx.journey.card_mut().unwrap();
    match action {
        Action::InsertChar(c) => overlay.keep_edit_push(*c),
        Action::DeleteBackward => overlay.keep_edit_pop(),
        Action::DeleteWordBackward => overlay.keep_edit_pop_word(),
        Action::ForwardChar => overlay.keep_edit_char_right(),
        Action::BackwardChar => overlay.keep_edit_char_left(),
        Action::ForwardWord => overlay.keep_edit_word_right(),
        Action::BackwardWord => overlay.keep_edit_word_left(),
        Action::DeleteWordForward => overlay.keep_edit_delete_word_forward(),
        Action::Newline => {
            let target = overlay.keep_edit_target();
            // Let the journey table decide whether a parked parent resumes.
            journey_accept(
                ctx,
                crate::overlay::OverlayKind::KeepName.accept_disposition(),
            );
            return Some(
                target
                    .map(|name| Effect::KeepVersion { name })
                    .unwrap_or(Effect::None),
            );
        }
        Action::Cancel => {
            // Cancel resumes a parked parent; dismiss would discard it.
            journey_cancel(ctx);
        }
        _ => {}
    }
    Some(Effect::None)
}
fn value_edit_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    ctx.journey.card().unwrap().value_edit.as_ref()?;
    let overlay = ctx.journey.card_mut().unwrap();
    match action {
        Action::InsertChar(c) => overlay.value_edit_push(*c),
        Action::DeleteBackward => overlay.value_edit_pop(),
        Action::DeleteWordBackward => overlay.value_edit_pop_word(),
        Action::ForwardChar => overlay.value_edit_char_right(),
        Action::BackwardChar => overlay.value_edit_char_left(),
        Action::ForwardWord => overlay.value_edit_word_right(),
        Action::BackwardWord => overlay.value_edit_word_left(),
        Action::DeleteWordForward => overlay.value_edit_delete_word_forward(),
        Action::Newline => {
            let target = overlay.value_edit_target();
            overlay.value_edit = None;
            return Some(
                target
                    .map(|(key, value)| Effect::SettingValueCommit { key, value })
                    .unwrap_or(Effect::None),
            );
        }
        Action::Cancel => overlay.value_edit_cancel(),
        _ => {}
    }
    Some(Effect::None)
}

/// Modal overlay input shared by the live app and `--keys` replay.
pub(super) fn overlay_intercept(ctx: &mut ActionCtx, action: &Action) -> Effect {
    if let Some(effect) = rename_edit_intercept(ctx, action) {
        return effect;
    }
    if let Some(effect) = link_edit_intercept(ctx, action) {
        return effect;
    }
    if let Some(effect) = keep_edit_intercept(ctx, action) {
        return effect;
    }
    if let Some(effect) = value_edit_intercept(ctx, action) {
        return effect;
    }
    if ctx.journey.card().unwrap().kind == crate::overlay::OverlayKind::Keybindings
        && let Some(eff) = keybindings_intercept(ctx, action)
    {
        return eff;
    }
    // The shape-aware intercept folds in what was a separate
    // `history_intercept`. See `workspace_nav::workspace_intercept`'s doc.
    if let Some(effect) = super::workspace_nav::workspace_intercept(ctx, action) {
        return effect;
    }
    if let Some(effect) = navigate_overlay(ctx, action) {
        return effect;
    }
    if ctx.journey.card().unwrap().kind == crate::overlay::OverlayKind::Context
        && !matches!(
            action,
            Action::Newline | Action::AcceptAlternate | Action::Cancel
        )
    {
        return Effect::None;
    }
    match action {
        Action::InsertChar(c) => {
            ctx.journey.card_mut().unwrap().push(*c);
            preview_move(ctx.journey.card_mut().unwrap());
            Effect::None
        }
        Action::DeleteBackward | Action::DeleteWordBackward => {
            let ov = ctx.journey.card().unwrap();
            // The switch-project picker is flat over the workspace's direct
            // children only, with no ascend affordance to leave that
            // boundary — Project only keeps the destination-navigator's
            // Backspace-ascends grammar for the Settings folder-VALUE picker
            // (`Bind::Path`), which walks the whole tree on purpose.
            let is_project_path_pick = ov.kind == crate::overlay::OverlayKind::Project
                && matches!(ctx.journey.bind(), Some(crate::overlay::Bind::Path { .. }));
            let navigable = ov.kind.is_folder_destination()
                || ov.kind == crate::overlay::OverlayKind::Browse
                || is_project_path_pick;
            if navigable && ov.query.is_empty() {
                if let Some(parent) = ascend_target(ov)
                    && let Some(next) = (ctx.browse_to)(ov.kind, parent)
                {
                    ctx.journey.relevel(next);
                }
                return Effect::None;
            }
            // The flat switch-project picker has nothing to pop on an EMPTY
            // query (no ascend, no typed filter) — an inert no-op, not a
            // fall-through to `pop()` below, which unconditionally resets
            // `selected` to row 0 and would silently bounce the highlight
            // back to the synthetic "." row on every idle Backspace.
            if ov.kind == crate::overlay::OverlayKind::Project && ov.query.is_empty() {
                return Effect::None;
            }
            if matches!(action, Action::DeleteWordBackward) {
                ctx.journey.card_mut().unwrap().pop_word();
            } else {
                ctx.journey.card_mut().unwrap().pop();
            }
            preview_move(ctx.journey.card_mut().unwrap());
            Effect::None
        }
        // `AcceptAlternate` (⇧↵) defaults to the SAME accept every
        // ordinary overlay kind already gives `Newline` (Goto opens, Theme
        // commits, …): only the shape-aware intercept above declares a
        // different meaning for bare `Newline` (History's timeline), and it
        // deliberately lets `AcceptAlternate` fall all the way through to
        // here so History's own accept — restoring the highlighted version —
        // fires exactly where bare Enter used to.
        Action::Newline | Action::AcceptAlternate => accept_overlay(ctx),
        Action::ForwardWord => {
            ctx.journey.card_mut().unwrap().query_word_right();
            preview_move(ctx.journey.card_mut().unwrap());
            Effect::None
        }
        Action::BackwardWord => {
            ctx.journey.card_mut().unwrap().query_word_left();
            preview_move(ctx.journey.card_mut().unwrap());
            Effect::None
        }
        Action::Cancel => cancel_overlay(ctx),
        _ => Effect::None,
    }
}

/// ESC / C-g — routed straight to the ONE lifecycle door
/// ([`crate::overlay::Journey::cancel`]), which reverts whatever the card was
/// auditioning and then lands wherever the table says: the editor, back on a
/// workspace's primary list, or on the parked parent at its exact position.
/// The only thing left here is the theme REPORT — the live App re-tints its GPU
/// pipelines and window title from `OverlayAccept`, and a revert changes the
/// active world just as a commit does.
fn cancel_overlay(ctx: &mut ActionCtx) -> Effect {
    let was_theme = ctx.journey.card().map(|o| o.kind) == Some(crate::overlay::OverlayKind::Theme);
    journey_cancel(ctx);
    if was_theme {
        Effect::OverlayAccept(
            crate::overlay::OverlayKind::Theme,
            crate::theme::active().name.to_string(),
        )
    } else {
        Effect::None
    }
}
fn navigate_overlay(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    match action {
        Action::NextLine => {
            ctx.journey.card_mut().unwrap().move_sel(1);
            preview_move(ctx.journey.card_mut().unwrap());
        }
        Action::PreviousLine => {
            ctx.journey.card_mut().unwrap().move_sel(-1);
            preview_move(ctx.journey.card_mut().unwrap());
        }
        Action::PageScrollDown => {
            ctx.journey.card_mut().unwrap().move_sel(OVERLAY_PAGE);
            preview_move(ctx.journey.card_mut().unwrap());
        }
        Action::PageScrollUp => {
            ctx.journey.card_mut().unwrap().move_sel(-OVERLAY_PAGE);
            preview_move(ctx.journey.card_mut().unwrap());
        }
        Action::LineStart | Action::BufferStart => {
            ctx.journey.card_mut().unwrap().select_first();
            preview_move(ctx.journey.card_mut().unwrap());
        }
        Action::LineEnd | Action::BufferEnd => {
            ctx.journey.card_mut().unwrap().select_last();
            preview_move(ctx.journey.card_mut().unwrap());
        }
        Action::ForwardChar => {
            if let Some(effect) = range_step(ctx, 1) {
                return Some(effect);
            }
            let ov = ctx.journey.card().unwrap();
            if ov.is_faceting() {
                ctx.journey.card_mut().unwrap().cycle_lens(1);
                preview_move(ctx.journey.card_mut().unwrap());
            } else if ov.kind.is_folder_destination() {
                if ov.selected_is_dir()
                    && let Some(name) = ov.selected_value().map(str::to_string)
                {
                    let child = descend_target(ov, &name);
                    if let Some(next) = (ctx.browse_to)(ov.kind, Some(child)) {
                        ctx.journey.relevel(next);
                    }
                }
            } else {
                ctx.journey.card_mut().unwrap().move_sel(1);
                preview_move(ctx.journey.card_mut().unwrap());
            }
        }
        Action::BackwardChar => {
            if let Some(effect) = range_step(ctx, -1) {
                return Some(effect);
            }
            let ov = ctx.journey.card().unwrap();
            if ov.is_faceting() {
                ctx.journey.card_mut().unwrap().cycle_lens(-1);
                preview_move(ctx.journey.card_mut().unwrap());
            } else if ov.kind.is_folder_destination() {
                if let Some(parent) = ascend_target(ov)
                    && let Some(next) = (ctx.browse_to)(ov.kind, parent)
                {
                    ctx.journey.relevel(next);
                }
            } else {
                ctx.journey.card_mut().unwrap().move_sel(-1);
                preview_move(ctx.journey.card_mut().unwrap());
            }
        }
        _ => return None,
    }
    Some(Effect::None)
}

fn accept_path_overlay(ctx: &mut ActionCtx) -> Option<Effect> {
    let ov = ctx.journey.card().unwrap();
    match ov.kind {
        crate::overlay::OverlayKind::Browse => {
            let effect = match ov.selected_value().map(str::to_string) {
                Some(name) if ov.selected_is_dir() => {
                    if let Some(next) =
                        (ctx.browse_to)(ov.kind, Some(join_browse(ov.browse_dir.as_deref(), &name)))
                    {
                        ctx.journey.relevel(next);
                    }
                    return Some(Effect::None);
                }
                Some(name) => Effect::OverlayAccept(
                    crate::overlay::OverlayKind::Goto,
                    join_browse(ov.browse_dir.as_deref(), &name),
                ),
                None => Effect::None,
            };
            dispose_after_accept(ctx);
            Some(effect)
        }
        // THE TWO DESTINATION NAVIGATORS: one folder answer
        // ([`move_dest_value`]), two things to put in it. The move rides the
        // generic accept; the export rides `Effect::Export`, because the folder
        // is one component of a request that also carries the FORMAT the summon
        // chose — carried on the card across every level change (`OverlayState::
        // carry_level_payload_from`).
        crate::overlay::OverlayKind::MoveDest => {
            let effect = dest_value(ov, true)
                .map(|dest| Effect::OverlayAccept(crate::overlay::OverlayKind::MoveDest, dest))
                .unwrap_or(Effect::None);
            dispose_after_accept(ctx);
            Some(effect)
        }
        crate::overlay::OverlayKind::ExportDest => {
            let effect = match (ov.export_format, dest_value(ov, true)) {
                (Some(format), Some(dest)) => Effect::Export(format, Some(dest)),
                _ => Effect::None,
            };
            dispose_after_accept(ctx);
            Some(effect)
        }
        // THE THIRD DESTINATION NAVIGATOR: the same walk and the same accept
        // arithmetic, and what lands in the folder is the PROJECT ITSELF. It
        // emits the switch under [`OverlayKind::Project`]'s own accept effect —
        // one owner of "switch to this root", whichever door reached it — so
        // the App, the replay classifier and the sidecar's project block need
        // to know nothing about this kind.
        crate::overlay::OverlayKind::ProjectBrowse => {
            let effect = dest_value(ov, false)
                .filter(|dest| !dest.is_empty())
                .map(|dest| Effect::OverlayAccept(crate::overlay::OverlayKind::Project, dest))
                .unwrap_or(Effect::None);
            dispose_after_accept(ctx);
            Some(effect)
        }
        crate::overlay::OverlayKind::Project => {
            let path_key = match ctx.journey.bind() {
                Some(crate::overlay::Bind::Path { key }) => Some(key.clone()),
                Some(crate::overlay::Bind::Value) | None => None,
            };
            // THE DOOR ROW (`RowMeta::ProjectDoor`) — the flat picker's one
            // reach past the workspace's direct children, and the reason the
            // flat accept below can stay flat. It DESCENDS rather than
            // switching: the folder navigator takes the stage and this card
            // parks at its exact row, so Esc comes back here instead of
            // dropping to the document. Asked before the row is read as a
            // project, and asked of the row's META, so the label is only a
            // label.
            if ov.selected_is_browse_door() {
                if let Some(child) =
                    (ctx.browse_to)(crate::overlay::OverlayKind::ProjectBrowse, None)
                {
                    ctx.journey.descend(child, crate::overlay::Bind::Value);
                }
                return Some(Effect::None);
            }
            // FLAT OVER DIRECT WORKSPACE CHILDREN ONLY: the switch-project
            // picker (no path key — a plain launch, never a Settings
            // folder-VALUE pick) never descends. A folder row IS the
            // project; accepting it switches immediately, so a grandchild
            // can never enter the roster. Deeper navigation is the door
            // above, and the Settings path-picker's own descend grammar
            // (`Bind::Path`, below).
            if path_key.is_none() && ov.selected_is_dir() {
                let target = ov.selected_value().map(|name| descend_target(ov, name));
                return Some(match target {
                    Some(path) => {
                        ctx.journey.navigate_away();
                        Effect::OverlayAccept(crate::overlay::OverlayKind::Project, path)
                    }
                    None => Effect::None,
                });
            }
            if ov.selected_is_dir() {
                if let Some(name) = ov.selected_value().map(str::to_string)
                    && let Some(next) = (ctx.browse_to)(ov.kind, Some(descend_target(ov, &name)))
                {
                    // A LEVEL, not a rung: `relevel` keeps whatever parent is
                    // parked and whatever config key this navigator is filling
                    // in, so a descend/ascend can no longer silently drop them.
                    ctx.journey.relevel(next);
                }
                return Some(Effect::None);
            }
            let dir = ctx
                .journey
                .card()
                .and_then(|o| o.browse_dir.clone())
                .filter(|dir| !dir.is_empty());
            let result = match (dir, path_key) {
                // A folder picked FOR A CONFIG KEY is a value commit, so it
                // lands wherever the table sends a value commit — back on the
                // Settings row you left, or in the document.
                (Some(path), Some(key)) => {
                    journey_accept(ctx, crate::overlay::AcceptDisposition::ValuePick);
                    Effect::SettingPathPick { key, path }
                }
                (Some(path), None) => {
                    ctx.journey.navigate_away();
                    Effect::OverlayAccept(crate::overlay::OverlayKind::Project, path)
                }
                (None, _) => {
                    journey_cancel(ctx);
                    Effect::None
                }
            };
            Some(result)
        }
        _ => None,
    }
}

fn accept_value_overlay(ctx: &mut ActionCtx) -> Effect {
    if let Some(effect) = accept_context(ctx) {
        return effect;
    }
    let ov = ctx.journey.card().unwrap();
    if ov.kind == crate::overlay::OverlayKind::Command {
        // Map through the platform-filtered catalog, then close before re-dispatch.
        let eff = ov
            .selected_corpus_index()
            .map(|i| Effect::RunAction(crate::commands::visible_action_of(i)))
            .unwrap_or(Effect::None);
        dispose_after_accept(ctx);
        return eff;
    }
    if ov.kind.previews_live_document() {
        // COMMIT: the highlighted value is ALREADY live (`preview_overlay`
        // applied it as the selection moved), so Enter just keeps it and
        // closes. Emit the committed name so the caller can re-tint its
        // GPU pipelines / window title to match. Asked of the audition's own
        // owner rather than naming its two members, so this arm and the
        // preview that makes it a KEEP cannot enrol different kinds.
        let eff = match ov.selected_value() {
            Some(v) => Effect::OverlayAccept(ov.kind, v.to_string()),
            None => Effect::None,
        };
        dispose_after_accept(ctx);
        return eff;
    }
    if ov.kind == crate::overlay::OverlayKind::Dictionary {
        let eff = match ov
            .selected_value()
            .and_then(crate::spell::DictVariant::from_label)
        {
            Some(dv) => {
                crate::spell::set_active_variant(dv);
                Effect::OverlayAccept(ov.kind, dv.label().to_string())
            }
            None => Effect::None,
        };
        dispose_after_accept(ctx);
        return eff;
    }
    if ov.kind == crate::overlay::OverlayKind::CjkLang {
        let eff = match ov
            .selected_value()
            .and_then(crate::frontmatter::Lang::from_label)
        {
            Some(lang) => {
                let promoted = crate::frontmatter::promote_cjk_priority(lang);
                crate::frontmatter::set_cjk_priority(&promoted);
                Effect::OverlayAccept(ov.kind, lang.code().to_string())
            }
            None => Effect::None,
        };
        dispose_after_accept(ctx);
        return eff;
    }
    if ov.kind == crate::overlay::OverlayKind::Date {
        let eff = match ov
            .selected_corpus_index()
            .and_then(|i| crate::dateformat::DateFormat::ALL.get(i).copied())
        {
            Some(fmt) => {
                crate::dateformat::set_active_format(fmt);
                Effect::OverlayAccept(ov.kind, fmt.config_name().to_string())
            }
            None => Effect::None,
        };
        dispose_after_accept(ctx);
        return eff;
    }
    if ov.kind == crate::overlay::OverlayKind::Goto && ov.selected_is_heading() {
        let eff = match ov.selected_line() {
            Some(line) => Effect::JumpToLine(line),
            None => Effect::None,
        };
        dispose_after_accept(ctx);
        return eff;
    }
    if ov.kind == crate::overlay::OverlayKind::Assets {
        // ASSET CLEANER: Enter REQUESTS the highlighted orphan be trashed. Emit
        // its root-relative path (the corpus value) for the App to trash +
        // remove the row; the picker STAYS OPEN (no `close_overlay`), and the
        // core never touches the row itself (the App removes it only after a
        // successful trash — see `Effect::TrashAsset`). An empty state (no
        // selection) is a calm no-op.
        return match ov.selected_value() {
            Some(rel) => Effect::TrashAsset {
                rel: rel.to_string(),
            },
            None => Effect::None,
        };
    }
    if ov.kind == crate::overlay::OverlayKind::History {
        let eff = match ov.selected_history_id() {
            Some(id) => Effect::OverlayAccept(ov.kind, id.to_string()),
            None => Effect::None,
        };
        dispose_after_accept(ctx);
        return eff;
    }
    // GENERIC fallthrough — reached by a Go-to FILE row (a non-heading Goto),
    // whose accept OPENS the file (Navigate). Routed through the shared
    // disposition owner so it closes the whole stack.
    let eff = match ov.selected_value() {
        Some(v) => Effect::OverlayAccept(ov.kind, v.to_string()),
        None => Effect::None,
    };
    dispose_after_accept(ctx);
    eff
}

fn accept_context(ctx: &mut ActionCtx) -> Option<Effect> {
    if ctx.journey.card().unwrap().kind != crate::overlay::OverlayKind::Context {
        return None;
    }
    let effect = ctx
        .journey
        .card()
        .unwrap()
        .selected_corpus_index()
        .and_then(|i| ctx.journey.card().unwrap().context_actions[i].clone())
        .map(Effect::RunAction)
        .unwrap_or(Effect::None);
    dispose_after_accept(ctx);
    Some(effect)
}

fn accept_overlay(ctx: &mut ActionCtx) -> Effect {
    {
        let ov = ctx.journey.card().unwrap();
        if ov.kind == crate::overlay::OverlayKind::Spell {
            // "Add '<word>' to dictionary" row: SIGNAL the add (the live App
            // silences the word + appends it to the on-disk personal
            // dictionary) and close — NEVER a buffer edit. The word rides
            // `add_word`, so this stays decoupled from the buffer span.
            if ov.selected_is_add_to_dictionary() {
                let word = ov.add_word.clone();
                dispose_after_accept(ctx);
                return word.map(Effect::AddToDictionary).unwrap_or(Effect::None);
            }
            let pick = ov.selected_value().map(|s| s.to_string());
            let target = ov.spell_target;
            if let (Some(word), Some((line, start, end))) = (pick, target) {
                let s = ctx.buffer.line_col_to_char(line, start);
                let e = ctx.buffer.line_col_to_char(line, end);
                ctx.buffer.replace_char_range(s, e, &word);
            }
            // A spell replace is a buffer EDIT (Navigate): close the whole
            // stack, never pop back to a summoning overlay.
            dispose_after_accept(ctx);
            return Effect::None;
        }
        if ov.kind == crate::overlay::OverlayKind::Settings {
            return settings_accept(ctx);
        }
        // THE UNION ROUND: a settings row reached via the COMMAND PALETTE
        // dispatches through the SAME owner Enter uses inside the Settings
        // menu (`dispatch_settings_row`) — with the breadcrumb set to `Command`
        // (so a Picker/Submenu/Path row it opens pops back to the palette on
        // Esc, mirroring how running "Switch theme…" from the palette behaves)
        // and `close_on_toggle: true` (activating a Toggle/Action CLOSES the
        // palette, the palette's own "running a row closes it" convention — the
        // Settings menu's OWN accept, just above, stays open instead). An
        // ordinary command row (not a setting) falls through to the RunAction
        // path below.
        if ov.kind == crate::overlay::OverlayKind::Command
            && let Some(row) = ov.selected_setting_row()
        {
            return dispatch_settings_row(ctx, row);
        }
    }
    if let Some(effect) = accept_path_overlay(ctx) {
        return effect;
    }
    accept_value_overlay(ctx)
}

/// Dispose of the card after an ACCEPT, per the highlighted kind's declared
/// [`crate::overlay::AcceptDisposition`] — the one place the accept
/// classification meets the lifecycle. Where each disposition LANDS is the
/// table's business, not this function's: `Navigate` ends the journey,
/// `ValuePick` returns to a parked parent or completes the errand depending on
/// what that parent was, and `StayOpen` leaves the card up. A no-op with no
/// card, since the table's editor row is all `Stay` but `Dismiss`.
pub(super) fn dispose_after_accept(ctx: &mut ActionCtx) {
    let Some(kind) = ctx.journey.card().map(|o| o.kind) else {
        return;
    };
    journey_accept(ctx, kind.accept_disposition());
}

fn range_ctx_value(
    id: crate::settings::SettingId,
    cell: crate::overlay::RangeCell,
    ctx: &ActionCtx,
) -> Option<f32> {
    Some(match id {
        crate::settings::SettingId::Zoom => *ctx.zoom,
        crate::settings::SettingId::ScrollSensitivity => crate::settings::scroll_sensitivity(),
        crate::settings::SettingId::PageWidthProse | crate::settings::SettingId::PageWidthCode => {
            crate::settings::range_spec(id)?.value_of_step(cell.step)
        }
        _ => return None,
    })
}
fn range_ctx_set(id: crate::settings::SettingId, ctx: &mut ActionCtx, v: f32) {
    if id == crate::settings::SettingId::Zoom {
        *ctx.zoom = v
    } else if id == crate::settings::SettingId::ScrollSensitivity {
        crate::settings::set_scroll_sensitivity(v);
    }
}
fn range_step(ctx: &mut ActionCtx, steps: i32) -> Option<Effect> {
    let cell = ctx.journey.card()?.selected_range()?;
    let spec = crate::settings::range_spec(cell.id)?;
    let key = crate::settings::value_key(cell.id)?;
    let cur = range_ctx_value(cell.id, cell, ctx)?;
    let next = spec.stepped(cur, steps);
    range_ctx_set(cell.id, ctx, next);
    ctx.journey
        .card_mut()?
        .set_selected_range(spec.step_of(next), spec.format(next));
    Some(Effect::SettingRangeStep {
        key: key.to_string(),
    })
}

fn settings_accept(ctx: &mut ActionCtx) -> Effect {
    let Some(ci) = ctx.journey.card().unwrap().selected_corpus_index() else {
        journey_cancel(ctx);
        return Effect::None;
    };
    let row = *crate::settings::visible_rows()[ci];
    dispatch_settings_row(ctx, row)
}

/// THE SHARED settings-row dispatcher — the ONE owner both [`settings_accept`]
/// (Enter inside the Settings menu itself) and the Command palette's own
/// settings-row accept call, so the two can never drift.
///
/// It used to take a `breadcrumb` kind and a `close_on_toggle` flag, because
/// the caller was the only thing that knew whether it was the Settings menu or
/// the palette. Both are gone: the journey already knows what surface it is
/// standing on, so a Picker/Submenu/Path row DESCENDS (parking the caller at
/// its exact row) and a Toggle asks the table, which keeps a workspace open and
/// completes a launcher's errand.
fn dispatch_settings_row(ctx: &mut ActionCtx, row: crate::settings::SettingRow) -> Effect {
    match row.kind {
        crate::settings::SettingKind::Toggle => {
            let key = crate::settings::toggle_key(row.id).expect(
                "Toggle row always resolves its config key — settings law \
                 every_toggle_has_a_config_key_and_nothing_else_does",
            );
            journey_toggled(ctx);
            Effect::SettingToggle {
                key: key.to_string(),
            }
        }
        crate::settings::SettingKind::Picker | crate::settings::SettingKind::Submenu => {
            let target = crate::settings::sub_overlay(row.id).expect(
                "Picker/Submenu row always resolves its sub-overlay — settings law \
                 pickers_and_submenus_open_a_sub_overlay_and_nothing_else_does",
            );
            if let Some(next) = (ctx.make_overlay)(target) {
                ctx.journey.descend(next, crate::overlay::Bind::Value);
            }
            Effect::None
        }
        crate::settings::SettingKind::Action => {
            ctx.journey.navigate_away();
            match row.id {
                crate::settings::SettingId::ReportProblem => Effect::ReportProblem,
                crate::settings::SettingId::EditConfigAsText => {
                    Effect::Buffer(BufferEffect::OpenSettings)
                }
                _ => Effect::None,
            }
        }
        crate::settings::SettingKind::Value | crate::settings::SettingKind::Range => {
            // The Cmd-P DEEP LINK; its whole argument is on
            // `deep_link_settings`.
            if super::workspace_nav::deep_link_settings(ctx, row) {
                return Effect::None;
            }
            let key = crate::settings::value_key(row.id).expect(
                "Value/Range row always resolves its config key — settings law \
                 value_and_path_keys_track_their_kinds",
            );
            ctx.journey
                .card_mut()
                .unwrap()
                .start_value_edit(key.to_string(), row.name.to_string());
            Effect::None
        }
        crate::settings::SettingKind::Path => {
            let key = crate::settings::path_key(row.id).expect(
                "Path row always resolves its config key — settings law \
                 value_and_path_keys_track_their_kinds",
            );
            if let Some(nav) = (ctx.browse_to)(crate::overlay::OverlayKind::Project, None) {
                let bind = crate::overlay::Bind::Path {
                    key: key.to_string(),
                };
                ctx.journey.descend(nav, bind);
            }
            Effect::None
        }
    }
}

pub(super) fn join_browse(dir: Option<&str>, name: &str) -> String {
    match dir {
        Some(d) if !d.is_empty() => format!("{d}/{name}"),
        _ => name.to_string(),
    }
}

pub(super) fn browse_parent(dir: Option<&str>) -> Option<Option<String>> {
    match dir {
        None => None, // already at root; nothing above
        Some(d) => match d.rsplit_once('/') {
            Some((parent, _)) => Some(Some(parent.to_string())),
            None => Some(None), // one level deep -> back to root
        },
    }
}

/// THE TWO ABSOLUTE-PATH WALKERS: `Project` (the Settings folder-VALUE picker's
/// levels) and `ProjectBrowse` (the switch-project door's) carry a whole
/// directory in `browse_dir`, so a level change is a path join, not a
/// root-relative string append.
fn walks_absolute(kind: crate::overlay::OverlayKind) -> bool {
    matches!(
        kind,
        crate::overlay::OverlayKind::Project | crate::overlay::OverlayKind::ProjectBrowse
    )
}

/// THE PATH A HIGHLIGHTED ROW NAMES. `name` is usually a child NAME joined onto
/// the level — but a switch-project REMEMBERED row (`overlay::build::recent`)
/// carries a whole ABSOLUTE path, and `Path::join` then answers with that path
/// itself. Relied on, not overlooked: a remembered root IS the project, wherever
/// it lives, so the level it is listed beside has no part in the answer.
pub(super) fn descend_target(ov: &OverlayState, name: &str) -> String {
    match walks_absolute(ov.kind) {
        true => std::path::Path::new(ov.browse_dir.as_deref().unwrap_or(""))
            .join(name)
            .to_string_lossy()
            .to_string(),
        false => join_browse(ov.browse_dir.as_deref(), name),
    }
}

pub(super) fn ascend_target(ov: &OverlayState) -> Option<Option<String>> {
    match walks_absolute(ov.kind) {
        true => std::path::Path::new(ov.browse_dir.as_deref().unwrap_or("/"))
            .parent()
            .map(|p| Some(p.to_string_lossy().to_string())),
        false => browse_parent(ov.browse_dir.as_deref()),
    }
}

/// THE FOLDER A DESTINATION NAVIGATOR'S ACCEPT NAMES: the highlighted folder,
/// else the level you are standing in.
///
/// `allow_new` is whether a TYPED name with no matching row counts as an answer.
/// A move or an export CREATES the folder it names, so they say yes; the
/// switch-project door says no, because there is no project to switch to in a
/// folder that does not exist — and its typed query is a filter that simply
/// matched nothing, which leaves the level itself as the honest answer.
pub(super) fn dest_value(ov: &OverlayState, allow_new: bool) -> Option<String> {
    if let Some(name) = ov.selected_value()
        && ov.selected_is_dir()
    {
        return Some(join_browse(ov.browse_dir.as_deref(), name));
    }
    let q = ov.query.text().trim();
    if allow_new && !q.is_empty() {
        return Some(join_browse(ov.browse_dir.as_deref(), q));
    }
    Some(ov.browse_dir.clone().unwrap_or_default())
}

/// THE LIVE AUDITION: apply the highlighted row's value to the running editor so
/// the page behind the card shows it before anything is committed.
///
/// The roster gate is [`crate::overlay::OverlayKind::previews_live_document`], not
/// the arms below: a kind that has not declared the audition never reaches its
/// own arm, so an arm added here alone is inert until the declaration is made —
/// and the declaration is what pins the card's backdrop crisp
/// (`keeps_backdrop_crisp`, asserted equal over the roster). Without the gate the
/// wildcard swallowed the difference, and a new previewing kind could blur the
/// only thing its rows were showing with every law still green.
pub(crate) fn preview_overlay(ov: &OverlayState) {
    if !ov.kind.previews_live_document() {
        return;
    }
    match ov.kind {
        crate::overlay::OverlayKind::Theme => {
            if let Some(name) = ov.selected_value() {
                crate::theme::set_active_by_name(name);
            }
        }
        crate::overlay::OverlayKind::Caret => {
            if let Some(m) = ov.selected_caret_mode() {
                crate::caret::set_mode(m);
            }
        }
        // Unreachable behind the gate above: a kind that answers yes and lands
        // here has declared an audition it never wired, which the roster law
        // catches by finding nothing changed.
        _ => {}
    }
}

pub(crate) fn preview_move(ov: &mut OverlayState) {
    preview_overlay(ov);
    ov.reanchor();
}
