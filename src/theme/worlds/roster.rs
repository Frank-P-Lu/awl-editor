use super::*;

pub const THEMES: [Theme; 19] = [
    TAWNY, MOPOKE, CURRAWONG, POTOROO, GUMTREE, BILBY, SALTPAN, QUOKKA, BOMBORA, BOWERBIRD, MULGA,
    MANGROVE, GALAH, MAGPIE, BROLGA, WAGTAIL, FIRETAIL, CASSOWARY, KITE,
];

const fn str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn world_index(name: &str) -> usize {
    let mut i = 0;
    while i < THEMES.len() {
        if str_eq(THEMES[i].name, name) {
            return i;
        }
        i += 1;
    }
    panic!("world_index: no world by that name")
}

pub const DEFAULT_THEME: usize = world_index("Saltpan");

pub fn world_names() -> Vec<&'static str> {
    THEMES.iter().map(|theme| theme.name).collect()
}
