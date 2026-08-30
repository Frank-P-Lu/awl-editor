use super::super::*;
use super::note_tmp;
use crate::buffers::BufferKey;

// --- QUICK NOTE: title slug, collision suffixing, auto-name on save --------

#[test]
fn note_stem_titles() {
    assert_eq!(note_stem("Japanese week 12"), "japanese-week-12");
    assert_eq!(note_stem("  Hello,  World!  "), "hello-world");
    assert_eq!(note_stem("UPPER Case"), "upper-case");
    // Punctuation-only / empty -> the "scratch" fallback.
    assert_eq!(note_stem("!!!"), "scratch");
    assert_eq!(note_stem(""), "scratch");
}

#[test]
fn real_disk_long_first_line_saves_below_name_max() {
    let _guard = crate::testlock::serial();
    let notes = note_tmp("long_first_line_real_disk");
    let mut buf = Buffer::scratch();
    buf.set_text(&"word ".repeat(70));

    buf.save_into_folder(&notes).unwrap();
    let name = buf.path().unwrap().file_name().unwrap().to_string_lossy();
    assert!(name.len() <= NOTE_STEM_MAX_BYTES + 3, "{name}");
    assert_eq!(
        std::fs::read_to_string(buf.path().unwrap()).unwrap(),
        "word ".repeat(70)
    );
}

#[test]
fn real_disk_scratch_fallback_never_clobbers_an_existing_scratch_md() {
    let _guard = crate::testlock::serial();
    let notes = note_tmp("scratch_collision_real_disk");
    std::fs::write(notes.join("scratch.md"), "existing\n").unwrap();
    let mut buf = Buffer::scratch();
    buf.set_text("!!!");

    buf.save_into_folder(&notes).unwrap();

    assert_eq!(buf.path().unwrap().file_name().unwrap(), "scratch-2.md");
    assert_eq!(
        std::fs::read_to_string(notes.join("scratch.md")).unwrap(),
        "existing\n"
    );
    assert_eq!(std::fs::read_to_string(buf.path().unwrap()).unwrap(), "!!!");
}

#[test]
fn failed_naming_write_leaves_every_identity_field_unchanged() {
    use std::sync::Arc;
    let notes = std::path::PathBuf::from("/notes");
    crate::fs::with_fs(Arc::new(crate::fs::UnwritableFs), || {
        let mut buf = Buffer::scratch();
        buf.start_fresh_doc(notes.clone());
        buf.set_text("irreplaceable prose");
        let before = (
            buf.path.clone(),
            buf.note_dir.clone(),
            buf.fresh_id,
            BufferKey::of(&buf),
            buf.is_dirty(),
            buf.text(),
        );

        assert!(buf.save().is_err());

        assert_eq!(
            (
                buf.path.clone(),
                buf.note_dir.clone(),
                buf.fresh_id,
                BufferKey::of(&buf),
                buf.is_dirty(),
                buf.text(),
            ),
            before,
            "a failed naming write cannot partially commit identity or text state"
        );
    });
}

#[test]
fn note_stem_cap_is_byte_aware_and_never_leaves_a_dash() {
    let lines = vec![
        "alpha beta gamma delta ".repeat(20),
        "日本語の長い段落".repeat(30),
        "a-".repeat(100),
        "!!!".to_string(),
    ];
    for line in lines {
        let stem = note_stem(&line);
        assert!(
            stem.len() <= NOTE_STEM_MAX_BYTES,
            "{stem:?} is {} bytes",
            stem.len()
        );
        assert!(!stem.ends_with('-'), "{stem:?}");
        assert!(stem.is_char_boundary(stem.len()));
    }
}

#[test]
fn stem_budget_measures_atomic_collision_and_quarantine_headroom() {
    const NAME_MAX_FLOOR: usize = 255;
    const MD_EXTENSION: usize = ".md".len();
    const MAX_U32_COLLISION: usize = "-4294967295".len();
    const ATOMIC_DECORATION: usize = ".".len() + ".awl-tmp".len();
    const QUARANTINE_RESERVE: usize = 64;
    let worst = NOTE_STEM_MAX_BYTES
        + MD_EXTENSION
        + MAX_U32_COLLISION
        + ATOMIC_DECORATION
        + QUARANTINE_RESERVE;
    assert_eq!(worst, 159, "keep the stated budget arithmetic honest");
    assert_eq!(NAME_MAX_FLOOR - worst, 96, "measured portable headroom");
}

#[test]
fn successful_naming_preserves_the_common_save_postconditions() {
    let _guard = crate::testlock::serial();
    let notes = note_tmp("naming_postconditions");
    let mut buf = Buffer::scratch();
    buf.start_fresh_doc(notes.to_path_buf());
    buf.set_text("Title\nline two\n");
    buf.set_eol(Eol::Crlf);
    let version = buf.version();
    let disk_bytes = buf.disk_bytes();

    buf.save().unwrap();

    let path = buf.path().expect("successful naming binds a path");
    assert_eq!(path.parent(), Some(notes.as_ref()));
    assert_eq!(path.file_name().unwrap(), "title.md");
    assert_eq!(std::fs::read(path).unwrap(), disk_bytes, "EOL encoding");
    assert_eq!(buf.text(), "Title\nline two\n", "rope text is unchanged");
    assert_eq!(buf.version(), version, "save is not an edit");
    assert!(!buf.is_dirty(), "successful common save marks clean");
    assert!(!buf.is_unnamed_fresh(), "note marker is retired");
    assert_eq!(buf.fresh_id(), None, "provisional identity is retired");
    assert!(matches!(BufferKey::of(&buf), BufferKey::Path(_)));
}

#[test]
fn first_nonempty_line_skips_blanks() {
    assert_eq!(
        first_nonempty_line("\n\n  \nReal title\nmore"),
        Some("Real title")
    );
    assert_eq!(first_nonempty_line("   \n\t"), None);
    assert_eq!(first_nonempty_line(""), None);
}

#[test]
fn unique_path_suffixes_on_collision() {
    // unique_path probes existence through the FILESYSTEM SEAM, so drive it with
    // an InMemoryFs (no temp dir).
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let dir = std::path::PathBuf::from("/notes");
    let mem = crate::fs::InMemoryFs::new().with_dir(&dir);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        // First is the bare name; once it exists, the next is -2, then -3.
        let p1 = unique_path(&dir, "japanese-week-12", "md");
        assert_eq!(p1.file_name().unwrap(), "japanese-week-12.md");
        mem.write(&p1, b"x").unwrap();
        let p2 = unique_path(&dir, "japanese-week-12", "md");
        assert_eq!(p2.file_name().unwrap(), "japanese-week-12-2.md");
        mem.write(&p2, b"x").unwrap();
        let p3 = unique_path(&dir, "japanese-week-12", "md");
        assert_eq!(p3.file_name().unwrap(), "japanese-week-12-3.md");
    });
}

#[test]
fn note_save_derives_filename_from_first_line() {
    // The quick-note save path (slug derivation + collision suffix + filename
    // lock), routed through the FILESYSTEM SEAM (InMemoryFs) — no temp dir.
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let dir = std::path::PathBuf::from("/notes");
    let mem = crate::fs::InMemoryFs::new().with_dir(&dir);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        // An EMPTY note writes nothing (no litter): save bails.
        let mut buf = Buffer::scratch();
        buf.set_note_dir(dir.clone());
        assert!(buf.is_unnamed_fresh());
        assert!(buf.save().is_err());
        assert!(buf.path().is_none());
        // Type a title; save now DERIVES <slug>.md and writes it.
        for c in "Japanese week 12".chars() {
            buf.insert_char(c);
        }
        buf.save().unwrap();
        let p = buf.path().unwrap().to_path_buf();
        assert_eq!(p.file_name().unwrap(), "japanese-week-12.md");
        assert!(mem.exists(&p));
        // Filename LOCKS: editing the first line + re-saving keeps the same path.
        buf.buffer_start();
        for c in "X ".chars() {
            buf.insert_char(c);
        }
        buf.save().unwrap();
        assert_eq!(
            buf.path().unwrap(),
            p,
            "filename must lock after first save"
        );
        // A SECOND fresh note with the same title collides -> -2 suffix.
        let mut buf2 = Buffer::scratch();
        buf2.set_note_dir(dir.clone());
        for c in "Japanese week 12".chars() {
            buf2.insert_char(c);
        }
        buf2.save().unwrap();
        assert_eq!(
            buf2.path().unwrap().file_name().unwrap(),
            "japanese-week-12-2.md"
        );
    });
}

#[test]
fn display_name_for_gutter_saved_derived_and_scratch() {
    // A SAVED file shows its bound file name.
    let mut saved = Buffer::scratch();
    saved.set_path(std::path::PathBuf::from("/tmp/notes/today.md"));
    assert_eq!(saved.display_name(), "today.md");
    // An UNSAVED note shows the name it WOULD derive on first save (<slug>.md).
    let note = Buffer::from_str("Grocery list\nmilk\n");
    assert_eq!(note.display_name(), "grocery-list.md");
    // An untitled / empty buffer falls back to the scratch placeholder.
    let blank = Buffer::scratch();
    assert_eq!(blank.display_name(), "scratch.md");
}

#[test]
fn note_one_word_first_line_names_file() {
    // A single-word first line yields <word>.md (no dash, no fallback).
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let dir = std::path::PathBuf::from("/notes");
    let mem = crate::fs::InMemoryFs::new().with_dir(&dir);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::scratch();
        buf.set_note_dir(dir.clone());
        for c in "foo".chars() {
            buf.insert_char(c);
        }
        buf.save().unwrap();
        assert_eq!(buf.path().unwrap().file_name().unwrap(), "foo.md");
        assert!(mem.exists(buf.path().unwrap()));
    });
}

#[test]
fn note_empty_writes_no_file() {
    // A truly empty note (only whitespace) NEVER writes — no litter.
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let dir = std::path::PathBuf::from("/notes");
    let mem = crate::fs::InMemoryFs::new().with_dir(&dir);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::scratch();
        buf.set_note_dir(dir.clone());
        for c in "   \n\t  ".chars() {
            buf.insert_char(c);
        }
        assert!(buf.save().is_err());
        assert!(buf.path().is_none());
        // Nothing landed in the fake filesystem.
        let count = mem.read_dir(&dir).map(|d| d.len()).unwrap_or(0);
        assert_eq!(count, 0, "empty note must not write a file");
    });
}

#[test]
fn note_content_without_title_falls_back_to_scratch() {
    // A first line with content but NO derivable title (punctuation only)
    // falls back to scratch.md, then scratch-2.md on the next such note.
    use std::sync::Arc;
    let dir = std::path::PathBuf::from("/notes");
    let mem = crate::fs::InMemoryFs::new().with_dir(&dir);
    crate::fs::with_fs(Arc::new(mem), || {
        let mut buf = Buffer::scratch();
        buf.set_note_dir(dir.clone());
        for c in "!!!".chars() {
            buf.insert_char(c);
        }
        buf.save().unwrap();
        assert_eq!(buf.path().unwrap().file_name().unwrap(), "scratch.md");
        // A second untitled-content note collides -> scratch-2.md.
        let mut buf2 = Buffer::scratch();
        buf2.set_note_dir(dir.clone());
        for c in "???".chars() {
            buf2.insert_char(c);
        }
        buf2.save().unwrap();
        assert_eq!(buf2.path().unwrap().file_name().unwrap(), "scratch-2.md");
    });
}

// --- SAVE-FEEDBACK round: `Buffer::save_into_folder` (scratch -> note on manual save) ---

#[test]
fn save_as_note_converts_a_true_scratch_buffer_and_writes_it() {
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let notes = std::path::PathBuf::from("/notes");
    let mem = crate::fs::InMemoryFs::new(); // notes dir does NOT exist yet
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::scratch();
        for c in "brain dump".chars() {
            buf.insert_char(c);
        }
        assert!(
            !buf.is_unnamed_fresh(),
            "a true scratch buffer starts as no note"
        );
        buf.save_into_folder(&notes).unwrap();
        // ONE-SHOT NAMING: the SAME call that derives the name also
        // clears the fresh-document marker — an ordinary pathed file from here.
        assert!(
            !buf.is_unnamed_fresh(),
            "named once — an ordinary file now, not a lasting note identity"
        );
        let p = buf.path().unwrap();
        assert_eq!(p.file_name().unwrap(), "brain-dump.md");
        assert!(p.starts_with(&notes));
        assert!(mem.exists(p), "the folder was created and the file written");
    });
}

#[test]
fn save_as_note_second_call_is_a_plain_save_same_path() {
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let notes = std::path::PathBuf::from("/notes");
    let mem = crate::fs::InMemoryFs::new().with_dir(&notes);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::scratch();
        for c in "first draft".chars() {
            buf.insert_char(c);
        }
        buf.save_into_folder(&notes).unwrap();
        let named = buf.path().unwrap().to_path_buf();
        for c in " continued".chars() {
            buf.insert_char(c);
        }
        // A SECOND save (now that it's a real note) never re-derives or
        // re-homes the filename — it's a plain `save()` at the same path.
        buf.save_into_folder(&notes).unwrap();
        assert_eq!(buf.path().unwrap(), named);
        assert_eq!(mem.read_to_string(&named).unwrap(), "first draft continued");
    });
}

#[test]
fn one_shot_naming_a_later_first_line_edit_never_renames() {
    // The one-shot naming law: `Buffer::save` derives the filename
    // from the first line EXACTLY ONCE. Editing the first line AFTER that
    // first save — even before a second save — never re-derives or
    // renames the file; the old LIVE-rename-to-title behavior is retired.
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let dir = std::path::PathBuf::from("/docs");
    let mem = crate::fs::InMemoryFs::new().with_dir(&dir);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::scratch();
        buf.start_fresh_doc(dir.clone());
        assert!(buf.is_unnamed_fresh(), "arranged: a fresh unnamed document");
        buf.set_text("first title");
        buf.save().unwrap();
        let named = buf.path().unwrap().to_path_buf();
        assert_eq!(named.file_name().unwrap(), "first-title.md");
        assert!(
            !buf.is_unnamed_fresh(),
            "one-shot: named once, ordinary now"
        );

        // Retitle the FIRST LINE entirely, then save again.
        buf.set_text("totally different title");
        buf.save().unwrap();

        assert_eq!(
            buf.path().unwrap(),
            named,
            "the path is UNCHANGED — a later title edit never renames"
        );
        assert!(
            !mem.exists(&dir.join("totally-different-title.md")),
            "no new file was ever created for the retitled first line"
        );
        assert_eq!(
            mem.read_to_string(&named).unwrap(),
            "totally different title",
            "the CONTENT still updated at the original path"
        );
    });
}

#[test]
fn save_as_note_already_a_note_is_untouched_by_the_conversion_step() {
    // A buffer that is ALREADY a note (e.g. C-x n) keeps its OWN note_dir —
    // `save_into_folder` must never re-home it at the passed-in folder.
    use std::sync::Arc;
    let own_dir = std::path::PathBuf::from("/project/scratch-notes");
    let other_folder = std::path::PathBuf::from("/notes");
    let mem = crate::fs::InMemoryFs::new()
        .with_dir(&own_dir)
        .with_dir(&other_folder);
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::scratch();
        buf.start_fresh_doc(own_dir.clone());
        for c in "already a note".chars() {
            buf.insert_char(c);
        }
        buf.save_into_folder(&other_folder).unwrap();
        assert!(
            buf.path().unwrap().starts_with(&own_dir),
            "kept its own note home"
        );
    });
}

/// A minimal [`crate::fs::FileSystem`] fake whose `write` ALWAYS fails —
/// standing in for a target folder that exists but isn't writable (a full
/// disk, a permissions error, …). `InMemoryFs` has no such mode (every
/// write always succeeds), so this is the smallest fake that can exercise
/// the failure path `Buffer::save`'s `write_atomic` call can genuinely
/// take. Every other method is a total no-op / `NotFound` — nothing this
/// test needs reads through them.

#[test]
fn save_as_note_unwritable_folder_surfaces_as_an_err_never_panics() {
    // A target folder that exists but can't be WRITTEN to surfaces the
    // failure as the same `Err` `save` already returns — the caller
    // (`App::convert_scratch_and_save`) turns it into a calm notice,
    // never a terminal print, never a panic.
    use std::sync::Arc;
    let notes = std::path::PathBuf::from("/notes");
    crate::fs::with_fs(Arc::new(crate::fs::UnwritableFs), || {
        let mut buf = Buffer::scratch();
        for c in "will not land".chars() {
            buf.insert_char(c);
        }
        assert!(buf.save_into_folder(&notes).is_err());
    });
}

#[test]
fn move_file_relocates_and_no_clobbers() {
    // The C-x m move (true rename + no-clobber + buffer re-point + save at new
    // home), all over the FILESYSTEM SEAM (InMemoryFs) — no real disk.
    use crate::fs::FileSystem;
    use std::sync::Arc;
    let root = std::path::PathBuf::from("/notes");
    let sub = root.join("archive");
    let old = root.join("idea.md");
    let mem = crate::fs::InMemoryFs::new()
        .with_dir(&sub)
        .with_file(&old, "body");
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        // A note at the root, opened into a buffer.
        let mut buf = Buffer::from_file(&old);
        // MOVE into archive/: a true rename — old path gone, new path present.
        let new = move_file(&old, &sub).unwrap();
        assert_eq!(new, sub.join("idea.md"));
        assert!(!mem.exists(&old), "old path must be gone after a move");
        assert!(mem.exists(&new), "new path must exist after a move");
        // The buffer re-points so future saves land at the new home.
        buf.set_path(new.clone());
        assert_eq!(buf.path().unwrap(), new);
        buf.insert_char('!');
        buf.save().unwrap();
        assert_eq!(mem.read_to_string(&new).unwrap(), "!body");
        // NO CLOBBER: moving a second `idea.md` into archive/ suffixes it.
        let other = root.join("idea.md");
        mem.write(&other, b"two").unwrap();
        let new2 = move_file(&other, &sub).unwrap();
        assert_eq!(new2.file_name().unwrap(), "idea-2.md");
        assert!(mem.exists(&new2) && !mem.exists(&other));
    });
}
