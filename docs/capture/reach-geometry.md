# Capture geometry reach

Read when planning canvas, DPI, picker, or find/replace geometry assertions.
[Capture reach](../harness-reach.md) · Source paths are repo-relative.

## Canvas, DPI, and zoom

Windowed launch, replay capture, and headless App capture resolve absent zoom
through `range::ZOOM.default` (1.0). Saved config or an explicit replay override
can change it. Record `font.zoom` with every measurement.

Both capture drivers honor `--capture-size` and `--capture-dpi`. The App capture
law `a_live_app_capture_honors_capture_size_and_the_dpi_meaning_holds` checks the
document's wrap count at equivalent logical windows (`1200x800 @1` and
`2400x1600 @2`). Do not extend that equality to every chrome dimension; read
`overlay.window` from the actual capture instead of scaling another measurement.
Live probes use App references at the surface and DPI reported by the real window.

## Picker rows

`overlay.window.band` and `overlay.window.rows` publish planned physical-pixel
rectangles: `{ display, item, x, y, w, h }`, with accessory lanes `label`, `value`,
and `rail`. Selection is reported once by `window.sel_row`, not duplicated on rows.
Both drivers use the same sidecar writer.

Use these facts for row pitch, band placement, label/control overlap, and planned
hit regions. They are shared with drawing and `overlay_row_at`; they cannot prove
that a downstream emitter actually painted visible ink. Use PNG arithmetic for
that separate claim.

## Find/replace panel

`search.panel` is null when closed. When open, it reports the exterior card, text
origin, shaped-row bands, and named field/button/checkbox spans. It includes only
controls actually present in the frame. `search.editing_replacement` owns field
focus; geometry does not duplicate it.

Read the reported responsive card. Its width yields on narrow canvases and its
placement clamps to the canvas; projection does not repair the drawn geometry.
The planner converts `PANEL_MARGIN` and `PANEL_PAD` through its panel metrics;
these are logical lengths, not fixed device-pixel offsets. See
[field details](geometry-fields.md) and the panel planner for exact semantics.
