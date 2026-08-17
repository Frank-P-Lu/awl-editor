//! The bundled-document-owner law, carved out of `open.rs` so that file's own
//! size measures production code. A sibling `tests.rs` is exempt from the
//! production ceiling by `code-health.py`'s `production()` rule, whose own comment
//! notes that counting one as production would defeat the point of carving an
//! inline test module out of an oversized file.
//!
//! THE BUNDLED-DOCUMENT-OWNER LAW — a source scan, in the shape of
//! `embedded_docs_law.rs`'s `embed_owner_is_the_only_include_str_site`, for
//! the OTHER half of the same discipline: not just where a document's BYTES
//! are embedded, but where they are OPENED. Guide and Reference both route
//! through `open_bundled_doc` (write the embedded text under
//! `fs::data_root()`, then `load_path` it) rather than each carrying its own
//! copy of that two-step shape — before that owner existed, `open_credits`
//! and `open_guide` each hand-rolled it, and `open_reference` would have been
//! a THIRD hand-roll. Credits has since left this family entirely: it opens
//! as a summoned read-only VIEWER (`OverlayKind::Credits`) rather than a
//! buffer, so it never reaches `load_path` at all. This test is what stops a
//! new bundled DOCUMENT BUFFER from reintroducing the write+load duplication
//! instead of reusing the owner.

use std::fs;
use std::path::PathBuf;

/// The `impl App` block only — truncated at this very law module's own
/// marker (mirroring `embedded_docs_law.rs`'s "the law module cites the
/// tokens it defends; skip its own text"), since this test's OWN doc
/// comments and assertion strings necessarily spell both needles and
/// would otherwise flag themselves.
fn this_file_source() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join("src/app/files/open.rs")).expect("read open.rs")
}

/// Extract `(name, body)` for every `fn` in the source whose SIGNATURE
/// LINE begins (after indentation) with `fn ` or with a `pub...` prefix
/// containing ` fn ` — anchoring on the start of the line, not a bare
/// `"fn "` substring search, specifically so a doc comment that merely
/// MENTIONS "fn" (this file has one: "Before this fn existed…") can never
/// be mistaken for a signature. The body is then the byte range from the
/// first `{` after the name to its balanced `}`, found by a raw brace
/// count.
fn functions(source: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut offset = 0usize;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let is_sig =
            trimmed.starts_with("fn ") || (trimmed.starts_with("pub") && trimmed.contains(" fn "));
        if is_sig {
            let fn_kw = offset + line.find("fn ").expect("a signature line names `fn`");
            let name_start = fn_kw + 3;
            let name_end = source[name_start..]
                .find(|c: char| c == '(' || c.is_whitespace())
                .map(|o| name_start + o)
                .unwrap_or(name_start);
            let name = source[name_start..name_end].to_string();
            if let Some(brace_rel) = source[name_end..].find('{') {
                let body_start = name_end + brace_rel;
                let mut depth = 0i32;
                let mut j = body_start;
                let mut body_end = body_start;
                while j < bytes.len() {
                    match bytes[j] {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                body_end = j + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                out.push((name, source[body_start..body_end].to_string()));
            }
        }
        offset += line.len();
    }
    out
}

/// THE LAW. `open_bundled_doc` performs "write the embedded text, then
/// `load_path` it" in one place; every OTHER function in this file is
/// scanned for that exact two-step shape (both `fs::write_atomic` AND
/// `self.load_path(` inside the SAME function body) and fails by name if
/// found — that is precisely what a hand-rolled fourth opener would need
/// to do instead of calling the owner.
///
/// WHAT THIS CANNOT SEE (a source scan is a grep, not a compiler, and its
/// own configuration is therefore a hypothesis, not a guarantee):
/// - A bypass that reaches the write/load pair through a local alias, a
///   renamed `use`, or an indirect helper never spells both literal
///   needles in one function body, so it would not match.
/// - A bypass added in a DIFFERENT file is invisible: this scan reads
///   only `src/app/files/open.rs`, the one file the module doc already
///   names as the owner's home.
/// - The brace counter is byte-level, not a Rust parser: an unbalanced
///   `{`/`}` inside a string or comment (this file currently has none —
///   every format-string brace pair like `"{label}"` is self-balanced)
///   would desync it. It catches the exact duplication shape this item's
///   own refactor eliminated, not every conceivable bypass.
#[test]
fn only_open_bundled_doc_writes_an_embedded_document_and_loads_it() {
    let source = this_file_source();
    let fns = functions(&source);
    assert!(
        fns.iter().any(|(n, _)| n == "open_bundled_doc"),
        "sanity: the extractor must find open_bundled_doc itself, or this \
         law is vacuous — got {:?}",
        fns.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );
    let mut offenders: Vec<String> = fns
        .iter()
        .filter(|(name, _)| name != "open_bundled_doc")
        .filter(|(_, body)| body.contains("fs::write_atomic") && body.contains("self.load_path("))
        .map(|(name, _)| name.clone())
        .collect();
    offenders.sort();
    assert!(
        offenders.is_empty(),
        "these functions write an embedded document and load it INLINE \
         instead of routing through `open_bundled_doc` (the one owner \
         Guide/Reference share): {offenders:?} — call \
         `self.open_bundled_doc(label, filename, content)` instead",
    );
}
