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
