//! WRITING STREAKS' App-side WIRING (native only — `cfg(not(target_arch =
//! "wasm32"))`, mirroring the odometer / daemon / session-restore gate): the
//! per-buffer word-delta SAMPLING on the autosave flush triggers and the live
//! year-view PUSH into the pipeline.
//!
//! The STATE is not here. [`crate::streaks`] owns the pure store +
//! calendar/intensity arithmetic + the (de)serializer, and
//! `app::usage::UsageLedger` owns the live ledger — the record, its
//! unflushed-changes stamp, and the word baseline. What remains in this file is
//! the seam that reaches the DOCUMENT for a word count and the GPU pipeline for
//! the card push.
//!
//! **The hooks (each takes the privacy gate as a [`super::usage::Recording`]
//! value from `ConfigurationRuntime` — the one reader of the toggle; the
//! streaks and the odometer share that single LOCAL-usage kill switch, since
//! both are native-only, private, never-uploaded personal state):**
//!  - [`Self::streaks_flush`] — on the SAME idle/blur/switch/quit triggers the
//!    autosave engine flushes on: sample the active buffer's word count, record
//!    the DELTA since the last sample under today's LOCAL calendar day, and
//!    re-anchor.
//!  - [`Self::streaks_reset_baseline`] / [`Self::streaks_anchor_now`] — on a
//!    buffer SWAP: drop the anchor (an opened file's existing words are not
//!    writing) or set it eagerly (an awl-created buffer's first keystrokes ARE).
//!  - [`Self::streaks_sync_card`] — every `sync_view`: push the live year-view
//!    so a summoned card this frame reads the real heatmap (live-only; a
//!    capture never calls `sync_view`, so the card shows the placeholder).
//!
//! **Why the streaks flush is NOT interchangeable with the odometer's:** the
//! word delta is BUFFER-SCOPED — words now minus words at the last sample of
//! *this* document — so three call sites (`load_path`, `start_fresh_document`,
//! and the card-summon effect) run [`Self::streaks_flush`] ALONE, before the
//! active buffer changes, where subtracting across two documents would be
//! meaningless. The odometer's counters are app-global and carry no such
//! deadline. See `docs/app-domains.md`.
//!
//! **Determinism:** all of it lives ONLY on the live `App`; the headless
//! capture never constructs a `UsageLedger`, so a `--screenshot`/`--keys`
//! capture is STRUCTURALLY incapable of touching `streaks.toml` — the same
//! boundary the odometer's `headless_replay_never_touches_the_stats_file`
//! tripwire pins.

use super::*;

/// The one door both streak word-count call sites route through — see
/// [`App::streaks_current_words`] for why it is [`crate::card::figures::word_count`]
/// and not the plain whitespace-split counter. A free function rather than a
/// second `App` method because [`App::streaks_flush`] borrows only
/// `self.document` ahead of a closure passed to `self.usage.flush_writing`; a
/// method taking `&self` there would capture the whole `App` and collide with
/// that call's own `&mut self.usage` borrow.
#[cfg(not(target_arch = "wasm32"))]
fn streaks_word_count(text: &str) -> usize {
    crate::card::figures::word_count(text)
}

impl App {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn streaks_flush_if_document(&mut self) {
        if self.document.has_active() {
            self.streaks_flush();
        }
    }

    /// The active buffer's whole-document word count — the SAME
    /// [`crate::card::figures::word_count`] the readout / held HUD use, not the
    /// plain whitespace-split `markdown::word_count` this used to call (which
    /// silently undercounted CJK-majority prose, since an unspaced run of
    /// ideographs has no whitespace to split on and so counted as one "word"
    /// no matter how long it ran). `figures::word_count` also strips a leading
    /// frontmatter block before counting — a deliberate improvement for the
    /// streak ledger too, since typing frontmatter isn't writing prose. A
    /// `String` alloc per call, but flushes are infrequent (idle/blur/switch/
    /// quit), so this is cheap — and with tracking off the ledger never asks
    /// for it at all.
    #[cfg(not(target_arch = "wasm32"))]
    fn streaks_current_words(&self) -> usize {
        streaks_word_count(&self.document.buffer().text())
    }

    /// Drop the word-delta ANCHOR to LAZY across a BUFFER SWAP into an OPENED
    /// FILE, so the arriving document's existing words re-anchor on the next
    /// flush rather than counting as freshly written. The first post-swap flush
    /// anchors at whatever the file holds THEN — correct because a file's content
    /// is already present at swap and (barring a rare open-then-type-within-1s)
    /// unchanged before that flush. Mirrors [`Self::stats_reset_caret_anchor`].
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn streaks_reset_baseline(&mut self) {
        self.usage.reset_word_baseline();
    }

    /// Anchor the word-delta baseline EAGERLY at the active buffer's CURRENT word
    /// count — the seam for an awl-CREATED buffer (a NEW NOTE, or the birth /
    /// restored-stash SCRATCH), whose birth content must NOT count as freshly
    /// written (0 for a new note; the restored stash's own words are yesterday's),
    /// yet whose FIRST post-birth keystrokes — typed BEFORE the first idle flush —
    /// MUST. This is the anchor-swallow fix: a lazy `None` anchor (see
    /// [`Self::streaks_reset_baseline`]) would anchor at the already-typed count on
    /// that first flush and lose everything written in the window, which is exactly
    /// what a short new-note session hit.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn streaks_anchor_now(&mut self) {
        let words = self.streaks_current_words();
        self.usage.anchor_words(words);
    }

    /// Sample the active buffer's word count and fold the DELTA since the last
    /// sample into today's record, then persist if anything changed — on the SAME
    /// idle/blur/switch/quit triggers the autosave flush uses. A no-op when the
    /// feature is off. The FIRST sample of a buffer only ANCHORS (records nothing),
    /// so a file's pre-existing words are never counted; every later sample records
    /// the net words added since the previous one (clamped for the day total, raw
    /// kept). Errors go to stderr, never disrupt.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn streaks_flush(&mut self) {
        let recording = self.config.usage_recording();
        let document = &self.document;
        self.usage
            .flush_writing(recording, || streaks_word_count(&document.buffer().text()));
    }

    /// Push the live year-VIEW into the pipeline so a summoned Writing streaks card
    /// this frame reads the real heatmap. Called every `sync_view` (LIVE-ONLY); a
    /// headless capture never calls this, so the pipeline field stays `None` and the
    /// card renders the synthetic [`crate::streaks::placeholder`] — the determinism
    /// boundary keeping a `--streaks` capture byte-stable. When the feature is OFF
    /// the ledger yields `None` too, so the card honestly shows the placeholder
    /// rather than a misleading empty grid. Cheap: the view is a small pure
    /// computation over the (catalog-sized) day map, like `stats_sync_hud`.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn streaks_sync_card(&mut self) {
        // Compute the view BEFORE borrowing the GPU (both read `self`).
        let view = self.usage.writing_view(self.config.usage_recording());
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_streaks(view);
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn writing_words_records_the_net_delta_after_anchoring() {
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            // First flush ANCHORS the empty scratch buffer — records nothing.
            app.streaks_flush();
            assert!(
                app.usage.writing().days.is_empty(),
                "the anchor flush records nothing"
            );
            let today = usage::local_today();

            // Write some words, then flush: the net delta is recorded under today.
            app.document.set_text("hello there friend");
            app.streaks_flush();
            assert_eq!(
                app.usage.writing().words_on(&today),
                3,
                "three net words added"
            );

            // Cut back to two words, flush: a net-cut flush never erodes the day
            // total (raw net still drops).
            app.document.set_text("hello there");
            app.streaks_flush();
            assert_eq!(
                app.usage.writing().words_on(&today),
                3,
                "a cut never lowers the day total"
            );
            assert!(app.usage.writing().days.get(&today).unwrap().raw_net <= 3);

            // Persisted to (and reloaded from) streaks.toml.
            let saved = crate::streaks::load(&crate::streaks::streaks_path());
            assert_eq!(saved.words_on(&today), 3);
        });
    }

    #[test]
    fn a_buffer_swap_reset_anchors_the_new_buffer() {
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            app.document.set_text("one two three four");
            // The birth scratch is eager-anchored at 0, so this first flush records
            // the 4 words typed into it (the anchor-swallow fix); `before` captures
            // whatever the day total is now — this test then proves the SWAP below
            // never ADDS the arriving doc's words to it.
            app.streaks_flush();
            let today = usage::local_today();
            let before = app.usage.writing().words_on(&today);
            // Simulate a swap into an OPENED file: reset the anchor LAZY, replace the
            // buffer with a big doc.
            app.streaks_reset_baseline();
            app.document
                .replace_buffer(crate::buffer::Buffer::from_str("a b c d e f g h i j"));
            app.streaks_flush(); // must ANCHOR the arriving words, not count them
            assert_eq!(
                app.usage.writing().words_on(&today),
                before,
                "opening a doc's existing words is anchored, never counted as written"
            );
        });
    }

    #[test]
    fn a_new_note_records_words_typed_before_the_first_flush() {
        // THE ANCHOR-SWALLOW BUG: an awl-CREATED buffer is born EMPTY, and the
        // user types into it BEFORE the first idle flush fires. A lazy first-flush
        // anchor (`None` → anchor at the current count) would swallow everything
        // typed in that window. A new note must anchor EAGERLY at birth (0 words),
        // so the first flush records the delta from 0 — the words the user wrote.
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            // Create a fresh note the REAL way (the C-x n path).
            app.new_document();
            let today = usage::local_today();
            // Type INTO the fresh note before any idle flush has fired.
            app.document.set_text("brand new words typed today");
            // The first idle flush of this awl-created note.
            app.streaks_flush();
            assert_eq!(
                app.usage.writing().words_on(&today),
                5,
                "words typed into a fresh note BEFORE its first flush must be recorded, \
                 not anchored away"
            );
        });
    }

    #[test]
    fn a_fresh_scratch_records_words_typed_before_the_first_flush() {
        // The same anchor-swallow, one layer up: the BIRTH scratch buffer awl
        // opens on a no-argument launch is also awl-created + empty, so words
        // typed into it before the first flush must count too.
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            let today = usage::local_today();
            // Type into the birth scratch before any idle flush.
            app.document.set_text("first words of the day");
            app.streaks_flush();
            assert_eq!(
                app.usage.writing().words_on(&today),
                5,
                "words typed into the birth scratch before its first flush are recorded"
            );
        });
    }

    #[test]
    fn summoning_the_card_flushes_so_today_is_live() {
        // CARD-SUMMON FRESHNESS: opening the Writing streaks card must FLUSH the
        // pending word-delta first, so "written today" reads LIVE rather than up
        // to ~1s stale (the idle flush may not have fired since the last
        // keystroke). Drives the REAL post-apply side effect the live app runs.
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            let today = usage::local_today_ymd();
            // Type into the birth scratch, but DON'T let an idle flush fire.
            app.document.set_text("live words not yet flushed today");
            // The delta is still pending — the store hasn't seen it.
            assert_eq!(
                app.usage.writing().view(today).today_words,
                0,
                "precondition: the pending delta is not yet in the store"
            );
            app.apply_live_effect(crate::actions::Effect::Persistence(
                crate::actions::PersistenceEffect::Preference(
                    crate::actions::PreferenceEffect::WritingStreaks,
                ),
            ));
            assert_eq!(
                app.usage.writing().view(today).today_words,
                6,
                "summoning FLUSHED the pending delta — the card reads live, not stale"
            );
        });
    }

    #[test]
    fn kill_switch_off_records_nothing_and_never_writes() {
        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let cfg = Config {
                stats: Some(false),
                ..Config::empty()
            };
            let mut app = App::new(None, PathBuf::from("/n"), None, None, cfg);
            app.document.set_text("some words here now");
            app.streaks_flush();
            assert!(app.usage.writing().days.is_empty(), "off: no recording");
            assert!(
                crate::fs::active()
                    .read(&crate::streaks::streaks_path())
                    .is_err(),
                "off: never writes streaks.toml"
            );
        });
    }

    #[test]
    fn cjk_prose_records_the_readout_s_word_count_not_a_whitespace_undercount() {
        // Unspaced Japanese: no ASCII/Unicode whitespace anywhere in the run,
        // so the plain whitespace-split counter this module used to call
        // would see the whole sentence as ONE "word". The readout's own
        // counter (`card::figures::word_count`) treats each ideograph as its
        // own token — 11 here (10 Han/Kana + the trailing `。`).
        let ja = "今日はいい天気ですね。";
        let readout_count = crate::card::figures::word_count(ja);
        assert_eq!(
            readout_count, 11,
            "sanity: pins the readout counter's own CJK figure this test compares against"
        );
        let old_whitespace_count = crate::markdown::word_count(ja);
        assert_eq!(
            old_whitespace_count, 1,
            "sanity: the retired whitespace-split counter collapses the whole \
             unspaced sentence to a single token — the undercount this fix closes"
        );

        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            app.streaks_flush(); // anchors the empty scratch buffer, records nothing
            let today = usage::local_today();
            app.document.set_text(ja);
            app.streaks_flush();
            assert_eq!(
                app.usage.writing().words_on(&today),
                readout_count as u64,
                "the streak delta must equal the readout's own CJK-aware count \
                 (11), not the whitespace-split undercount (1) the ledger used \
                 to accrue for this exact sentence"
            );
        });
    }

    #[test]
    fn plain_english_prose_records_the_same_count_either_counter_would_give() {
        // The switch is a no-op for ordinary spaced Latin prose: prove it by
        // actually running BOTH counters over the same fixture and asserting
        // they agree, rather than assuming whitespace tokenization and
        // `count_tokens` coincide.
        let prose = "the quick brown fox jumps over the lazy dog";
        let old_whitespace_count = crate::markdown::word_count(prose);
        let readout_count = crate::card::figures::word_count(prose);
        assert_eq!(
            old_whitespace_count, readout_count,
            "plain English: the retired whitespace-split counter and the \
             CJK-aware readout counter must agree on ordinary spaced prose"
        );
        assert_eq!(readout_count, 9);

        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            app.streaks_flush();
            let today = usage::local_today();
            app.document.set_text(prose);
            app.streaks_flush();
            assert_eq!(
                app.usage.writing().words_on(&today),
                readout_count as u64,
                "unchanged behavior on plain English prose"
            );
        });
    }

    #[test]
    fn frontmatter_words_never_tick_the_streak_ledger() {
        // Landing note: `card::figures::word_count` routes through
        // `manuscript()`, which strips a leading frontmatter block before
        // counting — a deliberate improvement for the streak ledger, since
        // typing frontmatter (`title:`, `tags:`, …) isn't writing prose. Pin
        // it: the frontmatter block here carries 6 words of its own
        // ("My Great Document", "alpha beta gamma"), and none of them may
        // reach the ledger.
        let doc =
            "---\ntitle: My Great Document\ntags: alpha beta gamma\n---\nthree real words here\n";
        let readout_count = crate::card::figures::word_count(doc);
        assert_eq!(
            readout_count, 4,
            "sanity: only the body's 4 words count, the frontmatter's 6 are stripped"
        );

        crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
            let mut app = App::new(None, PathBuf::from("/n"), None, None, Config::empty());
            app.streaks_flush(); // anchors the empty scratch buffer
            let today = usage::local_today();
            app.document.set_text(doc);
            app.streaks_flush();
            assert_eq!(
                app.usage.writing().words_on(&today),
                4,
                "the frontmatter block's own words must never tick the streak ledger"
            );
        });
    }
}
