//! src/app/files/dictionary.rs — spell-check DICTIONARY switching, the
//! PERSONAL (user-added) dictionary file (load at startup, "Add to
//! dictionary"), and the CJK ambiguity-ladder persist. Peeled out of
//! `files/settings.rs` to stay under the ~500-line ceiling (item 56).

use crate::app::*;

impl App {

    /// Persist the now-active DICTIONARY variant (write-on-change after the
    /// Dictionary picker commits).
    pub(in crate::app) fn persist_dictionary(&mut self) {
        let name = crate::config::dictionary_name(crate::spell::active_variant());
        self.persist_pref("dictionary", &format!("\"{name}\""));
    }


    /// Persist the now-active CJK ambiguity LADDER (write-on-change after the
    /// CJK-priority language picker commits) — mirrors `persist_dictionary`,
    /// except the value is a whole ORDERED LIST rather than one scalar: the
    /// core already promoted + set the live global
    /// (`frontmatter::set_cjk_priority`), so this just formats it as a TOML
    /// array RHS and writes it through the same format-preserving `write_pref`
    /// (which only cares that `value` is an already-formatted RHS — an array
    /// upserts exactly like a string/bool/number). The config file keeps the
    /// FULL ordered list (not just the promoted front), so hand-editing and an
    /// old config both keep working unchanged.
    pub(in crate::app) fn persist_cjk_priority(&mut self) {
        let ladder = crate::frontmatter::cjk_priority();
        let quoted: Vec<String> = ladder.iter().map(|l| format!("\"{}\"", l.code())).collect();
        self.persist_pref("cjk_priority", &format!("[{}]", quoted.join(", ")));
    }


    /// SWITCH the active spell-check dictionary: reconstruct the App's
    /// [`crate::spell::SpellChecker`] for `variant` (the ONE real per-switch cost —
    /// timed + reported here, so a live switch's latency is observable), then
    /// INVALIDATE the squiggle cache (`spell_checked_version`) and recompute
    /// IMMEDIATELY — a discrete picker commit deserves instant feedback —
    /// before persisting the sticky pref. A failed parse disables spell-check
    /// (reported to stderr), exactly like the `App::new` startup path.
    pub(in crate::app) fn set_dictionary(&mut self, variant: crate::spell::DictVariant) {
        let t0 = std::time::Instant::now();
        self.spell = match crate::spell::SpellChecker::new(variant) {
            Ok(sc) => Some(sc),
            Err(e) => {
                eprintln!("dictionary switch failed: {e}");
                None
            }
        };
        eprintln!(
            "dictionary switched to {}: parsed in {:.2}ms",
            crate::config::dictionary_name(variant),
            t0.elapsed().as_secs_f64() * 1000.0
        );
        // CACHE-KEY DISCIPLINE: `spell_checked_version` gates on the BUFFER's
        // version alone, which the dictionary switch never bumps — so without this
        // reset the stale cache would look "current" until the next edit. Clearing
        // it forces `run_spellcheck_now` to actually re-scan against the new
        // dictionary right away.
        self.active.extra.spell_checked_version = None;
        // Re-fold the user (personal) dictionary onto the FRESH checker: the switch
        // reconstructed it with an empty personal set, so without this the words the
        // user added would stop suppressing squiggles until the next launch.
        self.load_user_dictionary();
        self.run_spellcheck_now();
        self.persist_dictionary();
    }


    /// The USER (personal) DICTIONARY path — `dictionary.txt` beside `config.toml`
    /// (GLOBAL across projects). `None` when no config dir resolved (the
    /// `Config::empty` placeholder), so the add stays in-memory-only that session.
    pub(in crate::app) fn user_dictionary_path(&self) -> Option<std::path::PathBuf> {
        crate::config::dictionary_path(&self.config.path)
    }


    /// LOAD the user's personal dictionary from disk into the live
    /// [`crate::spell::SpellChecker`] — called at launch and after
    /// [`Self::set_dictionary`] rebuilds the checker. An ABSENT file loads as an
    /// empty list (no error — the file only exists once a word has been added).
    /// ZERO-NETWORK: a plain file read through the [`crate::fs`] seam, never a
    /// fetch. The one owner of "pull the word list into the checker".
    pub(in crate::app) fn load_user_dictionary(&mut self) {
        let Some(path) = self.user_dictionary_path() else { return };
        let text = crate::fs::active().read_to_string(&path).unwrap_or_default();
        let words = crate::spell::parse_dictionary(&text);
        if let Some(sc) = self.spell.as_mut() {
            sc.set_user_words(words);
        }
    }


    /// "Add '<word>' to dictionary" (the Cmd-`;` overlay row / the right-click
    /// summon): add `word` to the live checker AND persist it to the personal
    /// dictionary file, then rescan so the squiggle clears THIS frame. In-memory
    /// FIRST so a failed disk write still silences the word this session; the file
    /// append is skipped when the word was already known (no duplicate lines).
    /// GLOBAL across projects (the file is config-dir scoped); removal v1 =
    /// hand-edit the file. ZERO-NETWORK: an append through the [`crate::fs`] seam,
    /// never a fetch.
    pub(in crate::app) fn add_to_dictionary(&mut self, word: &str) {
        let word = word.trim();
        if word.is_empty() {
            return;
        }
        let newly = self.spell.as_mut().map(|sc| sc.add_user_word(word)).unwrap_or(false);
        if newly {
            if let Some(path) = self.user_dictionary_path() {
                if let Err(e) = Self::append_word_to_dictionary_file(&path, word) {
                    eprintln!("could not add '{word}' to dictionary at {}: {e}", path.display());
                }
            }
        }
        self.active.extra.spell_checked_version = None;
        self.run_spellcheck_now();
    }


    /// Append `word` as its own line to the personal dictionary FILE (creating the
    /// file + its config dir), through the [`crate::fs`] seam + atomic write (crash
    /// leaves the old or the new file, never a torn one — same durability the
    /// config writes get). A word already present (case-insensitively) is a no-op,
    /// so re-adding never duplicates a line. The existing text is preserved
    /// verbatim (hand-edited comments/order kept) with the new word appended after a
    /// terminating newline. Associated fn (no `self`) so it stays a pure path→disk
    /// unit, testable under the `InMemoryFs`.
    fn append_word_to_dictionary_file(path: &std::path::Path, word: &str) -> std::io::Result<()> {
        let fs = crate::fs::active();
        let existing = fs.read_to_string(path).unwrap_or_default();
        if crate::spell::parse_dictionary(&existing)
            .iter()
            .any(|w| w.eq_ignore_ascii_case(word))
        {
            return Ok(()); // already on disk — keep the file duplicate-free
        }
        if let Some(dir) = path.parent() {
            if !dir.as_os_str().is_empty() {
                let _ = fs.create_dir_all(dir);
            }
        }
        let mut out = existing;
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(word);
        out.push('\n');
        crate::fs::write_atomic(path, out.as_bytes())
    }
}
