# cosmic-text 0.18.2 local patch

This directory is the exact published `cosmic-text` 0.18.2 crate source plus a
small default-inert `BufferLine::hanging_inset` extension. The upstream crate
checksum recorded by the original lockfile is
`bbe782a9e7520cc7de2232c957a47f99d3a35e855552677d07a557bc1a3b66ed`.

The extension adds one non-negative per-paragraph inset. A positive value wraps
against `width - inset` and translates every LTR visual row by the same inset;
zero preserves upstream layout. Invalid values become zero and an inset can
never reduce the available wrap width below one pixel. RTL paragraphs reserve
the narrower width at the right edge without applying an LTR translation.
Changing the value invalidates only that line's layout cache.

`LICENSE-APACHE`, `LICENSE-MIT`, and the manifest-declared `README.md` are
preserved from the published crate (`UPSTREAM-README.md` is an identical audit
copy). Remove this patch and the root `[patch.crates-io]` entry when
upstream cosmic-text exposes an equivalent per-line hanging-indent API.
