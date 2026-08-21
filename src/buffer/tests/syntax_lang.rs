use super::*;

#[test]
fn syntax_lang_gates_code_only() {
    // The gate that controls whether the renderer emits ANY syntax spans: code
    // extensions highlight; markdown / txt / scratch must NOT. A path with a
    // non-markdown extension (.rs / .txt) is ALSO not markdown, so the markdown
    // and code styling passes stay mutually exclusive.
    let mut code = Buffer::from_str("fn main() {}");
    code.set_path("/p/main.rs".into());
    assert_eq!(code.syntax_lang(), Some(crate::syntax::Lang::Rust));
    assert!(!code.is_markdown(), "a .rs file is code, not markdown");

    let mut md = Buffer::from_str("# heading");
    md.set_path("/p/notes.md".into());
    assert!(
        md.syntax_lang().is_none(),
        "markdown must not syntax-highlight"
    );
    assert!(md.is_markdown(), "and it IS markdown");

    let mut txt = Buffer::from_str("plain prose");
    txt.set_path("/p/notes.txt".into());
    assert!(
        txt.syntax_lang().is_none(),
        ".txt must not syntax-highlight"
    );
    assert!(
        !txt.is_markdown(),
        "a .txt file is plain prose, not markdown"
    );

    // The bare scratch buffer (no path) now reads as markdown — the prose-first
    // writing surface — yet syntax is path-based, so it is never code-highlighted
    // (markdown and code remain mutually exclusive).
    let scratch = Buffer::from_str("scratch");
    assert!(scratch.syntax_lang().is_none());
    assert!(
        scratch.is_markdown(),
        "the scratch writing surface IS markdown"
    );
}

#[test]
fn page_class_mirrors_syntax_lang_presence() {
    // The prose/code page-width split (`crate::page::PageClass`): a recognized
    // CODE file is `Code`, everything else — markdown, an unrecognized plain-text
    // file, or the no-path scratch surface — is `Prose`. Mirrors
    // `syntax_lang_gates_code_only` exactly, since `page_class` is defined in
    // terms of `syntax_lang`.
    let mut code = Buffer::from_str("fn main() {}");
    code.set_path("/p/main.rs".into());
    assert_eq!(code.page_class(), crate::page::PageClass::Code);

    let mut md = Buffer::from_str("# heading");
    md.set_path("/p/notes.md".into());
    assert_eq!(md.page_class(), crate::page::PageClass::Prose);

    let mut txt = Buffer::from_str("plain prose");
    txt.set_path("/p/notes.txt".into());
    assert_eq!(txt.page_class(), crate::page::PageClass::Prose);

    let scratch = Buffer::from_str("scratch");
    assert_eq!(scratch.page_class(), crate::page::PageClass::Prose);
}

#[test]
fn note_is_markdown_from_first_keystroke() {
    // A QUICK NOTE is conceptually always markdown (it auto-saves as `.md`), so
    // it must read as markdown the instant it is summoned — BEFORE its first
    // save derives a path. While you type the title, styling must already apply.
    let dir = note_tmp("md_gate");
    let mut note = Buffer::scratch();
    note.start_fresh_doc(dir.to_path_buf());
    assert!(note.path().is_none(), "a fresh note has no path yet");
    assert!(
        note.is_markdown(),
        "an unsaved note is markdown from the start"
    );
    // ...and it must NOT be code-highlighted: syntax is path-based, a note has
    // no code extension, so markdown and code stay mutually exclusive.
    assert!(
        note.syntax_lang().is_none(),
        "a note never syntax-highlights"
    );

    // Once saved, the note's path ends in `.md`, so it stays markdown.
    let mut saved = Buffer::from_str("# titled");
    saved.set_path("/notes/titled.md".into());
    assert!(
        saved.is_markdown(),
        "a saved note keeps reading as markdown"
    );

    // The bare SCRATCH buffer (no note_dir, no path) is ALSO markdown now —
    // awl's blank launch surface is a prose-first writing surface, so `#` /
    // `**` style as you type. It is NOT a note, and (syntax is path-based) it
    // is never code-highlighted, so markdown and code stay mutually exclusive.
    let scratch = Buffer::scratch();
    assert!(
        scratch.is_markdown(),
        "the scratch writing surface IS markdown"
    );
    assert!(!scratch.is_unnamed_fresh(), "but it is not a quick note");
    assert!(
        scratch.syntax_lang().is_none(),
        "scratch is never code-highlighted"
    );
}
