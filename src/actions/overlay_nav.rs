//! Modal navigation-overlay actions, shared by live input and `--keys` replay.

use super::*;
const OVERLAY_PAGE: isize = 12;
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
            ctx.journey.dismiss();
            return Some(
                target
                    .map(|name| Effect::KeepVersion { name })
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

/// The modal OVERLAY INTERCEPT. When the summoned navigation overlay is open, it OWNS
/// every key: printable chars extend the overlay query (never the rope), Up/Down (and
/// C-n/C-p, which resolve to NextLine/PreviousLine) move the selection, Enter accepts
/// the highlighted item, Esc/C-g cancels. Routing this through the shared core (rather
/// than only in `App`) is exactly what makes the overlay drivable under `--keys` — the
/// same mistake the isearch panel made (its query routing lives in `App`, so `--keys`
/// can't type into it) is deliberately avoided here. Returns the one [`Effect`] the key
/// signals back; `apply_transition` returns it directly (the overlay is modal, so the key
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
    if ctx.journey.card().unwrap().kind == crate::overlay::OverlayKind::Keybindings
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
            ctx.journey.card_mut().unwrap().push(*c);
            preview_move(ctx.journey.card_mut().unwrap());
            Effect::None
        }
        Action::DeleteBackward | Action::DeleteWordBackward => {
            let ov = ctx.journey.card().unwrap();
            let navigable = matches!(
                ov.kind,
                crate::overlay::OverlayKind::Browse
                    | crate::overlay::OverlayKind::MoveDest
                    | crate::overlay::OverlayKind::Project
            );
            if navigable && ov.query.is_empty() {
                if let Some(parent) = ascend_target(ov)
                    && let Some(next) = (ctx.browse_to)(ov.kind, parent)
                {
                    ctx.journey.relevel(next);
                }
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
        Action::Newline => accept_overlay(ctx),
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
    ctx.journey.cancel(ctx.make_overlay);
    if was_theme {
        Effect::OverlayAccept(
            crate::overlay::OverlayKind::Theme,
            crate::theme::active().name.to_string(),
        )
    } else {
        Effect::None
    }
}
fn history_intercept(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let ov = ctx.journey.card().unwrap();
    if ov.kind != crate::overlay::OverlayKind::History || ov.selected_history_id().is_none() {
        return None;
    }
    let page = ctx.scroll_page_lines.max(1);
    let focused = ov.detail_focus;
    match action {
        Action::PageScrollDown => {
            let ov = ctx.journey.card_mut().unwrap();
            ov.diff_scroll = ov.diff_scroll.saturating_add(page);
            Some(Effect::None)
        }
        Action::PageScrollUp => {
            let ov = ctx.journey.card_mut().unwrap();
            ov.diff_scroll = ov.diff_scroll.saturating_sub(page);
            Some(Effect::None)
        }
        Action::CompareVersion | Action::InsertTab => {
            ctx.journey.toggle_detail();
            Some(Effect::None)
        }
        _ if focused => match action {
            Action::NextLine => {
                let ov = ctx.journey.card_mut().unwrap();
                ov.diff_scroll = ov.diff_scroll.saturating_add(1);
                Some(Effect::None)
            }
            Action::PreviousLine => {
                let ov = ctx.journey.card_mut().unwrap();
                ov.diff_scroll = ov.diff_scroll.saturating_sub(1);
                Some(Effect::None)
            }
            // Esc and Enter both fall through to the shared owners: the table
            // already says a cancel on the detail stage lands on the primary
            // list, and an accept there restores the version.
            Action::Cancel => None,
            Action::Newline => None,
            _ => Some(Effect::None),
        },
        _ => None,
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
            } else if ov.kind == crate::overlay::OverlayKind::MoveDest {
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
            } else if ov.kind == crate::overlay::OverlayKind::MoveDest {
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
        crate::overlay::OverlayKind::MoveDest => {
            let effect = move_dest_value(ov)
                .map(|dest| Effect::OverlayAccept(crate::overlay::OverlayKind::MoveDest, dest))
                .unwrap_or(Effect::None);
            dispose_after_accept(ctx);
            Some(effect)
        }
        crate::overlay::OverlayKind::Project => {
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
            let path_key = match ctx.journey.bind() {
                Some(crate::overlay::Bind::Path { key }) => Some(key.clone()),
                Some(crate::overlay::Bind::Value) | None => None,
            };
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
                    ctx.journey.accept(
                        crate::overlay::AcceptDisposition::ValuePick,
                        ctx.make_overlay,
                    );
                    Effect::SettingPathPick { key, path }
                }
                (Some(path), None) => {
                    ctx.journey.navigate_away();
                    Effect::OverlayAccept(crate::overlay::OverlayKind::Project, path)
                }
                (None, _) => {
                    ctx.journey.cancel(ctx.make_overlay);
                    Effect::None
                }
            };
            Some(result)
        }
        _ => None,
    }
}

fn accept_value_overlay(ctx: &mut ActionCtx) -> Effect {
    let ov = ctx.journey.card().unwrap();
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
    ctx.journey
        .accept(kind.accept_disposition(), ctx.make_overlay);
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
        ctx.journey.cancel(ctx.make_overlay);
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
            ctx.journey.toggled(ctx.make_overlay);
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
