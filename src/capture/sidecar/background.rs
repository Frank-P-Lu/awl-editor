use super::json_string;
use crate::theme::Background;

pub(super) fn background_json(bg: Background, phase: f32) -> String {
    match bg {
        Background::Gradient { .. }
        | Background::Dots { .. }
        | Background::Starfield { .. }
        | Background::Pinstripe { .. }
        | Background::Stripes { .. } => simple(bg),
        _ => rich(bg, phase),
    }
}

#[rustfmt::skip]
fn simple(bg: Background) -> String {
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
        Background::Starfield { from, to, dir, tint } => format!(
            concat!("{{ \"kind\": \"starfield\", \"from\": {}, \"to\": {}, ",
                "\"dir\": [{}, {}], \"tint\": {} }}"),
            hex(from), hex(to), dir.0, dir.1, hex(tint)),
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
fn rich(bg: Background, phase: f32) -> String {
    let hex = |c: crate::theme::Srgb| json_string(&c.hex());
    match bg {
        Background::Lava { ground, blob_lo, blob_hi, edge, dithered } => format!(
            concat!(
                "{{ \"kind\": \"lava\", \"ground\": {}, \"blob_lo\": {}, ",
                "\"blob_hi\": {}, \"edge\": \"{}\", \"dithered\": {}, \"phase\": {} }}"
            ),
            hex(ground), hex(blob_lo), hex(blob_hi), edge.as_str(), dithered, phase
        ), Background::Bands { tones, angle } => format!(
            "{{ \"kind\": \"bands\", \"tones\": [{}, {}, {}], \"angle\": {} }}",
            hex(tones[0]), hex(tones[1]), hex(tones[2]), angle
        ), Background::Waves { tones } => format!(
            "{{ \"kind\": \"waves\", \"tones\": [{}, {}, {}] }}",
            hex(tones[0]), hex(tones[1]), hex(tones[2])
        ), Background::Zigzag {
            from, to, dir, tint, period_px, amplitude_px, angle, density, banded,
        } => format!(
            concat!(
                "{{\"kind\":\"zigzag\",\"from\":{},\"to\":{},\"dir\":[{},{}],\"tint\":{},",
                "\"period_px\":{},\"amplitude_px\":{},\"angle\":{},\"density\":{},\"banded\":{}}}"
            ),
            hex(from), hex(to), dir.0, dir.1, hex(tint), period_px, amplitude_px,
            angle, density, banded
        ), Background::Organic { tones, scale_px, density } => format!(
            concat!(
                "{{\"kind\":\"organic\",\"tones\":[{},{},{}],",
                "\"scale_px\":{},\"density\":{},\"phase\":{}}}"
            ),
            hex(tones[0]), hex(tones[1]), hex(tones[2]), scale_px, density, phase
        ), Background::WarpedGrid { tones, spacing_px, density, curvature } => {
            let pose = crate::warpgrid::route_pose(phase);
            format!(
                concat!(
                    "{{ \"kind\": \"warped-grid\", \"tones\": [{}, {}, {}], ",
                    "\"spacing_px\": {}, \"density\": {}, \"curvature\": {}, ",
                    "\"phase\": {}, \"yaw\": {}, \"pitch\": {}, \"forward_cells\": {} }}"
                ),
                hex(tones[0]), hex(tones[1]), hex(tones[2]), spacing_px, density,
                curvature, phase, pose.yaw, pose.pitch, pose.forward_cells
            )
        },
        _ => unreachable!("rich background helper received a simple ground"),
    }
}
