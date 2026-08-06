//! THE SEMANTIC FOLD'S REACH BOUNDARY, as executable data.
//!
//! `docs/app-domains.md` counts FIELDS; this file counts REACH. Building the
//! accessibility tree is a read over the document and the summoned-UI ladder,
//! but it used to take `&App` — so every fold could also read the clipboard,
//! the daemon socket, the usage ledger and the GPU, and "what may the fold
//! read" was answerable only by reading every fold.
//!
//! `App::semantic_view` is now the one place the whole application state is
//! read on that side, and the folds below it receive
//! [`crate::app::semantic::SemanticView`]. Two things have to stay true for
//! that to be a boundary rather than a habit, and neither is expressible in
//! Rust's visibility system — `SemanticView` and `App` are both nameable from
//! anywhere under `crate::app`:
//!
//!  1. a fold file may not name `App` at all, and
//!  2. the view may not grow a handle to another domain, which would move the
//!     boundary without moving a single call site.
//!
//! The roster below is WILDCARD-FREE and swept against the directory, so a new
//! file under `src/app/semantic/` fails this test until someone chooses which
//! side of the line it is on.

/// Every production source file under `src/app/semantic/`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SemanticFile {
    /// The live doors and the one narrowing constructor.
    Mod,
    /// The `SemanticView` declaration; holds `App::semantic_view`.
    View,
    /// Decoded requests, applied through the App's existing owners.
    Requests,
    /// `--bench-a11y`; builds a whole `App` to measure against.
    Bench,
    /// The retained projection — the document half of the fold.
    Projection,
    /// The ACTIVE summoned surfaces.
    Surfaces,
    /// The PASSIVE surfaces.
    Passive,
}

/// Which side of the boundary a file is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    /// May name `App`. Every one of these is a place the narrowing happens or
    /// a mutation that genuinely ends in another owner's verb.
    ReadsTheApp,
    /// Sees the narrow view and nothing behind it.
    FoldOnly,
}

impl SemanticFile {
    const ROSTER: &'static [Self] = &[
        Self::Mod,
        Self::View,
        Self::Requests,
        Self::Bench,
        Self::Projection,
        Self::Surfaces,
        Self::Passive,
    ];

    /// No wildcard arm: a new roster member cannot join without choosing a
    /// side, and the sweep below will not let a new file stay off the roster.
    fn path_and_side(self) -> (&'static str, Side) {
        match self {
            Self::Mod => ("src/app/semantic/mod.rs", Side::ReadsTheApp),
            Self::View => ("src/app/semantic/view.rs", Side::ReadsTheApp),
            Self::Requests => ("src/app/semantic/requests.rs", Side::ReadsTheApp),
            Self::Bench => ("src/app/semantic/bench.rs", Side::ReadsTheApp),
            Self::Projection => ("src/app/semantic/projection.rs", Side::FoldOnly),
            Self::Surfaces => ("src/app/semantic/surfaces.rs", Side::FoldOnly),
            Self::Passive => ("src/app/semantic/passive.rs", Side::FoldOnly),
        }
    }
}

/// The fold reads the narrow view; only the narrowing sites read the `App`.
///
/// `App` is matched as a whole identifier with comments stripped, because
/// `SemanticRole::Application` contains those three letters and a doc comment
/// is allowed to name the type it deliberately no longer touches.
#[test]
fn semantic_fold_reads_only_the_narrow_view() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let view = std::fs::read_to_string(repo.join("src/app/semantic/view.rs"))
        .expect("the semantic view's source must be readable");
    assert_eq!(
        struct_field_names(&strip_comments(&view), "SemanticView"),
        ["document", "workspace_state", "card", "whichkey", "notice"],
        "the semantic view's roster drifted — a new member widens what every \
         fold may read, which is the boundary this file exists to hold"
    );

    let mut roster_paths = std::collections::BTreeSet::new();
    for file in SemanticFile::ROSTER {
        let (relative, side) = file.path_and_side();
        roster_paths.insert(relative.to_string());
        let source = std::fs::read_to_string(repo.join(relative))
            .unwrap_or_else(|e| panic!("{relative} must be readable: {e}"));
        let code = strip_comments(&source);
        let names_app = names_identifier(&code, "App");
        match side {
            Side::ReadsTheApp => assert!(
                names_app,
                "{relative} is on the ReadsTheApp side but never names `App` — \
                 move it to Side::FoldOnly rather than leaving a vacuous roster entry"
            ),
            Side::FoldOnly => {
                assert!(
                    !names_app,
                    "{relative} is a fold and must not reach past `SemanticView` — \
                     the whole `App` is read once, in `App::semantic_view`"
                );
                assert!(
                    names_identifier(&code, "SemanticView"),
                    "{relative} is a vacuous fold: it names neither `App` nor \
                     `SemanticView`, so this gate is not guarding it"
                );
            }
        }
    }

    let mut discovered = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(repo.join("src/app/semantic"))
        .expect("the semantic source directory must be readable")
    {
        let path = entry.expect("source entry must be readable").path();
        if path.is_dir() || !path.extension().is_some_and(|ext| ext == "rs") {
            continue;
        }
        discovered.insert(
            path.strip_prefix(&repo)
                .expect("source lives under the repo")
                .to_string_lossy()
                .to_string(),
        );
    }
    assert_eq!(
        discovered, roster_paths,
        "the no-wildcard semantic-file roster must cover every production file \
         under src/app/semantic/"
    );
}

/// Field names of `struct <name>`, in declaration order. Takes source with the
/// comments already stripped, so no prose can perturb the brace depth.
fn struct_field_names(source: &str, name: &str) -> Vec<String> {
    let marker = format!("struct {name}");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("missing `{marker}`"));
    let body_start = source[start..]
        .find('{')
        .map(|offset| start + offset + 1)
        .expect("struct body");
    let mut depth = 1usize;
    let mut end = source.len();
    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = body_start + offset;
                    break;
                }
            }
            _ => {}
        }
    }
    source[body_start..end]
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(field_name)
        .collect()
}

/// The declared name on one struct-field line, or `None` if the line declares
/// nothing. Splits at the field's own `:` — never at a path separator, which
/// `pub(in crate::app) x: T` puts first — and then drops the visibility.
fn field_name(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let at = (0..bytes.len()).find(|&i| {
        bytes[i] == b':'
            && bytes.get(i + 1) != Some(&b':')
            && (i == 0 || bytes[i - 1] != b':')
            && line[..i].matches('(').count() == line[..i].matches(')').count()
    })?;
    let name = line[..at].split_whitespace().next_back()?;
    (!name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .then(|| name.to_string())
}

fn strip_comments(source: &str) -> String {
    source
        .lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Does `code` use `needle` as a whole identifier? A substring match would
/// report `Application` as a use of `App`.
fn names_identifier(code: &str, needle: &str) -> bool {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    code.match_indices(needle).any(|(at, _)| {
        let before = code[..at].chars().next_back();
        let after = code[at + needle.len()..].chars().next();
        !before.is_some_and(is_word) && !after.is_some_and(is_word)
    })
}
