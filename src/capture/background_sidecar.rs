//! The `page.background` sidecar arm — one JSON shape per `theme::Background`.
//!
//! Carved out of `sidecar.rs` alongside its `scroll_sidecar`/`layout_sidecar`/
//! `replay_sidecar` siblings when item 158's Deckle ground added a tenth arm.
//! The split is mechanical: the same three functions, verbatim, with the
//! shared `json_string` helper imported instead of colocated.

use super::sidecar::json_string;

#[rustfmt::skip]
pub(super) fn background_json(bg: crate::theme::Background, lava_phase: f32) -> String {
    use crate::theme::Background;
    match bg {
        Background::Gradient { .. } | Background::Dots { .. } | Background::Starfield { .. }
        | Background::Pinstripe { .. } | Background::Stripes { .. } => simple_background_json(bg),
        _ => rich_background_json(bg, lava_phase),
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
fn rich_background_json(bg: crate::theme::Background, lava_phase: f32) -> String {
    use crate::theme::Background;
    let hex = |c: crate::theme::Srgb| json_string(&c.hex());
    match bg {
        Background::Lava {
            ground,
            blob_lo,
            blob_hi,
            edge,
            dithered,
        } => format!(
            concat!("{{ \"kind\": \"lava\", \"ground\": {}, \"blob_lo\": {}, \"blob_hi\": {}, ",
                "\"edge\": \"{}\", \"dithered\": {}, \"phase\": {} }}"),
            hex(ground),
            hex(blob_lo),
            hex(blob_hi),
            edge.as_str(),
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
            hex(tones[0]), hex(tones[1]), hex(tones[2]), scale_px, density, lava_phase
        ), Background::Deckle {
            ground, layer, deckle, weave, period_px, wander_px, density
        } => format!(
            concat!("{{\"kind\":\"deckle\",\"ground\":{},\"layer\":{},",
                "\"deckle\":{},\"weave\":\"{}\",\"period_px\":{},",
                "\"wander_px\":{},\"density\":{},\"static\":true}}"),
            hex(ground), hex(layer), hex(deckle), weave.as_str(), period_px, wander_px, density
        ), _ => unreachable!("rich background helper received a simple ground"),
    }
}
