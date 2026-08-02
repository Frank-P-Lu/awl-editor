//! Renderer-independent semantic UI state.
//!
//! The live editor, AccessKit, `--semantic-json`, and live-App capture sidecars
//! all read this one snapshot. It contains product meaning only: no pixels,
//! animation phase, GPU state, or callbacks.

use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

pub const SCHEMA: &str = "awl-semantic/1";
pub const ROOT_ID: &str = "app";
pub const DOCUMENT_ID: &str = "document";
pub const DOCUMENT_TEXT_ID: &str = "document.text";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSnapshot {
    pub schema: &'static str,
    pub root_id: String,
    pub focus_id: String,
    pub nodes: Vec<SemanticNode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticNode {
    pub id: String,
    pub role: SemanticRole,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<SemanticAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<SemanticSelection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub character_lengths: Vec<usize>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub focusable: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub focused: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub editable: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub multiline: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expanded: Option<bool>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl SemanticNode {
    pub fn new(id: impl Into<String>, role: SemanticRole, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role,
            name: name.into(),
            value: None,
            description: None,
            children: Vec::new(),
            controls: Vec::new(),
            actions: Vec::new(),
            selection: None,
            character_lengths: Vec::new(),
            focusable: false,
            focused: false,
            editable: false,
            multiline: false,
            selected: None,
            checked: None,
            expanded: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticRole {
    Application,
    Document,
    Text,
    Dialog,
    Group,
    TextInput,
    ListBox,
    Option,
    Button,
    CheckBox,
    Slider,
    Status,
    MenuBar,
    MenuItem,
    Heading,
    StaticText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticAction {
    Focus,
    Click,
    SetTextSelection,
    ReplaceSelectedText,
    SetValue,
    Increment,
    Decrement,
    Expand,
    Collapse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSelection {
    pub anchor: usize,
    pub focus: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticRequest {
    Focus {
        id: String,
    },
    Click {
        id: String,
    },
    SetTextSelection {
        id: String,
        anchor: usize,
        focus: usize,
    },
    ReplaceSelectedText {
        id: String,
        value: String,
    },
    SetValue {
        id: String,
        value: String,
    },
    Increment {
        id: String,
    },
    Decrement {
        id: String,
    },
    Expand {
        id: String,
    },
    Collapse {
        id: String,
    },
}

pub fn grapheme_lengths(text: &str) -> Vec<usize> {
    text.graphemes(true).map(str::len).collect()
}

pub fn char_to_grapheme(text: &str, char_offset: usize) -> usize {
    let byte = text
        .char_indices()
        .nth(char_offset)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len());
    text.grapheme_indices(true)
        .take_while(|(start, _)| *start < byte)
        .count()
}

pub fn grapheme_to_char(text: &str, grapheme_offset: usize) -> usize {
    let byte = text
        .grapheme_indices(true)
        .nth(grapheme_offset)
        .map(|(byte, _)| byte)
        .unwrap_or(text.len());
    text[..byte].chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grapheme_offsets_cover_combining_emoji_and_flags_without_splitting() {
        let text = "a e\u{301} 👨‍👩‍👧‍👦 🇯🇵 z";
        let graphemes: Vec<&str> = text.graphemes(true).collect();
        assert!(graphemes.contains(&"e\u{301}"));
        assert!(graphemes.contains(&"👨‍👩‍👧‍👦"));
        assert!(graphemes.contains(&"🇯🇵"));
        assert_eq!(grapheme_lengths(text).iter().sum::<usize>(), text.len());
        for index in 0..=graphemes.len() {
            let chars = grapheme_to_char(text, index);
            assert_eq!(char_to_grapheme(text, chars), index);
        }
    }
}
