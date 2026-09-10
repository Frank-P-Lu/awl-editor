# Typing and search measurements

Release measurements on Apple M1 Max, 2910×1720 at 2× scale. Builds and
benchmarks ran sequentially. These are headless pipeline timings, including
blocking GPU completion; they are not keyboard-to-display latency.

| Workload | Before median | Combined median | Ratio |
| --- | ---: | ---: | ---: |
| 50,029-word manuscript, buffer edit + spelling + view/frame | 34.539 ms | 8.194 ms | 4.2× |
| Same manuscript, rendering-only typing | 18.600 ms | 7.775 ms | 2.4× |
| 5,005-word single paragraph, 795 search matches | 139.918 ms | 4.774 ms | 29.3× |
| Same single paragraph, buffer edit + spelling + view/frame | 25.404 ms | 25.145 ms | 1.0× |

The combined result is from `f1aae14d`. The typing baseline includes the same
live-preparation benchmark before the retained spelling and geometry paths.
The search baseline was recorded before the search change. Timings vary with
machine load; work counts below establish what the faster paths actually did.

Across 30 manuscript edits, spelling tokenized 30 changed lines (825 bytes),
with no full spelling scans. Geometry patched 30 lines and 30 rows, using
1,170 binary-search probes. The pipeline still reshaped once per edit.
Search retained all 795 matches and performed all 10 measured steps.

Typing is not constant-time: it still scans inexpensive line keys and performs
other document-wide preparation. Code spelling and structural Markdown edits
retain full invalidation paths. A partially presented long paragraph rejects
row retention before constructing replacement rows; its typing cost is unchanged.

The suite completed 52 cells. Its existing documented skips remain code zoom
and single-paragraph resize. Regression laws compare retained geometry with a
full rebuild, including real pixels, and cover Unicode, history, width changes,
structural edits, concealment, and the partial-paragraph fallback. Work-count
laws detect repeated search gathering and discarded paragraph-row construction.

## Frame preparation, document context, and spelling geometry

Three later rounds measured named owners inside frame preparation before
changing them. Each reports a work count beside its timing, so the number is
tied to work that provably changed rather than to a share of a stage total.
Stage summaries remain separate statistics: they do not decompose the total
additively.

| Owner | Before | After | Work count after |
| --- | ---: | ---: | --- |
| Writing-nit protos, 50,029-word manuscript | — | — | 30 lines retokenized across 30 keys |
| Ornament lists (four passes to one sweep) | — | — | unchanged scope, strictly less work per reshape |
| Document context (CJK evidence) | 0.656 ms | 0.025 ms | 30 lines scanned across 30 keys |
| Spelling squiggle geometry | 46.815 ms / 30 keys | 2.435 ms / 30 keys | retained per line |

Manuscript `typing_live` medians across those rounds: 7.960 ms before the nit
and ornament work, 6.859 ms after it, 6.752 ms after context retention, and
4.772 ms after squiggle retention. The rendering-only typing median finished at
4.772 ms on the same tier.

What these paths do NOT cover is recorded deliberately. Markdown span parsing
still re-parses the whole document on every edit — 286,713 bytes and 1,719
lines on this manuscript — and is now instrumented rather than merely
suspected; retaining it needs correct nonlocal invalidation for fences,
frontmatter and references, which pulldown-cmark offers no incremental API for.
Documents carrying links, images, tables, frontmatter or code are ineligible
for nit and squiggle retention by construction and pay the untouched full scan.
A single-paragraph document is one logical line, so every per-line retention
degenerates to a no-op there; its typing cost is unchanged for that reason and
not because retention failed.

Squiggle retention additionally requires the row-geometry patch to have
succeeded before it may splice, because a rejected patch shifts every row below
the changed band. An eager first implementation of it regressed the zoom burst
by about half, since coalesced view updates each paid for a rebuild the next
discarded unread; the work defers to the next read instead.
