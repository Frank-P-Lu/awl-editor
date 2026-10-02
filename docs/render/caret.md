# Caret geometry

Read before changing caret sizing, raster anchors, or fallback metrics.

[Guide](../render.md) · Paths in code examples are relative to the repo root.

The Block caret uses a padded rectangle around the complete shaped grapheme's actual raster ink. Its width and height both follow the letter: lowercase x-height, capitals, descenders, accents, separately positioned combining marks, and resolved fallback CJK glyphs. The rule applies to proportional and monospaced faces alike. `caret/adaptive.rs` unions shaped glyph placements; the picker calls the same raster bounds helper. `caret_body.rs` supplies visibility floors and 1.25 logical pixels of top padding and 2.5 logical pixels at the sides and bottom, with one shared center offset so the bottom stays anchored. The shared 4.5-logical-pixel corner radius is clamped to half the body dimensions; rounded-boundary laws keep the complete raster ink and its antialias margin inside each curved corner.

`caret_cell_vertical` owns the resting height and vertical centre. `caret_geometry` owns the horizontal centre and width. Ink corrections enter at the spring's rest endpoint; movement retains its thin travel streak. All lengths use the stored zoom × DPI scale. Headings follow their own shaped font size.

Spaces, empty lines, end-of-line, and a ligature spanning independent graphemes use the stable insertion-cell fallback. No single character claims an entire ligature's ink. Filled blocks compose true-weight coverage from every positioned glyph of the anchored grapheme; ligature knockouts are clipped to the rectangle so neighboring characters keep their ink. InverseVideo worlds use the same rectangle in the post-text role-swap pass.

The picker offers Block and I-beam. The independent **Highlight previous character** checkbox defaults off. When on, Block anchors the previous Unicode grapheme on the current visual row; a row start uses the insertion bar. Selection drags and I-beam retain insertion-point anchoring. Legacy `caret_mode = "morph"` resolves to Block with the checkbox on unless an explicit checkbox value overrides it. A style save writes that preference before canonicalizing the old style key. There is no letter-shaped accent silhouette.

Geometry laws sweep the world roster, resolved glyph classes, 1x/2x DPI, headings, wrapping, glyphless cells, and reduced motion. Actual frame-difference laws require both visible rectangles and shorter lowercase carets. The Paperbark x/H/Å/W law rejects restoration of a fixed face envelope. Filled and InverseVideo pixel laws retain legibility checks. Wrapped-row comparisons must account for each row's own baseline.
