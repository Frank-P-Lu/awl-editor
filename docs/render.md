# Rendering guide

Read before changing layout, chrome, pickers, or render state. Use
[DESIGN.md](../DESIGN.md) for the product's visual rules and
[verification policy](verification.md) for outcome audits and visual smoke.
Read the focused reference for the mechanism you are changing.

## Shared rules

Theme variation is data in `RenderCaps`; render consumers do not branch on world
identity. Each geometry decision has one owner shared by drawing, hit testing,
and sidecar projection. Do not recreate a formula to avoid a shared file.

Per-frame work is O(visible). Cache keys include every input that changes their
answer; buffer versions need identity or invalidation on swap. A new `ViewState`
field gets an inert default in `ViewState::base()` and an explicit decision in
`sync_view`. Conceal changes can alter advances, so invalidate row geometry when
reshaping. Headless captures prove the frame they draw, not live invalidation or
presentation cadence.

Record canvas, DPI, zoom, and backend in geometry evidence. Source constants,
logical lengths, physical pixels, and shaped glyph positions are distinct units.
Use PNG arithmetic for visibility, contrast, and perceptual distinction; a correct
sidecar rectangle does not prove visible ink.

## Find the owner

| Change | Read | Principal owners |
| --- | --- | --- |
| Column placement, clipping, toasts, margin plates, count cues | [Geometry](render/geometry.md) | `column_left`, `content_clip`, `render/plan/floating`, `render/rowlayout` |
| Caret cell height, raster anchors, proportional fallback | [Caret](render/caret.md) | `caret_cell_vertical`, `caret_anchor_raster_box` |
| Backgrounds, ground coordinate spaces, lava frost | [Grounds](render/grounds.md) | `RenderCaps`, background descriptors/shaders, frost seed cache |
| Rotated location labels, fit and hit testing | [Labels](render/labels.md) | `rotated_label`, `prepare_rotated_location_label` |
| List, facet, pane, placard, and material expressions | [Chrome composition](render/chrome.md) | `ListStyle`, `FacetStyle`, `PaneSplit`, shared chrome resolvers |
| Picker blur footprint, shear, feather, and bounds | [Picker frost](render/frost.md) | `render/blur`, drawn-envelope geometry |
| Chrome units, card placement, hint spacing | [Chrome geometry](render/chrome-geometry.md) | shared card/row/pane geometry and logical-length owners |
| Workspace arrangement and settings controls | [Workspaces](render/workspaces.md) | `workspace_shape`, settings/range rosters |
| Scene planning and draw/hit/report agreement | [Planning](render/planning.md) | `render/plan` |
| Files journeys, pointer stability, preview timing | [Navigation](render/navigation.md) | navigation owners, `app/input/mouse/overlay`, presentation sampling |

## Review the changed behavior

Use roster-derived laws with nonempty enrollment and mutation evidence. Sweep the
changed axis: a new expression requires the surface roster; a geometry change
needs relevant narrow/wide and DPI cases. Degradation arms must remain visible,
usable, and tested. Read [capture reach](harness-reach.md) before choosing a driver.

Live theme switching, buffer swaps, resize/page drag, and redraw scheduling can
fail while independently rebuilt screenshots agree. Measure those transitions at
their real seam and state any remaining live-only evidence. The detailed references
retain mechanism descriptions and law names for targeted investigation.
