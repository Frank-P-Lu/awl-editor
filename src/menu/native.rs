use super::*;
use crate::menu_icons;
use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};

pub struct InstalledMenu {
    menu: Menu,
    markdown: Submenu,
    /// Every ROUTED item's live handle, keyed by its roster id — the seam
    /// [`Self::refresh_accelerators`] walks so a rebind can update each
    /// item's key equivalent in place rather than rebuilding the menu.
    /// `muda::MenuItemKind` rather than a concrete type because a routed id
    /// can build either a `MenuItem` or an `IconMenuItem` ([`to_menu_item`]),
    /// and both expose `set_accelerator` only on their concrete type.
    routed: Vec<(&'static str, muda::MenuItemKind)>,
}

impl InstalledMenu {
    pub fn set_markdown_enabled(&self, is_markdown: bool) {
        self.markdown
            .set_enabled(markdown_submenu_enabled(is_markdown));
    }

    /// Recompute and apply every routed item's native key equivalent from
    /// the CURRENT live keymap (`keys` = `Config::keys`) — the seam a rebind
    /// commit/reset and a keymap-flavor apply call so the AppKit menu never
    /// shows (or, worse, still intercepts via) a chord the live keymap no
    /// longer dispatches. Reads the exact same [`accelerator_for_id`] the
    /// initial [`build_menu`] used, so a fresh install and a post-rebind
    /// refresh can never compute a different answer for the same config.
    ///
    /// `set_accelerator` is a muda `Result`; the only failure mode is an
    /// accelerator muda itself refuses to encode, which cannot happen here —
    /// [`accelerator_for_id`] only ever hands back `None` or a value built
    /// from `Accelerator::new`/`FromStr`, both already-valid by construction —
    /// so a failure is silently dropped rather than panicking the live App
    /// over a menu refresh.
    pub fn refresh_accelerators(&self, keys: &[(String, Vec<String>)]) {
        for (id, kind) in &self.routed {
            let accelerator = accelerator_for_id(id, keys);
            match kind {
                muda::MenuItemKind::MenuItem(item) => {
                    let _ = item.set_accelerator(accelerator);
                }
                muda::MenuItemKind::Icon(item) => {
                    let _ = item.set_accelerator(accelerator);
                }
                _ => {}
            }
        }
    }
}

fn to_menu_item(
    id: &'static str,
    label: &'static str,
    icon: bool,
    keys: &[(String, Vec<String>)],
) -> Box<dyn muda::IsMenuItem> {
    let accelerator = accelerator_for_id(id, keys);
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

/// The native `muda::Accelerator` for a routed item's CURRENT effective
/// macOS chord — `keys` (`Config::keys`) resolved through
/// [`commands::native_accelerator_chord`], the SAME live-keymap-aware answer
/// [`super::item_chord_for_id`] gives the drawn bar, so the native menu's key
/// equivalent can never drift from what a rebind actually made the keymap
/// dispatch. `None` when the command has no chord under this config at all,
/// or its chord's key isn't in the covered set (see below). Builds a token
/// string [`muda::accelerator::Accelerator`]'s own `FromStr` accepts
/// (`"CONTROL+SHIFT+;"`) from the SAME `(Key, Modifiers)`
/// [`crate::keyspec::parse_chord`] gives every other chord consumer — never a
/// second hand-typed table, so the key-equivalent column can never name a
/// chord the keymap itself would not actually dispatch.
///
/// COVERED: every menu-routed command's default Mac chord is a single
/// character key (a letter, digit, or one of `; = - ,`) plus modifiers — the
/// whole roster, verified by `native_accelerator_law`. A future routed
/// command bound to a named key (arrows, Tab, …), default OR rebound, falls
/// through to `None` here — same as every item shows today — rather than
/// guessing a mapping untested against muda's own `Code` table.
fn accelerator_for_id(
    id: &str,
    keys: &[(String, Vec<String>)],
) -> Option<muda::accelerator::Accelerator> {
    let command = super::routed_command_for_id(id)?;
    let c = commands::COMMANDS.iter().find(|c| c.name == command)?;
    let spec = commands::native_accelerator_chord(c, keys)?;
    let (key, mods) = crate::keyspec::parse_chord(&spec).ok()?;
    let winit::keyboard::Key::Character(ch) = key else {
        return None;
    };
    let state = mods.state();
    let mut token = String::new();
    if state.contains(winit::keyboard::ModifiersState::CONTROL) {
        token.push_str("CONTROL+");
    }
    if state.contains(winit::keyboard::ModifiersState::ALT) {
        token.push_str("ALT+");
    }
    if state.contains(winit::keyboard::ModifiersState::SHIFT) {
        token.push_str("SHIFT+");
    }
    if state.contains(winit::keyboard::ModifiersState::SUPER) {
        token.push_str("SUPER+");
    }
    token.push_str(ch.as_str());
    token.parse().ok()
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

pub fn build_menu(keys: &[(String, Vec<String>)]) -> InstalledMenu {
    let mut markdown = None;
    let mut routed = Vec::new();
    let submenus: Vec<Submenu> = roster()
        .into_iter()
        .map(|menu| {
            let items: Vec<Box<dyn muda::IsMenuItem>> = menu
                .items
                .iter()
                .map(|item| to_native_item(item, &mut markdown, &mut routed, keys))
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
        routed,
    }
}

fn to_native_item(
    item: &RosterItem,
    markdown: &mut Option<Submenu>,
    routed: &mut Vec<(&'static str, muda::MenuItemKind)>,
    keys: &[(String, Vec<String>)],
) -> Box<dyn muda::IsMenuItem> {
    match item {
        RosterItem::Routed { id, label, icon } => {
            let built = to_menu_item(id, label, *icon, keys);
            routed.push((id, built.kind()));
            built
        }
        RosterItem::Separator => Box::new(PredefinedMenuItem::separator()),
        RosterItem::Predefined(kind) => Box::new(to_predefined(*kind)),
        RosterItem::Submenu { label, items } => {
            let children: Vec<Box<dyn muda::IsMenuItem>> = items
                .iter()
                .map(|item| to_native_item(item, markdown, routed, keys))
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
    keys: &[(String, Vec<String>)],
) -> InstalledMenu {
    let menu = build_menu(keys);
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
                let accel = accelerator_for_id(id, &[]).unwrap_or_else(|| {
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
                accelerator_for_id(id, &[]),
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
        assert_eq!(accelerator_for_id("awl.open", &[]), None);
    }

    #[test]
    fn unknown_id_has_no_accelerator() {
        assert_eq!(accelerator_for_id("awl.nonexistent", &[]), None);
    }

    /// ONE ROW of [`native_accelerator_table_tracks_a_live_keymap_change`]:
    /// does `accelerator_for_id`'s answer for `id` under `keys` agree with
    /// the drawn bar's own [`commands::menu_native_label`]? `None` when the
    /// row falls outside today's documented coverage (not a routed id, not a
    /// catalog command, or a named-key chord — the same gap the
    /// default-chord law above leaves untested) — never asserted. Otherwise
    /// `Some(chord_differs_from_the_static_default)`, so the caller can prove
    /// the sweep's `keys` scenarios actually moved something.
    ///
    /// `menu_native_label` is read with an EXPLICIT `Convention::Mac` +
    /// `Platform::Native` rather than through `item_chord_for_id` (which
    /// resolves `Convention::current()`): `native-gate.sh` also runs this
    /// same binary under `AWL_CONVENTION_FORCE=linux` to sweep the Linux
    /// label table through a real macOS build, and the native AppKit menu
    /// this file models is unconditionally Mac — pinning the convention
    /// keeps this law honest under that sweep instead of silently comparing
    /// against the wrong table.
    fn assert_row_tracks_the_live_keymap(id: &str, keys: &[(String, Vec<String>)]) -> Option<bool> {
        use crate::convention::Convention;
        let command = super::super::routed_command_for_id(id)?;
        let c = commands::COMMANDS.iter().find(|c| c.name == command)?;
        let label = commands::menu_native_label(
            c,
            keys,
            &[],
            Convention::Mac,
            commands::Platform::Native,
            crate::keymap::KeymapFlavor::Native,
        );
        let accel = accelerator_for_id(id, keys);
        let Some(spec) = commands::native_accelerator_chord(c, keys) else {
            assert!(
                label.is_empty(),
                "{id:?} ({command:?}): no chord under this config but the drawn bar still \
                 shows {label:?}"
            );
            assert_eq!(
                accel, None,
                "{id:?} ({command:?}): no chord under this config but accelerator_for_id \
                 returned {accel:?}"
            );
            return Some(false);
        };
        let (key, mods) = crate::keyspec::parse_chord(&spec).ok()?;
        let winit::keyboard::Key::Character(_) = key else {
            return None;
        };
        assert_eq!(
            label,
            crate::keyspec::mac_glyph_chord(&spec),
            "{id:?} ({command:?}): native accelerator spec {spec:?} disagrees with the drawn \
             bar's own label"
        );
        let got = accel.unwrap_or_else(|| {
            panic!(
                "{id:?} ({command:?}): spec {spec:?} parses as a covered single-character \
                 chord but accelerator_for_id returned None"
            )
        });
        let state = mods.state();
        let want = got.modifiers();
        assert_eq!(
            want.contains(muda::accelerator::Modifiers::CONTROL),
            state.contains(winit::keyboard::ModifiersState::CONTROL),
            "{id:?} ({command:?}): control modifier mismatch, spec {spec:?}"
        );
        assert_eq!(
            want.contains(muda::accelerator::Modifiers::ALT),
            state.contains(winit::keyboard::ModifiersState::ALT),
            "{id:?} ({command:?}): alt modifier mismatch, spec {spec:?}"
        );
        assert_eq!(
            want.contains(muda::accelerator::Modifiers::SHIFT),
            state.contains(winit::keyboard::ModifiersState::SHIFT),
            "{id:?} ({command:?}): shift modifier mismatch, spec {spec:?}"
        );
        assert_eq!(
            want.contains(muda::accelerator::Modifiers::SUPER),
            state.contains(winit::keyboard::ModifiersState::SUPER),
            "{id:?} ({command:?}): super/cmd modifier mismatch, spec {spec:?}"
        );
        Some(spec.as_str() != c.native)
    }

    /// LAW: the native accelerator table tracks a LIVE keymap change the
    /// same way the drawn bar does — swept over several representative
    /// `Config::keys` scenarios (no override; a command's own chord moved
    /// elsewhere, retiring its old default; a DIFFERENT command's chord
    /// rebound onto a chord another routed command still defaults to, a real
    /// collision; a command with no default gaining one), not just the
    /// static-default case the two laws above already cover. Pinned at the
    /// DATA layer (`accelerator_for_id`/`native_accelerator_chord`) rather
    /// than against a real `NSMenuItem`, per `build_menu`/`install`'s doc —
    /// muda's own object construction is main-thread-only and cannot run in
    /// this test process; the live AppKit key-equivalent interception this
    /// data feeds stays outside what this test — or any headless one — can
    /// prove.
    #[test]
    fn native_accelerator_table_tracks_a_live_keymap_change() {
        let scenarios: &[&[(&str, &[&str])]] = &[
            &[],
            // "Save" moves off its own default chord — the retired-default
            // case: the item that used to answer to Cmd-S must stop.
            &[("Save", &["Cmd-Shift-S"])],
            // "Go to…" claims Save's still-live default chord — a real
            // collision between two routed items' effective chords.
            &[("Go to…", &["Cmd-S"])],
            // "Open file…" has no default at all; give it one.
            &[("Open file…", &["Cmd-Shift-O"])],
        ];
        let mut covered = 0usize;
        let mut saw_a_non_default_answer = false;
        for scenario in scenarios {
            let keys: Vec<(String, Vec<String>)> = scenario
                .iter()
                .map(|(name, chords)| {
                    (
                        (*name).to_string(),
                        chords.iter().map(|c| (*c).to_string()).collect(),
                    )
                })
                .collect();
            for menu in super::super::roster() {
                for item in super::super::dropdown_items(&menu) {
                    let RosterItem::Routed { id, .. } = item else {
                        continue;
                    };
                    if let Some(non_default) = assert_row_tracks_the_live_keymap(id, &keys) {
                        covered += 1;
                        saw_a_non_default_answer |= non_default;
                    }
                }
            }
        }
        assert!(
            covered > 80,
            "expected a non-trivial roster swept across every scenario, found {covered} \
             (a filtering bug could make this loop cover nothing and still pass)"
        );
        assert!(
            saw_a_non_default_answer,
            "every scenario answered with the static default chord — the `keys` scenarios \
             above never actually changed anything, which would make this law pass whether \
             or not the native table tracks a live rebind at all"
        );
    }
}
