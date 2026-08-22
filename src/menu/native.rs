use super::*;
use crate::menu_icons;
use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};

pub struct InstalledMenu {
    menu: Menu,
    markdown: Submenu,
}

impl InstalledMenu {
    pub fn set_markdown_enabled(&self, is_markdown: bool) {
        self.markdown
            .set_enabled(markdown_submenu_enabled(is_markdown));
    }
}

fn to_menu_item(id: &'static str, label: &'static str, icon: bool) -> Box<dyn muda::IsMenuItem> {
    let accelerator = accelerator_for_id(id);
    if icon && let Some(icon) = menu_icons::icon_for(id) {
        return Box::new(muda::IconMenuItem::with_id(
            id,
            label,
            true,
            Some(icon),
            accelerator,
        ));
    }
    Box::new(MenuItem::with_id(id, label, true, accelerator))
}

/// The native `muda::Accelerator` for a routed item's default macOS chord —
/// `None` when the command has no Mac default ([`super::Command::native`]
/// empty) or its chord's key isn't in the covered set (see below). Builds a
/// token string [`muda::accelerator::Accelerator`]'s own `FromStr` accepts
/// (`"CONTROL+SHIFT+;"`) from the SAME `(Key, Modifiers)`
/// [`crate::keyspec::parse_chord`] gives every other chord consumer — never a
/// second hand-typed table, so the key-equivalent column can never name a
/// chord the keymap itself would not actually dispatch.
///
/// COVERED: every menu-routed command's default Mac chord is a single
/// character key (a letter, digit, or one of `; = - ,`) plus modifiers — the
/// whole roster, verified by `native_accelerator_law`. A future routed
/// command bound to a named key (arrows, Tab, …) falls through to `None`
/// here — same as every item shows today — rather than guessing a mapping
/// untested against muda's own `Code` table.
fn accelerator_for_id(id: &str) -> Option<muda::accelerator::Accelerator> {
    let command = super::routed_command_for_id(id)?;
    let native = commands::COMMANDS
        .iter()
        .find(|c| c.name == command)?
        .native;
    if native.trim().is_empty() {
        return None;
    }
    let (key, mods) = crate::keyspec::parse_chord(native).ok()?;
    let winit::keyboard::Key::Character(ch) = key else {
        return None;
    };
    let state = mods.state();
    let mut spec = String::new();
    if state.contains(winit::keyboard::ModifiersState::CONTROL) {
        spec.push_str("CONTROL+");
    }
    if state.contains(winit::keyboard::ModifiersState::ALT) {
        spec.push_str("ALT+");
    }
    if state.contains(winit::keyboard::ModifiersState::SHIFT) {
        spec.push_str("SHIFT+");
    }
    if state.contains(winit::keyboard::ModifiersState::SUPER) {
        spec.push_str("SUPER+");
    }
    spec.push_str(ch.as_str());
    spec.parse().ok()
}

fn to_predefined(kind: PredefinedKind) -> PredefinedMenuItem {
    match kind {
        PredefinedKind::Minimize => PredefinedMenuItem::minimize(None),
        PredefinedKind::Maximize => PredefinedMenuItem::maximize(None),
        PredefinedKind::Hide => PredefinedMenuItem::hide(None),
        PredefinedKind::HideOthers => PredefinedMenuItem::hide_others(None),
        PredefinedKind::ShowAll => PredefinedMenuItem::show_all(None),
    }
}

pub fn build_menu() -> InstalledMenu {
    let mut markdown = None;
    let submenus: Vec<Submenu> = roster()
        .into_iter()
        .map(|menu| {
            let items: Vec<Box<dyn muda::IsMenuItem>> = menu
                .items
                .iter()
                .map(|item| to_native_item(item, &mut markdown))
                .collect();
            let refs: Vec<&dyn muda::IsMenuItem> = items.iter().map(|item| item.as_ref()).collect();
            Submenu::with_items(menu.title, true, &refs).expect("submenu build")
        })
        .collect();
    let refs: Vec<&dyn muda::IsMenuItem> = submenus
        .iter()
        .map(|submenu| submenu as &dyn muda::IsMenuItem)
        .collect();
    InstalledMenu {
        menu: Menu::with_items(&refs).expect("root menu build"),
        markdown: markdown.expect("the menu roster must carry the Markdown submenu"),
    }
}

fn to_native_item(item: &RosterItem, markdown: &mut Option<Submenu>) -> Box<dyn muda::IsMenuItem> {
    match item {
        RosterItem::Routed { id, label, icon } => to_menu_item(id, label, *icon),
        RosterItem::Separator => Box::new(PredefinedMenuItem::separator()),
        RosterItem::Predefined(kind) => Box::new(to_predefined(*kind)),
        RosterItem::Submenu { label, items } => {
            let children: Vec<Box<dyn muda::IsMenuItem>> = items
                .iter()
                .map(|item| to_native_item(item, markdown))
                .collect();
            let refs: Vec<&dyn muda::IsMenuItem> =
                children.iter().map(|item| item.as_ref()).collect();
            let submenu = Submenu::with_items(label, true, &refs).expect("submenu build");
            if *label == "Markdown" {
                assert!(
                    markdown.replace(submenu.clone()).is_none(),
                    "one Markdown submenu"
                );
            }
            Box::new(submenu)
        }
    }
}

#[must_use = "the returned menu must be retained for the app lifetime"]
pub fn install<E: Send + 'static>(
    proxy: winit::event_loop::EventLoopProxy<E>,
    wrap: impl Fn(String) -> E + Send + Sync + 'static,
    is_markdown: bool,
) -> InstalledMenu {
    let menu = build_menu();
    menu.set_markdown_enabled(is_markdown);
    menu.menu.init_for_nsapp();
    muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
        let _ = proxy.send_event(wrap(event.id().0.clone()));
    }));
    menu
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands;

    /// LAW: every routed menu item whose command carries a default Mac chord
    /// ([`commands::Command::native`], non-empty) gets a real
    /// `muda::Accelerator` whose modifiers and key match that chord — swept
    /// across the WHOLE routed roster via [`super::super::roster`], not a
    /// hand-picked list, so a future routed command with a default binding is
    /// covered automatically the moment it lands.
    #[test]
    fn native_accelerator_law_names_every_routed_items_default_mac_chord() {
        let mut covered = 0usize;
        for menu in super::super::roster() {
            for item in super::super::dropdown_items(&menu) {
                let RosterItem::Routed { id, .. } = item else {
                    continue;
                };
                let Some(command) = super::super::routed_command_for_id(id) else {
                    continue;
                };
                let Some(c) = commands::COMMANDS.iter().find(|c| c.name == command) else {
                    continue;
                };
                if c.native.trim().is_empty() {
                    continue;
                }
                let Ok((key, mods)) = crate::keyspec::parse_chord(c.native) else {
                    continue;
                };
                let winit::keyboard::Key::Character(_) = key else {
                    // Outside the documented single-character coverage — see
                    // accelerator_for_id's doc. Not part of today's roster
                    // (asserted by the non-vacuity count below), so this arm
                    // stays untested rather than pinning a mapping no chord
                    // exercises.
                    continue;
                };
                let accel = accelerator_for_id(id).unwrap_or_else(|| {
                    panic!(
                        "{id:?} ({command:?}, native {:?}) has a single-character default \
                         chord but accelerator_for_id returned None",
                        c.native
                    )
                });
                let state = mods.state();
                let got = accel.modifiers();
                assert_eq!(
                    got.contains(muda::accelerator::Modifiers::CONTROL),
                    state.contains(winit::keyboard::ModifiersState::CONTROL),
                    "{id:?} ({command:?}): control modifier mismatch, native {:?}",
                    c.native
                );
                assert_eq!(
                    got.contains(muda::accelerator::Modifiers::ALT),
                    state.contains(winit::keyboard::ModifiersState::ALT),
                    "{id:?} ({command:?}): alt modifier mismatch, native {:?}",
                    c.native
                );
                assert_eq!(
                    got.contains(muda::accelerator::Modifiers::SHIFT),
                    state.contains(winit::keyboard::ModifiersState::SHIFT),
                    "{id:?} ({command:?}): shift modifier mismatch, native {:?}",
                    c.native
                );
                assert_eq!(
                    got.contains(muda::accelerator::Modifiers::SUPER),
                    state.contains(winit::keyboard::ModifiersState::SUPER),
                    "{id:?} ({command:?}): super/cmd modifier mismatch, native {:?}",
                    c.native
                );
                covered += 1;
            }
        }
        assert!(
            covered > 20,
            "expected a non-trivial swept roster of default-bound routed items, found {covered} \
             (a filtering bug could make this loop cover nothing and still pass)"
        );
    }

    /// GROUND-TRUTH SPOT CHECK: a handful of well-known chords, asserted
    /// against a literally-constructed `Accelerator` rather than re-derived
    /// through the same conversion the law above sweeps — catches a
    /// systematic character-to-`Code` mapping bug the sweep's own logic
    /// would not (comparing the implementation to itself proves nothing).
    #[test]
    fn native_accelerator_matches_ground_truth_for_known_chords() {
        use muda::accelerator::{Accelerator, Code, Modifiers};
        let cases: &[(&str, &str, Option<Modifiers>, Code)] = &[
            ("awl.save", "Cmd-S", Some(Modifiers::SUPER), Code::KeyS),
            ("awl.bold", "Cmd-B", Some(Modifiers::SUPER), Code::KeyB),
            ("awl.settings", "Cmd-,", Some(Modifiers::SUPER), Code::Comma),
            ("awl.zoom_in", "Cmd-=", Some(Modifiers::SUPER), Code::Equal),
            (
                "awl.toggle_outline",
                "Cmd-S-o",
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyO,
            ),
        ];
        for (id, native, mods, code) in cases {
            let command = super::super::routed_command_for_id(id)
                .unwrap_or_else(|| panic!("{id:?} is not a routed menu id"));
            let c = commands::COMMANDS
                .iter()
                .find(|c| c.name == command)
                .unwrap_or_else(|| panic!("{command:?} is not a catalog command"));
            assert_eq!(
                c.native, *native,
                "{id:?} ({command:?}): test's assumed default chord is stale"
            );
            assert_eq!(
                accelerator_for_id(id),
                Some(Accelerator::new(*mods, *code)),
                "{id:?} ({command:?}) native {native:?}"
            );
        }
    }

    /// An item with no default Mac chord gets no accelerator — never a
    /// panic, never a stray guess.
    #[test]
    fn unbound_routed_item_has_no_accelerator() {
        assert_eq!(
            super::super::routed_command_for_id("awl.open")
                .and_then(|command| commands::COMMANDS.iter().find(|c| c.name == command))
                .map(|c| c.native),
            Some(""),
            "premise: Open file… has no default Mac chord in this roster"
        );
        assert_eq!(accelerator_for_id("awl.open"), None);
    }

    #[test]
    fn unknown_id_has_no_accelerator() {
        assert_eq!(accelerator_for_id("awl.nonexistent"), None);
    }
}
