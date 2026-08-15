use winit::keyboard::{Key, ModifiersState, SmolStr};

use super::Chord;

pub(super) fn canon_key(key: &Key) -> Key {
    match key {
        Key::Character(s) => Key::Character(SmolStr::new(s.to_lowercase())),
        other => other.clone(),
    }
}

fn key_is_char(key: &Key, c: char) -> bool {
    matches!(key, Key::Character(s) if s.eq_ignore_ascii_case(&c.to_string()))
}

/// Parse a config chord into the override-map representation. Single chords
/// and the two supported prefix families share the headless keyspec grammar.
pub fn parse_binding(spec: &str) -> Result<Chord, String> {
    let toks: Vec<&str> = spec.split_whitespace().collect();
    match toks.as_slice() {
        [one] => {
            let (key, modifiers) = crate::keyspec::parse_chord(one).map_err(|e| e.to_string())?;
            Ok(Chord::Single(canon_key(&key), modifiers.state()))
        }
        [first, second] => {
            let (prefix, modifiers) =
                crate::keyspec::parse_chord(first).map_err(|e| e.to_string())?;
            let is_cx = modifiers.state() == ModifiersState::CONTROL && key_is_char(&prefix, 'x');
            let is_cc = modifiers.state() == ModifiersState::CONTROL && key_is_char(&prefix, 'c');
            if !is_cx && !is_cc {
                return Err(format!(
                    "only the C-x / C-c prefixes are supported for two-chord bindings, \
                     got {first:?}"
                ));
            }
            let (key, modifiers) =
                crate::keyspec::parse_chord(second).map_err(|e| e.to_string())?;
            Ok(if is_cx {
                Chord::Cx(canon_key(&key), modifiers.state())
            } else {
                Chord::Cc(canon_key(&key), modifiers.state())
            })
        }
        [] => Err("empty binding".to_string()),
        _ => Err(format!(
            "expected one chord or 'C-x <key>', got {} chords",
            toks.len()
        )),
    }
}
