pub const GAP_CHARS: usize = 2;
pub const PRIMARY_MIN_CHARS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    Full { primary: usize },
    Split { primary: usize },
    Measure,
}
pub fn plan(total_chars: usize, widest_secondary_chars: Option<usize>) -> Plan {
    let Some(widest) = widest_secondary_chars else {
        return Plan::Full {
            primary: full_budget(total_chars),
        };
    };
    let primary = total_chars.saturating_sub(1 + widest + GAP_CHARS);
    if primary >= PRIMARY_MIN_CHARS {
        Plan::Split { primary }
    } else {
        Plan::Measure
    }
}
pub fn full_budget(total_chars: usize) -> usize {
    total_chars.saturating_sub(1).max(4)
}
pub fn fits(text_w: f32, gap_px: f32, primary_px: f32, secondary_px: f32) -> bool {
    if secondary_px <= 0.0 {
        return primary_px <= text_w;
    }
    primary_px + gap_px + secondary_px <= text_w
}
pub fn fit_primary(text: &str, budget: usize) -> String {
    crate::overlay::elide_path(text, budget)
}

pub fn fit_primary_end(text: &str, budget: usize) -> String {
    if text.chars().count() <= budget {
        return text.to_string();
    }
    if let Some(head) = subtitle_head(text) {
        if head.chars().count() <= budget {
            return head.to_string();
        }
        return elide_end(head, budget);
    }
    elide_end(text, budget)
}

fn subtitle_head(text: &str) -> Option<&str> {
    [" — ", " – "]
        .iter()
        .filter_map(|sep| text.find(sep))
        .min()
        .map(|i| text[..i].trim_end())
}

fn elide_end(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        return s.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let head: String = chars[..max - 1].iter().collect();
    format!("{head}…")
}

/// PIXEL-TRUTH variant of [`fit_primary_end`]: [`fit_primary_end`] alone trusts a
/// CHAR-COUNT budget against a MEAN glyph-width estimate, which under-predicts a
/// title carrying disproportionately WIDE glyphs (the margin outline's own bug: a
/// heading shaped wider than its char-fit budget promised, forcing an unwanted
/// word-wrap onto a second visual row — see `render/chrome/outline.rs`). This
/// shrinks the candidate — one char at a time, re-querying the caller's real
/// shaped-pixel `measure` after each step — until it genuinely fits `budget_px`, or
/// nothing is left to shave. Pure over its OWN decision; the actual shaping stays
/// the CALLER's (the same "decision here, measurement there" split [`fits`] already
/// keeps — `measure` is typically a `cosmic-text` `layout_runs().line_w` probe).
///
/// Safe to feed an ALREADY [`fit_primary_end`]-produced string back in as
/// `candidate` (this function's own shrink loop does exactly that): an appended
/// `…` never re-triggers the subtitle-divider scan, so re-applying with a smaller
/// budget just shaves further, correctly, every time.
pub fn fit_primary_end_to_px(
    candidate: &str,
    budget_px: f32,
    mut measure: impl FnMut(&str) -> f32,
) -> String {
    let mut text = candidate.to_string();
    loop {
        if measure(&text) <= budget_px {
            return text;
        }
        let n = text.chars().count();
        if n == 0 {
            return text;
        }
        text = fit_primary_end(&text, n - 1);
    }
}

/// The bottom-left page-mode GUTTER's hard floor, in chars, at the LABEL font
/// scale it renders at: below this the margin can't hold even a stub filename, so
/// the whole gutter hides rather than draw confetti (`render/chrome.rs`'s
/// `GutterLayout`). Deliberately much smaller than [`PRIMARY_MIN_CHARS`] — this is
/// quiet LABEL-size chrome living in a margin, not a picker's primary content.
pub const GUTTER_MIN_NAME_CHARS: usize = 6;

/// The persistent margin OUTLINE's hard floor, in chars, at the LABEL font scale it
/// renders at: below this the left margin can't hold even a stub heading TITLE, so
/// the whole outline hides rather than draw a useless sliver (`render/chrome/outline.rs`).
/// Matched to [`GUTTER_MIN_NAME_CHARS`] on purpose — the outline and the gutter are
/// the two margin surfaces (top vs bottom), so they should appear and collapse
/// TOGETHER at the same margin width rather than one lingering while the other hides.
/// (A TASTE TUNABLE — the exact "too cramped to bother" width is a live-review call.)
pub const OUTLINE_MIN_CHARS: usize = GUTTER_MIN_NAME_CHARS;

/// The persistent margin OUTLINE's PREFERRED rail width, in chars at the same
/// LABEL font scale — comfortable enough to hold a typical heading label
/// ("## Some Section Title") without crowding, the target the ADAPTIVE-COLUMN
/// placement policy (`render::geometry::adaptive_column_left`) tries to grant
/// the outline once the window has room to spare it. Deliberately expressed
/// as a MULTIPLE of [`OUTLINE_MIN_CHARS`] — never a parallel magic number —
/// so the hard "too cramped to bother" floor and the "comfortable" target
/// scale together if the floor is ever retuned. (A TASTE TUNABLE — the exact
/// multiplier is a live-review call, flagged like `OUTLINE_MIN_CHARS` itself.)
pub const OUTLINE_PREFERRED_CHARS: usize = OUTLINE_MIN_CHARS * 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GutterPlan {
    /// The filename's one-line char budget. [`fit_primary`] against it is always
    /// a safe, wrap-free door — a no-op whenever the name already fits.
    pub name_budget: usize,
    pub project_budget: usize,
    pub show_project: bool,
}

pub fn gutter_plan(avail_chars: usize) -> Option<GutterPlan> {
    if avail_chars < GUTTER_MIN_NAME_CHARS {
        return None;
    }
    Some(GutterPlan {
        name_budget: avail_chars,
        project_budget: avail_chars,
        show_project: true,
    })
}

const RAIL_W_LH: f32 = 3.2;
const RAIL_H_LH: f32 = 0.09;
const THUMB_W_LH: f32 = 0.22;
const THUMB_H_LH: f32 = 0.50;
const RAIL_GAP_LH: f32 = 0.45;
const RAIL_HIT_PAD_LH: f32 = 0.55;

/// The rail and its fixed internal gap occupy this much of an accessory cluster
/// before that cluster's measured value text begins.
pub fn rail_accessory_width(lh: f32) -> f32 {
    (RAIL_W_LH + RAIL_GAP_LH) * lh.max(0.0)
}

/// WHICH WAY A COLUMN'S INK GROWS from the edge it hangs on.
///
/// A row's name and its accessory hang on OPPOSITE ends of one cluster and grow
/// toward each other, so a single signed answer places both — and mirroring the
/// whole composition is [`Self::mirrored`], applied once, rather than a second
/// sign in every consumer to drift from the first the day one of them is edited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnFlow {
    /// The ink BEGINS at its anchor and runs right — an ascending spine's
    /// accessory column, and every left-aligned name.
    Rightward,
    /// The ink ENDS at its anchor: every upright world's right-aligned secondary
    /// column, a descending spine's accessory, and an ascending spine's names.
    Leftward,
}

impl ColumnFlow {
    /// The LEFT edge of ink `w` wide hanging on `anchor` — a text area's origin.
    pub fn origin(self, anchor: f32, w: f32) -> f32 {
        match self {
            Self::Rightward => anchor,
            Self::Leftward => anchor - w,
        }
    }

    /// That ink's `(left, right)` extent.
    pub fn span(self, anchor: f32, w: f32) -> (f32, f32) {
        let left = self.origin(anchor, w);
        (left, left + w)
    }

    pub fn mirrored(self) -> Self {
        match self {
            Self::Rightward => Self::Leftward,
            Self::Leftward => Self::Rightward,
        }
    }

    /// +1 rightward, −1 leftward — for stepping INTO the ink from its anchor.
    /// A law's own question: the draw path steps by placing ink, not by sign.
    #[cfg(test)]
    pub fn sign(self) -> f32 {
        match self {
            Self::Rightward => 1.0,
            Self::Leftward => -1.0,
        }
    }

    /// The glyph alignment that lands ink on this flow's anchor when a buffer of
    /// the same width is seated by [`Self::origin`]. Shaping and placement give
    /// ONE answer, so a mirrored column cannot align its ink one way and seat its
    /// buffer the other.
    pub fn align(self) -> glyphon::cosmic_text::Align {
        match self {
            Self::Rightward => glyphon::cosmic_text::Align::Left,
            Self::Leftward => glyphon::cosmic_text::Align::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rail {
    pub track: [f32; 4],
    pub fill: [f32; 4],
    pub thumb: [f32; 4],
    pub hit: [f32; 4],
    pub x0: f32,
    pub x1: f32,
}

/// The Range rail, seated one fixed gap INWARD of its own value text: both hang
/// on the accessory column's `anchor` and grow along `flow`, so the rail always
/// sits between the value and the row's name. `x0` stays the track's LEFT edge
/// at either orientation — a slider's fill direction belongs to the value it
/// carries, not to the composition it is seated in.
pub fn rail_geom(
    anchor: f32,
    flow: ColumnFlow,
    value_w: f32,
    avail: f32,
    row_top: f32,
    lh: f32,
    frac: f32,
) -> Option<Rail> {
    if lh <= 0.0 {
        return None;
    }
    let w = RAIL_W_LH * lh;
    let gap = RAIL_GAP_LH * lh;
    if avail < w + gap * 2.0 {
        return None;
    }
    let (value_left, value_right) = flow.span(anchor, value_w);
    let (x0, x1) = match flow {
        ColumnFlow::Leftward => (value_left - gap - w, value_left - gap),
        ColumnFlow::Rightward => (value_right + gap, value_right + gap + w),
    };
    let frac = if frac.is_nan() {
        0.0
    } else {
        frac.clamp(0.0, 1.0)
    };
    let th = (RAIL_H_LH * lh).max(1.0);
    let ty = row_top + (lh - th) * 0.5;
    let tw = (THUMB_W_LH * lh).max(2.0);
    let thh = (THUMB_H_LH * lh).max(3.0);
    let cx = x0 + frac * w;
    let pad = RAIL_HIT_PAD_LH * lh;
    Some(Rail {
        track: [x0, ty, w, th],
        fill: [x0, ty, (w * frac).max(0.0), th],
        thumb: [cx - tw * 0.5, row_top + (lh - thh) * 0.5, tw, thh],
        hit: [x0 - pad, row_top, w + pad * 2.0, lh],
        x0,
        x1,
    })
}

pub fn rail_frac_at(px: f32, x0: f32, x1: f32) -> f32 {
    if x1 <= x0 {
        return 0.0;
    }
    ((px - x0) / (x1 - x0)).clamp(0.0, 1.0)
}

pub fn rail_hit(rail: &Rail, px: f32, py: f32) -> bool {
    let [x, y, w, h] = rail.hit;
    px >= x && px <= x + w && py >= y && py <= y + h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::OverlayKind;

    fn rows_for(kind: OverlayKind) -> (Vec<String>, Option<usize>) {
        let widest = |v: &[String]| v.iter().map(|s| s.chars().count()).max().unwrap_or(0);
        match kind {
            OverlayKind::Goto => (
                vec![
                    "src/main.rs".into(),
                    "very/deeply/nested/directory/structure/for/testing/some_quite_long_filename.md".into(),
                ],
                Some("2 days ago".chars().count()),
            ),
            OverlayKind::Project => (vec!["awl-next/".into(), "projects/".into()], Some(0)),
            OverlayKind::Browse => (vec!["src/".into(), "README.md".into()], Some(0)),
            OverlayKind::MoveDest => (vec!["notes/".into(), "archive/".into()], Some(0)),
            OverlayKind::Theme => (
                crate::theme::THEMES.iter().map(|t| t.name.to_string()).collect(),
                Some(0),
            ),
            OverlayKind::Caret => {
                let names: Vec<String> = crate::caret::CaretMode::ALL
                    .iter()
                    .map(|m| m.label().to_string())
                    .collect();
                let descs: Vec<String> = crate::caret::CaretMode::ALL
                    .iter()
                    .map(|m| m.description().to_string())
                    .collect();
                let w = widest(&descs);
                (names, Some(w))
            }
            OverlayKind::Dictionary => {
                let names: Vec<String> = crate::spell::DictVariant::ALL
                    .iter()
                    .map(|v| v.label().to_string())
                    .collect();
                let descs: Vec<String> = crate::spell::DictVariant::ALL
                    .iter()
                    .map(|v| v.description().to_string())
                    .collect();
                let w = widest(&descs);
                (names, Some(w))
            }
            OverlayKind::CjkLang => {
                let names: Vec<String> = crate::frontmatter::DEFAULT_CJK_PRIORITY
                    .iter()
                    .map(|l| l.label().to_string())
                    .collect();
                let descs: Vec<String> = crate::frontmatter::DEFAULT_CJK_PRIORITY
                    .iter()
                    .map(|l| l.description().to_string())
                    .collect();
                let w = widest(&descs);
                (names, Some(w))
            }
            OverlayKind::Date => {
                let (y, m, d) = crate::dateformat::CAPTURE_PLACEHOLDER_YMD;
                let names: Vec<String> = crate::dateformat::DateFormat::ALL
                    .iter()
                    .map(|f| f.format(y, m, d))
                    .collect();
                let descs: Vec<String> = crate::dateformat::DateFormat::ALL
                    .iter()
                    .map(|f| f.label().to_string())
                    .collect();
                let w = widest(&descs);
                (names, Some(w))
            }
            OverlayKind::Command | OverlayKind::Keybindings => {
                let names = crate::commands::names();
                let binds = crate::commands::effective_bindings(&[], &[]);
                let w = widest(&binds);
                (names, Some(w))
            }
            OverlayKind::Spell => (
                vec!["thoroughgoing".into(), "thoroughgoingly".into()],
                Some(0),
            ),
            OverlayKind::History => (
                vec!["yesterday".into(), "2 days ago".into()],
                Some("+204 −683".chars().count()),
            ),
            OverlayKind::Settings => {
                let names = crate::settings::names();
                let widest_value = "English (Australia)".chars().count();
                (names, Some(widest_value))
            }
            OverlayKind::Assets => (
                vec!["photo.png".into(), "a-rather-long-screenshot-name.png".into()],
                Some("12.3 KB · notes/deeply/nested/assets".chars().count()),
            ),
            OverlayKind::Rename => (
                vec!["a-rather-long-note-title-being-renamed.md".into()],
                None,
            ),
            OverlayKind::InsertLink => (
                vec!["https://example.com/a/rather-long/path/to/something".into()],
                None,
            ),
            OverlayKind::KeepName => (
                vec!["a rather long name for the draft I want back".into()],
                None,
            ),
            OverlayKind::Conflict => (
                crate::overlay::CONFLICT_ROWS.iter().map(|r| r.to_string()).collect(),
                None,
            ),
            OverlayKind::Context => (
                vec!["Collapse other sections".into(), "Page width settings…".into()],
                Some("unavailable".chars().count()),
            ),
        }
    }

    /// The whole roster, derived from the enum's own declaration rather than
    /// hand-kept beside it. The hand-written list this replaces had silently
    /// omitted `OverlayKind::Date` — a kind that ships, with rows and a
    /// secondary column, swept by neither law below — which is exactly the
    /// failure a parallel roster produces and the reason there is no second one.
    const ALL_KINDS: [OverlayKind; OverlayKind::ALL.len()] = OverlayKind::ALL;

    const NARROW_TOTAL: usize = 28;
    const WIDE_TOTAL: usize = 40;

    #[test]
    fn law_no_overlap_secondary_yields_first_for_every_kind() {
        for kind in ALL_KINDS {
            let (names, widest_secondary) = rows_for(kind);
            for total in [NARROW_TOTAL, WIDE_TOTAL] {
                let plan = plan(total, widest_secondary);
                if let Plan::Split { primary } = plan {
                    let widest = widest_secondary.unwrap_or(0);
                    assert!(
                        primary + GAP_CHARS + widest < total,
                        "{kind:?}@{total}: split overlaps (primary {primary} + gap + secondary {widest} > {total})"
                    );
                    assert!(
                        primary >= PRIMARY_MIN_CHARS,
                        "{kind:?}@{total}: secondary granted while primary starves ({primary})"
                    );
                }
                let floor = match plan {
                    Plan::Split { primary } => primary,
                    Plan::Full { .. } => full_budget(total),
                    Plan::Measure => full_budget(total), // the yield fallback
                };
                for name in &names {
                    if name.chars().count() <= PRIMARY_MIN_CHARS {
                        assert_eq!(
                            &fit_primary(name, floor),
                            name,
                            "{kind:?}@{total}: short primary {name:?} elided (budget {floor})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn law_wide_budgets_match_the_historical_math() {
        for kind in ALL_KINDS {
            let (_, widest_secondary) = rows_for(kind);
            let Some(widest) = widest_secondary else {
                assert!(
                    matches!(plan(WIDE_TOTAL, widest_secondary), Plan::Full { .. }),
                    "{kind:?}: a label-less kind must plan Full at the wide budget"
                );
                continue;
            };
            let historical = WIDE_TOTAL.saturating_sub(1 + widest + GAP_CHARS).max(4);
            match plan(WIDE_TOTAL, widest_secondary) {
                Plan::Split { primary } => assert_eq!(
                    primary, historical,
                    "{kind:?}: wide Split budget must equal the historical formula"
                ),
                Plan::Measure => assert!(
                    historical < PRIMARY_MIN_CHARS,
                    "{kind:?}: only an already-broken wide budget ({historical}) may re-measure"
                ),
                Plan::Full { .. } => unreachable!("{kind:?}: Full needs no label column"),
            }
        }
    }

    #[test]
    fn secondary_yields_monotonically_as_the_budget_narrows() {
        let widest = Some("rounded square + trailing underline".chars().count());
        let mut granted = true;
        for total in (8..=80).rev() {
            match plan(total, widest) {
                Plan::Split { primary } => {
                    assert!(
                        granted,
                        "a withdrawn secondary must not re-grant at {total}"
                    );
                    assert!(primary >= PRIMARY_MIN_CHARS);
                }
                Plan::Measure => granted = false,
                Plan::Full { .. } => unreachable!(),
            }
        }
    }

    #[test]
    fn fits_charges_the_gap_only_when_a_secondary_shows() {
        assert!(fits(100.0, 10.0, 60.0, 30.0));
        assert!(
            !fits(100.0, 10.0, 61.0, 30.0),
            "gap + cells past the width must fail"
        );
        assert!(
            fits(100.0, 10.0, 100.0, 0.0),
            "a lone primary may fill the full width"
        );
        assert!(!fits(100.0, 10.0, 100.5, 0.0));
    }

    #[test]
    fn full_budget_matches_the_historical_lone_column() {
        assert_eq!(full_budget(40), 39);
        assert_eq!(full_budget(5), 4);
        assert_eq!(full_budget(0), 4);
    }

    #[test]
    fn fit_primary_end_drops_the_subtitle_then_end_elides() {
        let title = "WORLDS.md — the themes we ship, in plain flavour";
        let head = "WORLDS.md";
        let budget = head.chars().count() + 6;
        assert_eq!(
            fit_primary_end(title, budget),
            head,
            "the subtitle is dropped whole"
        );
        assert_ne!(
            fit_primary_end(title, budget),
            fit_primary(title, budget),
            "the prose variant must differ from the filename middle-elide"
        );

        let tight = 5;
        let out = fit_primary_end(title, tight);
        assert!(out.chars().count() <= tight);
        assert!(out.ends_with('…'), "end-elision keeps the front: {out:?}");
        assert!(out.starts_with("WORL"), "the front survives: {out:?}");

        let plain = "a-long-hyphenated-prose-heading-with-no-subtitle";
        let out = fit_primary_end(plain, 12);
        assert_eq!(out.chars().count(), 12);
        assert!(out.ends_with('…') && out.starts_with("a-long"), "{out:?}");

        assert_eq!(fit_primary_end("Short title", 40), "Short title");
        assert_eq!(fit_primary_end("Ornament faces", 40), "Ornament faces");
    }

    /// A fake `measure` charging WIDE glyphs (e.g. the outline's own repeated-⌘
    /// repro) more than plain ones per char — a pure stand-in for a real shaped
    /// probe, so this test needs no font system at all.
    fn wide_glyph_measure(s: &str) -> f32 {
        s.chars().map(|c| if c == '⌘' { 3.0 } else { 1.0 }).sum()
    }

    #[test]
    fn fit_primary_end_to_px_shrinks_by_measured_width_not_char_count() {
        let wide = "⌘".repeat(20);
        let out = fit_primary_end_to_px(&wide, 20.0, wide_glyph_measure);
        assert!(
            wide_glyph_measure(&out) <= 20.0,
            "the shrunk candidate must genuinely fit the pixel budget: {out:?} measures {}",
            wide_glyph_measure(&out)
        );
        assert!(
            out.chars().count() < wide.chars().count(),
            "it must have actually shrunk: {out:?}"
        );
        assert!(
            out.ends_with('…'),
            "shrinking always leaves a trailing ellipsis: {out:?}"
        );

        let short = "⌘⌘⌘";
        assert_eq!(
            fit_primary_end_to_px(short, 20.0, wide_glyph_measure),
            short,
            "a candidate already within budget is a no-op"
        );

        let out = fit_primary_end_to_px(&wide, 0.0, wide_glyph_measure);
        assert_eq!(out, "", "an impossible budget shrinks all the way to empty");
    }

    #[test]
    fn fit_primary_end_to_px_composes_with_an_already_char_fit_candidate() {
        let title = "Head — a rather long subtitle that would normally be dropped";
        let char_fit = fit_primary_end(title, 10);
        assert_eq!(
            char_fit, "Head",
            "sanity: the char-count fit dropped the subtitle"
        );
        let out = fit_primary_end_to_px(&char_fit, 2.0, |s| s.chars().count() as f32);
        assert!(out.chars().count() as f32 <= 2.0);
        assert!(out.ends_with('…') || out.is_empty(), "{out:?}");
    }

    #[test]
    fn fit_primary_is_the_only_elision_door() {
        assert_eq!(fit_primary("Block", 27), "Block");
        let deep = "very/deeply/nested/dir/some_quite_long_filename_here.md";
        let out = fit_primary(deep, 27);
        assert_eq!(out, crate::overlay::elide_path(deep, 27));
        assert!(out.chars().count() <= 27);
        assert!(out.ends_with(".md"), "the extension survives: {out}");
    }

    #[test]
    fn gutter_plan_hides_below_the_hard_floor() {
        assert_eq!(gutter_plan(GUTTER_MIN_NAME_CHARS - 1), None);
        assert_eq!(gutter_plan(0), None);
        assert!(gutter_plan(GUTTER_MIN_NAME_CHARS).is_some());
    }

    #[test]
    fn gutter_short_lines_never_elided_or_hidden() {
        let name = "DESIGN.md"; // 9 chars
        let project = "awl"; // 3 chars
        for avail in GUTTER_MIN_NAME_CHARS..=40 {
            let plan = gutter_plan(avail).expect("avail is at/above the hard floor");
            assert!(
                plan.show_project,
                "the gutter never hides the project from width pressure alone"
            );
            if name.chars().count() <= avail {
                assert_eq!(
                    fit_primary(name, plan.name_budget),
                    name,
                    "a name that fits must render whole at avail={avail}"
                );
            }
            if project.chars().count() <= avail {
                assert_eq!(
                    fit_primary(project, plan.project_budget),
                    project,
                    "a project that fits must render whole at avail={avail}"
                );
            }
        }
    }

    #[test]
    fn gutter_name_elides_when_narrow_while_project_stays_visible() {
        let name = "a-fairly-long-descriptive-filename.md";
        let project = "awl-next"; // short enough to stay whole at the avail below
        let avail = GUTTER_MIN_NAME_CHARS + 2;
        assert!(
            avail < name.chars().count(),
            "fixture must land the name in its eliding band"
        );
        assert!(
            project.chars().count() <= avail,
            "fixture project must stay whole at this avail"
        );

        let plan = gutter_plan(avail).unwrap();
        assert!(
            plan.show_project,
            "the project must never be hidden just because the name is eliding"
        );
        let fitted_name = fit_primary(name, plan.name_budget);
        assert_ne!(
            fitted_name, name,
            "a name this long at avail={avail} must actually elide"
        );
        assert!(fitted_name.chars().count() <= avail);
        assert!(
            fitted_name.ends_with(".md"),
            "elision preserves the extension: {fitted_name:?}"
        );
        assert_eq!(
            fit_primary(project, plan.project_budget),
            project,
            "the project is unaffected by the name eliding alongside it"
        );
    }

    #[test]
    fn gutter_project_elides_when_narrow_while_name_stays_visible() {
        let name = "short.md";
        let project = "a-fairly-long-project-directory-name";
        let avail = GUTTER_MIN_NAME_CHARS + 2;
        assert!(
            avail < project.chars().count(),
            "fixture must land the project in its eliding band"
        );
        assert!(
            name.chars().count() <= avail,
            "fixture name must stay whole at this avail"
        );

        let plan = gutter_plan(avail).unwrap();
        assert!(plan.show_project);
        assert_eq!(
            fit_primary(name, plan.name_budget),
            name,
            "the name is unaffected by the project eliding alongside it"
        );
        let fitted_project = fit_primary(project, plan.project_budget);
        assert_ne!(
            fitted_project, project,
            "a project this long at avail={avail} must actually elide"
        );
        assert!(fitted_project.chars().count() <= avail);
    }

    #[test]
    fn gutter_both_lines_elide_independently_when_both_are_long() {
        let name = "a-fairly-long-descriptive-filename.md";
        let project = "a-fairly-long-project-directory-name";
        let avail = GUTTER_MIN_NAME_CHARS + 2;

        let plan = gutter_plan(avail).unwrap();
        assert!(
            plan.show_project,
            "the project line never disappears from width pressure alone"
        );
        let fitted_name = fit_primary(name, plan.name_budget);
        let fitted_project = fit_primary(project, plan.project_budget);
        assert_ne!(fitted_name, name);
        assert_ne!(fitted_project, project);
        assert!(fitted_name.chars().count() <= avail);
        assert!(fitted_project.chars().count() <= avail);
    }

    /// `fit_primary` is the gutter's only elision door too — never a bespoke
    /// wrap/truncate implementation in `render/chrome.rs`.
    #[test]
    fn gutter_name_elision_preserves_the_extension() {
        let long = "some-quite-long-note-title-that-overflows.md";
        let plan = gutter_plan(GUTTER_MIN_NAME_CHARS).unwrap();
        let out = fit_primary(long, plan.name_budget);
        assert_eq!(out, crate::overlay::elide_path(long, plan.name_budget));
        assert!(out.chars().count() <= GUTTER_MIN_NAME_CHARS);
        assert!(out.ends_with(".md"), "the extension survives: {out}");
        assert!(
            !out.contains('\n'),
            "the fitted name must always be ONE line"
        );
    }
}
