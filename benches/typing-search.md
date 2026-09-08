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
