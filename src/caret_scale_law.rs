//! src/caret_scale_law.rs — THE ONE-DISPLAY-SCALE LAW for caret geometry.
//!
//! WHY: [`crate::render::Metrics`] STORES `scale` (`zoom * dpi`) so that, in its
//! own words, "a consumer cannot invent a second one". Three caret sites invented
//! one anyway — dividing an already-scaled length back down
//! (`metrics.caret_h / CARET_H`) and handing the quotient to [`Logical::px`] —
//! and two doc comments asserted the quotient was "exactly `Metrics::scale`".
//!
//! IT IS NOT. `(CARET_H * s) / CARET_H` is not a round trip in `f32`:
//! [`recovering_the_display_scale_by_division_is_not_bit_exact`] measures the
//! whole authored zoom grid against six real display factors and finds nine
//! (zoom, dpi) pairs where the quotient is one ULP away from the stored value.
//! The disagreement is ~1e-6 device pixels on a 3-logical-pixel pad — far below
//! anything a rasterizer can show, so this is a FALSE CHECKABLE CLAIM rather
//! than a rendering defect. The two laws here make the claim true and keep it
//! true: one measures the arithmetic, the other refuses a fourth site.
//!
//! WHY IT WAS INVISIBLE, and this is the transferable part: the mismatch is
//! empty at dpi 1 and dpi 2 — multiplying by 1 or 2 only moves a binary
//! exponent, so the division undoes it exactly — and dpi 1 and 2 are the only
//! two factors any headless capture uses. Fractional Wayland scaling (1.25 /
//! 1.5 / 1.75) and a 3× display are ordinary values of
//! `gpu.window.scale_factor()`, and four of them mismatch. The sweep therefore
//! pins the exact scales the evidence pipeline cannot reach.
#![cfg(test)]

use crate::range::ZOOM;
use crate::render::{CARET_H, Metrics, clamp_zoom};
use crate::srgb_eotf_law::{repo_root, src_rs_files, strip_cfg_test_blocks, strip_line_comments};

/// Real display factors, INCLUDING the fractional ones no capture ever uses.
/// `1.25`/`1.5`/`1.75` are what a fractional-scaling Wayland compositor hands
/// `winit`, and `3.0` is a real macOS/Android density; `1.0` and `2.0` are the
/// two the harness runs at, kept in the sweep precisely so the law records that
/// they are the exact ones — an oracle that swept only those would report this
/// whole family as clean.
const DPIS: [f32; 6] = [1.0, 1.25, 1.5, 1.75, 2.0, 3.0];

/// Every authored zoom on `range::ZOOM`'s own grid, derived from the spec rather
/// than typed out, so a widened band or a finer step enrols itself.
fn authored_zooms() -> Vec<f32> {
    (ZOOM.min_step()..=ZOOM.max_step())
        .map(|k| ZOOM.value_of_step(k))
        .collect()
}

/// THE CHAIN, bit-exactly: `scale` is the product of the clamped zoom and the
/// DPI, and every enrolled length is that stored factor times its constant. This
/// is what makes `metrics.scale` the ONE factor a caret site is entitled to read;
/// without it, "read `scale` instead of dividing" would just be a preference
/// between two unverified numbers.
#[test]
fn metrics_scale_is_the_stored_product_and_every_length_is_built_from_it() {
    let zooms = authored_zooms();
    assert_eq!(
        zooms.len(),
        ZOOM.step_count() as usize,
        "the zoom grid enrolled {} of {} authored positions — the sweep has stopped \
         covering the band it claims to",
        zooms.len(),
        ZOOM.step_count()
    );
    for &dpi in &DPIS {
        for &zoom in &zooms {
            let m = Metrics::with_dpi(zoom, dpi);
            let s = clamp_zoom(zoom) * dpi;
            assert_eq!(
                m.scale.to_bits(),
                s.to_bits(),
                "Metrics::with_dpi({zoom}, {dpi}).scale is {} but zoom*dpi is {s} — the \
                 stored factor is no longer the product it is documented as",
                m.scale
            );
            assert_eq!(
                m.caret_h.to_bits(),
                (CARET_H * m.scale).to_bits(),
                "caret_h at (zoom {zoom}, dpi {dpi}) is not CARET_H * scale"
            );
        }
    }
}

/// THE MEASUREMENT that the source scan below exists for. Dividing an
/// already-scaled length back down does NOT recover the stored factor
/// bit-exactly, and the set of pairs where it fails is a property of `f32`, not
/// of this host: `1.0` and `2.0` are exact for the whole grid, and the failures
/// land on the fractional and 3× factors no capture reaches.
///
/// The law asserts BOTH halves on purpose. "Some pairs mismatch" alone would
/// stay green if the exact pairs became inexact, and "dpi 1 and 2 are exact"
/// alone would stay green if the division became a perfect round trip and made
/// the scan pointless. Together they pin the reason.
#[test]
fn recovering_the_display_scale_by_division_is_not_bit_exact() {
    let zooms = authored_zooms();
    let mut mismatches: Vec<(f32, f32, i64)> = Vec::new();
    let mut worst_pad_delta = 0.0f32;
    for &dpi in &DPIS {
        for &zoom in &zooms {
            let m = Metrics::with_dpi(zoom, dpi);
            let recovered = m.caret_h / CARET_H;
            let ulps = recovered.to_bits() as i64 - m.scale.to_bits() as i64;
            if ulps != 0 {
                mismatches.push((dpi, zoom, ulps));
            }
            // The widest caret pad this factor is ever applied to, so the report
            // carries the real magnitude rather than an abstract ULP count.
            let pad = crate::render::CARET_INK_PAD;
            worst_pad_delta = worst_pad_delta.max((pad.px(recovered) - pad.px(m.scale)).abs());
        }
    }

    let exact_at = |dpi: f32| {
        zooms.iter().all(|&z| {
            let m = Metrics::with_dpi(z, dpi);
            (m.caret_h / CARET_H).to_bits() == m.scale.to_bits()
        })
    };
    assert!(
        exact_at(1.0) && exact_at(2.0),
        "dpi 1 and dpi 2 are supposed to be the EXACT factors — the two the capture \
         harness runs at, and the reason this family was silent. If they are now inexact, \
         every byte-identity claim taken at those scales needs re-reading"
    );

    assert!(
        !mismatches.is_empty(),
        "dividing caret_h by CARET_H now recovers Metrics::scale bit-exactly at all \
         {} (zoom, dpi) pairs. If that is genuinely true of f32 on this target, the \
         source scan beside this law is guarding nothing and should be re-argued rather \
         than kept as ceremony",
        zooms.len() * DPIS.len()
    );
    assert!(
        mismatches
            .iter()
            .all(|&(dpi, _, _)| dpi != 1.0 && dpi != 2.0),
        "a mismatch appeared at dpi 1 or 2, contradicting the exactness assertion above: \
         {mismatches:?}"
    );
    assert!(
        mismatches.iter().all(|&(_, _, u)| u.abs() == 1),
        "the recovery error grew past one ULP, so this is no longer only a false claim: \
         {mismatches:?}"
    );
    // The magnitude, pinned loosely on purpose: the claim being defended is "this
    // is not a rendering defect", and one thousandth of a device pixel is the
    // generous side of that.
    assert!(
        worst_pad_delta < 1.0e-3,
        "the recovered scale now moves a caret pad by {worst_pad_delta} device pixels — \
         past the point where this family can be called invisible, and the item that \
         filed it as a documentation defect needs reopening as a rendering one"
    );
}

/// The recovery's spelling, whitespace-normalized so `a.caret_h / CARET_H` and
/// `a.caret_h/CARET_H` are the same needle.
const NEEDLE: &str = "caret_h/CARET_H";

/// THE LAW. No PRODUCTION source file re-derives the display scale from an
/// already-scaled caret length: every caret site reads the one stored
/// `Metrics::scale`. A fourth site fails here by name.
///
/// Test paths are excluded, not blessed: a test that recovers the scale by
/// division is asserting against a factor one ULP from the one the product uses,
/// which is a real (if equally invisible) oracle divergence at dpi 1.5. It is out
/// of scope here rather than correct.
///
/// WHAT THIS CANNOT SEE — a source scan is a grep, not a compiler:
/// - A recovery spelled through a differently named local (`let h = m.caret_h;
///   h / CARET_H`) or through another already-scaled length
///   (`m.line_height / LINE_HEIGHT`) is a different needle and passes.
/// - Only `src/**/*.rs` is scanned; a shader doing the same division is outside
///   its reach.
/// - Comment text is stripped before scanning, so prose may still DESCRIBE the
///   old recovery. Cutting each line at its first `//` means a `//` inside a
///   string literal truncates that line early — an arithmetic bypass cannot hide
///   there, only a needle written inside a string.
#[test]
fn no_production_site_recovers_the_display_scale_by_division() {
    let root = repo_root();
    let mut offenders: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    let mut test_paths = 0usize;
    for rel in src_rs_files(&root) {
        if rel == "src/caret_scale_law.rs" {
            continue; // this law's own doc names the needle it forbids
        }
        if crate::srgb_eotf_law::is_test_path(&rel) {
            test_paths += 1;
            continue;
        }
        let text = std::fs::read_to_string(root.join(&rel)).expect("read src file");
        scanned += 1;
        let code = strip_line_comments(&strip_cfg_test_blocks(&text));
        let squeezed: String = code.chars().filter(|c| !c.is_whitespace()).collect();
        let sites = squeezed.matches(NEEDLE).count();
        if sites > 0 {
            offenders.push(format!("{rel} ({sites} site(s))"));
        }
    }
    // Enrolment, named in the failure message rather than assumed: a scan that
    // silently stopped finding files would otherwise pass by sweeping nothing.
    assert!(
        scanned > 200 && test_paths > 20,
        "the scan enrolled {scanned} production and {test_paths} test files under src/ — \
         the tree walk has stopped seeing the tree, and this law is checking almost nothing"
    );
    offenders.sort();
    assert!(
        offenders.is_empty(),
        "these production files re-derive the display scale by dividing an already-scaled \
         caret length ({NEEDLE}): {offenders:?} — read `metrics.scale`, the ONE stored \
         factor, which the quotient is not bit-equal to at dpi 1.25/1.5/1.75/3 (see \
         `recovering_the_display_scale_by_division_is_not_bit_exact`). Scanned {scanned} \
         production files"
    );
}
