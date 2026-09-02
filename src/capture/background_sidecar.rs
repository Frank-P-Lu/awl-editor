//! The `page.background` sidecar arm — one JSON shape per `theme::Background`.
//!
//! Carved out of `sidecar.rs` alongside its `scroll_sidecar`/`layout_sidecar`/
//! `replay_sidecar` siblings when Deckle added the background's tenth arm.
//! The split is mechanical: the same three functions, verbatim, with the
//! shared `json_string` helper imported instead of colocated.

use super::sidecar::json_string;

#[rustfmt::skip]
pub(super) fn background_json(
    bg: crate::theme::Background,
    lava_phase: f32,
    warp: crate::warpgrid::WarpRender,
) -> String {
    use crate::theme::Background;
    match bg {
        Background::Gradient { .. } | Background::Dots { .. }
        | Background::Pinstripe { .. } | Background::Stripes { .. } => simple_background_json(bg),
        _ => rich_background_json(bg, lava_phase, warp),
    }
}
#[rustfmt::skip]
fn simple_background_json(bg: crate::theme::Background) -> String {
    use crate::theme::Background;
    let hex = |c: crate::theme::Srgb| json_string(&c.hex());
    match bg {
        Background::Gradient { from, to, dir } => format!(
            concat!("{{ \"kind\": \"gradient\", \"from\": {}, \"to\": {}, ",
                "\"dir\": [{}, {}] }}"),
            hex(from), hex(to), dir.0, dir.1),
        Background::Dots { from, to, dir, tint, edge } => format!(
            concat!("{{ \"kind\": \"dots\", \"from\": {}, \"to\": {}, ",
                "\"dir\": [{}, {}], \"tint\": {}, \"edge\": {} }}"),
            hex(from), hex(to), dir.0, dir.1, hex(tint), edge),
        Background::Pinstripe { from, to, dir, tint } => format!(
            concat!("{{ \"kind\": \"pinstripe\", \"from\": {}, \"to\": {}, ",
                "\"dir\": [{}, {}], \"tint\": {} }}"),
            hex(from), hex(to), dir.0, dir.1, hex(tint)),
        Background::Stripes { from, to, band, angle } => format!(
            concat!("{{ \"kind\": \"stripes\", \"from\": {}, \"to\": {}, ",
                "\"band\": {}, \"angle\": {} }}"),
            hex(from), hex(to), hex(band), angle),
        _ => unreachable!("simple background helper received a rich ground"),
    }
}
#[rustfmt::skip]
fn rich_background_json(
    bg: crate::theme::Background,
    lava_phase: f32,
    warp: crate::warpgrid::WarpRender,
) -> String {
    use crate::theme::Background;
    let hex = |c: crate::theme::Srgb| json_string(&c.hex());
    match bg {
        Background::Lava {
            ground,
            blob_lo,
            blob_hi,
            dithered,
        } => format!(
            concat!("{{ \"kind\": \"lava\", \"ground\": {}, \"blob_lo\": {}, \"blob_hi\": {}, ",
                "\"dithered\": {}, \"phase\": {} }}"),
            hex(ground),
            hex(blob_lo),
            hex(blob_hi),
            dithered,
            lava_phase
        ), Background::Bands { tones, angle } => format!(
            "{{ \"kind\": \"bands\", \"tones\": [{}, {}, {}], \"angle\": {} }}",
            hex(tones[0]),
            hex(tones[1]),
            hex(tones[2]),
            angle
        ), Background::Waves { tones } => format!(
            "{{ \"kind\": \"waves\", \"tones\": [{}, {}, {}] }}",
            hex(tones[0]),
            hex(tones[1]),
            hex(tones[2])
        ), Background::Zigzag {
            from,
            to,
            dir,
            tint,
            period_px,
            amplitude_px,
            angle,
            density,
            banded,
        } => format!(
            concat!(
                "{{\"kind\":\"zigzag\",\"from\":{},\"to\":{},\"dir\":[{},{}],\"tint\":{},",
                "\"period_px\":{},\"amplitude_px\":{},\"angle\":{},\"density\":{},\"banded\":{}}}"
            ),
            hex(from),
            hex(to),
            dir.0,
            dir.1,
            hex(tint),
            period_px,
            amplitude_px,
            angle,
            density,
            banded
        ), Background::Organic { tones, scale_px, density } => format!(
            concat!(
                "{{\"kind\":\"organic\",\"tones\":[{},{},{}],",
                "\"scale_px\":{},\"density\":{},\"phase\":{}}}"
            ),
            hex(tones[0]), hex(tones[1]), hex(tones[2]),
            scale_px, density, lava_phase
        ), Background::Deckle {
            ground, layer, deckle, weave, period_px, wander_px, density
        } => format!(
            concat!("{{\"kind\":\"deckle\",\"ground\":{},\"layer\":{},",
                "\"deckle\":{},\"weave\":\"{}\",\"period_px\":{},",
                "\"wander_px\":{},\"density\":{},\"static\":true}}"),
            hex(ground), hex(layer), hex(deckle), weave.as_str(),
            period_px, wander_px, density
        ), Background::WarpedGrid {
            ground, minor, major, tunnel, spacing_px, density, fold, twist, forward_drift, ribs
        } => format!(
            concat!("{{\"kind\":\"warped-grid\",\"ground\":{},\"minor\":{},",
                "\"major\":{},\"tunnel\":\"{}\",\"spacing_px\":{},",
                "\"density\":{},\"fold\":{},\"twist\":{},\"forward_drift\":{},",
                "\"ribs\":{},\"forward_cells\":{},",
                "\"vanishing_point\":{{\"x\":{},\"y\":{}}},",
                "\"holding\":{},\"from\":\"{}\",\"to\":\"{}\",\"transit_t\":{},",
                "\"calm\":{}}}"),
            hex(ground), hex(minor), hex(major), tunnel.as_str(), spacing_px,
            density, fold, twist, forward_drift, ribs, warp.travel_cells,
            warp.axis_frac.0, warp.axis_frac.1,
            warp.holding, warp.from.as_str(), warp.to.as_str(), warp.transit_t,
            warp.calm
        ), _ => unreachable!("rich background helper received a simple ground"),
    }
}

#[cfg(test)]
mod tests {
    use super::background_json;
    use crate::warpgrid::{VpCorner, WarpRender};

    fn holding_at(corner: VpCorner, travel: f32) -> WarpRender {
        WarpRender {
            axis_frac: corner.frac(),
            travel_cells: travel,
            holding: true,
            from: corner,
            to: corner,
            transit_t: 0.0,
            calm: false,
        }
    }

    #[test]
    fn kite_sidecar_reports_the_resolved_pose_and_forward_travel() {
        let json = background_json(crate::theme::KITE.background, 9.0, holding_at(VpCorner::TopRight, 12.5));
        for field in [
            "\"kind\":\"warped-grid\"",
            "\"tunnel\":\"fixed\"",
            "\"forward_cells\":12.5",
            "\"vanishing_point\":{\"x\":0.8,\"y\":0.24}",
            "\"holding\":true",
            "\"from\":\"top-right\"",
            "\"to\":\"top-right\"",
            "\"calm\":false",
            "\"fold\":0.34",
            "\"twist\":0.72",
            "\"ribs\":58",
        ] {
            assert!(json.contains(field), "missing {field}: {json}");
        }
        for retired in ["curvature", "yaw", "pitch"] {
            assert!(!json.contains(retired), "retired field {retired}: {json}");
        }
    }

    #[test]
    fn calm_pose_reports_calm_true() {
        let calm = WarpRender {
            axis_frac: VpCorner::TopRight.frac(),
            travel_cells: 0.0,
            holding: true,
            from: VpCorner::TopRight,
            to: VpCorner::TopRight,
            transit_t: 0.0,
            calm: true,
        };
        let json = background_json(crate::theme::KITE.background, 0.0, calm);
        assert!(json.contains("\"calm\":true"), "{json}");
    }
}
