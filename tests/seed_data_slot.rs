//! tests/seed_data_slot.rs — THE DATA-ROOT SEED SLOT, on the real binary.
//!
//! `docs/harness-reach.md` records that **no
//! capture tier could reach an external-change conflict**. Tier 1 never builds a
//! disk baseline; tier 2 cannot raise a conflict mid-run, because the change has
//! to come from outside awl and a capture drives chords only; and it could not
//! START conflicted either, because `--screenshot-app`'s hermetic sandbox was
//! seeded from exactly two CLI paths and awl's own data root was not one of them.
//! The store was not merely unseeded — it was **unseedable**. Slice 1 drove that
//! for real: a record placed beside a diverging file photographed the DISK text
//! and no conflict.
//!
//! `--seed-data DIR` is the narrowing that opens it (`scenario::data_root_seeds`).
//! This file proves it on the REAL binary, through the real
//! `parse_args` → sandbox-install → config-load → `App::new` → capture pipeline,
//! and — the half that makes the other half mean something — proves the SAME
//! command without the flag still cannot see the conflict.

use std::path::{Path, PathBuf};

mod common;
use common::ScratchDir;

/// What the file says. Deliberately the same byte length as [`MINE`], so nothing
/// here can pass on a stat comparison rather than on content.
const DISK: &str = "somebody else typed this\n";
/// What awl was holding when it was killed.
const MINE: &str = "what I had typed inste\n";

fn tmp_dir(tag: &str) -> ScratchDir {
    ScratchDir::claim(
        &std::env::temp_dir(),
        &format!("awl-seed-data-{tag}-{}", std::process::id()),
    )
}

/// A document that has moved on disk, plus a seed directory holding the
/// unresolved-change record that belongs to it — exactly the state a relaunch
/// after a kill would find.
fn arrange(dir: &Path) -> (PathBuf, PathBuf) {
    let doc = dir.join("draft.md");
    std::fs::write(&doc, DISK).unwrap();
    let seed = dir.join("seed");
    std::fs::create_dir_all(&seed).unwrap();
    std::fs::write(
        seed.join("unresolved-change.md"),
        format!("awl-unresolved-change 1\n{}\n{MINE}", doc.display()),
    )
    .unwrap();
    (doc, seed)
}

fn run(home: &Path, args: &[&str]) -> std::process::Output {
    let cache = home.parent().unwrap().join("xdg-cache");
    std::fs::create_dir_all(&cache).unwrap();
    common::awl_in_home(home)
        .args(args)
        .env("XDG_DATA_HOME", home.join(".local").join("share"))
        .env("XDG_CACHE_HOME", &cache)
        .env("AWL_CONVENTION_FORCE", "mac")
        .output()
        .expect("failed to spawn the awl binary under CARGO_BIN_EXE_awl")
}

fn run_ok(home: &Path, args: &[&str]) {
    let out = run(home, args);
    assert!(
        out.status.success(),
        "awl {args:?} failed: {}\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn sidecar(png: &Path) -> serde_json::Value {
    let json = std::fs::read_to_string(png.with_extension("json")).expect("sidecar exists");
    serde_json::from_str(&json).expect("sidecar parses")
}

/// **THE SLOT OPENS THE STATE — and the same run without it still cannot.**
///
/// Both arms are one test on purpose. An arm that only showed the seeded run
/// reaching a conflict would not distinguish "the slot works" from "this
/// document was conflicted anyway"; the unseeded arm is the exact measurement
/// slice 1 recorded, kept as the anti-vacuity half.
#[test]
fn a_seeded_data_root_starts_a_live_app_capture_already_conflicted() {
    let root = tmp_dir("reach");
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let (doc, seed) = arrange(&root);

    // ── WITH the slot: the record is found, so the App holds the user's text
    //    and the guard has latched against the file that moved underneath it.
    let seeded = root.join("seeded.png");
    run_ok(
        &home,
        &[
            "--screenshot-app",
            seeded.to_str().unwrap(),
            doc.to_str().unwrap(),
            "--seed-data",
            seed.to_str().unwrap(),
        ],
    );
    let json = sidecar(&seeded);
    assert_eq!(json["driver"].as_str(), Some("live-app"));
    assert_eq!(
        json["text"].as_str(),
        Some(MINE),
        "the capture starts from the RECORD's text, not the disk's"
    );
    assert_eq!(
        json["gutter"]["changed"].as_bool(),
        Some(true),
        "…and the persistent `changed elsewhere` affordance is up, which is the \
         state oracle this slot was added to make reachable"
    );
    assert_eq!(
        json["gutter"]["name"].as_str(),
        Some("dr….md"),
        "the affordance really is beside this document's elided filename at the default zoom"
    );

    // ── WITHOUT the slot: byte-identical command, and the conflict is
    //    structurally out of reach. This is slice 1's own measurement.
    let bare = root.join("bare.png");
    run_ok(
        &home,
        &[
            "--screenshot-app",
            bare.to_str().unwrap(),
            doc.to_str().unwrap(),
        ],
    );
    let bare_json = sidecar(&bare);
    assert_eq!(
        bare_json["text"].as_str(),
        Some(DISK),
        "with no seeded store the capture photographs the DISK text — if this \
         ever reads the held text, the arm above has stopped proving anything"
    );
    assert_eq!(
        bare_json["gutter"]["changed"].as_bool(),
        Some(false),
        "…and no conflict, exactly as item 204 slice 1 measured"
    );
}

/// **THE SURFACE THE SLOT WAS OPENED FOR.** From that seeded conflict, the
/// palette's gated "Review the change" row opens the conflict workspace, and
/// each of its three rows serves ITS OWN view — read straight out of the
/// sidecar's `overlay.preview_view` and the previewed document text.
#[test]
fn the_conflict_workspace_and_its_three_views_are_photographable() {
    let root = tmp_dir("views");
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let (doc, seed) = arrange(&root);

    // (view tag, a fragment the transcript must carry, one it must NOT)
    let expected: [(&str, &str, Option<&str>); 3] = [
        ("diff", "Differences", None),
        ("mine", "Your version", Some("somebody else")),
        ("theirs", "Version on disk", Some("I had typed")),
    ];
    for (row, (tag, title, forbidden)) in expected.iter().enumerate() {
        let png = root.join(format!("view-{row}.png"));
        let downs = std::iter::repeat_n("Down", row)
            .collect::<Vec<_>>()
            .join(" ");
        let keys = format!("s-p R e v i e w Enter {downs}");
        run_ok(
            &home,
            &[
                "--screenshot-app",
                png.to_str().unwrap(),
                doc.to_str().unwrap(),
                "--seed-data",
                seed.to_str().unwrap(),
                "--keys",
                keys.trim(),
            ],
        );
        let json = sidecar(&png);
        let overlay = &json["overlay"];
        assert_eq!(
            overlay["mode"].as_str(),
            Some("conflict"),
            "row {row}: the gated palette row opens the conflict workspace"
        );
        assert_eq!(
            overlay["title"].as_str(),
            Some("changed elsewhere"),
            "row {row}: and it titles itself with the affordance's own words"
        );
        assert_eq!(
            overlay["workspace"].as_bool(),
            Some(true),
            "row {row}: it is drawn as a relocated workspace, not a card"
        );
        assert_eq!(
            overlay["items"].as_array().map(|a| a.len()),
            Some(3),
            "row {row}: three views, one at a time"
        );
        assert_eq!(
            overlay["selected_index"].as_u64(),
            Some(row as u64),
            "row {row}: the walk landed where it meant to"
        );
        assert_eq!(
            overlay["preview_view"].as_str(),
            Some(*tag),
            "row {row}: the sidecar names the VIEW, which is the one fact that \
             distinguishes three previews of one subject"
        );
        assert_eq!(
            overlay["preview_id"].as_str(),
            Some(doc.to_string_lossy().as_ref()),
            "row {row}: …of this file"
        );
        let text = json["text"].as_str().expect("previewed document text");
        assert!(
            text.starts_with(&format!("# {title}")),
            "row {row}: the previewed prose names itself — got {:?}",
            text.lines().next()
        );
        if let Some(absent) = forbidden {
            assert!(
                !text.contains(absent),
                "row {row}: a whole-version view must show ONE manuscript, but it \
                 carried {absent:?}: {text}"
            );
        }
    }

    // AND THE FILE WAS NEVER WRITTEN. Previews are read-only; a capture that
    // drove them must leave the user's document exactly as it found it.
    assert_eq!(
        std::fs::read_to_string(&doc).unwrap(),
        DISK,
        "reading the two versions may never write either of them"
    );
}

/// The slot means something only on a hermetic door, so naming it anywhere else
/// is REFUSED rather than silently ignored: a run that named a store and did not
/// get one would photograph the wrong starting state and read as a product bug.
#[test]
fn seed_data_is_refused_where_it_would_do_nothing() {
    let root = tmp_dir("refuse");
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let (doc, seed) = arrange(&root);
    let png = root.join("ordinary.png");
    let out = run(
        &home,
        &[
            "--screenshot",
            png.to_str().unwrap(),
            doc.to_str().unwrap(),
            "--seed-data",
            seed.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "an ordinary capture must refuse --seed-data rather than ignore it"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--seed-data"),
        "…and the refusal must name the flag: {stderr}"
    );
}

/// HERMETICITY IS NOT WEAKENED BY THE SLOT. The seeded store lives in the
/// sandbox; awl's REAL data root under the canary home must be untouched, even
/// though this run adopted a recovery record and opened a workspace over it.
#[test]
fn seeding_a_store_never_writes_the_real_one() {
    let root = tmp_dir("canary");
    let home = root.join("home");
    let real_store = home.join(".local").join("share").join("awl");
    std::fs::create_dir_all(&real_store).unwrap();
    // Bait: a record for a DIFFERENT file, which a leaky implementation would
    // either read (and adopt) or overwrite.
    let bait = "awl-unresolved-change 1\n/somewhere/else.md\nbait\n";
    std::fs::write(real_store.join("unresolved-change.md"), bait).unwrap();

    let (doc, seed) = arrange(&root);
    let png = root.join("canary.png");
    run_ok(
        &home,
        &[
            "--screenshot-app",
            png.to_str().unwrap(),
            doc.to_str().unwrap(),
            "--seed-data",
            seed.to_str().unwrap(),
            "--keys",
            "s-p R e v i e w Enter",
        ],
    );
    assert_eq!(
        std::fs::read_to_string(real_store.join("unresolved-change.md")).unwrap(),
        bait,
        "the real store was written"
    );
    // The bait record names a DIFFERENT file, so a run that read the REAL store
    // would find nothing for this document, latch no conflict, and hide the
    // gated palette row — the workspace simply would not open.
    let json = sidecar(&png);
    assert_eq!(
        json["overlay"]["mode"].as_str(),
        Some("conflict"),
        "…and the SEEDED record is the one that was read, not the real one"
    );
    assert!(
        json["text"]
            .as_str()
            .is_some_and(|t| t.contains(MINE.trim())),
        "…and the prose it opened on is about the seeded text: {:?}",
        json["text"].as_str()
    );
}
