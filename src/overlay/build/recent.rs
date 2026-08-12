//! How a REMEMBERED project root becomes a switch-project row, and how that row
//! reads. Its own file beside `rowdisplay`, for the same reason: one owner, kept
//! out of the builder it is called from.

use std::path::Path;

/// **THE TWO ROUTES ONTO THE SWITCH-PROJECT ROSTER**, and the one question that
/// tells them apart.
///
/// A LEVEL row is a child NAME read from the directory the card is standing on.
/// That read is one level deep and must stay that way — it is what keeps the
/// flat picker flat.
///
/// A REMEMBERED row is a whole ABSOLUTE PATH the persisted recent-projects MRU
/// ([`crate::recents`]) kept from a switch already made: any depth, inside the
/// workspace or not, and it reaches the roster without anything descending. A
/// project nested below a direct child is unreachable by the read BY DESIGN and
/// findable here anyway, which is the whole of the Recent lens.
///
/// Absoluteness is the discriminator and it is TOTAL: [`crate::index::
/// list_dir_level`] yields leaf names, which carry no separator at all, and
/// [`resolve`] refuses a remembered root that is not absolute — so no row can be
/// both and none can be neither.
pub(crate) fn is_remembered_root(accept: &str) -> bool {
    Path::new(accept).is_absolute()
}

/// **WHICH REMEMBERED ROOTS CAN STAND AS ROWS AT ALL**, resolved against the
/// disk here — at the level read, the one seam already touching it — rather than
/// inside [`crate::overlay::OverlayState::new_project`], which stays a pure
/// function of what it is handed.
///
/// The verdict per MRU entry, in MRU order (the order IS the Recent lens's sort
/// key, so it is preserved rather than re-derived):
///
/// * **not absolute** — dropped. A relative root would resolve against whatever
///   the process's working directory happens to be, which is not a project.
/// * **the level's OWN directory** — dropped. The accept-this-folder row already
///   IS that answer; a second row naming the same folder reads as a duplicate.
/// * **a direct CHILD of the level** — kept, and it MARKS the row the read
///   already produced instead of becoming a second one.
/// * **anywhere else, still a directory** — kept; it becomes its own row,
///   carrying its whole path.
/// * **anywhere else, gone or no longer a directory** — dropped. A remembered
///   project that has been deleted, renamed, or lives on an unmounted volume can
///   only offer a row whose Enter fails, and an absent row is the kinder answer —
///   the same verdict the roster already reaches for an unresolvable symlink.
///
/// The git tag is the same rule `list_dir_level` applies to a child, asked of
/// the remembered root, so the same folder is tagged the same way whichever
/// route put it on the roster.
pub(in crate::overlay) fn resolve(
    dir: &str,
    folders: &[(String, bool)],
    roots: &[String],
) -> Vec<(String, bool)> {
    let base = Path::new(dir);
    let mut out: Vec<(String, bool)> = Vec::new();
    for root in roots {
        let rp = Path::new(root);
        if !rp.is_absolute() || rp == base || out.iter().any(|(kept, _)| kept == root) {
            continue;
        }
        if let Some((_, git)) = folders.iter().find(|(name, _)| base.join(name) == rp) {
            out.push((root.clone(), *git));
        } else if crate::fs::active().is_dir(rp) {
            let git = crate::fs::active().exists(&rp.join(".git"));
            out.push((root.clone(), git));
        }
    }
    out
}

/// **THE ENROLMENT.** Fold [`resolve`]'s verdicts into a corpus that already
/// holds the accept-this-folder row and the level's children: a root that names
/// a child marks that child, and every other root appends its own row. Returns
/// the corpus indices in MRU order — `refilter`'s MRU tiebreak reads exactly
/// that, so Recent lists newest-first without any switch-project-specific
/// ranking code.
///
/// Index 0 (the accept-this-folder row) is never a candidate: its corpus string
/// is `.`, not a name, and the level it stands for is already refused by
/// `resolve`.
pub(in crate::overlay) fn enrol(
    dir_abs: &str,
    corpus: &mut Vec<String>,
    git: &mut Vec<bool>,
    is_dir: &mut Vec<bool>,
    roots: &[(String, bool)],
) -> Vec<usize> {
    let base = Path::new(dir_abs);
    let mut recent: Vec<usize> = Vec::new();
    for (root, root_git) in roots {
        let rp = Path::new(root);
        let ci = match (1..corpus.len()).find(|&i| base.join(&corpus[i]) == rp) {
            Some(child) => child,
            None => {
                corpus.push(root.clone());
                git.push(*root_git);
                is_dir.push(true);
                corpus.len() - 1
            }
        };
        if !recent.contains(&ci) {
            recent.push(ci);
        }
    }
    recent
}

/// **HOW A REMEMBERED ROOT READS**: relative to the LEVEL the card is standing
/// on when it lives under it (`code2026/awl-next`), else relative to HOME
/// (`~/notes`), else the path itself.
///
/// This is the one switch-project row that shows a path at all, and it shows it
/// from the top down. The parents are the informative half of a directory
/// readout — `code2026/awl-next` and `archive/awl-next` are different projects
/// and the leaf cannot say so — which is also why it does not go through
/// [`super::elide_path`], whose bias keeps the leaf and eats the parents.
pub(in crate::overlay) fn label(root: &str, level: Option<&str>) -> String {
    label_with_home(root, level, crate::fs::home_dir().as_deref())
}

/// The PURE half of [`label`], with `home` injected — so the laws assert both
/// branches instead of asserting whichever one the machine running them happens
/// to produce.
///
/// `Path::strip_prefix` matches whole COMPONENTS, never bytes, so a sibling
/// whose name merely extends the level's (`/ws-archive` beside `/ws`) keeps its
/// absolute form rather than being sliced into nonsense. A root that strips to
/// NOTHING falls through to the next form rather than becoming an empty row —
/// unreachable in the product, since [`resolve`] refuses the level itself, and
/// the honest degradation if it ever is.
pub(in crate::overlay) fn label_with_home(
    root: &str,
    level: Option<&str>,
    home: Option<&Path>,
) -> String {
    let rp = Path::new(root);
    if let Some(rel) = level.and_then(|l| rp.strip_prefix(l).ok())
        && rel.components().next().is_some()
    {
        return rel.to_string_lossy().to_string();
    }
    if let Some(rel) = home.and_then(|h| rp.strip_prefix(h).ok())
        && rel.components().next().is_some()
    {
        return format!("~/{}", rel.to_string_lossy());
    }
    root.to_string()
}
