//! src/module_map_law.rs — THE MODULE-MAP LAW for `ARCHITECTURE.md`.
//!
//! WHY: `ARCHITECTURE.md` declared `search.rs` and `theme.rs` as flat files long
//! after both became directory modules — the doc's whole job is to tell a reader
//! where code lives, and it was pointing at two paths that do not exist. The
//! file is the FIRST thing a new reader opens, so a wrong path there costs more
//! than a wrong sentence anywhere else.
//!
//! THE CHECK reads the document's own declaration grammar rather than scanning
//! every backtick. A module-map entry opens a top-level bullet with the module's
//! path in code ticks, and the tick spelling says which KIND it is:
//!
//! ```text
//! - `keymap.rs` — …      a FILE:      src/keymap.rs must exist and be a file
//! - `search/` — …        a DIRECTORY: src/search must exist and be a directory
//! ```
//!
//! The two arms assert DIFFERENT filesystem predicates on purpose, so the law
//! catches the drift in both directions: a directory module still written as
//! `foo.rs` fails, and a `foo/` with no such directory fails too. A single
//! "something named foo exists somewhere" check would have passed the very bug
//! this law is named for — `src/search/` exists, so any forgiving predicate
//! reads `search.rs` as fine.
//!
//! WHAT THE DIR ARM DOES NOT DECIDE, stated because the obvious reading of the
//! sentence above is wrong: several modules are BOTH (`render.rs` beside
//! `render/`, `rotated_label.rs` beside `rotated_label/`), and the document
//! spells those inconsistently — `render.rs` with its `→ render/:` sub-bullet,
//! `rotated_label/` outright. Both spellings resolve, so this law accepts both;
//! which one reads better is a style call, not a wrong path, and a law that
//! forced one would be asserting taste.
//!
//! NOT COVERED, deliberately: the prose beside each entry, the `→ dir:`
//! sub-bullets listing submodules (a partial list there is a summary, not a
//! false path), and citations inside running prose — those are relative to
//! whatever module the sentence is about, and resolving them would need the
//! grammar to know which. Path citations across the whole doc web are
//! `embedded_docs_law::docs_links_resolve`'s subject, for `.md` targets.
#![cfg(test)]

use std::path::PathBuf;

/// What a module-map bullet declares, by how it is spelled.
#[derive(Debug, PartialEq)]
enum Declared {
    /// `` `foo.rs` `` / `` `foo/bar.rs` `` — a source FILE under `src/`.
    File(String),
    /// `` `foo/` `` — a DIRECTORY module under `src/`.
    Dir(String),
}

/// Every module-map declaration in `text`: a line opening `- ` followed
/// immediately by one code-ticked path that is either `*.rs` or ends in `/`.
///
/// Anything else at the head of a bullet is prose or a concept name and is not a
/// declaration — the grammar is deliberately narrow so the law never invents a
/// path to check.
fn declarations(text: &str) -> Vec<Declared> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("- `") else {
            continue;
        };
        let Some(end) = rest.find('`') else { continue };
        let path = &rest[..end];
        if path.is_empty() || path.starts_with('/') || path.contains(char::is_whitespace) {
            continue;
        }
        if let Some(dir) = path.strip_suffix('/') {
            out.push(Declared::Dir(dir.to_string()));
        } else if path.ends_with(".rs") {
            out.push(Declared::File(path.to_string()));
        }
    }
    out
}

/// THE LAW. Every path `ARCHITECTURE.md`'s module map declares resolves under
/// `src/`, AS THE KIND IT IS DECLARED AS.
#[test]
fn architecture_md_module_map_paths_resolve_as_declared() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let doc = std::fs::read_to_string(root.join("ARCHITECTURE.md")).expect("read ARCHITECTURE.md");
    let declared = declarations(&doc);
    assert!(
        declared.len() > 10,
        "ARCHITECTURE.md's module map yielded only {} declarations — the \
         bullet grammar this law reads (`- ` then one code-ticked `foo.rs` or \
         `foo/`) has changed, and the law is now checking almost nothing",
        declared.len()
    );

    let mut wrong: Vec<String> = Vec::new();
    for d in &declared {
        match d {
            Declared::File(rel) => {
                let p = root.join("src").join(rel);
                if !p.is_file() {
                    let as_dir = root.join("src").join(rel.trim_end_matches(".rs"));
                    let hint = if as_dir.is_dir() {
                        format!(
                            " — but `src/{}/` IS a directory: the module was \
                             decomposed and the entry should read `{}/`",
                            rel.trim_end_matches(".rs"),
                            rel.trim_end_matches(".rs")
                        )
                    } else {
                        String::new()
                    };
                    wrong.push(format!(
                        "ARCHITECTURE.md declares `{rel}` as a file, but \
                         `src/{rel}` is not one{hint}"
                    ));
                }
            }
            Declared::Dir(rel) => {
                let p = root.join("src").join(rel);
                if !p.is_dir() {
                    let as_file = root.join("src").join(format!("{rel}.rs"));
                    let hint = if as_file.is_file() {
                        format!(" — but `src/{rel}.rs` IS a file: the entry should read `{rel}.rs`")
                    } else {
                        String::new()
                    };
                    wrong.push(format!(
                        "ARCHITECTURE.md declares `{rel}/` as a directory \
                         module, but `src/{rel}` is not a directory{hint}"
                    ));
                }
            }
        }
    }
    wrong.sort();
    assert!(
        wrong.is_empty(),
        "ARCHITECTURE.md's module map points at paths that are not what it \
         says they are ({} of {} declarations):\n{}",
        wrong.len(),
        declared.len(),
        wrong.join("\n")
    );
}

/// The grammar's own unit check: both arms must actually be reachable, and they
/// must classify differently. Without this, a regression that made
/// [`declarations`] return only `File`s would leave the law above green while
/// silently no longer checking any directory module.
#[test]
fn the_module_map_grammar_tells_a_file_from_a_directory() {
    let found = declarations(
        "- `keymap.rs` — a flat module\n- `search/` — a directory module\n- not a declaration\n- `Action` — a concept\n",
    );
    assert_eq!(
        found,
        vec![
            Declared::File("keymap.rs".to_string()),
            Declared::Dir("search".to_string()),
        ]
    );
    let doc =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ARCHITECTURE.md"))
            .expect("read ARCHITECTURE.md");
    let real = declarations(&doc);
    assert!(
        real.iter().any(|d| matches!(d, Declared::File(_)))
            && real.iter().any(|d| matches!(d, Declared::Dir(_))),
        "ARCHITECTURE.md's module map must exercise BOTH arms — it declares \
         {real:?}, and a law that only ever sees one kind proves nothing about \
         the other"
    );
}
