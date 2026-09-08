//! Retained per-line spelling projection for prose buffers.
//!
//! The buffer's [`RunTable`](crate::semantic::runs::RunTable) already records
//! which logical lines changed. Ordinary prose typing therefore re-tokenizes
//! only those lines, while a cheap revision scan still visits every line. Any
//! edit that can change non-local scope (line structure, a fence delimiter, or
//! frontmatter) takes the exact full-scan path. Code buffers also stay on that
//! path because their lexer-owned prose ranges can cross logical lines.

use super::{SpellChecker, SpellVerdict};
use crate::semantic::runs::RunId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpellRefreshWork {
    /// Integer revision comparisons; this remains O(logical lines).
    pub(crate) line_keys_scanned: u64,
    /// Lines whose actual text was read and tokenized.
    pub(crate) lines_tokenized: u64,
    /// Bytes in those tokenized lines (or in the exact full-scan document).
    pub(crate) bytes_tokenized: u64,
    pub(crate) full_scans: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Slot {
    id: RunId,
    rev: u64,
    fence: bool,
    in_fence_before: bool,
    /// Line-local verdicts: `span.line` is always zero until materialization.
    verdicts: Vec<SpellVerdict>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpellProjection {
    table_id: Option<u64>,
    content_rev: u64,
    shape_rev: u64,
    syntax_lang: Option<crate::syntax::Lang>,
    checker_generation: u64,
    spellcheck_generation: u64,
    slots: Vec<Slot>,
    frontmatter_runs: Vec<RunId>,
    ambiguous_frontmatter_opener: bool,
    work: SpellRefreshWork,
}

impl SpellProjection {
    pub(crate) fn invalidate(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn work(&self) -> SpellRefreshWork {
        self.work
    }

    pub(crate) fn is_current(
        &self,
        buffer: &crate::buffer::Buffer,
        checker: &SpellChecker,
    ) -> bool {
        self.table_id == Some(buffer.runs().state_key().0)
            && self.content_rev == buffer.runs().content_rev()
            && self.shape_rev == buffer.runs().shape_rev()
            && self.syntax_lang == buffer.syntax_lang()
            && self.checker_generation == checker.generation()
            && self.spellcheck_generation == super::spellcheck_generation()
    }

    pub(crate) fn refresh(
        &mut self,
        buffer: &crate::buffer::Buffer,
        checker: &SpellChecker,
    ) -> Vec<SpellVerdict> {
        self.work = SpellRefreshWork::default();
        if !super::spellcheck_on() {
            let table = buffer.runs();
            let (table_id, content_rev) = table.state_key();
            self.table_id = Some(table_id);
            self.content_rev = content_rev;
            self.shape_rev = table.shape_rev();
            self.syntax_lang = buffer.syntax_lang();
            self.checker_generation = checker.generation();
            self.spellcheck_generation = super::spellcheck_generation();
            self.slots.clear();
            self.frontmatter_runs.clear();
            self.ambiguous_frontmatter_opener = false;
            return Vec::new();
        }

        let table = buffer.runs();
        let (table_id, content_rev) = table.state_key();
        let code = buffer.syntax_lang().is_some();
        if self.table_id != Some(table_id)
            || self.slots.len() != table.runs().len()
            || self.shape_rev != table.shape_rev()
            || self.syntax_lang != buffer.syntax_lang()
            || self.checker_generation != checker.generation()
            || self.spellcheck_generation != super::spellcheck_generation()
            || code
        {
            return self.rebuild(buffer, checker);
        }
        if self.content_rev == content_rev {
            return self.materialize();
        }

        self.work.line_keys_scanned = table.runs().len() as u64;
        let mut changed = Vec::new();
        for (line, run) in table.runs().iter().enumerate() {
            if self.slots[line].rev != run.rev {
                changed.push((line, *run, line_text(buffer, line)));
            }
        }

        if changed.iter().any(|(line, _, text)| {
            let slot = &self.slots[*line];
            slot.fence
                || crate::markdown::is_fence_line(text)
                || self.frontmatter_runs.contains(&slot.id)
                || (self.frontmatter_runs.is_empty()
                    && (self.ambiguous_frontmatter_opener
                        || (*line == 0 && text.trim_end_matches('\r') == "---")))
        }) {
            return self.rebuild(buffer, checker);
        }

        for (line, run, text) in changed {
            self.work.lines_tokenized += 1;
            self.work.bytes_tokenized += text.len() as u64;
            let verdicts = if self.slots[line].in_fence_before {
                Vec::new()
            } else {
                line_verdicts(&text, checker)
            };
            self.slots[line].rev = run.rev;
            self.slots[line].verdicts = verdicts;
        }
        self.content_rev = content_rev;
        self.materialize()
    }

    fn rebuild(
        &mut self,
        buffer: &crate::buffer::Buffer,
        checker: &SpellChecker,
    ) -> Vec<SpellVerdict> {
        let text = buffer.text();
        let table = buffer.runs();
        let (table_id, content_rev) = table.state_key();
        let verdicts = refresh_text_cache(&text, buffer.syntax_lang(), checker);

        self.work = SpellRefreshWork {
            line_keys_scanned: table.runs().len() as u64,
            lines_tokenized: table.runs().len() as u64,
            bytes_tokenized: text.len() as u64,
            full_scans: 1,
        };
        self.slots.clear();
        self.slots.reserve(table.runs().len());
        self.frontmatter_runs.clear();

        let frontmatter = crate::frontmatter::detect(&text);
        let first = line_text(buffer, 0);
        self.ambiguous_frontmatter_opener =
            frontmatter.is_none() && first.trim_end_matches('\r') == "---";
        let frontmatter_end = frontmatter.as_ref().map(|fm| fm.range.end);
        let mut byte_start = 0usize;
        let mut in_fence = false;
        for (line, run) in table.runs().iter().enumerate() {
            let raw = buffer.run_text(line);
            let body = raw.strip_suffix('\n').unwrap_or(&raw);
            let in_frontmatter = frontmatter_end.is_some_and(|end| byte_start < end);
            let fence = !in_frontmatter && crate::markdown::is_fence_line(body);
            if in_frontmatter {
                self.frontmatter_runs.push(run.id);
            }
            self.slots.push(Slot {
                id: run.id,
                rev: run.rev,
                fence,
                in_fence_before: in_fence,
                verdicts: Vec::new(),
            });
            if fence {
                in_fence = !in_fence;
            }
            byte_start += raw.len();
        }
        for mut verdict in verdicts {
            let line = verdict.span.line;
            verdict.span.line = 0;
            if let Some(slot) = self.slots.get_mut(line) {
                slot.verdicts.push(verdict);
            }
        }
        self.table_id = Some(table_id);
        self.content_rev = content_rev;
        self.shape_rev = table.shape_rev();
        self.syntax_lang = buffer.syntax_lang();
        self.checker_generation = checker.generation();
        self.spellcheck_generation = super::spellcheck_generation();
        self.materialize()
    }

    fn materialize(&self) -> Vec<SpellVerdict> {
        let capacity = self.slots.iter().map(|slot| slot.verdicts.len()).sum();
        let mut out = Vec::with_capacity(capacity);
        for (line, slot) in self.slots.iter().enumerate() {
            out.extend(slot.verdicts.iter().cloned().map(|mut verdict| {
                verdict.span.line = line;
                verdict
            }));
        }
        out
    }
}

fn line_text(buffer: &crate::buffer::Buffer, line: usize) -> String {
    let raw = buffer.run_text(line);
    raw.strip_suffix('\n').unwrap_or(&raw).to_string()
}

fn line_verdicts(line: &str, checker: &SpellChecker) -> Vec<SpellVerdict> {
    let mut spans = Vec::new();
    super::scan_line(line, 0, &|word| checker.check(word), &mut spans);
    super::keyed(line, spans)
}

/// Full-scan test oracle for [`SpellProjection`]. The projection's initial seed
/// and exact invalidation fallbacks share [`refresh_text_cache`] with this
/// wrapper, while ordinary live refreshes retain unchanged line verdicts.
#[cfg(test)]
fn refresh_buffer_cache(
    buffer: &crate::buffer::Buffer,
    checker: &SpellChecker,
) -> Vec<SpellVerdict> {
    let text = buffer.text();
    refresh_text_cache(&text, buffer.syntax_lang(), checker)
}

fn refresh_text_cache(
    text: &str,
    lang: Option<crate::syntax::Lang>,
    checker: &SpellChecker,
) -> Vec<SpellVerdict> {
    let spans = checker.misspellings_for(text, lang);
    super::keyed(text, spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_exact(
        projection: &mut SpellProjection,
        buffer: &crate::buffer::Buffer,
        checker: &SpellChecker,
        context: &str,
    ) {
        assert_eq!(
            projection.refresh(buffer, checker),
            super::refresh_buffer_cache(buffer, checker),
            "{context}",
        );
    }

    fn char_index(text: &str, needle: &str) -> usize {
        let byte = text.find(needle).expect("fixture needle");
        text[..byte].chars().count()
    }

    #[test]
    fn retained_projection_matches_full_scan_across_edit_invalidations() {
        let _g = crate::testlock::serial();
        let checker = SpellChecker::new(super::super::DictVariant::EnUs).unwrap();
        let mut projection = SpellProjection::default();
        let mut buffer = crate::buffer::Buffer::from_str(
            "---\ntitle: Tale\n---\nhelo world\n```\ndefinately hidden\n```\nnaïve mispeled\n",
        );
        assert_exact(&mut projection, &buffer, &checker, "seed");

        buffer.set_cursor(char_index(&buffer.text(), "helo") + 4);
        buffer.insert_char('o');
        assert_exact(&mut projection, &buffer, &checker, "ordinary edit");
        assert_eq!(projection.work().lines_tokenized, 1);
        assert_eq!(projection.work().full_scans, 0);

        buffer.insert_text("\nnew mispeled line");
        assert_exact(&mut projection, &buffer, &checker, "multiline paste");
        assert_eq!(projection.work().full_scans, 1);
        buffer.undo();
        assert_exact(&mut projection, &buffer, &checker, "undo");
        buffer.redo();
        assert_exact(&mut projection, &buffer, &checker, "redo");

        buffer.set_cursor(char_index(&buffer.text(), "naïve") + 5);
        buffer.insert_char('é');
        assert_exact(&mut projection, &buffer, &checker, "Unicode edit");

        buffer.set_cursor(0);
        buffer.insert_char('x');
        assert_exact(&mut projection, &buffer, &checker, "frontmatter change");

        let fence = char_index(&buffer.text(), "```");
        buffer.set_cursor(fence);
        buffer.insert_char('x');
        assert_exact(&mut projection, &buffer, &checker, "fence change");
    }

    #[test]
    fn ordinary_first_line_edit_stays_incremental_until_it_becomes_an_opener() {
        let _g = crate::testlock::serial();
        let checker = SpellChecker::new(super::super::DictVariant::EnUs).unwrap();
        let mut projection = SpellProjection::default();
        let mut buffer = crate::buffer::Buffer::from_str("helo world\nsecond line\n");
        assert_exact(&mut projection, &buffer, &checker, "seed");
        buffer.set_cursor(4);
        buffer.insert_char('o');
        assert_exact(&mut projection, &buffer, &checker, "first-line prose edit");
        assert_eq!(projection.work().full_scans, 0);
        assert_eq!(projection.work().lines_tokenized, 1);

        let first_line_end = char_index(&buffer.text(), "\n");
        buffer.replace_char_range(0, first_line_end, "---");
        assert_exact(&mut projection, &buffer, &checker, "new frontmatter opener");
        assert_eq!(projection.work().full_scans, 1);
    }

    #[test]
    fn ordinary_edit_work_is_one_line_across_document_sizes_and_positions() {
        let _g = crate::testlock::serial();
        let checker = SpellChecker::new(super::super::DictVariant::EnUs).unwrap();
        for lines in [1usize, 64, 1024] {
            for line in [0, lines / 2, lines - 1] {
                let text = "ordinary mispeled prose\n".repeat(lines);
                let mut buffer = crate::buffer::Buffer::from_str(&text);
                let mut projection = SpellProjection::default();
                assert_exact(&mut projection, &buffer, &checker, "matrix seed");
                buffer.set_cursor(buffer.line_col_to_char(line, 8));
                buffer.insert_char('x');
                assert_exact(&mut projection, &buffer, &checker, "matrix edit");
                let work = projection.work();
                assert_eq!(work.full_scans, 0, "{lines} lines, position {line}");
                assert_eq!(work.lines_tokenized, 1, "{lines} lines, position {line}");
                assert_eq!(
                    work.line_keys_scanned,
                    buffer.line_count() as u64,
                    "cheap revision scan remains explicitly document-sized",
                );
            }
        }
    }

    #[test]
    fn moving_the_frontmatter_boundary_falls_back_and_then_stabilizes() {
        let _g = crate::testlock::serial();
        let checker = SpellChecker::new(super::super::DictVariant::EnUs).unwrap();
        let mut projection = SpellProjection::default();
        let mut buffer = crate::buffer::Buffer::from_str(
            "---\ntitle: Tale\n---\nbody: mispeled\nmore: value\n---\nafter mispeled\n",
        );
        assert_exact(&mut projection, &buffer, &checker, "short frontmatter");
        let closer = char_index(&buffer.text(), "---\nbody");
        buffer.replace_char_range(closer, closer + 3, "x:y");
        assert_exact(&mut projection, &buffer, &checker, "extended frontmatter");
        assert_eq!(projection.work().full_scans, 1);

        let after = char_index(&buffer.text(), "after") + 5;
        buffer.set_cursor(after);
        buffer.insert_char('x');
        assert_exact(&mut projection, &buffer, &checker, "edit after boundary");
        assert_eq!(projection.work().full_scans, 0);
        assert_eq!(projection.work().lines_tokenized, 1);
    }

    #[test]
    fn toggle_dictionary_and_buffer_identity_cannot_reuse_stale_verdicts() {
        let _g = crate::testlock::serial();
        crate::spell::set_spellcheck_on(true);
        let mut checker = SpellChecker::new(super::super::DictVariant::EnUs).unwrap();
        let mut projection = SpellProjection::default();
        let a = crate::buffer::Buffer::from_str("mispeled prose\n");
        let b = crate::buffer::Buffer::from_str("ordinary prose\n");
        assert_exact(&mut projection, &a, &checker, "buffer A");
        assert_exact(&mut projection, &b, &checker, "same-revision buffer B");
        assert_eq!(projection.work().full_scans, 1, "identity forced a seed");

        crate::spell::set_spellcheck_on(false);
        assert!(projection.refresh(&b, &checker).is_empty());
        crate::spell::set_spellcheck_on(true);
        assert_exact(&mut projection, &b, &checker, "toggle back on");
        assert_eq!(projection.work().full_scans, 1);

        assert!(checker.add_user_word("mispeled"));
        projection.invalidate();
        assert_exact(&mut projection, &a, &checker, "personal dictionary change");
        assert!(projection.refresh(&a, &checker).is_empty());
    }

    #[test]
    fn code_scope_uses_the_exact_full_scan_fallback() {
        let _g = crate::testlock::serial();
        let checker = SpellChecker::new(super::super::DictVariant::EnUs).unwrap();
        let mut projection = SpellProjection::default();
        let mut buffer =
            crate::buffer::Buffer::from_str("// a mispeled prose comment\nfn mispeled_name() {}\n");
        buffer.set_path(std::path::PathBuf::from("sample.rs"));
        assert_exact(&mut projection, &buffer, &checker, "code seed");
        buffer.set_cursor(5);
        buffer.insert_char('x');
        assert_exact(&mut projection, &buffer, &checker, "code edit");
        assert_eq!(projection.work().full_scans, 1);

        buffer.set_path(std::path::PathBuf::from("sample.md"));
        assert_exact(
            &mut projection,
            &buffer,
            &checker,
            "code to prose path change",
        );
        assert_eq!(projection.work().full_scans, 1);
    }
}
