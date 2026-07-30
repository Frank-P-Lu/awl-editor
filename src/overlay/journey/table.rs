//! THE TRANSITION TABLE — the whole of "what does Esc/Back/Enter do here",
//! written out cell by cell with no wildcard. See [`super`] for the lifecycle
//! this table drives.

/// WHAT THE CARD HOLDING ATTENTION IS.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Surface {
    /// A BRIEF contextual overlay — summoned, used, gone on pick (go-to,
    /// browse, spell, the command palette, a value picker).
    Contextual,
    /// A SUSTAINED summoned workspace with its PRIMARY list focused.
    Workspace,
    /// A SUSTAINED summoned workspace with its DETAIL stage focused — the pane
    /// beside the list on a wide stage, the pushed stage on a narrow one. The
    /// lifecycle deliberately does not distinguish those two: see the note on
    /// width in [`landing_of`].
    WorkspaceDetail,
}

/// WHAT IS PARKED BENEATH IT — the return policy, as a fact rather than a
/// breadcrumb the caller has to remember to carry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Beneath {
    /// Nothing: this surface was summoned straight from the editor.
    Editor,
    /// A sustained WORKSPACE you were configuring. Both a cancel and a commit
    /// come back to it, because it is a place, not an errand.
    Workspace,
    /// A brief LAUNCHER (the command palette). A cancel comes back — you never
    /// chose — but a commit COMPLETES the errand and lands in the editor,
    /// never back in the launcher (which would re-appear on its Recent lens:
    /// the reported "Switch theme → recent files menu" bug).
    Launcher,
}

/// THE LIFECYCLE STATE — the table's row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    /// Nothing summoned: the document owns the keyboard.
    Editor,
    /// A card holds attention, over whatever is parked beneath it.
    Summoned { surface: Surface, beneath: Beneath },
}

/// THE LIFECYCLE EVENT vocabulary — every way a summoned journey advances.
/// Deliberately closed and small: an in-place edit (a Value/Range row's inline
/// editor, a rename/link/keep prompt, a rebind capture) is CONTENT, changes no
/// rung, and so has no event here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    /// Esc / C-g / a Back gesture. The one event that REVERTS a live audition.
    Cancel,
    /// Enter on a row that takes you SOMEWHERE (open a file, jump to a heading,
    /// restore a version, switch project, run a command, open the config file).
    AcceptNavigate,
    /// Enter on a row that COMMITS a value (a world, a caret look, a
    /// dictionary, a CJK language, a date format, a folder for a config key).
    AcceptValue,
    /// Enter on a row that acts and leaves the surface up (trash an asset,
    /// rebind a key).
    AcceptStayOpen,
    /// Enter on a row that flips a setting IN PLACE.
    Toggle,
    /// A row opens a CHILD over this surface.
    Descend,
    /// Tab (or the compare-version chord): move focus between a workspace's
    /// primary list and its detail stage.
    ToggleDetail,
    /// Everything summoned goes at once — the menu bar took over, or the buffer
    /// swapped underneath.
    Dismiss,
}

/// WHERE a `(state, event)` pair lands. Closed, so a new rung cannot be
/// half-wired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing {
    /// The event does not apply in this state — nothing moves.
    Stay,
    /// Everything summoned goes; the editor gets the keyboard back with its
    /// buffer and view untouched.
    Editor,
    /// The workspace keeps attention with its PRIMARY list focused.
    Primary,
    /// The workspace keeps attention with its DETAIL stage focused.
    Detail,
    /// The parked parent resumes AT ITS EXACT POSITION.
    Resume,
    /// This surface is parked and a child takes the stage. Reachable only
    /// through `Journey::descend`, which carries the child.
    Suspend,
}

/// THE TABLE. Wildcard-free over `State × Event`: the editor's eight cells plus
/// `Surface × Beneath × Event` = 3 × 3 × 8, so 80 cells, every one written out.
/// A new surface, a new parent policy or a new event fails to compile here,
/// which is the whole reason the three enums are closed.
///
/// **On width.** A workspace's detail stage is presented BESIDE the list when
/// there is room and PUSHED OVER it when there is not — and Back means the same
/// thing in both: return to the primary list, same position. That invariance is
/// achieved by construction rather than by agreement, because width is not an
/// input to this function and no arm can branch on what it cannot see.
///
/// **On depth.** `Descend` suspends from every summoned state, and a suspend
/// replaces whatever was parked. The stack is therefore exactly one deep by
/// construction: this is shared workspace machinery, not a route stack.
pub fn landing_of(state: State, event: Event) -> Landing {
    use Beneath as B;
    use Event as E;
    use Landing as L;
    use Surface as F;
    let State::Summoned { surface, beneath } = state else {
        // NOTHING SUMMONED. Esc in the document clears a selection, Enter
        // inserts a newline — the journey is not involved in either. Only a
        // dismissal applies, and it is idempotent.
        return match event {
            E::Cancel => L::Stay,
            E::AcceptNavigate => L::Stay,
            E::AcceptValue => L::Stay,
            E::AcceptStayOpen => L::Stay,
            E::Toggle => L::Stay,
            E::Descend => L::Stay,
            E::ToggleDetail => L::Stay,
            E::Dismiss => L::Editor,
        };
    };
    match (surface, beneath, event) {
        // ── A BRIEF OVERLAY summoned from the editor: an errand. Anything
        //    that completes it drops to the document.
        (F::Contextual, B::Editor, E::Cancel) => L::Editor,
        (F::Contextual, B::Editor, E::AcceptNavigate) => L::Editor,
        (F::Contextual, B::Editor, E::AcceptValue) => L::Editor,
        (F::Contextual, B::Editor, E::AcceptStayOpen) => L::Stay,
        (F::Contextual, B::Editor, E::Toggle) => L::Editor,
        (F::Contextual, B::Editor, E::Descend) => L::Suspend,
        (F::Contextual, B::Editor, E::ToggleDetail) => L::Stay,
        (F::Contextual, B::Editor, E::Dismiss) => L::Editor,
        // ── A CHILD AUDITION over a parked WORKSPACE: you were configuring,
        //    so both outcomes return you to it.
        (F::Contextual, B::Workspace, E::Cancel) => L::Resume,
        (F::Contextual, B::Workspace, E::AcceptNavigate) => L::Editor,
        (F::Contextual, B::Workspace, E::AcceptValue) => L::Resume,
        (F::Contextual, B::Workspace, E::AcceptStayOpen) => L::Stay,
        (F::Contextual, B::Workspace, E::Toggle) => L::Resume,
        (F::Contextual, B::Workspace, E::Descend) => L::Suspend,
        (F::Contextual, B::Workspace, E::ToggleDetail) => L::Stay,
        (F::Contextual, B::Workspace, E::Dismiss) => L::Editor,
        // ── A CHILD AUDITION over a parked LAUNCHER: cancel returns, commit
        //    completes.
        (F::Contextual, B::Launcher, E::Cancel) => L::Resume,
        (F::Contextual, B::Launcher, E::AcceptNavigate) => L::Editor,
        (F::Contextual, B::Launcher, E::AcceptValue) => L::Editor,
        (F::Contextual, B::Launcher, E::AcceptStayOpen) => L::Stay,
        (F::Contextual, B::Launcher, E::Toggle) => L::Editor,
        (F::Contextual, B::Launcher, E::Descend) => L::Suspend,
        (F::Contextual, B::Launcher, E::ToggleDetail) => L::Stay,
        (F::Contextual, B::Launcher, E::Dismiss) => L::Editor,
        // ── A SUSTAINED WORKSPACE summoned from the editor: a PLACE. You keep
        //    configuring; only Esc or going somewhere leaves.
        (F::Workspace, B::Editor, E::Cancel) => L::Editor,
        (F::Workspace, B::Editor, E::AcceptNavigate) => L::Editor,
        (F::Workspace, B::Editor, E::AcceptValue) => L::Primary,
        (F::Workspace, B::Editor, E::AcceptStayOpen) => L::Stay,
        (F::Workspace, B::Editor, E::Toggle) => L::Stay,
        (F::Workspace, B::Editor, E::Descend) => L::Suspend,
        (F::Workspace, B::Editor, E::ToggleDetail) => L::Detail,
        (F::Workspace, B::Editor, E::Dismiss) => L::Editor,
        // ── A WORKSPACE over a parked WORKSPACE (no journey reaches this
        //    today): the place you are in wins for in-place work, the place
        //    beneath wins for anything that ends this one.
        (F::Workspace, B::Workspace, E::Cancel) => L::Resume,
        (F::Workspace, B::Workspace, E::AcceptNavigate) => L::Editor,
        (F::Workspace, B::Workspace, E::AcceptValue) => L::Resume,
        (F::Workspace, B::Workspace, E::AcceptStayOpen) => L::Stay,
        (F::Workspace, B::Workspace, E::Toggle) => L::Stay,
        (F::Workspace, B::Workspace, E::Descend) => L::Suspend,
        (F::Workspace, B::Workspace, E::ToggleDetail) => L::Detail,
        (F::Workspace, B::Workspace, E::Dismiss) => L::Editor,
        // ── A WORKSPACE launched from the palette: Esc returns to the palette,
        //    but a row you flip in place keeps you configuring.
        (F::Workspace, B::Launcher, E::Cancel) => L::Resume,
        (F::Workspace, B::Launcher, E::AcceptNavigate) => L::Editor,
        (F::Workspace, B::Launcher, E::AcceptValue) => L::Editor,
        (F::Workspace, B::Launcher, E::AcceptStayOpen) => L::Stay,
        (F::Workspace, B::Launcher, E::Toggle) => L::Stay,
        (F::Workspace, B::Launcher, E::Descend) => L::Suspend,
        (F::Workspace, B::Launcher, E::ToggleDetail) => L::Detail,
        (F::Workspace, B::Launcher, E::Dismiss) => L::Editor,
        // ── THE DETAIL STAGE. Esc is a BACK, never a close, at every parent
        //    policy — the arm that used to be an exceptional `Esc` branch
        //    inside `history_intercept`, and the one item 114's narrow stage
        //    depends on.
        (F::WorkspaceDetail, B::Editor, E::Cancel) => L::Primary,
        (F::WorkspaceDetail, B::Editor, E::AcceptNavigate) => L::Editor,
        (F::WorkspaceDetail, B::Editor, E::AcceptValue) => L::Primary,
        (F::WorkspaceDetail, B::Editor, E::AcceptStayOpen) => L::Stay,
        (F::WorkspaceDetail, B::Editor, E::Toggle) => L::Stay,
        (F::WorkspaceDetail, B::Editor, E::Descend) => L::Suspend,
        (F::WorkspaceDetail, B::Editor, E::ToggleDetail) => L::Primary,
        (F::WorkspaceDetail, B::Editor, E::Dismiss) => L::Editor,
        (F::WorkspaceDetail, B::Workspace, E::Cancel) => L::Primary,
        (F::WorkspaceDetail, B::Workspace, E::AcceptNavigate) => L::Editor,
        (F::WorkspaceDetail, B::Workspace, E::AcceptValue) => L::Resume,
        (F::WorkspaceDetail, B::Workspace, E::AcceptStayOpen) => L::Stay,
        (F::WorkspaceDetail, B::Workspace, E::Toggle) => L::Stay,
        (F::WorkspaceDetail, B::Workspace, E::Descend) => L::Suspend,
        (F::WorkspaceDetail, B::Workspace, E::ToggleDetail) => L::Primary,
        (F::WorkspaceDetail, B::Workspace, E::Dismiss) => L::Editor,
        (F::WorkspaceDetail, B::Launcher, E::Cancel) => L::Primary,
        (F::WorkspaceDetail, B::Launcher, E::AcceptNavigate) => L::Editor,
        (F::WorkspaceDetail, B::Launcher, E::AcceptValue) => L::Editor,
        (F::WorkspaceDetail, B::Launcher, E::AcceptStayOpen) => L::Stay,
        (F::WorkspaceDetail, B::Launcher, E::Toggle) => L::Stay,
        (F::WorkspaceDetail, B::Launcher, E::Descend) => L::Suspend,
        (F::WorkspaceDetail, B::Launcher, E::ToggleDetail) => L::Primary,
        (F::WorkspaceDetail, B::Launcher, E::Dismiss) => L::Editor,
    }
}

/// The full rosters, for the law sweep. Kept beside the table so a new member
/// is added here in the same edit that makes the table stop compiling.
#[cfg(test)]
impl Surface {
    pub const ALL: &'static [Surface] = &[
        Surface::Contextual,
        Surface::Workspace,
        Surface::WorkspaceDetail,
    ];
}

#[cfg(test)]
impl Beneath {
    pub const ALL: &'static [Beneath] = &[Beneath::Editor, Beneath::Workspace, Beneath::Launcher];
}

#[cfg(test)]
impl Event {
    pub const ALL: &'static [Event] = &[
        Event::Cancel,
        Event::AcceptNavigate,
        Event::AcceptValue,
        Event::AcceptStayOpen,
        Event::Toggle,
        Event::Descend,
        Event::ToggleDetail,
        Event::Dismiss,
    ];
}

#[cfg(test)]
impl State {
    /// Every state: the editor, plus every `Surface × Beneath` pair.
    pub fn all() -> Vec<State> {
        let mut all = vec![State::Editor];
        for &surface in Surface::ALL {
            for &beneath in Beneath::ALL {
                all.push(State::Summoned { surface, beneath });
            }
        }
        all
    }
}
