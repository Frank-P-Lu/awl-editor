//! END-OF-DOCUMENT BREATHING ROOM — the virtual space past the last line.
//!
//! Typing at the end of a note used to leave the caret riding the window's
//! bottom edge, because the default cursor-follow reveals the caret's row box
//! and nothing more. [`TextPipeline::end_pad_px`] adds air below the last line
//! and [`TextPipeline::scroll_to_show_row_pos`] spends it, so the caret settles
//! clear of the edge.
//!
//! **The whole feature's non-negotiable half is that the air is VIRTUAL.** It is
//! a term in a scroll coordinate and nothing else: no rope insert, no view text,
//! no `disk_bytes` addend. [`end_pad_is_virtual_and_no_byte_of_it_reaches_disk`]
//! is the headline here and it drives a REAL save through the real filesystem
//! seam rather than inspecting a struct, because "the buffer looks unchanged" and
//! "the file is unchanged" are different claims and autosave writes the second.
//!
//! The other laws pin what the pad is allowed to change: how deep the document
//! scrolls (one owner, so a wheel, a drag and `Cmd-Down` cannot disagree), how it
//! behaves at every document height around the viewport boundary, that typewriter
//! mode and the pad do not fight, and — over real pixels — that the caret is
//! DRAWN clear of the bottom edge and that the air it sits in is the PAGE's own
//! ground rather than a frame.

use super::super::*;
use super::{H, headless_dqp, headless_pipeline, pixeldiff, view};
use crate::render::scroll::END_PAD_ROWS;

/// A document of `n` short, non-wrapping lines — one visual row each — so the
/// arithmetic below is exact rather than wrap-dependent.
fn rows(n: usize) -> String {
    (0..n)
        .map(|i| format!("row {i}\n"))
        .collect::<Vec<_>>()
        .concat()
}

/// The visual row the caret occupies at the very end of `text`.
fn last_row(p: &TextPipeline) -> usize {
    p.total_visual_rows() - 1
}

/// THE HEADLINE. The breathing room never becomes a byte.
///
/// The oracle is the FILE, read back through the same `FileSystem` the product
/// wrote it through, after a real `Buffer::save` — not `buffer.text()`, not a
/// length field. Autosave is what makes that distinction load-bearing: the way
/// this feature could hurt someone is by committing junk whitespace to their
/// note, and only a save can do that.
///
/// Non-vacuity is the other half and it is what makes the law able to fail: the
/// pad must actually be ENGAGED at the moment of the save. So the fixture proves,
/// before saving, that the settled cursor-follow scroll has entered the virtual
/// space (`scroll_top > doc_height - viewport`) — a document that never reached
/// the pad would round-trip its bytes no matter what the pad did.
#[test]
fn end_pad_is_virtual_and_no_byte_of_it_reaches_disk() {
    let _g = crate::testlock::serial();
    use crate::buffer::Buffer;
    use crate::fs::FileSystem;
    use std::sync::Arc;

    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping end_pad_is_virtual_and_no_byte_of_it_reaches_disk: no wgpu adapter");
        return;
    };

    // Deliberately ends WITHOUT a trailing newline: an implementation that made
    // the pad real by appending blank lines would be free to call the result
    // "just a trailing newline", and this fixture refuses that cover.
    let raw = format!("{}last line, no trailing newline", rows(120));
    let path = std::path::PathBuf::from("/notes/journal.md");
    let mem = crate::fs::InMemoryFs::new().with_file(&path, &raw);

    crate::fs::with_fs(Arc::new(mem.clone()), || {
        let mut buf = Buffer::from_file(&path);
        assert_eq!(buf.text(), raw, "the fixture loads byte-identical");

        // Drive the REAL default cursor-follow with the caret at the document's
        // very end — the journaling posture the report was filed about.
        let end_line = buf.line_count() - 1;
        p.set_view(&view(&buf.text(), end_line, 0));
        let row = last_row(&p);
        let settled = p.scroll_to_show_row_pos(row, ScrollPos::default(), H);

        // PRESENCE FLOOR: the pad is real and engaged, so the save below is
        // being asked the question this law exists to ask.
        let pad = p.end_pad_px(H);
        assert!(
            pad > 0.0,
            "the fixture must be tall enough for the pad to exist (doc {:.1}px, \
             viewport {:.1}px)",
            p.total_doc_height(),
            p.viewport_avail_px(H)
        );
        let into_virtual =
            p.scroll_top_px(settled) - (p.total_doc_height() - p.viewport_avail_px(H));
        assert!(
            (into_virtual - pad).abs() <= 1.0,
            "cursor-follow must settle a full pad into the VIRTUAL space, else \
             this law's save proves nothing (entered {into_virtual:.2}px of a \
             {pad:.2}px pad)"
        );

        // THE ASSERTION. A real save, then the real bytes.
        buf.save().unwrap();
        let on_disk = mem.read(&path).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&on_disk),
            raw,
            "the end-of-document pad reached the FILE — it is virtual space drawn \
             by the renderer, and `Buffer::disk_bytes` is the sole author of what \
             lands on disk"
        );
        assert_eq!(
            on_disk.len(),
            raw.len(),
            "on-disk byte length changed by {} — no part of the {pad:.1}px pad may \
             become a byte",
            on_disk.len() as i64 - raw.len() as i64
        );
        // And the in-memory document is untouched too: the scroll computation
        // above must not have been able to reach the rope at all.
        assert_eq!(buf.text(), raw, "the pad never enters the buffer either");
    });
}

/// The pad exists exactly when the document OVERFLOWS the window, at every
/// document height around that boundary and at both DPI tiers.
///
/// The four heights are genuinely different products: an empty document and a
/// one-line document have nothing to breathe past; a document exactly one screen
/// tall already has its air and must not be made to scroll; one line taller is
/// the first height where the pad is owed. Swept at DPI 1 and 2 because the pad
/// is `line_height`-derived and a scale-blind spelling would pass at one tier.
#[test]
fn end_pad_appears_only_once_the_document_overflows_the_window() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping end_pad_appears_only_once_the_document_overflows_the_window: no wgpu adapter"
        );
        return;
    };

    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        // Calibrate the fixture against THIS scale's own row pitch rather than a
        // hardcoded count: the number of rows that fills the window is a metric,
        // not a constant.
        p.set_view(&view(&rows(4), 0, 0));
        let line_h = p.metrics.line_height;
        let avail = p.viewport_avail_px(H);
        let fits = (avail / line_h).floor() as usize;
        // NON-VACUITY: every equality below is trivially true at a zero pad, and a
        // zero pad IS the bug this file exists to prevent returning.
        assert!(
            END_PAD_ROWS.0 >= 1.0,
            "the pad is {} rows — at zero this law asserts 0 == 0 in every cell",
            END_PAD_ROWS.0
        );
        assert!(
            fits >= 4,
            "dpi {dpi}: the window must hold several rows for this sweep to mean \
             anything (fits {fits})"
        );

        for (label, text, overflows) in [
            ("empty", String::new(), false),
            ("one line", "only".to_string(), false),
            // `rows(n)` ends with '\n', so it lays out n+1 visual rows; ask for
            // exactly `fits` rows of document.
            ("exactly one screen", rows(fits - 1), false),
            ("one row taller", rows(fits), true),
            ("much taller", rows(fits * 6), true),
        ] {
            p.set_view(&view(&text, 0, 0));
            let doc_h = p.total_doc_height();
            let pad = p.end_pad_px(H);
            let at = format!("dpi {dpi} · {label} · doc {doc_h:.1}px · viewport {avail:.1}px");

            if overflows {
                assert!(
                    doc_h > avail,
                    "fixture mis-calibrated — {label} does not overflow — {at}"
                );
                assert_eq!(
                    pad.to_bits(),
                    (line_h * END_PAD_ROWS.0).to_bits(),
                    "an overflowing document owes exactly {} rows of air — {at}",
                    END_PAD_ROWS.0
                );
            } else {
                assert!(
                    doc_h <= avail,
                    "fixture mis-calibrated — {label} overflows — {at}"
                );
                assert_eq!(pad, 0.0, "a document that fits invents no air — {at}");
                // …and therefore does not scroll at all: the whole point of the
                // gate is that a one-screen note stays still.
                assert_eq!(
                    p.scroll_by_px(ScrollPos::default(), 1_000_000.0, H),
                    ScrollPos::default(),
                    "a document that fits must not become scrollable — {at}"
                );
                let row = last_row(&p);
                assert_eq!(
                    p.scroll_to_show_row_pos(row, ScrollPos::default(), H),
                    ScrollPos::default(),
                    "following the caret to the end of a document that fits must \
                     not move the view — {at}"
                );
            }
        }
    }
    p.set_dpi(1.0);
}

/// The WHEEL, a selection DRAG and `Cmd-Down` all agree about where the document
/// ends.
///
/// They are three different code paths onto one quantity — the wheel and the drag
/// carry pixel deltas through `canonicalize_incremental`, `Cmd-Down` moves the
/// caret and lets cursor-follow resolve an absolute coordinate through
/// `scroll_pos_at_q` — and before the pad had one owner they would have
/// disagreed by exactly the pad. This is the law that says they cannot.
#[test]
fn wheel_drag_and_buffer_end_land_on_the_same_deepest_scroll() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping wheel_drag_and_buffer_end_land_on_the_same_deepest_scroll: no wgpu adapter"
        );
        return;
    };
    let text = rows(200);
    p.set_view(&view(&text, 200, 0));
    let row = last_row(&p);
    let pad = p.end_pad_px(H);
    assert!(pad > 0.0, "the fixture must overflow the viewport");

    // WHEEL: one enormous packet, and also many small ones — the incremental
    // carry and the absolute resolver must stop at the same place.
    let wheel = p.scroll_by_px(ScrollPos::default(), 1_000_000.0, H);
    let mut ratchet = ScrollPos::default();
    for _ in 0..4000 {
        ratchet = p.scroll_by_px(ratchet, 3.0, H);
    }
    assert_eq!(
        wheel, ratchet,
        "a flick and a slow scroll must reach the same document end"
    );

    // `Cmd-Down` (`Action::BufferEnd`): the caret lands on the last row and the
    // DEFAULT cursor-follow resolves the scroll.
    let buffer_end = p.scroll_to_show_row_pos(row, ScrollPos::default(), H);
    assert_eq!(
        buffer_end, wheel,
        "Cmd-Down and a wheel scrolled to the bottom must settle identically \
         (buffer_end={buffer_end:?}, wheel={wheel:?})"
    );

    // A selection DRAG is cursor-follow too (typewriter off ⇒ ShowRow), reached
    // from wherever the drag started; arriving from ANY depth lands in the same
    // place, so a drag to the end and a Cmd-Down cannot differ.
    for from in [0usize, 40, 120, row] {
        let start = p.scroll_by_px(ScrollPos::at_row(from), 0.0, H);
        assert_eq!(
            p.scroll_to_show_row_pos(row, start, H),
            buffer_end,
            "following the caret to the last row from scroll row {from} must \
             settle at the document's one deepest scroll"
        );
    }

    // And that deepest scroll IS the pad: the last line's bottom sits a full pad
    // above the window's bottom edge.
    let gap = p.viewport_avail_px(H) - (p.total_doc_height() - p.scroll_top_px(wheel));
    assert!(
        (gap - pad).abs() <= 1.0,
        "at the deepest scroll the last line must clear the bottom edge by the \
         pad (gap {gap:.2}px, pad {pad:.2}px)"
    );
}

/// The pad ramps CONTINUOUSLY into the end of the document rather than switching
/// on at the last row, and following the caret downward never scrolls up.
///
/// The axis the author of a "caret on the last line" rule would not think of is
/// the row BEFORE the last one: a pad applied only to the final row makes the
/// view jump by a pad-and-a-row on one keystroke. Here the follow scroll is
/// required to be monotonic in the row, and the slack strictly between zero and
/// the full pad somewhere in the approach — so a step function fails.
#[test]
fn the_pad_ramps_into_the_document_end_and_follow_stays_monotonic() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping the_pad_ramps_into_the_document_end_and_follow_stays_monotonic: \
             no wgpu adapter"
        );
        return;
    };
    p.set_view(&view(&rows(200), 0, 0));
    let total = p.total_visual_rows();
    let pad = p.end_pad_px(H);
    assert!(pad > 0.0, "the fixture must overflow the viewport");

    let mut previous = 0.0f32;
    let mut partial = 0usize;
    for row in 0..total {
        let slack = p.end_pad_below_row(row, H);
        assert!(
            (0.0..=pad).contains(&slack),
            "row {row}: slack {slack} escapes [0, {pad}]"
        );
        if slack > 0.0 && slack < pad {
            partial += 1;
        }
        let settled = p.scroll_top_px(p.scroll_to_show_row_pos(row, ScrollPos::default(), H));
        assert!(
            settled >= previous,
            "row {row}: following the caret downward scrolled UP \
             ({settled:.2} after {previous:.2})"
        );
        previous = settled;
    }
    assert_eq!(
        p.end_pad_below_row(0, H),
        0.0,
        "a row with a whole document beneath it borrows no air"
    );
    assert_eq!(
        p.end_pad_below_row(total - 1, H).to_bits(),
        pad.to_bits(),
        "the last row borrows the whole pad"
    );
    assert!(
        partial >= 2,
        "the ramp swept nothing — {partial} rows carried a PARTIAL slack, so a \
         last-row-only step function would pass this law"
    );
}

/// The pad and TYPEWRITER mode do not fight.
///
/// Both hold the caret off the bottom edge, so the question is what happens when
/// both are in force. They compose through one clamp: the typewriter pin centres
/// the caret row and then resolves through the same `max_scroll_q` the pad
/// widened, so at the document END the two land on the SAME scroll — the pin has
/// asked to go deeper than the document allows and the pad is exactly how much
/// deeper it may go. Away from the end the pin is strictly deeper, which is what
/// makes it a different mode rather than a louder pad.
#[test]
fn typewriter_and_the_end_pad_compose_through_one_clamp() {
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping typewriter_and_the_end_pad_compose_through_one_clamp: no wgpu adapter");
        return;
    };
    p.set_view(&view(&rows(200), 0, 0));
    let total = p.total_visual_rows();
    let last = total - 1;
    let pad = p.end_pad_px(H);
    assert!(pad > 0.0, "the fixture must overflow the viewport");

    let mid = total / 2;
    assert!(
        p.scroll_top_px(p.scroll_to_center_row_pos(mid, H))
            > p.scroll_top_px(p.scroll_to_show_row_pos(mid, ScrollPos::default(), H)),
        "mid-document the typewriter pin must still scroll further than the \
         default follow — otherwise the pad has swallowed the mode"
    );
    assert_eq!(
        p.scroll_to_center_row_pos(last, H),
        p.scroll_to_show_row_pos(last, ScrollPos::default(), H),
        "at the document end both mechanisms rest on the same clamp; neither may \
         pull the view somewhere the other would not"
    );
    // The pin is still bounded by the widened clamp, never past it.
    for row in 0..total {
        let pin = p.scroll_top_px(p.scroll_to_center_row_pos(row, H));
        assert!(
            pin <= p.total_doc_height() + pad - p.viewport_avail_px(H) + 0.5,
            "row {row}: the typewriter pin escaped the document's deepest scroll"
        );
    }
}

/// REAL PIXELS. At the settled end-of-document follow, the caret is DRAWN clear
/// of the window's bottom edge, and the air it sits in is the PAGE's own ground.
///
/// The caret is located by ARITHMETIC over two rendered frames rather than by
/// asking the renderer where it put it: the same scene at the same scroll, once
/// with the caret at the document end and once with it parked far above the
/// viewport, differ in exactly the caret's ink. The bounding box of that
/// difference is the caret, in pixels.
///
/// The second half answers the other constraint the decision carried — the air
/// must read as PAGE, not as frame. Two adjacent bands are compared in a slice of
/// the writing column the fixture's short lines never reach: blank page above,
/// virtual space below, and they must be perceptually the same ground. That claim
/// alone is satisfiable by a frame that renders nothing at all, so it is paired
/// with a PRESENCE floor over the same two bands in the slice where the glyphs
/// ARE — text above, air below, which must be plainly different.
#[test]
fn the_caret_at_the_document_end_is_drawn_clear_of_the_bottom_edge() {
    let _g = crate::testlock::serial();
    // The caret's own drawn form is mode-keyed, and its settle is an animation:
    // pin BLOCK and settle, or the frames below carry no caret at all and the
    // difference this law measures is empty. `TogglesRestore` returns the
    // ambient mode on the unwinding path too.
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(crate::caret::CaretMode::Block);
    const W: u32 = 1200;
    const CH: u32 = 800;
    let Some((device, queue, mut p)) = headless_dqp(W as f32, CH as f32) else {
        eprintln!(
            "skipping the_caret_at_the_document_end_is_drawn_clear_of_the_bottom_edge: \
             no wgpu adapter"
        );
        return;
    };

    let text = rows(200);
    let end_line = text.matches('\n').count();
    let mut v = view(&text, end_line, 0);
    p.set_view(&v);
    let row = last_row(&p);
    let pad = p.end_pad_px(CH as f32);
    assert!(pad > 0.0, "the fixture must overflow the viewport");
    let settled = p.scroll_to_show_row_pos(row, ScrollPos::default(), CH as f32);

    // Both frames at the SAME scroll, so the document's own ink is identical and
    // only the caret can differ.
    v.scroll = settled;
    p.set_view(&v);
    p.settle_caret();
    p.prepare(&device, &queue, W, CH).unwrap();
    let with_caret = pixeldiff::render_frame(&mut p, &device, &queue, W, CH);

    let mut parked = view(&text, 0, 0);
    parked.scroll = settled;
    p.set_view(&parked);
    p.settle_caret();
    p.prepare(&device, &queue, W, CH).unwrap();
    let without_caret = pixeldiff::render_frame(&mut p, &device, &queue, W, CH);

    // The caret's pixel bounding box.
    let (mut top, mut bottom, mut count) = (i64::MAX, i64::MIN, 0usize);
    for y in 0..CH as i64 {
        for x in 0..W as i64 {
            let i = (y * W as i64 + x) as usize;
            if with_caret[i] != without_caret[i] {
                top = top.min(y);
                bottom = bottom.max(y);
                count += 1;
            }
        }
    }
    assert!(
        count > 20,
        "the two frames differ in {count} pixels — the caret was not drawn, so \
         this law located nothing"
    );
    let clearance = CH as f32 - bottom as f32;
    assert!(
        clearance >= pad - 1.0,
        "the caret's drawn ink reaches y={bottom} of a {CH}px canvas — only \
         {clearance:.1}px of clearance where the pad owes {pad:.1}px (caret band \
         {top}..={bottom})"
    );

    // The air is the PAGE's ground. Sample two adjacent bands one pad tall: the
    // virtual space at the very bottom, and the band directly above it.
    let band_h = pad.floor().max(2.0);
    let col_left = p.column_left();
    let col_w = p.page_column_width();
    assert!(
        col_w > 40.0,
        "the writing column must be wide enough to have an empty right slice \
         (width {col_w})"
    );
    let air = |x: f32, w: f32| pixeldiff::Region::new(x, CH as f32 - band_h, w, band_h);
    let above = |x: f32, w: f32| pixeldiff::Region::new(x, CH as f32 - 2.0 * band_h, w, band_h);

    let mean = |r: pixeldiff::Region| mean_pixel(&without_caret, W as i64, CH as i64, r);

    // GLYPH slice — the fixture's `row NNN` lines live in the column's left
    // quarter. Text above, air below: plainly different. This is the presence
    // floor that stops the sameness claim below being satisfied by a blank frame.
    let ink_w = col_w * 0.25;
    let ink_above = mean(above(col_left, ink_w));
    let ink_air = mean(air(col_left, ink_w));
    let ink_de = pixeldiff::delta_e(ink_above, ink_air);
    assert!(
        ink_de > pixeldiff::CLASSIC_JND,
        "the band above the air and the air itself measure ΔE {ink_de:.2} apart in \
         the GLYPH slice — the fixture drew no text there, so the sameness check \
         below would be a claim about nothing (above {ink_above:?}, air {ink_air:?})"
    );

    // EMPTY slice — the column's right side, which no `row NNN` line reaches.
    // Blank page above, virtual space below: the SAME ground.
    let empty_x = col_left + col_w * 0.6;
    let empty_w = col_w * 0.35;
    let ground_above = mean(above(empty_x, empty_w));
    let ground_air = mean(air(empty_x, empty_w));
    let de = pixeldiff::delta_e(ground_above, ground_air);
    assert!(
        de < pixeldiff::CLASSIC_JND,
        "the virtual space is ΔE {de:.2} from the page directly above it — the \
         pad must extend the PAGE's ground, never let the caret float over the \
         world's margins (page {ground_above:?}, air {ground_air:?})"
    );
}

/// The mean colour of a region, clamped to the buffer's bounds — the ground a
/// band of page averages to. Averaging (not [`pixeldiff::dominant_ink_color`]'s
/// mode) is the right instrument here because the question is about a broad
/// area's ground rather than about which colour glyphs are drawn in: a few
/// glyphs in an otherwise blank band must MOVE the answer.
fn mean_pixel(pixels: &[[u8; 4]], width: i64, height: i64, r: pixeldiff::Region) -> [u8; 4] {
    let x0 = r.x.max(0);
    let y0 = r.y.max(0);
    let x1 = (r.x + r.w).min(width);
    let y1 = (r.y + r.h).min(height);
    let mut sums = [0u64; 4];
    let mut n = 0u64;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = pixels[(y * width + x) as usize];
            for c in 0..4 {
                sums[c] += u64::from(p[c]);
            }
            n += 1;
        }
    }
    assert!(n > 0, "mean_pixel: empty region {r:?}");
    std::array::from_fn(|c| (sums[c] / n) as u8)
}
