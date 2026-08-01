pub(super) struct CoreRun {
    pub(super) transition: crate::actions::Transition,
    pub(super) theme_overlay_before: bool,
    pub(super) theme_before: crate::theme::Theme,
    pub(super) history_overlay_before: bool,
}

pub(super) struct CoreBefore {
    pub(super) theme_overlay_before: bool,
    pub(super) theme_before: crate::theme::Theme,
    pub(super) history_overlay_before: bool,
}

impl CoreBefore {
    pub(super) fn of(overlay: Option<&crate::overlay::OverlayState>) -> Self {
        Self {
            theme_overlay_before: overlay
                .is_some_and(|state| state.kind == crate::overlay::OverlayKind::Theme),
            theme_before: crate::theme::active(),
            history_overlay_before: overlay
                .is_some_and(|state| state.kind == crate::overlay::OverlayKind::History),
        }
    }
}
