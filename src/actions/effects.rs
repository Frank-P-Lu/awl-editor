use crate::keymap::Action;

/// Persistence work described by the editor transition and owned by an
/// interpreter. No variant is allowed to write from `apply_transition`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveKind {
    Manual,
    Finish,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferenceEffect {
    CaretMode,
    PageMode,
    PageWidth,
    PageReset,
    Outline,
    MenuBar,
    Typewriter,
    Spellcheck,
    WritingNits,
    WritingStreaks,
}

/// WHICH WAY an unresolved external change is being settled. Both arms destroy
/// nothing: one writes the buffer over the file, the other replaces the buffer
/// as a single undoable edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// Write the buffer over the file — after rechecking that the file is still
    /// what the user was shown.
    KeepMine,
    /// Replace the buffer with the file, as one undoable edit.
    TakeTheirs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceEffect {
    Save(SaveKind),
    Preference(PreferenceEffect),
    /// Settle the one unresolved external change. Distinct from `Save` because
    /// it is not one: it is the door out of a state in which saving is refused,
    /// and only one of its two arms writes anything at all.
    ResolveExternalChange(Resolution),
    /// SUMMON the conflict workspace over the one unresolved external change.
    /// A read, not a write — but it lives here beside its resolutions because
    /// what it can show is the App's own latched conflict, which is the same
    /// live-only fact, and a replay that "opened" it would show an empty card.
    ReviewExternalChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardEffect {
    /// Mirror the core-owned kill ring to the platform clipboard.
    WriteKillRing,
    /// Resolve an image paste externally, then feed either an image-reference
    /// or text-yank continuation back through the shared transition.
    PasteImage,
}

/// The exact core-owned text inserted for a pasted image reference. The image
/// lands on its own line, and the caret lands on a fresh line after it.
pub(crate) fn image_reference_text(at_line_start: bool, reference: &str) -> String {
    let lead = if at_line_start { "" } else { "\n" };
    format!("{lead}![]({reference})\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferEffect {
    /// Switch to the previous live buffer. `finished` preserves whether the
    /// request is the final leg of Finish, so headless truthfulness can name
    /// that unsupported composite without consulting the originating Action.
    Previous {
        finished: bool,
    },
    NewDocument,
    OpenSettings,
    OpenCredits,
    OpenGuide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonEffect {
    NotifyFinished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceEffect {
    /// Show the platform's About surface. The transition does not decide
    /// whether that is the macOS panel or awl's in-app card.
    ShowAbout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoticeEffect {
    Toast(String),
    Sticky(String),
    Clear,
}

/// WHICH KIND of notice is on screen — one owner, read by the live `App`'s frame
/// state, the render layer's treatment, the capture fold and the sidecar, so a
/// notice's kind cannot be spelled two ways.
///
/// The distinction is a LIFETIME, not a severity: a `Toast` clears itself on a
/// wall-clock deadline, a `Sticky` is held until its owner clears it. So a
/// lifetime can never explain an unseen sticky notice, and the two kinds have to
/// be probed separately whenever the question is "was this ever seen".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NoticeKind {
    /// A timed acknowledgement of something that already succeeded.
    Toast,
    /// Held until its owner clears it — the default, because a notice whose
    /// kind is unknown must not silently expire.
    #[default]
    Sticky,
}

impl NoticeKind {
    /// The sidecar / semantic spelling. One owner, so the JSON and any prose
    /// about it read the same word.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Toast => "toast",
            Self::Sticky => "sticky",
        }
    }
}

impl NoticeEffect {
    /// The kind this effect raises, or `None` for [`NoticeEffect::Clear`] (which
    /// raises nothing). The no-wildcard match is what makes a new arm a
    /// compile error here rather than a silently mis-kinded notice.
    pub fn kind(&self) -> Option<NoticeKind> {
        match self {
            Self::Toast(_) => Some(NoticeKind::Toast),
            Self::Sticky(_) => Some(NoticeKind::Sticky),
            Self::Clear => None,
        }
    }

    /// This effect's message, or `None` for [`NoticeEffect::Clear`].
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Toast(text) | Self::Sticky(text) => Some(text),
            Self::Clear => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderEffect {
    /// Rebuild the live view from the transitioned state. `follow` has the same
    /// meaning as `App::sync_view`: keep the caret visible when true.
    SyncView {
        follow: bool,
    },
    /// Re-measure the live page before the following sync.
    Reshape,
    /// Re-anchor and mark the already-applied zoom value for rendering.
    ZoomChanged,
    Redraw,
    EditStreak,
}

/// One closed effect vocabulary shared by the live and headless interpreters.
/// A transition can carry several effects because one action may request, for
/// example, save + daemon notification + buffer switch + repaint.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    None,
    Quit,
    Persistence(PersistenceEffect),
    Clipboard(ClipboardEffect),
    Buffer(BufferEffect),
    Daemon(DaemonEffect),
    Surface(SurfaceEffect),
    Notice(NoticeEffect),
    Render(RenderEffect),
    RunAction(Action),
    OverlayAccept(crate::overlay::OverlayKind, String),
    JumpToLine(usize),
    AddToDictionary(String),
    RebindCommit {
        slug: String,
        binding: String,
        confirmed: bool,
    },
    RebindReset {
        slug: String,
    },
    Recoil(crate::caret::RecoilDir),
    TypeImpact,
    DeleteSquash,
    Gulp,
    LineLand,
    KeepVersion {
        name: Option<String>,
    },
    FollowLink(String),
    ReportProblem,
    DownloadFile,
    Export(crate::export::Format),
    CheckForUpdates,
    CopyPulse,
    SettingToggle {
        key: String,
    },
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
    TrashAsset {
        rel: String,
    },
    RenameNoteCommit {
        new_name: String,
    },
    DuplicateNote,
    InsertDate,
}

/// The complete result of one shared editor transition.
#[must_use = "interpret every effect or explicitly select the semantic primary"]
#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    primary: Effect,
    effects: Vec<Effect>,
}

/// Shared depth-first ordering for effect interpreters. When an effect asks to
/// run another action, [`Self::descend`] puts that action ahead of the current
/// transition's remaining effects without discarding the remainder.
pub(crate) struct EffectWorklist {
    stack: Vec<EffectWork>,
}

enum EffectWork {
    Action(Action),
    Effects {
        owner: Action,
        effects: std::vec::IntoIter<Effect>,
    },
}

pub(crate) enum EffectWorkItem {
    Action(Action),
    Effect { owner: Action, effect: Effect },
}

/// Visit a live transition's complete effect stream in order. A nested
/// `App::apply` runs inside `visit`; returning from that callback cannot return
/// from this driver, so the outer transition's remainder is never dropped.
pub(crate) fn visit_transition_effects(transition: Transition, mut visit: impl FnMut(Effect)) {
    for effect in transition.into_effects() {
        visit(effect);
    }
}

impl EffectWorklist {
    pub(crate) fn root(action: Action) -> Self {
        Self {
            stack: vec![EffectWork::Action(action)],
        }
    }

    #[cfg(test)]
    pub(crate) fn from_transition(action: Action, transition: Transition) -> Self {
        let mut work = Self { stack: Vec::new() };
        work.expand(action, transition);
        work
    }

    pub(crate) fn expand(&mut self, owner: Action, transition: Transition) {
        self.stack.push(EffectWork::Effects {
            owner,
            effects: transition.into_effects().into_iter(),
        });
    }

    pub(crate) fn descend(&mut self, action: Action) {
        self.stack.push(EffectWork::Action(action));
    }

    pub(crate) fn next(&mut self) -> Option<EffectWorkItem> {
        loop {
            match self.stack.pop()? {
                EffectWork::Action(action) => return Some(EffectWorkItem::Action(action)),
                EffectWork::Effects { owner, mut effects } => {
                    let Some(effect) = effects.next() else {
                        continue;
                    };
                    self.stack.push(EffectWork::Effects {
                        owner: owner.clone(),
                        effects,
                    });
                    return Some(EffectWorkItem::Effect { owner, effect });
                }
            }
        }
    }
}

impl Transition {
    fn new(primary: Effect) -> Self {
        let mut effects = Vec::with_capacity(4);
        if primary != Effect::None {
            effects.push(primary.clone());
        }
        Self { primary, effects }
    }

    fn push(&mut self, effect: Effect) {
        if effect != Effect::None {
            self.effects.push(effect);
        }
    }

    #[cfg(test)]
    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn into_effects(self) -> Vec<Effect> {
        self.effects
    }

    /// The action's primary semantic outcome. Focused tests and the caret
    /// preview select this explicitly from the complete transition.
    pub fn primary(&self) -> Effect {
        self.primary.clone()
    }

    pub fn contains(&self, predicate: impl Fn(&Effect) -> bool) -> bool {
        self.effects.iter().any(predicate)
    }
}

pub(super) fn complete(primary: Effect, action: &Action) -> Transition {
    let mut transition = Transition::new(primary);
    if matches!(action, Action::FinishBuffer) {
        transition.push(Effect::Daemon(DaemonEffect::NotifyFinished));
        transition.push(Effect::Buffer(BufferEffect::Previous { finished: true }));
    }
    if matches!(
        action,
        Action::DeleteWordBackward
            | Action::KillLine
            | Action::CopyRegion
            | Action::KillRegion
            | Action::CopyLinkDestination
    ) {
        transition.push(Effect::Clipboard(ClipboardEffect::WriteKillRing));
    }
    if matches!(action, Action::DeleteWordBackward) {
        transition.push(Effect::Render(RenderEffect::EditStreak));
    }
    if matches!(action, Action::ToggleSpellcheck) {
        transition.push(Effect::Persistence(PersistenceEffect::Preference(
            PreferenceEffect::Spellcheck,
        )));
    }
    if matches!(action, Action::ToggleWritingNits) {
        transition.push(Effect::Persistence(PersistenceEffect::Preference(
            PreferenceEffect::WritingNits,
        )));
    }
    if matches!(
        action,
        Action::TogglePageMode | Action::PageWider | Action::PageNarrower | Action::PageReset
    ) {
        transition.push(Effect::Render(RenderEffect::Reshape));
    }
    if matches!(action, Action::ZoomIn | Action::ZoomOut | Action::ZoomReset) {
        transition.push(Effect::Render(RenderEffect::ZoomChanged));
    }
    if !matches!(
        action,
        Action::ZoomIn | Action::ZoomOut | Action::ZoomReset | Action::Yank
    ) {
        transition.push(Effect::Render(RenderEffect::SyncView {
            follow: !matches!(action, Action::ToggleWritingNits),
        }));
    }
    transition.push(Effect::Render(RenderEffect::Redraw));
    transition
}

#[cfg(test)]
mod ordering_tests {
    use super::*;

    fn transition(effects: Vec<Effect>) -> Transition {
        Transition {
            primary: effects.first().cloned().unwrap_or(Effect::None),
            effects,
        }
    }

    #[test]
    fn nested_actions_are_depth_first_and_never_drop_the_outer_remainder() {
        let outer = Action::OpenCommandPalette;
        let nested = Action::Save;
        let mut work = EffectWorklist::from_transition(
            outer.clone(),
            transition(vec![
                Effect::Notice(NoticeEffect::Toast("before".into())),
                Effect::RunAction(nested.clone()),
                Effect::Notice(NoticeEffect::Sticky("after".into())),
            ]),
        );
        let mut seen = Vec::new();
        while let Some(item) = work.next() {
            match item {
                EffectWorkItem::Action(action) => {
                    seen.push(format!("action:{action:?}"));
                    work.expand(
                        action,
                        transition(vec![Effect::Notice(NoticeEffect::Clear)]),
                    );
                }
                EffectWorkItem::Effect { effect, .. } => {
                    seen.push(format!("effect:{effect:?}"));
                    if let Effect::RunAction(action) = effect {
                        work.descend(action);
                    }
                }
            }
        }
        assert_eq!(
            seen,
            vec![
                "effect:Notice(Toast(\"before\"))",
                "effect:RunAction(Save)",
                "action:Save",
                "effect:Notice(Clear)",
                "effect:Notice(Sticky(\"after\"))",
            ],
            "the shared worklist must visit before → nested transition → after"
        );
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let live = std::fs::read_to_string(root.join("app/apply.rs")).unwrap();
        let headless = std::fs::read_to_string(root.join("main/run.rs")).unwrap();
        assert!(
            live.contains("visit_transition_effects(transition")
                && headless.contains("EffectWorklist::root"),
            "MUTATION TRAP: live and headless interpreters must use the shared depth-first drivers"
        );
    }

    #[test]
    fn live_driver_cannot_drop_the_outer_remainder_after_a_nested_action() {
        let nested = Action::Save;
        let seen = std::cell::RefCell::new(Vec::new());
        visit_transition_effects(
            transition(vec![
                Effect::Notice(NoticeEffect::Toast("before".into())),
                Effect::RunAction(nested),
                Effect::Notice(NoticeEffect::Sticky("after".into())),
            ]),
            |effect| {
                seen.borrow_mut().push(format!("outer:{effect:?}"));
                if matches!(effect, Effect::RunAction(Action::Save)) {
                    visit_transition_effects(
                        transition(vec![Effect::Notice(NoticeEffect::Clear)]),
                        |nested_effect| {
                            seen.borrow_mut().push(format!("nested:{nested_effect:?}"));
                        },
                    );
                }
            },
        );
        assert_eq!(
            seen.into_inner(),
            vec![
                "outer:Notice(Toast(\"before\"))",
                "outer:RunAction(Save)",
                "nested:Notice(Clear)",
                "outer:Notice(Sticky(\"after\"))",
            ],
            "MUTATION TRAP: live nested dispatch returned before the outer effect remainder"
        );
    }
}
