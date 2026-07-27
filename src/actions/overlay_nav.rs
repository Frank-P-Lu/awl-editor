//! Modal navigation-overlay actions, shared by live input and `--keys` replay.

use super::*;

const OVERLAY_PAGE: isize = 12;

fn rename_edit_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    ctx.overlay.as_ref().unwrap().rename_edit.as_ref()?;
    let overlay = ctx.overlay.as_mut().unwrap();
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
            *ctx.overlay = None;
            return Some(
                target
                    .map(|new_name| Effect::RenameNoteCommit { new_name })
                    .unwrap_or(Effect::None),
            );
        }
        Action::Cancel => *ctx.overlay = None,
        _ => {}
    }
    Some(Effect::None)
}

fn link_edit_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    ctx.overlay.as_ref().unwrap().link_edit.as_ref()?;
    let overlay = ctx.overlay.as_mut().unwrap();
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
            *ctx.overlay = None;
            if let Some((url, mode)) = target {
                let text = ctx.buffer.text();
                let result = crate::actions::link::commit(&text, &mode, &url);
                ctx.buffer
                    .apply_format(&result.text, result.anchor, result.cursor);
            }
        }
        Action::Cancel => *ctx.overlay = None,
        _ => {}
    }
    Some(Effect::None)
}

fn keep_edit_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    ctx.overlay.as_ref().unwrap().keep_edit.as_ref()?;
    let overlay = ctx.overlay.as_mut().unwrap();
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
            *ctx.overlay = None;
            return Some(
                target
                    .map(|name| Effect::KeepVersion { name })
                    .unwrap_or(Effect::None),
            );
        }
        Action::Cancel => *ctx.overlay = None,
        _ => {}
    }
    Some(Effect::None)
}

fn value_edit_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    ctx.overlay.as_ref().unwrap().value_edit.as_ref()?;
    let overlay = ctx.overlay.as_mut().unwrap();
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

/// The modal OVERLAY INTERCEPT. When the summoned navigation overlay is open, it OWNS
/// every key: printable chars extend the overlay query (never the rope), Up/Down (and
/// C-n/C-p, which resolve to NextLine/PreviousLine) move the selection, Enter accepts
/// the highlighted item, Esc/C-g cancels. Routing this through the shared core (rather
/// than only in `App`) is exactly what makes the overlay drivable under `--keys` — the
/// same mistake the isearch panel made (its query routing lives in `App`, so `--keys`
/// can't type into it) is deliberately avoided here. Returns the one [`Effect`] the key
/// signals back; `apply_core` returns it directly (the overlay is modal, so the key
/// never reaches the buffer).
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
    if ctx.overlay.as_ref().unwrap().kind == crate::overlay::OverlayKind::Keybindings
        && let Some(eff) = keybindings_intercept(ctx, action)
    {
        return eff;
    }
    if let Some(effect) = history_intercept(ctx, action) {
        return effect;
    }
    if let Some(effect) = navigate_overlay(ctx, action) {
        return effect;
    }
    match action {
        Action::InsertChar(c) => {
            ctx.overlay.as_mut().unwrap().push(*c);
            preview_move(ctx.overlay.as_mut().unwrap());
            Effect::None
        }
        Action::DeleteBackward | Action::DeleteWordBackward => {
            let ov = ctx.overlay.as_ref().unwrap();
            let navigable = matches!(
                ov.kind,
                crate::overlay::OverlayKind::Browse
                    | crate::overlay::OverlayKind::MoveDest
                    | crate::overlay::OverlayKind::Project
            );
            if navigable && ov.query.is_empty() {
                let bc = Breadcrumb::of(ov);
                if let Some(parent) = ascend_target(ov)
                    && let Some(mut next) = (ctx.browse_to)(ov.kind, parent)
                {
                    bc.apply(&mut next);
                    *ctx.overlay = Some(next);
                }
                return Effect::None;
            }
            if matches!(action, Action::DeleteWordBackward) {
                ctx.overlay.as_mut().unwrap().pop_word();
            } else {
                ctx.overlay.as_mut().unwrap().pop();
            }
            preview_move(ctx.overlay.as_mut().unwrap());
            Effect::None
        }
        Action::Newline => accept_overlay(ctx),
        Action::ForwardWord => {
            ctx.overlay.as_mut().unwrap().query_word_right();
            preview_move(ctx.overlay.as_mut().unwrap());
            Effect::None
        }
        Action::BackwardWord => {
            ctx.overlay.as_mut().unwrap().query_word_left();
            preview_move(ctx.overlay.as_mut().unwrap());
            Effect::None
        }
        Action::Cancel => cancel_overlay(ctx),
        _ => Effect::None,
    }
}

fn cancel_overlay(ctx: &mut ActionCtx) -> Effect {
    let ov = ctx.overlay.as_ref().unwrap();
    let effect = if ov.kind == crate::overlay::OverlayKind::Theme {
        if let Some(theme) = ov.original_theme {
            crate::theme::set_active(theme);
        }
        Effect::OverlayAccept(
            crate::overlay::OverlayKind::Theme,
            crate::theme::active().name.to_string(),
        )
    } else if ov.kind == crate::overlay::OverlayKind::Caret {
        if ov.original_caret_was_auto {
            crate::caret::clear_override();
        } else if let Some(caret) = ov.original_caret {
            crate::caret::set_mode(caret);
        }
        Effect::None
    } else {
        Effect::None
    };
    close_overlay(ctx);
    effect
}
fn history_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let ov = ctx.overlay.as_ref().unwrap();
    if ov.kind != crate::overlay::OverlayKind::History || ov.selected_history_id().is_none() {
        return None;
    }
    let page = ctx.scroll_page_lines.max(1);
    let focused = ov.diff_focus;
    match action {
        Action::PageScrollDown => {
            let ov = ctx.overlay.as_mut().unwrap();
            ov.diff_scroll = ov.diff_scroll.saturating_add(page);
            Some(Effect::None)
        }
        Action::PageScrollUp => {
            let ov = ctx.overlay.as_mut().unwrap();
            ov.diff_scroll = ov.diff_scroll.saturating_sub(page);
            Some(Effect::None)
        }
        Action::CompareVersion | Action::InsertTab => {
            let ov = ctx.overlay.as_mut().unwrap();
            ov.diff_focus = !ov.diff_focus;
            Some(Effect::None)
        }
        _ if focused => match action {
            Action::NextLine => {
                let ov = ctx.overlay.as_mut().unwrap();
                ov.diff_scroll = ov.diff_scroll.saturating_add(1);
                Some(Effect::None)
            }
            Action::PreviousLine => {
                let ov = ctx.overlay.as_mut().unwrap();
                ov.diff_scroll = ov.diff_scroll.saturating_sub(1);
                Some(Effect::None)
            }
            Action::Cancel => {
                ctx.overlay.as_mut().unwrap().diff_focus = false;
                Some(Effect::None)
            }
            Action::Newline => None,
            _ => Some(Effect::None),
        },
        _ => None,
    }
}

fn navigate_overlay(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    match action {
        Action::NextLine => {
            ctx.overlay.as_mut().unwrap().move_sel(1);
            preview_move(ctx.overlay.as_mut().unwrap());
        }
        Action::PreviousLine => {
            ctx.overlay.as_mut().unwrap().move_sel(-1);
            preview_move(ctx.overlay.as_mut().unwrap());
        }
        Action::PageScrollDown => {
            ctx.overlay.as_mut().unwrap().move_sel(OVERLAY_PAGE);
            preview_move(ctx.overlay.as_mut().unwrap());
        }
        Action::PageScrollUp => {
            ctx.overlay.as_mut().unwrap().move_sel(-OVERLAY_PAGE);
            preview_move(ctx.overlay.as_mut().unwrap());
        }
        Action::LineStart | Action::BufferStart => {
            ctx.overlay.as_mut().unwrap().select_first();
            preview_move(ctx.overlay.as_mut().unwrap());
        }
        Action::LineEnd | Action::BufferEnd => {
            ctx.overlay.as_mut().unwrap().select_last();
            preview_move(ctx.overlay.as_mut().unwrap());
        }
        Action::ForwardChar => {
            if let Some(effect) = range_step(ctx, 1) {
                return Some(effect);
            }
            let ov = ctx.overlay.as_ref().unwrap();
            if ov.is_faceting() {
                ctx.overlay.as_mut().unwrap().cycle_lens(1);
                preview_move(ctx.overlay.as_mut().unwrap());
            } else if ov.kind == crate::overlay::OverlayKind::MoveDest {
                if ov.selected_is_dir()
                    && let Some(name) = ov.selected_value().map(str::to_string)
                {
                    let child = descend_target(ov, &name);
                    if let Some(next) = (ctx.browse_to)(ov.kind, Some(child)) {
                        *ctx.overlay = Some(next);
                    }
                }
            } else {
                ctx.overlay.as_mut().unwrap().move_sel(1);
                preview_move(ctx.overlay.as_mut().unwrap());
            }
        }
        Action::BackwardChar => {
            if let Some(effect) = range_step(ctx, -1) {
                return Some(effect);
            }
            let ov = ctx.overlay.as_ref().unwrap();
            if ov.is_faceting() {
                ctx.overlay.as_mut().unwrap().cycle_lens(-1);
                preview_move(ctx.overlay.as_mut().unwrap());
            } else if ov.kind == crate::overlay::OverlayKind::MoveDest {
                if let Some(parent) = ascend_target(ov)
                    && let Some(next) = (ctx.browse_to)(ov.kind, parent)
                {
                    *ctx.overlay = Some(next);
                }
            } else {
                ctx.overlay.as_mut().unwrap().move_sel(-1);
                preview_move(ctx.overlay.as_mut().unwrap());
            }
        }
        _ => return None,
    }
    Some(Effect::None)
}

fn accept_path_overlay(ctx: &mut ActionCtx) -> Option<Effect> {
    let ov = ctx.overlay.as_ref().unwrap();
    match ov.kind {
        crate::overlay::OverlayKind::Browse => {
            let effect = match ov.selected_value().map(str::to_string) {
                Some(name) if ov.selected_is_dir() => {
                    if let Some(next) =
                        (ctx.browse_to)(ov.kind, Some(join_browse(ov.browse_dir.as_deref(), &name)))
                    {
                        *ctx.overlay = Some(next);
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
        crate::overlay::OverlayKind::MoveDest => {
            let effect = move_dest_value(ov)
                .map(|dest| Effect::OverlayAccept(crate::overlay::OverlayKind::MoveDest, dest))
                .unwrap_or(Effect::None);
            dispose_after_accept(ctx);
            Some(effect)
        }
        crate::overlay::OverlayKind::Project => {
            if ov.selected_is_dir() {
                let breadcrumb = Breadcrumb::of(ov);
                if let Some(name) = ov.selected_value().map(str::to_string)
                    && let Some(mut next) =
                        (ctx.browse_to)(ov.kind, Some(descend_target(ov, &name)))
                {
                    breadcrumb.apply(&mut next);
                    *ctx.overlay = Some(next);
                }
                return Some(Effect::None);
            }
            let path_key = ov.setting_path_key.clone();
            let result = match ov.browse_dir.clone().filter(|dir| !dir.is_empty()) {
                Some(path) if path_key.is_some() => {
                    close_overlay(ctx);
                    Effect::SettingPathPick {
                        key: path_key.unwrap(),
                        path,
                    }
                }
                Some(path) => {
                    close_to_buffer(ctx);
                    Effect::OverlayAccept(crate::overlay::OverlayKind::Project, path)
                }
                None => {
                    close_overlay(ctx);
                    Effect::None
                }
            };
            Some(result)
        }
        _ => None,
    }
}

fn accept_value_overlay(ctx: &mut ActionCtx) -> Effect {
    let ov = ctx.overlay.as_ref().unwrap();
    if ov.kind == crate::overlay::OverlayKind::Command {
        // RUN the highlighted command. The corpus is `commands::visible()`
        // (the platform-filtered view — see `commands.rs`'s "PLATFORM-SCOPED
        // COMMANDS" section), so the selected corpus index maps back through
        // `commands::visible_action_of`, never a raw `COMMANDS[i]` index (which
        // would silently mis-map once some rows are hidden on web). Close the
        // palette FIRST so the caller's re-dispatch lands with the slot empty
        // (an overlay-opening command can then open into it); a no-match closes
        // silently.
        let eff = ov
            .selected_corpus_index()
            .map(|i| Effect::RunAction(crate::commands::visible_action_of(i)))
            .unwrap_or(Effect::None);
        dispose_after_accept(ctx);
        return eff;
    }
    if ov.kind == crate::overlay::OverlayKind::Theme {
        // COMMIT: the highlighted world is ALREADY active (live preview
        // applied it as the selection moved), so Enter just keeps it and
        // closes. Emit the committed name so the caller can re-tint its
        // GPU pipelines / window title to match.
        let eff = match ov.selected_value() {
            Some(v) => Effect::OverlayAccept(ov.kind, v.to_string()),
            None => Effect::None,
        };
        dispose_after_accept(ctx);
        return eff;
    }
    if ov.kind == crate::overlay::OverlayKind::Caret {
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

fn accept_overlay(ctx: &mut ActionCtx) -> Effect {
    {
        let ov = ctx.overlay.as_ref().unwrap();
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
            return dispatch_settings_row(ctx, row, crate::overlay::OverlayKind::Command, true);
        }
    }
    if let Some(effect) = accept_path_overlay(ctx) {
        return effect;
    }
    accept_value_overlay(ctx)
}

/// POP the summoned overlay — if it carries a `return_to` BREADCRUMB (opened as a
/// sub-picker from Settings, or run from the command palette), RE-SUMMON that parent
/// instead of closing to the buffer. The ONE owner of the breadcrumb POP, so every
/// Esc/cancel path and every VALUE-PICKING accept honors the breadcrumb identically.
/// SINGLE-LEVEL: the re-summoned parent (built fresh via `make_overlay`, so its value
/// cells reflect the change the sub-picker just committed) carries no breadcrumb of
/// its own, so there is no N-deep stack and no A→B→A loop. A `None` breadcrumb (every
/// normal top-level summon) closes to the buffer exactly as `*ctx.overlay = None`
/// always did.
pub(super) fn close_overlay(ctx: &mut ActionCtx) {
    let back = ctx.overlay.as_ref().and_then(|o| o.return_to);
    *ctx.overlay = match back {
        Some(kind) => (ctx.make_overlay)(kind),
        None => None,
    };
}

/// CLOSE the whole overlay stack to the buffer, IGNORING any `return_to` breadcrumb —
/// the disposition of a NAVIGATING accept (open a file, jump to a heading, switch the
/// project, restore a version, move a note, run a command). You asked to go somewhere,
/// so you land there, never back in the overlay that summoned this one. The
/// counterpart to [`close_overlay`] (which pops); the two are the pop-vs-close-all
/// pair the breadcrumb rule turns on.
pub(super) fn close_to_buffer(ctx: &mut ActionCtx) {
    *ctx.overlay = None;
}

/// Dispose of the overlay after an ACCEPT, per the highlighted kind's declared
/// [`crate::overlay::AcceptDisposition`] — the ONE owner routing every ordinary
/// accept through the single pop-vs-close-all classification. `Navigate` closes the
/// whole stack ([`close_to_buffer`]); `ValuePick` pops to the summoning overlay
/// ([`close_overlay`]) ONLY when that overlay
/// [retains its value-pick child](crate::overlay::OverlayKind::retains_value_pick_child)
/// (Settings), else closes to the buffer (a palette-launched or direct value-pick is
/// complete on commit); `StayOpen` leaves it untouched (the caller keeps the picker
/// up). A no-op with no overlay. (The `Project` navigator's Settings-PATH override —
/// pop back to Settings rather than close-all — is handled at that one accept seam,
/// not here, since it depends on `setting_path_key`, not the kind.)
pub(super) fn dispose_after_accept(ctx: &mut ActionCtx) {
    let Some(kind) = ctx.overlay.as_ref().map(|o| o.kind) else {
        return;
    };
    match kind.accept_disposition() {
        crate::overlay::AcceptDisposition::Navigate => close_to_buffer(ctx),
        // A VALUE-PICK accept POPS back to the summoning overlay ONLY when that
        // overlay wants its value-pick child re-summoned on commit — true just for
        // SETTINGS (keep configuring). A value-pick launched from the COMMAND palette
        // (a one-shot launcher) or summoned DIRECTLY (no breadcrumb) COMPLETES the
        // action, so it lands in the buffer rather than re-opening the launcher (which
        // re-appears on its Recent lens — the reported "Switch theme → recent files
        // menu" bug). Gated on the stored `return_to` VALUE, never enum position, so a
        // retired sibling variant can never re-aim this. (Esc still pops back
        // universally via `close_overlay`; only ACCEPT differs.)
        crate::overlay::AcceptDisposition::ValuePick => {
            let pop_back = ctx
                .overlay
                .as_ref()
                .and_then(|o| o.return_to)
                .is_some_and(|parent| parent.retains_value_pick_child());
            if pop_back {
                close_overlay(ctx);
            } else {
                close_to_buffer(ctx);
            }
        }
        crate::overlay::AcceptDisposition::StayOpen => {}
    }
}

/// Stamp a `return_to` BREADCRUMB onto an overlay that a palette/menu re-dispatch
/// just opened. The command palette's Enter CLOSES the palette then returns
/// [`Effect::RunAction`]; the caller (live `App::apply` / headless `replay_keys`)
/// re-dispatches that action, which opens any sub-overlay into the now-empty slot —
/// at which point THIS stamps `parent` (always `Command`) onto it so a later pop
/// returns to the palette. Only stamps when an overlay actually opened AND it carries
/// no breadcrumb of its own yet (a Settings sub-picker sets its own `return_to =
/// Settings` in place and must not be overwritten); a terminal command (no overlay)
/// or a `None` parent is a calm no-op. Shared by both re-dispatch seams so they can't
/// drift.
pub(crate) fn stamp_return_to(
    overlay: &mut Option<OverlayState>,
    parent: Option<crate::overlay::OverlayKind>,
) {
    if let (Some(parent), Some(ov)) = (parent, overlay.as_mut())
        && ov.return_to.is_none()
    {
        ov.return_to = Some(parent);
    }
}

fn range_ctx_value(id: crate::settings::SettingId, ctx: &ActionCtx) -> Option<f32> {
    Some(match id {
        crate::settings::SettingId::Zoom => *ctx.zoom,
        crate::settings::SettingId::ScrollSensitivity => crate::settings::scroll_sensitivity(),
        _ => return None,
    })
}

/// The write half of [`range_ctx_value`]. The value written is ALWAYS one the spec
/// produced (stepped/quantized), never a raw pointer/keyboard number.
fn range_ctx_set(id: crate::settings::SettingId, ctx: &mut ActionCtx, v: f32) {
    if id == crate::settings::SettingId::Zoom {
        *ctx.zoom = v
    } else if id == crate::settings::SettingId::ScrollSensitivity {
        crate::settings::set_scroll_sensitivity(v);
    }
}

fn range_step(ctx: &mut ActionCtx, steps: i32) -> Option<Effect> {
    let cell = ctx.overlay.as_ref()?.selected_range()?;
    let spec = crate::settings::range_spec(cell.id)?;
    let key = crate::settings::value_key(cell.id)?;
    let cur = range_ctx_value(cell.id, ctx)?;
    let next = spec.stepped(cur, steps);
    range_ctx_set(cell.id, ctx, next);
    ctx.overlay
        .as_mut()?
        .set_selected_range(spec.step_of(next), spec.format(next));
    Some(Effect::SettingRangeStep {
        key: key.to_string(),
    })
}

fn settings_accept(ctx: &mut ActionCtx) -> Effect {
    let Some(ci) = ctx.overlay.as_ref().unwrap().selected_corpus_index() else {
        close_overlay(ctx);
        return Effect::None;
    };
    let row = *crate::settings::visible_rows()[ci];
    dispatch_settings_row(ctx, row, crate::overlay::OverlayKind::Settings, false)
}

/// THE UNION ROUND: the SHARED settings-row dispatcher — the ONE owner both
/// [`settings_accept`] (Enter inside the Settings menu itself) AND the Command
/// palette's own settings-row accept (see the `OverlayKind::Command` arm below) call,
/// so the two can never drift (dispatch parity BY CONSTRUCTION, never a second copy).
/// `breadcrumb` is the overlay a Picker/Submenu/Path row's sub-picker pops back to on
/// Esc (`Settings` from the Settings menu itself; `Command` from the palette, so
/// canceling a theme-pick reached via the palette returns to the palette, mirroring
/// how running "Switch theme…" from the palette itself behaves via `stamp_return_to`).
/// `close_on_toggle` additionally CLOSES the overlay outright after a Toggle/Action —
/// `false` for the Settings menu (a persistent surface you keep configuring), `true`
/// for the palette (its own "activation closes it" convention, matching how running an
/// ordinary command row closes it).
fn dispatch_settings_row(
    ctx: &mut ActionCtx,
    row: crate::settings::SettingRow,
    breadcrumb: crate::overlay::OverlayKind,
    close_on_toggle: bool,
) -> Effect {
    match row.kind {
        crate::settings::SettingKind::Toggle => {
            let key = crate::settings::toggle_key(row.id).expect(
                "Toggle row always resolves its config key — settings law \
                 every_toggle_has_a_config_key_and_nothing_else_does",
            );
            if close_on_toggle {
                *ctx.overlay = None;
            }
            Effect::SettingToggle {
                key: key.to_string(),
            }
        }
        crate::settings::SettingKind::Picker | crate::settings::SettingKind::Submenu => {
            let target = crate::settings::sub_overlay(row.id).expect(
                "Picker/Submenu row always resolves its sub-overlay — settings law \
                 pickers_and_submenus_open_a_sub_overlay_and_nothing_else_does",
            );
            if let Some(mut next) = (ctx.make_overlay)(target) {
                next.return_to = Some(breadcrumb);
                *ctx.overlay = Some(next);
            }
            Effect::None
        }
        crate::settings::SettingKind::Action => {
            *ctx.overlay = None;
            match row.id {
                crate::settings::SettingId::ReportProblem => Effect::ReportProblem,
                crate::settings::SettingId::EditConfigAsText => Effect::OpenSettings,
                _ => Effect::None,
            }
        }
        crate::settings::SettingKind::Value | crate::settings::SettingKind::Range => {
            let key = crate::settings::value_key(row.id).expect(
                "Value/Range row always resolves its config key — settings law \
                 value_and_path_keys_track_their_kinds",
            );
            ctx.overlay
                .as_mut()
                .unwrap()
                .start_value_edit(key.to_string(), row.name.to_string());
            Effect::None
        }
        crate::settings::SettingKind::Path => {
            let key = crate::settings::path_key(row.id).expect(
                "Path row always resolves its config key — settings law \
                 value_and_path_keys_track_their_kinds",
            );
            if let Some(mut nav) = (ctx.browse_to)(crate::overlay::OverlayKind::Project, None) {
                nav.return_to = Some(breadcrumb);
                nav.setting_path_key = Some(key.to_string());
                *ctx.overlay = Some(nav);
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

/// A folder navigator's Settings BREADCRUMB (`return_to` + `setting_path_key`),
/// SNAPSHOTTED off the current level so it can be re-applied to a freshly-rebuilt
/// one AFTER the previous overlay's borrow has ended (the borrow checker forbids
/// reading `prev` while writing `*ctx.overlay`). A navigator opened FROM a Settings
/// PATH row must keep writing THAT config key (and return to Settings) even as you
/// descend / ascend to find the folder — a rebuilt level starts with both fields
/// `None`, so without carrying them a descend/ascend would silently drop the
/// breadcrumb. A plain navigator (both already `None`) carries nothing — a no-op.
/// The ONE owner of this carry-forward: applied at the Project descend (Enter) seam
/// and the shared ascend (Backspace) seam — the only rebuilds a Settings-opened
/// navigator can reach (Browse / MoveDest are never opened from a Settings row).
struct Breadcrumb {
    return_to: Option<crate::overlay::OverlayKind>,
    setting_path_key: Option<String>,
}

impl Breadcrumb {
    fn of(ov: &OverlayState) -> Self {
        Self {
            return_to: ov.return_to,
            setting_path_key: ov.setting_path_key.clone(),
        }
    }
    fn apply(self, next: &mut OverlayState) {
        next.return_to = self.return_to;
        next.setting_path_key = self.setting_path_key;
    }
}

pub(super) fn descend_target(ov: &OverlayState, name: &str) -> String {
    match ov.kind {
        crate::overlay::OverlayKind::Project => {
            std::path::Path::new(ov.browse_dir.as_deref().unwrap_or(""))
                .join(name)
                .to_string_lossy()
                .to_string()
        }
        _ => join_browse(ov.browse_dir.as_deref(), name),
    }
}

pub(super) fn ascend_target(ov: &OverlayState) -> Option<Option<String>> {
    match ov.kind {
        crate::overlay::OverlayKind::Project => {
            std::path::Path::new(ov.browse_dir.as_deref().unwrap_or("/"))
                .parent()
                .map(|p| Some(p.to_string_lossy().to_string()))
        }
        _ => browse_parent(ov.browse_dir.as_deref()),
    }
}

pub(super) fn move_dest_value(ov: &OverlayState) -> Option<String> {
    if let Some(name) = ov.selected_value()
        && ov.selected_is_dir()
    {
        return Some(join_browse(ov.browse_dir.as_deref(), name));
    }
    let q = ov.query.text().trim();
    if !q.is_empty() {
        return Some(join_browse(ov.browse_dir.as_deref(), q));
    }
    Some(ov.browse_dir.clone().unwrap_or_default())
}

pub(crate) fn preview_overlay(ov: &OverlayState) {
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
        _ => {}
    }
}

pub(crate) fn preview_move(ov: &mut OverlayState) {
    preview_overlay(ov);
    ov.reanchor();
}
