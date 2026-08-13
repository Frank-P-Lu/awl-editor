//! How one picker row reads: the corpus string a row carries turned into the
//! string a user sees. Carved out of `build.rs` to keep that file under its
//! frozen baseline; it is one owner and belongs beside its builder, not in it.

use crate::overlay::*;

/// **HOW ONE PICKER ROW READS** — the corpus string a row carries turned into
/// the string a user sees. One owner, because the drawn rows, the sidecar's
/// `overlay.items` and the accessibility tree are all this same answer, and a
/// second derivation is how they come to disagree.
///
/// Most rows are their own accept string, with a folder's trailing `/`. The
/// five that are not: the switch-project ACCEPT-THIS-FOLDER row
/// ([`here_folder_label`]), a switch-project REMEMBERED row (whose accept is a
/// whole absolute path, made readable by [`super::recent::label`]), an Assets
/// row (shown by its leaf), a palette Settings row, and a marker-prefixed Go-to
/// HEADING row.
pub(in crate::overlay) fn row_display(
    kind: OverlayKind,
    row: &OverlayRow,
    browse_dir: Option<&str>,
) -> String {
    // The accept-this-folder row's corpus string is not a name at all: it
    // carries `.` so the path math and the dotfile exemption have one stable
    // string to compare against. The only place `browse_dir` reaches eyes.
    if kind == OverlayKind::Project && row.accept == HERE_ACCEPT {
        return here_folder_label(browse_dir);
    }
    if kind == OverlayKind::Assets {
        let rel = &row.accept;
        return rel.rsplit('/').next().unwrap_or(rel).to_string();
    }
    if matches!(row.meta, RowMeta::CommandSetting { .. }) {
        return row.accept.clone();
    }
    if matches!(row.meta, RowMeta::GotoHeading { .. }) {
        return format!("{}{}", OverlayKind::HEADING_MARKER_PREFIX, row.accept);
    }
    // A REMEMBERED root carries its whole absolute path (that path IS the
    // project, wherever it lives), so it is the one switch-project row that
    // reads as a path rather than as a name — shortened against the level or
    // against home so the row says where the project is without spelling out
    // where the machine is.
    let mut s = match kind == OverlayKind::Project && is_remembered_root(&row.accept) {
        true => super::recent::label(&row.accept, browse_dir),
        false => row.accept.clone(),
    };
    if row.is_dir {
        s.push('/');
    }
    s
}

/// **THE ACCEPT-THIS-FOLDER ROW'S ACCEPT STRING.** The switch-project
/// navigator's first row is synthetic: it stands for the directory the level
/// itself is, so accepting it commits [`OverlayState::browse_dir`] rather than
/// any listed child. `.` is what the corpus carries — never what the user
/// reads (see [`here_folder_label`]) — and it is a string the row's own
/// consumers compare against (the dotfile filter's exemption, the
/// default-selection skip), so it is named once here rather than spelled as a
/// literal at each of them.
pub const HERE_ACCEPT: &str = ".";

/// The invariant half of [`here_folder_label`] — what the accept-this-folder
/// row says it DOES, with no folder named. Its own constant because the laws
/// that pin the row's copy and the label builder must not be able to drift.
pub const HERE_LABEL: &str = "use this folder";

/// **WHAT THE ACCEPT-THIS-FOLDER ROW READS AS** — the row's user-facing copy,
/// naming both what pressing it does and which folder it would do it to:
/// `use this folder — notes`.
///
/// It is the switch-project card's ONE statement of where it is standing.
/// [`OverlayState::browse_dir`] is otherwise a fact only the sidecar could
/// see: the title names the task (`switch project`), the rows name the
/// children, and nothing named the directory those children are children OF.
/// The row that ACTS on that directory is where naming it costs no extra
/// figure — a card stays calm by carrying few — so this is deliberately not a
/// second heading line.
///
/// The folder's NAME, never its path. A path here would inherit
/// [`elide_path`]'s bias — it keeps the leaf and elides the parents, right for
/// a filename and wrong for a directory readout, where the parent is the
/// informative half. The leaf is also what awl calls a folder everywhere else
/// (the gutter's project name, through the same owner,
/// [`crate::project::folder_name`]), and one level holds one directory, so
/// there is nothing here to disambiguate a parent from.
///
/// Correct at any depth: the label is rebuilt with the level, so it names the
/// workspace on a card that cannot browse and the browsed-to folder on one
/// that can (the Settings folder-VALUE navigator descends today).
pub fn here_folder_label(dir: Option<&str>) -> String {
    match dir.map(|d| crate::project::folder_name(std::path::Path::new(d))) {
        Some(name) if !name.is_empty() => format!("{HERE_LABEL} \u{2014} {name}"),
        // A level with no directory to name still says what the row does. Not
        // reachable through `new_project` (which always carries its dir), and
        // the honest degradation if it ever is.
        _ => HERE_LABEL.to_string(),
    }
}

/// Middle-truncate `s` to at most `max` CHARS with a single `…`, keeping the HEAD and
/// the TAIL — so a filename keeps its extension end. `s` already within `max` is returned
/// unchanged. Used for the directory prefix AND (when the filename alone overflows) the
/// filename itself.
fn elide_middle(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    if max == 1 {
        return "…".to_string();
    }
    let rem = max - 1; // room besides the one ellipsis
    let tail = rem / 2 + rem % 2; // bias the TAIL so the extension survives
    let head = rem - tail;
    let head_s: String = chars[..head].iter().collect();
    let tail_s: String = chars[chars.len() - tail..].iter().collect();
    format!("{head_s}…{tail_s}")
}

/// Elide a file-picker ROW to at most `max` CHARS on ONE line, PRESERVING the filename
/// (the text after the last `/`) and its extension and keeping as much LEADING directory
/// as fits. A row that already fits is returned whole. Otherwise the DIRECTORY is
/// middle-truncated (a single `…`) while the whole filename rides at the end; only when
/// the filename ALONE overflows is the filename itself middle-truncated (still one `…`,
/// still keeping its extension). The last `/` in the result is the figure/ground split
/// point ([`row_split`]): everything through it is the muted directory, the rest the
/// content-ink filename.
pub fn elide_path(path: &str, max: usize) -> String {
    let total = path.chars().count();
    if total <= max {
        return path.to_string();
    }
    match path.rfind('/') {
        Some(byte_slash) => {
            let dir = &path[..=byte_slash]; // through the trailing '/'
            let file = &path[byte_slash + 1..]; // filename + extension
            let file_len = file.chars().count();
            // No room for the whole filename beside an ellipsis → drop the dir and
            // middle-truncate the filename itself (keeping its extension end).
            if file_len + 1 > max {
                return elide_middle(file, max);
            }
            // Keep the WHOLE filename; middle-elide the directory to what's left. The
            // dir's trailing '/' rides its tail, so the split point survives.
            let dir_budget = max - file_len;
            format!("{}{file}", elide_middle(dir, dir_budget))
        }
        None => elide_middle(path, max),
    }
}

/// Elide a DIRECTORY readout to at most `max` chars while preserving the two
/// facts that make it read as a path: a `/` and a recognizable tail of the
/// final folder. Unlike [`elide_path`], this never drops the separator when the
/// leaf alone exceeds the allowance; file-picker rows keep their filename- and
/// extension-biased policy unchanged.
pub fn elide_directory_path(path: &str, max: usize) -> String {
    let total = path.chars().count();
    if total <= max {
        return path.to_string();
    }
    if max == 0 {
        return String::new();
    }
    if max == 1 {
        return "…".to_string();
    }
    if max == 2 {
        return "…/".to_string();
    }

    let leaf = path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path);
    if leaf.chars().count() + 2 <= max {
        return elide_path(path, max);
    }
    let leaf_budget = max - 2; // `…/` is the path-identity prefix.
    format!("…/{}", elide_middle(leaf, leaf_budget))
}

/// The figure/ground split of a (possibly elided) picker row: the byte index just PAST
/// the last `/` — everything before it is the DIRECTORY prefix (muted ink), everything
/// from it on is the FILENAME (content ink). `0` when the row has no `/` (a bare
/// filename → all content ink).
pub fn row_split(row: &str) -> usize {
    // THE UNION ROUND: a settings row's marker PREFIX (`"§ "`, `OverlayKind::
    // SETTINGS_MARKER_PREFIX`) is figure/ground-split exactly like a directory
    // prefix — the glyph recedes to muted ink, the setting name stays content ink.
    // Checked first (a setting name never itself contains a `/`).
    if row.starts_with(OverlayKind::SETTINGS_MARKER_PREFIX) {
        return OverlayKind::SETTINGS_MARKER_PREFIX.len();
    }
    // A Go-to HEADING row's marker PREFIX (`"❡ "`, `OverlayKind::
    // HEADING_MARKER_PREFIX`) is figure/ground-split the same way — the glyph
    // recedes to muted ink, the (indented) title stays content ink. Checked next
    // (a heading title never itself starts with the settings glyph).
    if row.starts_with(OverlayKind::HEADING_MARKER_PREFIX) {
        return OverlayKind::HEADING_MARKER_PREFIX.len();
    }
    row.rfind('/').map(|i| i + 1).unwrap_or(0)
}
