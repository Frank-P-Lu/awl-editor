use super::*;

pub(super) fn apply_deferred_action(ctx: &mut ActionCtx, action: &Action) -> Option<Effect> {
    let effect = match action {
        Action::LastBuffer => Effect::Buffer(BufferEffect::Previous { finished: false }),
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
        Action::OpenSettings => Effect::Buffer(BufferEffect::OpenSettings),
        Action::OpenCredits => Effect::Buffer(BufferEffect::OpenCredits),
        Action::OpenGuide => Effect::Buffer(BufferEffect::OpenGuide),
        Action::OpenSettingsMenu => {
            ctx.journey.enter((ctx.make_overlay)(OverlayKind::Settings));
            Effect::None
        }
        Action::FinishBuffer => Effect::Persistence(PersistenceEffect::Save(SaveKind::Finish)),
        Action::ResolveKeepMine => Effect::Persistence(PersistenceEffect::ResolveExternalChange(
            Resolution::KeepMine,
        )),
        Action::ResolveTakeTheirs => Effect::Persistence(PersistenceEffect::ResolveExternalChange(
            Resolution::TakeTheirs,
        )),
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
