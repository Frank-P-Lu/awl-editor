use super::{Action, Command};

// SENTENCE MOTION — no macOS-native convention exists to double (unlike
// word motion's retired ⌥←/⌥→), so both slots ship empty by default;
// Linux's `keymap = "emacs"` flavor still reaches these via the classic
// M-e/M-a/M-k Meta seed (`keymap::platform::LINUX_EMACS_META_SEED`), and
// `[keys] sentence_forward = "..."` reaches them everywhere else. See
// `buffer::sentence`'s module doc for the UAX #29 boundary rule.
pub(super) const SENTENCE_FORWARD: Command = Command {
    name: "Sentence forward",
    action: Action::ForwardSentence,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some("Move the caret to the start of the following sentence."),
};
pub(super) const SENTENCE_BACKWARD: Command = Command {
    name: "Sentence backward",
    action: Action::BackwardSentence,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some(
        "Move the caret to the start of the current sentence, or the previous one if already there.",
    ),
};

// SENTENCE DELETE, the mutating siblings of the sentence MOTIONS above.
// Forward reclaims the classic emacs kill-sentence chord on Linux
// (`delete_sentence_forward = "M-k"`, seeded by default there — see the
// Meta-seed table); backward has no classic binding and ships silent,
// like `delete_word_backward`.
pub(super) const DELETE_SENTENCE_FORWARD: Command = Command {
    name: "Delete sentence forward",
    action: Action::DeleteSentenceForward,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some(
        "Delete to the start of the following sentence; a selection deletes instead.",
    ),
};
pub(super) const DELETE_SENTENCE_BACKWARD: Command = Command {
    name: "Delete sentence backward",
    action: Action::DeleteSentenceBackward,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some("Delete to the start of the current sentence; a selection deletes instead."),
};
