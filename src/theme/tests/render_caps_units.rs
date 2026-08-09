//! THE `RenderCaps` FAMILY'S UNITS — the question a chrome declaration sweep
//! structurally cannot ask.
//!
//! `render/tests/chrome_pixel_space.rs` grades authored CONSTANTS in a
//! set of swept source files: every one states its unit family in its type, and
//! no length is resolved against `zoom` alone. A theme capability is neither —
//! it is a FIELD carrying per-world DATA, so widening that sweep's file list
//! reaches it in no way at all. `spell_underline_gap` proved the gap the
//! expensive way: it is the distance the spell squiggle's band hangs below the
//! glyph cell, it rode the user's zoom and never met the panel's density, and it
//! sat in the SAME `let` block as three shape terms that were repaired onto the
//! display scale — leaving a correctly-doubled wave a half-size gap from the
//! word it marks, a mismatch that had not existed while the whole family was
//! uniformly wrong.
//!
//! **TWO ARMS, AND THE ENROLMENT OF BOTH IS THE FAMILY'S OWN DECLARATIONS.**
//! [`family_fields`] walks `model.rs` from `RenderCaps` through every type it
//! reaches and reads the fields off those declarations, so a field added
//! tomorrow is enrolled by being declared — never by being named here. That
//! matters more than it sounds: a name list is the enrolment failure this
//! repository has recorded four times, and the names in THIS family are actively
//! misleading. `FoldAfford::chevron_lift` and `tail_lift` read as lengths and are
//! colour-lift fractions clamped to `0..=1`.
//!
//!   * [`no_render_caps_field_is_resolved_against_zoom_alone`] — the READ-SITE
//!     rule, and the one that fails on the defect. `zoom` is the user's type
//!     size; `Metrics::scale` is `zoom * dpi`. A theme length multiplied by the
//!     first holds its device size as the panel gets denser.
//!   * [`every_f32_field_in_the_render_caps_family_carries_a_unit_verdict`] —
//!     the CENSUS. Every `f32` the family declares is either not a length, or a
//!     length with the owner that resolves it named, and the verdict was read
//!     off that owner rather than off the field's name. A field with no verdict
//!     fails here, which is what makes the sweep enumerate the work instead of a
//!     reader guessing at it.
//!   * [`every_struct_field_in_the_render_caps_family_has_a_product_reader`] —
//!     the CONSUMPTION CENSUS. A struct field carried as theme data must appear
//!     at a product read site; an authored dial with no reader fails instead of
//!     earning an "unconsumed" classification.
//!
//! ⚠️ A length field whose read site resolves it correctly is recorded, not
//! typed. Typing the whole family would move read sites in files this round does
//! not own, and a verdict that names the owner it was measured against expires
//! the same way: the owner is greppable, and a reader who changes it finds this
//! table by name.

use std::collections::BTreeSet;

/// What a `RenderCaps` `f32` field is, and the evidence.
///
/// The distinction that matters is not "px in the name" — it is what the read
/// site multiplies the value by before it reaches a quad.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Verdict {
    /// Not a length: a fraction, an angle, a brightness, a colour lift, a
    /// multiple of a font size. No display factor belongs anywhere near it.
    NotALength,
    /// A length, resolved on the ONE display scale (`zoom * dpi`) by the named
    /// owner. Correct today; the name is where a reader checks.
    LengthOnTheScale,
    /// A length resolved against the DISPLAY DENSITY ALONE — dpi-correct and
    /// zoom-blind, so it holds its size as the reader turns the type up. A
    /// different axis from the one this file grades, recorded so it is not
    /// rediscovered as new.
    LengthDpiOnly,
}

/// Every `f32` field the family declares, with its verdict and the owner the
/// verdict was read off. Graded for staleness in both directions, so an entry
/// cannot outlive its field and a field cannot dodge a verdict.
const UNIT_VERDICTS: &[(&str, Verdict, &str)] = &[
    (
        "AmbientStyle.cell_px",
        Verdict::LengthOnTheScale,
        "render/layers.rs::prepare_stars_layer, `* (metrics.zoom * dpi)`",
    ),
    (
        "AmbientStyle.size_px",
        Verdict::LengthOnTheScale,
        "render/layers.rs::prepare_stars_layer, `* (metrics.zoom * dpi)`",
    ),
    (
        "CardShape.cut_px",
        Verdict::LengthDpiOnly,
        "render/chrome/mod.rs::card_shape_texture, `* dpi.max(1.0)` — a grow-only \
         resolution that never meets the user's zoom",
    ),
    (
        "CardTexture.cell_px",
        Verdict::LengthDpiOnly,
        "render/chrome/mod.rs::card_shape_texture, `* dpi.max(1.0)`, same shape as \
         the chamfer beside it",
    ),
    (
        "CardTexture.angle_deg",
        Verdict::NotALength,
        "an ANGLE — chrome/mod.rs calls `.to_radians()` on it",
    ),
    (
        "CardTexture.density",
        Verdict::NotALength,
        "a fraction of the halftone cell that is ink",
    ),
    (
        "HighlightTexture.density",
        Verdict::NotALength,
        "a fraction of the stipple cell that is ink",
    ),
    (
        "AmbientStyle.density",
        Verdict::NotALength,
        "stars per cell — a count, and stars::layout takes it beside an already-scaled cell",
    ),
    (
        "AmbientStyle.peak",
        Verdict::NotALength,
        "a brightness at the top of the twinkle, 0..=1",
    ),
    (
        "AmbientStyle.floor",
        Verdict::NotALength,
        "a brightness at the bottom of the twinkle, 0..=1",
    ),
    (
        "FoldAfford.chevron_lift",
        Verdict::NotALength,
        "a COLOUR lift toward the ink, `.clamp(0.0, 1.0)` in theme::derive — the \
         name reads like a distance and is not one",
    ),
    (
        "FoldAfford.tail_lift",
        Verdict::NotALength,
        "the same colour lift for the fold tail, `.clamp(0.0, 1.0)` in theme::derive",
    ),
    (
        "CardAnchor.x_frac",
        Verdict::NotALength,
        "a fraction of the card's own travel span, clamped 0..=1 in chrome/overlay.rs",
    ),
    (
        "TitleStyle.scale",
        Verdict::NotALength,
        "a multiple of the placard's font size, which is already scaled",
    ),
];

/// A `Type.field` name for every field the `RenderCaps` family declares, paired
/// with the declared type text — walked from `RenderCaps` through the types it
/// reaches, out of `model.rs`'s own source.
///
/// The walk is the enrolment. It is deliberately not a list of names: a field
/// added to any type the family reaches shows up here the moment it is declared,
/// and a type that leaves the family stops being graded.
/// `(field, declared type)` for every field in one brace-balanced type body —
/// including a variant payload written INLINE on one line.
///
/// Not line-based, and that is the whole point of it being its own function: a
/// first draft split each line on `:` and so read `Inset { x_frac: f32 }` as a
/// field called `Inset { x_frac`, silently missing FOUR of the family's `f32`
/// fields — `weight_px`, `cut_px`, `x_frac` and one `density` — every one of them
/// a payload on a single-line enum variant. The census reported clean over the
/// exact shape it exists to grade, and only the table's staleness arm, firing
/// from the other end, said so.
fn fields_in(body: &str) -> Vec<(String, String)> {
    let stripped: String = body
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("pub ", "");
    let b = stripped.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        // A field name starts a declaration only after a brace, a comma or a
        // newline — never mid-path, which is what keeps `crate::render::Logical`
        // from being read as a field called `render`.
        let boundary = i == 0 || matches!(b[i - 1], b'{' | b',' | b'\n' | b' ' | b'\t');
        if !boundary || !(b[i].is_ascii_lowercase() || b[i] == b'_') {
            i += 1;
            continue;
        }
        let start = i;
        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
            i += 1;
        }
        let name = &stripped[start..i];
        let mut j = i;
        while j < b.len() && (b[j] == b' ' || b[j] == b'\t') {
            j += 1;
        }
        if j >= b.len() || b[j] != b':' || stripped[j..].starts_with("::") {
            continue;
        }
        // The type runs to the next `,` or `}` at depth zero, so a generic or a
        // nested brace cannot end it early.
        let mut k = j + 1;
        let mut depth = 0i32;
        while k < b.len() {
            match b[k] {
                b'<' | b'(' | b'{' => depth += 1,
                b'>' | b')' => depth -= 1,
                b'}' if depth == 0 => break,
                b'}' => depth -= 1,
                b',' if depth == 0 => break,
                _ => {}
            }
            k += 1;
        }
        out.push((name.to_string(), stripped[j + 1..k].trim().to_string()));
        i = k;
    }
    out
}

fn family_fields() -> Vec<(String, String)> {
    let src = include_str!("../model.rs");
    // Every `pub struct` / `pub enum` in the file, with its brace-balanced body.
    let mut bodies: Vec<(String, String)> = Vec::new();
    let mut at = 0usize;
    while let Some(found) = src[at..].find("\npub ") {
        let start = at + found + 1;
        let head = &src[start..];
        let is_ty = head.starts_with("pub struct ") || head.starts_with("pub enum ");
        at = start + 4;
        if !is_ty {
            continue;
        }
        let name: String = head
            .split_whitespace()
            .nth(2)
            .unwrap_or("")
            .trim_end_matches([';', '{'])
            .to_string();
        let Some(open) = head.find('{') else {
            continue; // a unit struct: no fields to grade
        };
        let mut depth = 0usize;
        let mut end = open;
        for (i, c) in head[open..].char_indices() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    end = open + i;
                    break;
                }
            }
        }
        bodies.push((name, head[open..=end].to_string()));
    }
    assert!(
        bodies.len() > 20,
        "parsed only {} type bodies out of theme/model.rs — the file's spelling \
         changed and this walk is enrolling almost nothing",
        bodies.len()
    );
    let mut out: Vec<(String, String)> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec!["RenderCaps".to_string()];
    while let Some(ty) = stack.pop() {
        if !seen.insert(ty.clone()) {
            continue;
        }
        let Some((_, body)) = bodies.iter().find(|(n, _)| *n == ty) else {
            continue;
        };
        for (name, decl) in fields_in(body) {
            let short = decl.rsplit("::").next().unwrap_or(&decl).to_string();
            out.push((format!("{ty}.{name}"), decl.clone()));
            if bodies.iter().any(|(n, _)| *n == short) {
                stack.push(short);
            }
        }
    }
    assert!(
        out.len() > 25,
        "the family walk reached only {} fields — `RenderCaps` and the types it \
         carries declare more than that, so the walk is not following them",
        out.len()
    );
    out
}

/// Every product `.rs` file under `src/`, `tests` directories and `tests.rs`
/// dropped. Mirrors the chrome sweep's own product-source rule and for the same
/// reason: a test that multiplies a field by a zoom in order to DESCRIBE the
/// defect is not a counter-example to it.
fn product_sources() -> Vec<(String, String)> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut out: Vec<(String, String)> = Vec::new();
    let mut stack = vec![manifest.join("src")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("src readable") {
            let path = entry.expect("dir entry").path();
            let rel = path
                .strip_prefix(manifest)
                .unwrap_or(&path)
                .display()
                .to_string();
            if path.is_dir() {
                if !rel.ends_with("/tests") {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|e| e == "rs") && !rel.ends_with("tests.rs") {
                out.push((
                    rel,
                    std::fs::read_to_string(&path).expect("source readable"),
                ));
            }
        }
    }
    assert!(
        out.len() > 60,
        "the product-source scan found only {} files",
        out.len()
    );
    out.sort();
    out
}

/// Every field belonging to a struct reached from `RenderCaps`.
///
/// Enum payloads are consumed through exhaustive pattern matches, where their
/// binding names are ordinary locals. Struct fields keep their identity at the
/// read site as `.field`, so they can carry the stronger unread-data law below.
fn family_struct_fields() -> Vec<String> {
    let src = include_str!("../model.rs");
    let structs: BTreeSet<&str> = src
        .lines()
        .filter_map(|line| {
            line.trim_start()
                .strip_prefix("pub struct ")
                .and_then(|rest| rest.split(|c: char| c == '{' || c.is_whitespace()).next())
        })
        .collect();
    family_fields()
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| {
            name.split_once('.')
                .is_some_and(|(owner, _)| structs.contains(owner))
        })
        .collect()
}

/// **ARM 2 — EVERY STRUCT DIAL IN THE FAMILY HAS A PRODUCT READER.**
///
/// A declaration and a default do not make a capability real. Struct fields
/// retain a `.field` spelling at their read sites, so the declaration-derived
/// family walk can require at least one non-test product read for every one.
/// This is deliberately separate from the unit verdict: "unconsumed" is not a
/// unit and may not be used to make a dead dial look classified.
#[test]
fn every_struct_field_in_the_render_caps_family_has_a_product_reader() {
    let _g = crate::testlock::serial();
    let fields = family_struct_fields();
    let sources = product_sources();
    let mut unread = Vec::new();
    for qualified in &fields {
        let field = qualified.rsplit('.').next().unwrap_or_default();
        let needle = format!(".{field}");
        let read = sources.iter().any(|(_, src)| {
            src.lines()
                .any(|line| !line.trim_start().starts_with("//") && line.contains(&needle))
        });
        if !read {
            unread.push(qualified.clone());
        }
    }
    assert!(
        unread.is_empty(),
        "RenderCaps-family struct fields carry authored data but have no product reader: {unread:?}"
    );
    assert!(
        fields.len() > 20,
        "the struct-field consumption census enrolled only {} fields ({fields:?})",
        fields.len()
    );
}

/// **ARM 1 — NO `RenderCaps` FIELD IS RESOLVED AGAINST ZOOM ALONE.**
///
/// The rule the chrome laws hold over authored constants, asked of theme DATA.
/// `metrics.zoom` is the reader's type size and nothing else; the factor a drawn
/// length wants is `Metrics::scale`, which folds the panel's density in. A field
/// multiplied by the first renders at `1/dpi` of the size it was tuned at on
/// every dense panel — and does it while the constants beside it are correct,
/// which is worse than being uniformly wrong because the two detach.
#[test]
fn no_render_caps_field_is_resolved_against_zoom_alone() {
    let _g = crate::testlock::serial();
    let fields: Vec<String> = family_fields()
        .into_iter()
        .map(|(k, _)| k.rsplit('.').next().unwrap_or_default().to_string())
        .collect();
    let mut offenders: Vec<String> = Vec::new();
    let mut fields_read = BTreeSet::new();
    for (path, src) in product_sources() {
        for (i, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            for f in &fields {
                let needle = format!(".{f}");
                if !line.contains(&needle) {
                    continue;
                }
                fields_read.insert(f.clone());
                let words: Vec<&str> = line
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .collect();
                if line.contains('*')
                    && words.contains(&"zoom")
                    && !words.contains(&"scale")
                    && !words.contains(&"dpi")
                {
                    offenders.push(format!("{path}:{}: {}", i + 1, line.trim()));
                }
            }
        }
    }
    offenders.sort();
    offenders.dedup();
    assert!(
        offenders.is_empty(),
        "a theme capability resolved against ZOOM alone — it holds its device \
         size as the panel gets denser, beside constants that do not:\n{}",
        offenders.join("\n")
    );
    // NON-VACUITY, and it is the half that has failed elsewhere: a scan that
    // matches no read site reports a clean sweep of nothing. The floor is well
    // under the count so a refactor moving one field does not fail here, and far
    // enough above zero that a broken needle does.
    assert!(
        fields_read.len() >= 8,
        "the scan found read sites for only {} of the family's fields ({:?}) — it \
         is not reading the product it thinks it is",
        fields_read.len(),
        fields_read
    );
    eprintln!(
        "RenderCaps zoom-alone sweep: {} fields enrolled, {} with product read sites",
        fields.len(),
        fields_read.len()
    );
}

/// **ARM 3 — EVERY `f32` THE FAMILY DECLARES CARRIES A UNIT VERDICT.**
///
/// The census, and the reason a new theme length cannot repeat this item. A
/// bare `f32` in the family is exactly as silent as the bare `f32` chrome
/// constants were before their own sweep existed, and the fix has the same
/// shape: enrol from the declarations, require a verdict, and grade the table
/// from both ends so neither a field nor an entry can go missing.
///
/// The verdict is read off the READ SITE, never off the field's name. Two
/// entries exist only because the name lies: `chevron_lift` and `tail_lift`
/// carry no distance at all.
#[test]
fn every_f32_field_in_the_render_caps_family_carries_a_unit_verdict() {
    let _g = crate::testlock::serial();
    let all = family_fields();
    let f32s: Vec<String> = all
        .iter()
        .filter(|(_, decl)| decl == "f32")
        .map(|(k, _)| k.clone())
        .collect();
    let unit_typed: Vec<String> = all
        .iter()
        .filter(|(_, decl)| {
            matches!(
                decl.rsplit("::").next().unwrap_or(decl),
                "Logical" | "LogicalGrowOnly" | "Physical"
            )
        })
        .map(|(k, _)| k.clone())
        .collect();
    let mut problems: Vec<String> = Vec::new();
    for name in &f32s {
        if !UNIT_VERDICTS.iter().any(|(n, _, _)| n == name) {
            problems.push(format!(
                "`{name}: f32` is a theme capability with no unit verdict. A \
                 capability is DATA, so no declaration sweep over source files \
                 reaches it: say here whether it is a length and, if it is, name \
                 the owner that resolves it against `Metrics::scale` — or declare \
                 it `render::Logical` and let `Metrics::px` do it."
            ));
        }
    }
    for (name, _, reason) in UNIT_VERDICTS {
        if !f32s.iter().any(|n| n == name) {
            problems.push(format!(
                "`{name}` has a unit verdict and is not an `f32` field of the \
                 family any more — a stale entry silently excuses a field nobody \
                 declared (its recorded reason was: {reason})"
            ));
        }
        assert!(
            reason.len() > 20,
            "`{name}`'s verdict carries no usable reason: {reason:?}"
        );
    }
    problems.sort();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    // NON-VACUITY OF THE MECHANISM ITSELF: the family must actually contain a
    // unit-typed length, or this census is a table beside a door nobody uses and
    // the next length has no precedent to follow.
    assert!(
        !unit_typed.is_empty(),
        "no field of the `RenderCaps` family carries a unit type — the typed door \
         this census points a new length at does not exist"
    );
    assert!(
        f32s.len() >= 14,
        "the census graded only {} f32 fields ({f32s:?}) — the walk is not \
         reaching the family's payloads",
        f32s.len()
    );
    let lengths = UNIT_VERDICTS
        .iter()
        .filter(|(_, v, _)| *v != Verdict::NotALength)
        .count();
    eprintln!(
        "RenderCaps unit census: {} f32 fields, {lengths} of them lengths, {} \
         unit-typed fields ({unit_typed:?})",
        f32s.len(),
        unit_typed.len()
    );
}
