# Mixed Markdown typing scale

Run from this checkout on macOS:

```sh
AWL_BENCH_TYPING_SCALE=1 cargo run --offline --locked --release -- --bench-suite
```

The probe reuses Awl's instrumented typing path with synthetic English/Japanese
paragraphs, headings, and fenced Rust. Each size gets one untimed warmup and 30
timed insertions on the same penultimate line. Timings include the edit,
retained spelling, view preparation and blocking offscreen GPU completion.
They do not measure native input-to-display latency. The default suite and
checked-in baseline are unchanged.

## Measured on the same arm64 Mac Studio

| Lines | Before p50 / p95 | After p50 / p95 |
| ---: | ---: | ---: |
| 100 | 2.621 / 3.666 ms | 2.576 / 4.285 ms |
| 1,000 | 25.065 / 26.743 ms | 4.110 / 5.635 ms |
| 10,000 | 16,944.230 / 26,997.889 ms | 23.448 / 24.856 ms |

The small tiers use medians of three interleaved A/B runs. The slow 10,000-line
baseline is one full run; the after column is the median of three full runs.
The baseline ran while the machine also compiled the candidates and tests, so
do not overinterpret the exact ratio or the old tail. Even its fastest
10,000-line sample was 16,578.749 ms. This fixture intentionally has many
closed fences and headings; the ratio is not expected for typical documents.

The before source is the benchmark-only commit 06be1eb4. On the large tier,
checking overlap before the nested fence lookup brought p50 to 198.786 ms.
Using an ordered sweep of actual line intersections brought it to 52.448 ms.
Counting heading line offsets in one forward pass brought it to 23.448 ms.
The full Markdown parser is still called for every edit; its median stage
on the large tier is about 2.3 ms. The dense case is not constant-time.

## Correctness evidence

Every run checked actual Buffer edits, caret position, reshape count and
changed pixels. Before and after builds produced identical edited-text hashes,
caret coordinates and before/after frame hashes for every tier. Seven separate
synthetic replay captures matched both complete JSON sidecars and PNGs
byte-for-byte, including a fence-body caret, cross-block selection, Japanese
preedit and end-of-document input.

The targeted release tests cover span ordering, heading offsets, WYSIWYG,
image concealment, ordered lists, thematic rules and document IME; 78 distinct
tests passed. This is not a full native gate, wasm validation or a physical
keyboard/IME check. Local logs and images are in target/typing-perf/.
