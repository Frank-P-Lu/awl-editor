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
    if icon && let Some(icon) = menu_icons::icon_for(id) {
        return Box::new(muda::IconMenuItem::with_id(
            id,
            label,
            true,
            Some(icon),
            None,
        ));
    }
    Box::new(MenuItem::with_id(id, label, true, None))
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
