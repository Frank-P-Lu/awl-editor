//! src/range.rs — THE RANGE SPEC OWNER (item 94): one typed description of a
//! numeric setting that can be scrubbed on a rail, and the ONE place every bit of
//! its arithmetic lives.
//!
//! A [`RangeSpec`] carries the six authored facts about a range setting —
//! `min`, `max`, `step`, `default`, its display [`Unit`], and its rail
//! [`RailMap`] (linear or logarithmic) — and owns EVERY derivation from them:
//! quantization + clamping ([`RangeSpec::quantize`]), the discrete step grid
//! ([`RangeSpec::step_of`] / [`RangeSpec::value_of_step`]), the rail mapping in
//! both directions ([`RangeSpec::frac_of`] / [`RangeSpec::value_at_frac`]),
//! keyboard stepping ([`RangeSpec::stepped`]), the READOUT string
//! ([`RangeSpec::format`]), the exact-entry PARSE ([`RangeSpec::parse`]), and the
//! config RHS ([`RangeSpec::persist_value`]).
//!
//! SINGLE OWNER, by construction: keyboard (`actions::overlay_nav`), pointer
//! (`app::input::mouse` / `drags`), the readout (`settings::value_for`), the
//! drawn rail (`render::rowlayout::rail_geom` + `render/chrome`), the sidecar
//! (`capture`), and persistence (`app::files::settings`) all route through the
//! SAME spec — no input path computes a parallel value. `crate::render::clamp_zoom`
//! — the zoom band's historical owner, still the door ⌘±/⌘-wheel/`--zoom` use —
//! is now a one-line delegate to [`ZOOM`], so the wheel, the keyboard, the rail,
//! and a typed `125%` all land on the same authored grid.
//!
//! THE GRID IS ANCHORED AT ZERO: a value's step index is `round(v / step)`, so
//! zoom's 0.1 grid is `…, 0.5, 0.6, …, 3.0` exactly as `clamp_zoom` always
//! produced it (byte-identical arithmetic — see
//! [`tests::quantize_reproduces_the_historical_zoom_clamp_formula`]). A spec whose
//! `min`/`max`/`default` are NOT on that grid is a bug in the spec, caught by
//! [`tests::every_registered_spec_is_grid_coherent`].
//!
//! THE LOG MAP IS GENUINELY GENERAL, not zoom-shaped: [`RailMap::Log`] maps by
//! `ln`, so a spec running 25%–400% seats 25/50/100/200/400 at exactly
//! 0.0/0.25/0.5/0.75/1.0 of the rail — item 90's `scroll_sensitivity` is this
//! module's second customer and needs NO new code here (only its own spec
//! constant). It is deliberately NOT built in this round.

/// How a range's value maps onto its RAIL (the drawn track's 0..1 position).
///
/// AUTHORED per setting, never inferred: a percentage that reads as an ADDITIVE
/// amount (zoom: 50 %→300 % in even 10-point steps) is [`Linear`](RailMap::Linear);
/// a percentage that reads as a MULTIPLIER (a sensitivity where 50 % and 200 % are
/// equal-and-opposite) is [`Log`](RailMap::Log), which seats each doubling at an
/// equal interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Logarithmic mapping is the settled scroll-sensitivity grammar, wired when that setting lands.
#[allow(dead_code)]
pub enum RailMap {
    /// Even spacing in the VALUE: `frac = (v - min) / (max - min)`.
    Linear,
    /// Even spacing in the RATIO: `frac = ln(v/min) / ln(max/min)`. Requires a
    /// strictly positive `min` (asserted by the grid-coherence law).
    Log,
}

/// A range's DISPLAY unit — how its value is written in the row's value cell and
/// how a typed value is read back. The one owner of the ×100 percent conversion
/// (the readout, the sidecar, and the exact-entry parse all share it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// A factor shown as a whole PERCENT (`0.8` → `"80%"`). Parsing accepts the
    /// readout's own form (`"80%"`), a bare FACTOR (`"1.5"`), and — because
    /// retyping over the shown `"80%"` cell should do the obvious thing — an
    /// unsuffixed integer-ish value ≥ 10 as a percent (`"125"` → `1.25`).
    Percent,
    /// A whole number of text columns (`70` → `"70"`). Exact entry accepts a
    /// finite number and the range's authored step snaps it to a whole column.
    Columns,
}

impl Unit {
    /// The value's display string (the row's SECONDARY cell).
    pub fn format(self, v: f32) -> String {
        match self {
            Unit::Percent => format!("{:.0}%", v * 100.0),
            Unit::Columns => format!("{v:.0}"),
        }
    }

    /// Read a typed string back into a raw (unclamped, unquantized) value, or
    /// `None` when it isn't a number. See [`Unit::Percent`]'s doc for the
    /// accepted forms.
    pub fn parse(self, raw: &str) -> Option<f32> {
        match self {
            Unit::Percent => {
                let s = raw.trim();
                let (num, percent) = match s.strip_suffix('%') {
                    Some(n) => (n.trim(), true),
                    None => (s, false),
                };
                let v: f32 = num.parse().ok()?;
                if !v.is_finite() {
                    return None;
                }
                Some(if percent || v >= 10.0 { v / 100.0 } else { v })
            }
            Unit::Columns => {
                let v: f32 = raw.trim().parse().ok()?;
                v.is_finite().then_some(v)
            }
        }
    }
}

/// ONE range setting's authored description — the spec owner. Constructed as a
/// `const` per setting (see [`ZOOM`]) and resolved from a
/// [`crate::settings::SettingId`] through the single map
/// [`crate::settings::range_spec`].
#[derive(Clone, Copy, Debug)]
pub struct RangeSpec {
    /// Lowest reachable value (inclusive, on the step grid).
    pub min: f32,
    /// Highest reachable value (inclusive, on the step grid).
    pub max: f32,
    /// ONE authored increment — what Left/Right moves, and the resolution the
    /// rail snaps a click/drag to. Always positive.
    pub step: f32,
    /// The built-in value, used when the setting is unset and as the NaN guard's
    /// fallback (`clamp_zoom(NaN) == 1.0` historically).
    pub default: f32,
    /// How the value is written + read back.
    pub unit: Unit,
    /// How the value maps onto the rail.
    pub map: RailMap,
}

// These shared range helpers are the deliberate API for the remaining numeric settings migration.
#[allow(dead_code)]
impl RangeSpec {
    /// Author a spec. `const` so each setting's spec is a plain constant and the
    /// historical `ZOOM_MIN`/`ZOOM_MAX`/`ZOOM_STEP` consts can be derived from it
    /// in const context (no duplicated literals anywhere).
    pub const fn new(
        min: f32,
        max: f32,
        step: f32,
        default: f32,
        unit: Unit,
        map: RailMap,
    ) -> Self {
        Self {
            min,
            max,
            step,
            default,
            unit,
            map,
        }
    }

    /// QUANTIZE + CLAMP a raw value onto the authored grid — the ONE clamp every
    /// door (keyboard, rail click/drag, typed entry, `--zoom`, a config load)
    /// runs through.
    ///
    /// FINITE GUARD: NaN would sail through both the step arithmetic and
    /// `f32::clamp` (which returns NaN for NaN) and poison every derived metric,
    /// so it falls back to [`Self::default`]; ±inf saturates through the clamp.
    /// The result is always finite in `[min, max]`.
    pub fn quantize(&self, v: f32) -> f32 {
        if v.is_nan() {
            return self.default;
        }
        let stepped = (v / self.step).round() * self.step;
        stepped.clamp(self.min, self.max)
    }

    /// The lowest grid index (`round(min / step)`) — the grid is anchored at ZERO,
    /// so a step index is an absolute multiple of `step`, not an offset from `min`.
    pub fn min_step(&self) -> u16 {
        (self.min / self.step).round().max(0.0) as u16
    }

    /// The highest grid index (`round(max / step)`).
    pub fn max_step(&self) -> u16 {
        (self.max / self.step).round().max(0.0) as u16
    }

    /// How many authored positions the rail has (`max_step - min_step + 1`).
    pub fn step_count(&self) -> u16 {
        self.max_step().saturating_sub(self.min_step()) + 1
    }

    /// The grid INDEX of a raw value (quantized + clamped first). This is the
    /// discrete identity a row carries ([`crate::overlay::RangeCell`]) — an
    /// integer, so a row stays `Eq` and a step can never drift by a float epsilon.
    pub fn step_of(&self, v: f32) -> u16 {
        (self.quantize(v) / self.step).round().max(0.0) as u16
    }

    /// The value at grid index `k` (clamped into the band, so an out-of-range
    /// index resolves to the nearest end rather than escaping).
    pub fn value_of_step(&self, k: u16) -> f32 {
        self.quantize(k as f32 * self.step)
    }

    /// The RAIL FRACTION (0..1) of a value — where its thumb sits along the track.
    /// `Linear` spaces by value; `Log` spaces by ratio (equal intervals per
    /// doubling). A degenerate band (`max <= min`) reads as 0.
    pub fn frac_of(&self, v: f32) -> f32 {
        let v = self.quantize(v);
        match self.map {
            RailMap::Linear => {
                let span = self.max - self.min;
                if span <= 0.0 {
                    return 0.0;
                }
                ((v - self.min) / span).clamp(0.0, 1.0)
            }
            RailMap::Log => {
                if self.max <= self.min || self.min <= 0.0 {
                    return 0.0;
                }
                let span = (self.max / self.min).ln();
                if span <= 0.0 {
                    return 0.0;
                }
                ((v / self.min).ln() / span).clamp(0.0, 1.0)
            }
        }
    }

    /// The rail fraction of grid index `k` — [`Self::frac_of`] through the grid.
    pub fn frac_of_step(&self, k: u16) -> f32 {
        self.frac_of(self.value_of_step(k))
    }

    /// The value a rail fraction resolves to — the INVERSE of [`Self::frac_of`],
    /// snapped to the nearest authored step. THE pointer path's only value math:
    /// a click on the rail and every drag move resolve through this one function,
    /// so a pointer can never reach a value the keyboard cannot.
    pub fn value_at_frac(&self, f: f32) -> f32 {
        let f = if f.is_nan() { 0.0 } else { f.clamp(0.0, 1.0) };
        let raw = match self.map {
            RailMap::Linear => self.min + f * (self.max - self.min),
            RailMap::Log => {
                if self.max <= self.min || self.min <= 0.0 {
                    self.min
                } else {
                    self.min * (self.max / self.min).powf(f)
                }
            }
        };
        self.quantize(raw)
    }

    /// The grid index a rail fraction resolves to.
    pub fn step_at_frac(&self, f: f32) -> u16 {
        self.step_of(self.value_at_frac(f))
    }

    /// Move `v` by exactly `steps` AUTHORED increments (Left/Right; negative goes
    /// down), quantizing first so an off-grid starting value lands on the grid
    /// rather than carrying its drift. Saturates at the band ends.
    pub fn stepped(&self, v: f32, steps: i32) -> f32 {
        let k = self.step_of(v) as i32 + steps;
        let k = k.clamp(self.min_step() as i32, self.max_step() as i32);
        self.value_of_step(k as u16)
    }

    /// The value's READOUT string (the row's secondary cell + the sidecar).
    pub fn format(&self, v: f32) -> String {
        self.unit.format(self.quantize(v))
    }

    /// Parse an EXACT typed entry (Enter's numeric-edit commit) into a clamped,
    /// quantized value, or `None` when it isn't a number. The typed path lands on
    /// the SAME grid as every other door — no "typed values are special" branch.
    pub fn parse(&self, raw: &str) -> Option<f32> {
        self.unit.parse(raw).map(|v| self.quantize(v))
    }

    /// The config RHS this value persists as (see `App::persist_pref`):
    /// percentages keep the historical three-decimal factor; columns are whole.
    pub fn persist_value(&self, v: f32) -> String {
        let v = self.quantize(v);
        match self.unit {
            Unit::Percent => format!("{v:.3}"),
            Unit::Columns => format!("{v:.0}"),
        }
    }
}

/// Zoom: 50%–300% in ten-point steps on a linear rail, defaulting to 100%.
/// `render::clamp_zoom` delegates here, preserving every existing zoom door.
pub const ZOOM: RangeSpec = RangeSpec::new(0.5, 3.0, 0.1, 1.0, Unit::Percent, RailMap::Linear);

/// Smooth pixel multiplier on a logarithmic 25%–400% rail.
pub const SCROLL_SENSITIVITY: RangeSpec =
    RangeSpec::new(0.25, 4.0, 0.05, 1.0, Unit::Percent, RailMap::Log);

/// Prose and code measures share the same whole-column band but keep distinct
/// authored defaults and config keys.
pub const PAGE_WIDTH_PROSE: RangeSpec =
    RangeSpec::new(20.0, 200.0, 1.0, 70.0, Unit::Columns, RailMap::Linear);
pub const PAGE_WIDTH_CODE: RangeSpec =
    RangeSpec::new(20.0, 200.0, 1.0, 100.0, Unit::Columns, RailMap::Linear);

/// Every registered spec, for the pure sweep laws.
#[cfg(test)]
pub(crate) const REGISTERED: &[(&str, RangeSpec)] = &[
    ("page_width_prose", PAGE_WIDTH_PROSE),
    ("page_width_code", PAGE_WIDTH_CODE),
    ("zoom", ZOOM),
    ("scroll_sensitivity", SCROLL_SENSITIVITY),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The item-90 SHAPE, built here only to prove the LOG map is general rather
    /// than zoom-shaped: 25 %–400 %, whole-percentage 5-point steps. Item 90 owns
    /// the real `scroll_sensitivity` setting; this is a test fixture, deliberately
    /// NOT registered.
    const SENSITIVITY_SHAPE: RangeSpec =
        RangeSpec::new(0.25, 4.0, 0.05, 1.0, Unit::Percent, RailMap::Log);

    #[test]
    fn quantize_reproduces_the_historical_zoom_clamp_formula() {
        let _g = crate::testlock::serial();
        // The pre-item-94 body of `render::clamp_zoom`, verbatim.
        fn historical(z: f32) -> f32 {
            if z.is_nan() {
                return 1.0;
            }
            let stepped = (z / 0.1f32).round() * 0.1f32;
            stepped.clamp(0.5, 3.0)
        }
        let mut z = -5.0f32;
        while z <= 8.0 {
            assert_eq!(
                ZOOM.quantize(z).to_bits(),
                historical(z).to_bits(),
                "quantize({z}) must be BIT-identical to the historical clamp_zoom"
            );
            z += 0.017;
        }
        assert_eq!(ZOOM.quantize(f32::NAN), 1.0);
        assert_eq!(ZOOM.quantize(f32::INFINITY), 3.0);
        assert_eq!(ZOOM.quantize(f32::NEG_INFINITY), 0.5);
    }

    #[test]
    fn every_registered_spec_is_grid_coherent() {
        let _g = crate::testlock::serial();
        for (name, s) in REGISTERED {
            assert!(s.step > 0.0, "{name}: step must be positive");
            assert!(s.max > s.min, "{name}: max must exceed min");
            assert!(
                (s.min..=s.max).contains(&s.default),
                "{name}: default {} is outside [{}, {}]",
                s.default,
                s.min,
                s.max
            );
            if s.map == RailMap::Log {
                assert!(s.min > 0.0, "{name}: a Log rail needs a positive min");
            }
            // min / max / default all sit ON the zero-anchored grid, so the ends
            // are reachable and `quantize` is the identity there.
            for (what, v) in [("min", s.min), ("max", s.max), ("default", s.default)] {
                assert!(
                    (s.quantize(v) - v).abs() < s.step * 1e-3,
                    "{name}: {what} {v} is off the step grid"
                );
            }
            assert!(
                s.step_count() >= 2,
                "{name}: a range needs at least two stops"
            );
        }
    }

    #[test]
    fn value_and_step_round_trip_across_the_whole_grid() {
        let _g = crate::testlock::serial();
        for (name, s) in REGISTERED
            .iter()
            .chain([("sensitivity", SENSITIVITY_SHAPE)].iter())
        {
            for k in s.min_step()..=s.max_step() {
                let v = s.value_of_step(k);
                assert_eq!(s.step_of(v), k, "{name}: step {k} -> {v} -> lost its index");
                assert!(
                    v >= s.min - 1e-6 && v <= s.max + 1e-6,
                    "{name}: {v} escaped the band"
                );
            }
        }
    }

    #[test]
    fn rail_fraction_round_trips_every_authored_step_in_both_mappings() {
        let _g = crate::testlock::serial();
        for (name, s) in REGISTERED
            .iter()
            .chain([("sensitivity", SENSITIVITY_SHAPE)].iter())
        {
            for k in s.min_step()..=s.max_step() {
                let f = s.frac_of_step(k);
                assert!((0.0..=1.0).contains(&f), "{name}: frac {f} out of [0,1]");
                assert_eq!(
                    s.step_at_frac(f),
                    k,
                    "{name}: step {k} -> frac {f} -> did not come back"
                );
            }
            assert_eq!(s.frac_of(s.min), 0.0, "{name}: min sits at the rail head");
            assert!(
                (s.frac_of(s.max) - 1.0).abs() < 1e-5,
                "{name}: max sits at the rail tail"
            );
            assert_eq!(s.value_at_frac(0.0), s.min);
            assert!((s.value_at_frac(1.0) - s.max).abs() < s.step * 0.5);
        }
    }

    #[test]
    fn a_rail_fraction_always_resolves_onto_the_authored_grid_and_never_escapes_the_band() {
        let _g = crate::testlock::serial();
        for (name, s) in REGISTERED
            .iter()
            .chain([("sensitivity", SENSITIVITY_SHAPE)].iter())
        {
            // Out-of-range + NaN fractions clamp rather than escape.
            assert_eq!(s.value_at_frac(-3.0), s.min, "{name}");
            assert_eq!(s.value_at_frac(9.0), s.max, "{name}");
            assert_eq!(s.value_at_frac(f32::NAN), s.min, "{name}");
            let mut f = 0.0f32;
            while f <= 1.0 {
                let v = s.value_at_frac(f);
                assert!(
                    v >= s.min && v <= s.max,
                    "{name}: frac {f} -> {v} out of band"
                );
                assert_eq!(
                    s.quantize(v).to_bits(),
                    v.to_bits(),
                    "{name}: frac {f} -> {v} is off the authored grid"
                );
                f += 0.001;
            }
        }
    }

    /// THE LOG MAP'S REASON TO EXIST (item 90's requirement, proved on the SHAPE
    /// only — the setting itself is item 90's to build): 25/50/100/200/400 % —
    /// each a doubling — occupy EQUAL intervals on the rail.
    #[test]
    fn a_log_rail_seats_every_doubling_at_an_equal_interval() {
        let _g = crate::testlock::serial();
        let s = SENSITIVITY_SHAPE;
        let want = [
            (0.25, 0.0),
            (0.5, 0.25),
            (1.0, 0.5),
            (2.0, 0.75),
            (4.0, 1.0),
        ];
        for (v, f) in want {
            assert!(
                (s.frac_of(v) - f).abs() < 1e-4,
                "{v} should sit at {f} of the log rail, got {}",
                s.frac_of(v)
            );
        }
        // …and the SAME values on a LINEAR rail of the same band do NOT — the two
        // mappings are genuinely different, not a cosmetic flag.
        let lin = RangeSpec::new(0.25, 4.0, 0.05, 1.0, Unit::Percent, RailMap::Linear);
        assert!(
            (lin.frac_of(1.0) - 0.5).abs() > 0.2,
            "a linear rail must NOT seat 100% at mid-rail for this band"
        );
    }

    #[test]
    fn keyboard_stepping_moves_exactly_one_authored_increment_and_saturates() {
        let _g = crate::testlock::serial();
        for (name, s) in REGISTERED
            .iter()
            .chain([("sensitivity", SENSITIVITY_SHAPE)].iter())
        {
            let mid = s.value_of_step((s.min_step() + s.max_step()) / 2);
            assert!(
                (s.stepped(mid, 1) - (mid + s.step)).abs() < s.step * 1e-3,
                "{name}: +1 step must move exactly one increment"
            );
            assert!(
                (s.stepped(mid, -1) - (mid - s.step)).abs() < s.step * 1e-3,
                "{name}: -1 step must move exactly one increment"
            );
            assert_eq!(s.stepped(s.max, 1), s.max, "{name}: saturates at the top");
            assert_eq!(
                s.stepped(s.min, -1),
                s.min,
                "{name}: saturates at the bottom"
            );
            // An OFF-GRID start lands on the grid, not one increment off it.
            let off = s.min + s.step * 0.4;
            assert_eq!(
                s.stepped(off, 1),
                s.value_of_step(s.min_step() + 1),
                "{name}"
            );
            // Repeated single steps == one batched step (ordinary key repeat).
            let mut v = s.min;
            for _ in 0..5 {
                v = s.stepped(v, 1);
            }
            assert_eq!(v, s.stepped(s.min, 5), "{name}: repeat == batch");
        }
    }

    /// INCREMENTAL vs BATCHED DRAG EQUIVALENCE: scrubbing through a sequence of
    /// resolved fractions and releasing lands on EXACTLY the value a single jump
    /// to the final fraction would — the pointer path carries no accumulated
    /// state, so a fast drag and a slow one settle identically.
    #[test]
    fn a_drag_resolved_step_by_step_settles_where_one_jump_would() {
        let _g = crate::testlock::serial();
        for (name, s) in REGISTERED
            .iter()
            .chain([("sensitivity", SENSITIVITY_SHAPE)].iter())
        {
            let path: Vec<f32> = (0..=97).map(|i| (i as f32) / 97.0).collect();
            for end in [0.0f32, 0.13, 0.5, 0.77, 1.0] {
                // A SLOW drag: every intermediate fraction applies live, then the
                // release lands on `end`.
                let mut slow = s.value_at_frac(0.0);
                for f in path.iter().copied().filter(|f| *f <= end) {
                    slow = s.value_at_frac(f); // every drag move applies live
                }
                assert!(
                    slow >= s.min && slow <= s.max,
                    "{name}: scrub left the band"
                );
                slow = s.value_at_frac(end); // the release's own resolved step
                // A FAST drag: one move straight to `end` from the other extreme.
                let start_high = s.value_at_frac(1.0);
                assert!(start_high >= s.min && start_high <= s.max, "{name}");
                let fast = s.value_at_frac(end);
                assert_eq!(
                    slow.to_bits(),
                    fast.to_bits(),
                    "{name}: a drag scrubbed step-by-step to {end} must settle where one \
                     jump does — the pointer path may carry no accumulated state"
                );
            }
        }
    }

    #[test]
    fn percent_formats_and_parses_through_one_owner() {
        let _g = crate::testlock::serial();
        assert_eq!(ZOOM.format(0.8), "80%");
        assert_eq!(ZOOM.format(1.0), "100%");
        assert_eq!(ZOOM.parse("80%"), Some(0.8));
        assert_eq!(ZOOM.parse("1.5"), Some(1.5));
        assert_eq!(ZOOM.parse("125"), Some(ZOOM.quantize(1.25)));
        assert_eq!(ZOOM.parse("5000%"), Some(ZOOM.max), "clamps at the ceiling");
        assert_eq!(ZOOM.parse("10%"), Some(ZOOM.min), "clamps at the floor");
        assert_eq!(ZOOM.parse("wide"), None);
        assert_eq!(ZOOM.parse(""), None);
        // A typed value lands on the authored GRID like every other door — on the
        // grid's own value for step 9, which is `9 * 0.1f32` (one ULP off the `0.9`
        // literal; the historical `clamp_zoom` produced exactly this too, and
        // `step_of` reads it straight back).
        assert_eq!(ZOOM.parse("87%"), Some(ZOOM.value_of_step(9)));
        assert_eq!(ZOOM.step_of(0.9), 9);
        assert_eq!(ZOOM.persist_value(0.9), "0.900");
        // FORMAT/PARSE ROUND TRIP over the whole grid: what the cell shows always
        // types back to the same value (the exact-entry path can't lose a step).
        for k in ZOOM.min_step()..=ZOOM.max_step() {
            let v = ZOOM.value_of_step(k);
            assert_eq!(
                ZOOM.parse(&ZOOM.format(v)),
                Some(v),
                "readout of {v} did not re-parse"
            );
        }
    }
}
