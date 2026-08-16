//! src/scenario.rs — the HERMETIC SCENARIO FILESYSTEM: one seam that decides
//! scenario-vs-real fs.
//!
//! A SCENARIO run (today: `--screenshot --keys … --strict-replay`, the strict
//! door the storyboard phases build on) is HERMETIC BY DEFAULT: before its
//! config loads, the process-global filesystem ([`crate::fs::active`]) is
//! swapped to an [`InMemoryFs`] SANDBOX seeded from exactly the inputs the
//! command line names — the launch file's bytes and an explicitly-passed
//! config (`--config` / `$AWL_CONFIG`). Every fs consumer downstream — the
//! config load, the buffer open, the project `.git` probe, the index walk, a
//! replayed save, a History read, a Settings open — reads and writes the
//! sandbox, so a scenario NEVER touches the user's real files (config, notes,
//! history, session, scratch stash, autosave; the crash hook and daemon are
//! live-App-only doors no capture opens). External handoffs (URL open, mailto,
//! Trash, download) are already observed-not-performed via
//! [`crate::replay::classify_for`]'s Intercepted class — together the two seams
//! make a scenario run's only real side effects the PNG + JSON it was asked to
//! write (the harness deliverable itself deliberately bypasses the app seam:
//! `capture` writes it with `std::fs`/`image`, so the sandbox can't swallow
//! the artifact the caller named).
//!
//! The ordinary paths still read their explicitly named file and config, but
//! replay owns no filesystem-write capability: typed Save/Finish requests are
//! skipped and opening an absent config never materializes it. A strict
//! scenario is the only replay door explicitly granted isolated write
//! capability, and that capability targets this sandbox.
//!
//! STRUCTURAL hermeticity: every production call to [`install_hermetic_fs`]
//! lives in ONE function — `args::parse_args`, BEFORE `Config::load` — so
//! "which fs does this run see" is decided in one place, once, and no later fs
//! consumer can dodge the sandbox (they all go through `fs::active()`). Three
//! arms select it today: the strict-replay arm, the storyboard arm, and (item
//! 188) the `--screenshot-app` live-`App` capture, whose claim on the sandbox
//! is the strongest of the three — that mode drives a real `App`, which
//! PERFORMS the writes a replay only records. The `.git` probe rides the same seam
//! (`project::Project::resolve`), so a sandboxed root resolves as non-git and
//! the read-only `git` SUBPROCESSES (`git_branch`/`git_dirty` — the one fs
//! reader that bypasses the trait) are structurally never spawned.
//! `tests/hermetic_canary.rs` proves the whole contract on the real binary:
//! a save-bearing strict scenario under a canary HOME/XDG leaves the canary
//! tree byte-identical.
//!
//! Storyboard seeding (phase 5) extends the SEEDS, not the seam: a storyboard
//! hands [`build_sandbox`] more files (fixtures, config, history) through the
//! same door.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::fs::{FileSystem, InMemoryFs};

/// One seeded file: a storyboard input's path + the bytes it carries into the
/// sandbox. Paths are seeded VERBATIM (the sandbox stores keys as given), so
/// the run resolves them exactly as the CLI spelled them.
pub struct Seed {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

/// Gather the CLI-named storyboard inputs from the REAL disk, exactly once.
/// Deliberately `std::fs` (not the seam): seeding is the one boundary crossing
/// INTO the sandbox, performed before it exists. A missing/unreadable input
/// yields no seed — the scenario then sees an absent file, the same degrade
/// `Buffer::from_file` / `Config::load` give the legacy path.
///
/// `data_seed` is the THIRD slot: a real directory whose files
/// are carried in at awl's OWN [`crate::fs::data_root`] paths — see
/// [`data_root_seeds`] for why that slot has to exist at all.
pub fn cli_seeds(
    file: Option<&Path>,
    config: Option<&Path>,
    data_seed: Option<&Path>,
) -> Vec<Seed> {
    let mut seeds = Vec::new();
    for path in [file, config].into_iter().flatten() {
        if let Ok(bytes) = std::fs::read(path) {
            seeds.push(Seed {
                path: path.to_path_buf(),
                bytes,
            });
        }
    }
    seeds.extend(data_root_seeds(data_seed));
    seeds
}

/// THE DATA-ROOT SEED SLOT: carry the files in `dir` into the sandbox at awl's
/// own machine-state paths — `data_root()/<name>` for each entry.
///
/// # Why this slot exists
///
/// A scenario sandbox is seeded from exactly the paths the command line names,
/// and awl's data root is not one of them. That is fine for a document and a
/// config, and it is a wall for anything whose whole premise is state awl
/// already had: the unresolved-change record, the scratch stash, a session, a
/// history log. The consequence was measured rather than assumed: without this
/// slot, a live-`App` capture of a file with a conflict record beside it under
/// `$XDG_DATA_HOME/awl/` photographs the DISK text and no conflict, because
/// `recovery::read()` looks inside the sandbox and the sandbox has never heard
/// of that path. The store was not merely unseeded — it was unseedable.
///
/// # Why it is an explicit directory and not the machine's real data root
///
/// Seeding the developer's actual data root would make a capture depend on
/// whatever that machine happens to remember, which is the exact property
/// `new_headless_capture` pins `session_restore` off to avoid. The harness names
/// the store, so a capture's starting state is written down in its own command
/// line.
///
/// FLAT by design: entries are taken one directory deep and directories inside
/// are skipped. Every consumer of the data root — `recovery::record_path`,
/// `fs::scratch_stash_path`, `session.rs`, `updates.rs` — puts a plain file
/// directly under it, and a slot that quietly walked deeper would be inventing a
/// layout awl does not have.
pub fn data_root_seeds(dir: Option<&Path>) -> Vec<Seed> {
    let Some(dir) = dir else {
        return Vec::new();
    };
    let root = crate::fs::data_root();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut seeds: Vec<Seed> = entries
        .flatten()
        .filter_map(|e| {
            let from = e.path();
            let bytes = std::fs::read(&from).ok()?;
            Some(Seed {
                path: root.join(from.file_name()?),
                bytes,
            })
        })
        .collect();
    // Deterministic order, so two runs of one command seed identically and a
    // failure is reproducible from the command line alone.
    seeds.sort_by(|a, b| a.path.cmp(&b.path));
    seeds
}

/// How many files a `--seed-tree` directory may carry in, and how many bytes in
/// total. A scenario sandbox is an in-memory map built before the first frame,
/// and a slot that walked an arbitrary directory would let a mistyped path pull
/// a whole home directory into a capture. Both bounds fail LOUDLY (naming the
/// directory and the count) rather than truncating, because a silently trimmed
/// tree photographs a working set that is not the one the command line asked
/// for — the same failure mode `--seed-data`'s refusal-outside-a-hermetic-door
/// exists to prevent.
pub const MAX_TREE_SEED_FILES: usize = 256;
pub const MAX_TREE_SEED_BYTES: u64 = 4 * 1024 * 1024;

/// THE PROJECT-TREE SEED SLOT: carry every file under `dir` into the sandbox at
/// its OWN path, recursively.
///
/// # Why this slot exists
///
/// The other three slots each carry ONE file's worth of premise: a document, a
/// config, a flat data root. Nothing carries a *project* — and a hermetic door
/// therefore could not photograph any state whose premise is several files
/// under one root. `--root` alone seeds a directory MARKER (see
/// [`build_sandbox`]), so a live-`App` capture pointed at a real project sees an
/// empty folder: Go to lists nothing, so nothing can be opened, so a multi-file
/// working set is unreachable at the one door that can witness it. The gap was
/// measured, not assumed — the margin working set is App-owned, tier-1
/// classifies its switching Unsupported, and `--screenshot-app` is hermetic
/// unconditionally.
///
/// # Why the paths are verbatim rather than remapped
///
/// [`data_root_seeds`] remaps because its consumers read awl's own machine-state
/// paths. A project has no such fixed home: the run names the root, the root is
/// what `--root` resolves against, and the sandbox must answer at exactly the
/// paths the command line spelled. So this slot is the verbatim shape the
/// document and config slots already have, extended to a subtree.
///
/// RECURSIVE by design (unlike the flat data root): the working set's whole
/// point is that a file below the root reads by its root-relative path, which
/// needs a real nested tree to be true of. Symlinks are not followed — a link
/// out of the named directory would seed a path the command line never named.
pub fn tree_seeds(dir: Option<&Path>) -> anyhow::Result<Vec<Seed>> {
    let Some(dir) = dir else {
        return Ok(Vec::new());
    };
    let mut seeds = Vec::new();
    let mut bytes: u64 = 0;
    let mut pending = vec![dir.to_path_buf()];
    while let Some(cur) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&cur) else {
            continue;
        };
        for e in entries.flatten() {
            let from = e.path();
            // `symlink_metadata` rather than `metadata`: a followed link is a
            // path outside the named tree wearing a path inside it.
            let Ok(meta) = std::fs::symlink_metadata(&from) else {
                continue;
            };
            if meta.is_symlink() {
                continue;
            }
            if meta.is_dir() {
                pending.push(from);
                continue;
            }
            let Ok(content) = std::fs::read(&from) else {
                continue;
            };
            bytes += content.len() as u64;
            seeds.push(Seed {
                path: from,
                bytes: content,
            });
            if seeds.len() > MAX_TREE_SEED_FILES {
                anyhow::bail!(
                    "--seed-tree {}: more than {} files — name a smaller fixture tree",
                    dir.display(),
                    MAX_TREE_SEED_FILES
                );
            }
            if bytes > MAX_TREE_SEED_BYTES {
                anyhow::bail!(
                    "--seed-tree {}: more than {} bytes — name a smaller fixture tree",
                    dir.display(),
                    MAX_TREE_SEED_BYTES
                );
            }
        }
    }
    // Deterministic order, so two runs of one command seed identically and a
    // failure is reproducible from the command line alone (`read_dir` order is
    // not specified, and the directory stack above visits siblings in whatever
    // order it hands back).
    seeds.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(seeds)
}

/// Build the sandbox: every seed written at its own path (parent dirs implied,
/// exactly like a native write into an existing tree), plus a directory marker
/// per named `root` so `read_dir`/`is_dir` on an explicit `--root` see an
/// (empty) directory rather than an error. Pure over its inputs — the caller
/// decides whether to install it.
pub fn build_sandbox(seeds: &[Seed], roots: &[&Path]) -> InMemoryFs {
    let fs = InMemoryFs::new();
    for r in roots {
        // Infallible on the in-memory backend; `let _` keeps the signature simple.
        let _ = fs.create_dir_all(r);
    }
    for s in seeds {
        let _ = fs.write(&s.path, &s.bytes);
    }
    fs
}

/// THE ONE PRODUCTION DOOR: swap the process-global fs to a hermetic sandbox
/// seeded from the CLI-named inputs. Called once, from `args::parse_args`'s
/// strict-replay arm, BEFORE `Config::load` — so the config itself already
/// loads through the sandbox.
///
/// `config_arg` is the explicit `--config` flag; `$AWL_CONFIG` (the same
/// explicit opt-in `config::config_path` honours, in the same precedence) is
/// folded in here so a deliberately pointed-at test config still reaches the
/// scenario. The user's IMPLICIT `~/.config/awl/config.toml` never does: the
/// sandbox simply has no file at the XDG path, so `Config::load` degrades to
/// pure defaults exactly like a machine with no config.
pub fn install_hermetic_fs(
    file: Option<&Path>,
    config_arg: Option<&Path>,
    root: Option<&Path>,
    data_seed: Option<&Path>,
    tree_seed: Option<&Path>,
) -> anyhow::Result<()> {
    let explicit_config: Option<PathBuf> = config_arg
        .map(Path::to_path_buf)
        .or_else(|| std::env::var_os("AWL_CONFIG").map(PathBuf::from));
    let mut seeds = cli_seeds(file, explicit_config.as_deref(), data_seed);
    // The tree goes in FIRST so a document/config named on the same command line
    // wins on a collision: the CLI file is the one the run is about, and a
    // fixture tree that also contains it must not overwrite the bytes the other
    // slot already read.
    let mut all = tree_seeds(tree_seed)?;
    all.append(&mut seeds);
    let roots: Vec<&Path> = root.into_iter().collect();
    crate::fs::set_active(Arc::new(build_sandbox(&all, &roots)));
    Ok(())
}

// NOTE (phase 5): the storyboard runner reuses THIS same door — its document
// rides the `file` seed slot, and the sandbox's `write` already marks every
// seeded file's parent as a directory, so the runner's root resolution + index
// walk see the document's own directory with no extra marker.

/// WHICH DOCUMENT SEEDS THE SANDBOX for the door that is opening it. A
/// STORYBOARD run seeds the BOARD's own document (`board_file`, already resolved
/// against the board file's own directory) and nothing else — a bare CLI file is
/// not part of that scenario. Every other door — `--strict-replay` and
/// `--screenshot-app` — seeds the CLI-named `cli_file`. One owner rather than a
/// branch at the call site, so the three doors in `args::parse_args` cannot
/// drift on it, and that one call stays a single unconditional statement.
pub fn seed_document<'a>(
    is_storyboard: bool,
    board_file: Option<&'a Path>,
    cli_file: Option<&'a Path>,
) -> Option<&'a Path> {
    if is_storyboard { board_file } else { cli_file }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testscratch::ScratchDir;

    /// A fresh, uniquely-named real tempdir for arranging seed inputs, owned
    /// by a [`ScratchDir`] guard that removes it on drop.
    fn tmp_dir(tag: &str) -> ScratchDir {
        let dir = std::env::temp_dir().join(format!("awl-scenario-{tag}-{}", std::process::id()));
        ScratchDir::new(dir)
    }

    /// **THE DATA-ROOT SEED SLOT**: files named in a real directory
    /// arrive at awl's OWN data-root paths, so `recovery::read()` /
    /// `fs::scratch_stash_path()` / `session.toml` find them where they look.
    ///
    /// The mapping is the whole claim. A slot that seeded the source paths
    /// verbatim — the shape the two existing slots have — would put the record
    /// somewhere nothing reads, and the run would look exactly like an unseeded
    /// one.
    #[test]
    fn a_data_root_seed_lands_at_awls_own_paths() {
        let dir = tmp_dir("data-root");
        std::fs::write(dir.join("unresolved-change.md"), "a record\n").unwrap();
        std::fs::write(dir.join("scratch.md"), "a stash\n").unwrap();
        // A DIRECTORY inside is skipped: every consumer of the data root puts a
        // plain file directly under it, and walking deeper would invent a layout
        // awl does not have.
        std::fs::create_dir_all(dir.join("nested")).unwrap();
        std::fs::write(dir.join("nested").join("deep.md"), "unreachable\n").unwrap();

        let root = crate::fs::data_root();
        let seeds = data_root_seeds(Some(&dir));
        let paths: Vec<&Path> = seeds.iter().map(|s| s.path.as_path()).collect();
        assert_eq!(
            paths,
            vec![
                root.join("scratch.md").as_path(),
                root.join("unresolved-change.md").as_path()
            ],
            "flat, at the data root, and in a deterministic order"
        );
        assert_eq!(seeds[1].bytes, b"a record\n", "bytes carried verbatim");
        // …and the record really is where `recovery` looks for it.
        assert_eq!(seeds[1].path, crate::recovery::record_path());
        assert_eq!(seeds[0].path, crate::fs::scratch_stash_path());

        // ANTI-VACUITY: no directory named, nothing seeded — so an ordinary run
        // is untouched by the slot's existence.
        assert!(data_root_seeds(None).is_empty());
        assert!(
            data_root_seeds(Some(&dir.join("nope"))).is_empty(),
            "a missing directory degrades to no seeds, never an error"
        );
    }

    /// **THE PROJECT-TREE SEED SLOT**: a real nested directory arrives in the
    /// sandbox at its OWN paths, recursively — and the sandbox then answers
    /// `read_dir` for the subfolders, which is the whole point. A capture door
    /// that can only see the root's top level cannot photograph a working set
    /// whose members read by their root-relative path.
    ///
    /// The axis swept here is the one the flat data-root slot does NOT have:
    /// DEPTH. A slot that stopped one directory down (the shape `data_root_seeds`
    /// deliberately has) would seed `notes/a.md` and silently drop
    /// `notes/journal/field-notes.md`, and the capture would show a shorter
    /// working set than the command line asked for with nothing reporting why.
    #[test]
    fn a_tree_seed_carries_a_nested_project_in_at_its_own_paths() {
        let dir = tmp_dir("tree-root");
        let root = dir.join("notes");
        std::fs::create_dir_all(root.join("journal")).unwrap();
        std::fs::create_dir_all(root.join("research").join("sources")).unwrap();
        std::fs::write(root.join("index.md"), "# index\n").unwrap();
        std::fs::write(root.join("journal").join("field-notes.md"), "# field\n").unwrap();
        std::fs::write(
            root.join("research").join("sources").join("deep.md"),
            "# deep\n",
        )
        .unwrap();

        let seeds = tree_seeds(Some(&root)).unwrap();
        let paths: Vec<&Path> = seeds.iter().map(|s| s.path.as_path()).collect();
        assert_eq!(
            paths,
            vec![
                root.join("index.md").as_path(),
                root.join("journal").join("field-notes.md").as_path(),
                root.join("research")
                    .join("sources")
                    .join("deep.md")
                    .as_path(),
            ],
            "every depth, verbatim paths, deterministic order"
        );

        // The claim that matters at a capture door: the SANDBOX lists the
        // subfolders, so Go to can walk into one and open the file below it.
        let fs = build_sandbox(&seeds, &[&root]);
        assert_eq!(
            fs.read_to_string(&root.join("journal").join("field-notes.md"))
                .unwrap(),
            "# field\n",
            "a file two levels down is readable at the path the command line spelled"
        );
        assert!(
            fs.is_dir(&root.join("research").join("sources")),
            "an intermediate directory reads as a directory, so a picker can descend"
        );

        // ANTI-VACUITY: no directory named, nothing seeded — an ordinary run is
        // untouched by the slot's existence.
        assert!(tree_seeds(None).unwrap().is_empty());
        assert!(
            tree_seeds(Some(&root.join("nope"))).unwrap().is_empty(),
            "a missing directory degrades to no seeds, never an error"
        );
    }

    /// The bounds fail LOUDLY rather than truncating. A silently trimmed tree
    /// photographs a working set that is not the one the command line asked for,
    /// and nothing downstream could tell that from a product bug.
    #[test]
    fn a_tree_seed_over_its_file_bound_is_refused_by_name() {
        let dir = tmp_dir("tree-bound");
        let root = dir.join("many");
        std::fs::create_dir_all(&root).unwrap();
        for i in 0..=MAX_TREE_SEED_FILES {
            std::fs::write(root.join(format!("f{i:04}.md")), "x").unwrap();
        }
        let err = match tree_seeds(Some(&root)) {
            Ok(s) => panic!("expected a refusal, got {} seeds", s.len()),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("--seed-tree") && err.contains(&MAX_TREE_SEED_FILES.to_string()),
            "the refusal names the flag and the bound it hit: {err}"
        );
        // …and one file FEWER is accepted, so the bound is the cliff and not a
        // blanket refusal of any sizeable fixture.
        std::fs::remove_file(root.join(format!("f{:04}", MAX_TREE_SEED_FILES) + ".md")).unwrap();
        assert_eq!(
            tree_seeds(Some(&root)).unwrap().len(),
            MAX_TREE_SEED_FILES,
            "exactly at the bound is fine"
        );
    }

    /// The slot composes with the two that were already there rather than
    /// replacing either: a scenario can name a document, a config AND a store.
    #[test]
    fn the_three_seed_slots_compose() {
        let dir = tmp_dir("compose");
        let doc = dir.join("doc.md");
        let cfg = dir.join("cfg.toml");
        let store = dir.join("store");
        std::fs::write(&doc, "# body\n").unwrap();
        std::fs::write(&cfg, "theme = \"Bombora\"\n").unwrap();
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(store.join("unresolved-change.md"), "held\n").unwrap();

        let seeds = cli_seeds(Some(&doc), Some(&cfg), Some(&store));
        assert_eq!(seeds.len(), 3, "document, config, and store");
        let fs = build_sandbox(&seeds, &[]);
        assert_eq!(fs.read_to_string(&doc).unwrap(), "# body\n");
        assert_eq!(
            fs.read_to_string(&crate::recovery::record_path()).unwrap(),
            "held\n",
            "the seeded store is readable at the path awl's own reader uses"
        );
    }

    #[test]
    fn cli_seeds_reads_the_named_inputs_and_skips_missing_ones() {
        let dir = tmp_dir("seeds");
        let doc = dir.join("doc.md");
        let cfg = dir.join("cfg.toml");
        std::fs::write(&doc, "# body\n").unwrap();
        std::fs::write(&cfg, "theme = \"Bombora\"\n").unwrap();

        // Both present: two seeds, verbatim bytes, in (file, config) order.
        let seeds = cli_seeds(Some(&doc), Some(&cfg), None);
        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0].path, doc);
        assert_eq!(seeds[0].bytes, b"# body\n");
        assert_eq!(seeds[1].path, cfg);
        assert_eq!(seeds[1].bytes, b"theme = \"Bombora\"\n");

        // A missing input yields NO seed (the scenario sees an absent file —
        // the same degrade the legacy path gives), never an error.
        let missing = dir.join("nope.md");
        assert!(cli_seeds(Some(&missing), None, None).is_empty());
        assert_eq!(cli_seeds(None, Some(&cfg), None).len(), 1);
        assert!(cli_seeds(None, None, None).is_empty());
    }

    #[test]
    fn sandbox_contains_exactly_the_seeds_and_the_root_marker() {
        let doc = PathBuf::from("/proj/doc.md");
        let seeds = vec![Seed {
            path: doc.clone(),
            bytes: b"alpha\n".to_vec(),
        }];
        let root = PathBuf::from("/proj");
        let fs = build_sandbox(&seeds, &[&root]);
        // The seed is readable at its verbatim path; its parent doubles as the
        // (marked) root dir, so the index walk sees exactly the seeded input.
        assert_eq!(fs.read_to_string(&doc).unwrap(), "alpha\n");
        assert!(fs.is_dir(&root), "the named root is a directory");
        let names: Vec<String> = fs
            .read_dir(&root)
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert_eq!(names, vec!["doc.md".to_string()]);
        // NOTHING else: the user-config shape of path is absent, so a config
        // load inside the sandbox degrades to pure defaults.
        fs.read_to_string(Path::new("/home/u/.config/awl/config.toml"))
            .unwrap_err();
        // And the root carries no `.git`, so `Project::resolve` classifies it
        // non-git and never spawns the read-only git subprocesses.
        assert!(!fs.exists(&root.join(".git")));
    }

    #[test]
    fn seeded_documents_parent_reads_as_a_directory_with_no_extra_marker() {
        // The storyboard runner resolves its project root from the seeded
        // document's own directory — the sandbox's `write` marks every seeded
        // file's ancestors as dirs, so no storyboard-specific door is needed.
        let doc = PathBuf::from("scenarios/demo.md");
        let seeds = vec![Seed {
            path: doc.clone(),
            bytes: b"seeded\n".to_vec(),
        }];
        let fs = build_sandbox(&seeds, &[]);
        assert!(
            fs.is_dir(Path::new("scenarios")),
            "parent implied by the seed write"
        );
        let names: Vec<String> = fs
            .read_dir(Path::new("scenarios"))
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert_eq!(names, vec!["demo.md".to_string()]);
    }

    #[test]
    fn install_hermetic_fs_swaps_the_active_backend_to_the_seeded_sandbox() {
        let dir = tmp_dir("install");
        let doc = dir.join("doc.md");
        std::fs::write(&doc, "real bytes\n").unwrap();
        // FsGuard::capture() restores whatever `install_hermetic_fs` swaps in
        // — even on a failed assert — so no sibling test ever sees the sandbox.
        // `capture()` rather than `install(fs::active())`: the argument form
        // read the global BEFORE taking the guard.
        let _restore = crate::fs::FsGuard::capture();
        install_hermetic_fs(Some(&doc), None, Some(&dir), None, None).unwrap();
        // The active backend now serves the seeded copy…
        assert_eq!(
            crate::fs::active().read_to_string(&doc).unwrap(),
            "real bytes\n"
        );
        // …and a write through the seam lands in the sandbox, NEVER on disk.
        crate::fs::active().write(&doc, b"sandbox edit\n").unwrap();
        assert_eq!(
            crate::fs::active().read_to_string(&doc).unwrap(),
            "sandbox edit\n"
        );
        assert_eq!(
            std::fs::read_to_string(&doc).unwrap(),
            "real bytes\n",
            "the REAL file keeps every byte"
        );
    }
}
