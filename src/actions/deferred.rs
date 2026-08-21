use super::*;

pub(super) fn apply_deferred_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let effect = match action {
        Action::LastBuffer => Effect::Buffer(BufferEffect::Previous),
        Action::NewDocument => Effect::Buffer(BufferEffect::NewDocument),
        Action::KeepTutorial => Effect::RunAction(Action::OpenProject),
        Action::MoveFile => {
            ctx.journey
                .enter((ctx.browse_to)(OverlayKind::MoveDest, None));
            Effect::None
        }
        Action::OpenRenameNote => {
            if let Some(path) = ctx.buffer.path() {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                ctx.journey.enter(Some(OverlayState::new_rename(name)));
            }
            Effect::None
        }
        Action::DuplicateNote => Effect::DuplicateNote,
        Action::SaveCopy => {
            let card = (ctx.browse_to)(OverlayKind::ExportDest, None).map(|mut card| {
                card.save_copy = true;
                card
            });
            ctx.journey.enter(card);
            Effect::None
        }
        // Resolved HERE (the pure core), not by the live App: a path-less
        // scratch buffer signals `Effect::None`, the exact `FollowLink`
        // shape, so a headless replay of this action against a scratch
        // buffer records no handoff at all rather than an Intercepted one
        // live would never perform.
        Action::RevealInFileManager => ctx
            .buffer
            .path()
            .map(|p| Effect::RevealInFileManager(p.to_path_buf()))
            .unwrap_or(Effect::None),
        Action::OpenSettings => Effect::Buffer(BufferEffect::OpenSettings),
        Action::OpenSettingsMenu => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Settings));
            Effect::None
        }
        Action::FinishBuffer => Effect::Persistence(PersistenceEffect::Save(SaveKind::Finish)),
        Action::ReviewChange => Effect::Persistence(PersistenceEffect::ReviewExternalChange),
        Action::ResolveKeepMine => Effect::Persistence(PersistenceEffect::ResolveExternalChange(
            Resolution::KeepMine,
        )),
        Action::ResolveTakeTheirs => Effect::Persistence(PersistenceEffect::ResolveExternalChange(
            Resolution::TakeTheirs,
        )),
        Action::FollowLink => {
            let text = ctx.buffer.text();
            let byte = ctx.buffer.cursor_byte();
            if let Some(line) = crate::markdown::footnote_target_at(&text, byte) {
                Effect::JumpToLine(line)
            } else {
                crate::markdown::link_at(&text, byte)
                    .map(Effect::FollowLink)
                    .unwrap_or(Effect::None)
            }
        }
        Action::BeginPrefix | Action::Ignore => Effect::None,
        _ => return None,
    };
    Some(effect)
}
