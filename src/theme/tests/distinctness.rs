use super::super::*;

/// TAWNY↔MOPOKE DIFFERENTIATION law (see MOPOKE's own
/// doc comment in `worlds.rs`): the pair once shipped a BYTE-IDENTICAL caret
/// (`#FFC05E`) and selection (`#3A6FD8`), measuring only 24.6 RMS redmean
/// whole-palette distance apart — awl's tightest pair. Locks the separation
/// so it can never regress back to identity: Mopoke's caret and selection
/// (RGB, ignoring the unchanged selection alpha) must each differ from
/// Tawny's, and the pair's whole-palette RMS (the SAME `redmean`/`tokens`
/// recipe [`firetail_palette_is_numerically_distinct_from_every_other_world`]
/// uses) must clear a floor comfortably above the old identical-pair value —
/// measured ~76.1 post-change, floor set at 60 for margin.
#[test]
fn tawny_and_mopoke_carets_and_selections_are_now_numerically_distinct() {
    fn redmean(a: Srgb, b: Srgb) -> f32 {
        let rbar = (a.r as f32 + b.r as f32) * 0.5;
        let dr = a.r as f32 - b.r as f32;
        let dg = a.g as f32 - b.g as f32;
        let db = a.b as f32 - b.b as f32;
        ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
            .sqrt()
    }
    fn tokens(t: &Theme) -> [Srgb; 10] {
        [
            t.base_100,
            t.base_200,
            t.base_300,
            t.base_content,
            t.muted,
            t.faint,
            t.primary,
            t.primary_content,
            t.error,
            Srgb::rgb(
                t.selection_document.r,
                t.selection_document.g,
                t.selection_document.b,
            ),
        ]
    }

    assert_ne!(
        TAWNY.primary, MOPOKE.primary,
        "the caret must no longer be byte-identical between Tawny and Mopoke"
    );
    assert_ne!(
        (
            TAWNY.selection_document.r,
            TAWNY.selection_document.g,
            TAWNY.selection_document.b
        ),
        (
            MOPOKE.selection_document.r,
            MOPOKE.selection_document.g,
            MOPOKE.selection_document.b
        ),
        "the selection tint must no longer be byte-identical between Tawny and Mopoke"
    );
    assert_eq!(
        TAWNY.selection_document.a, MOPOKE.selection_document.a,
        "the selection ALPHA must stay identical between Tawny and Mopoke — only the \
         hue differentiates them"
    );

    let (tawny, mopoke) = (tokens(&TAWNY), tokens(&MOPOKE));
    let rms = (tawny
        .iter()
        .zip(mopoke)
        .map(|(&a, b)| redmean(a, b).powi(2))
        .sum::<f32>()
        / tawny.len() as f32)
        .sqrt();
    assert!(
        rms >= 60.0,
        "Tawny-Mopoke whole-palette distance is only {rms:.1} RMS redmean (floor 60, \
         set with margin above the pair's own once-identical value of 24.6 — see \
         this module's doc)"
    );
}

/// The ROSTER-WIDE distinctness floor — every pair of worlds, not a hand-picked
/// near-pair.
///
/// **THE THRESHOLD IS CHOSEN, and here is the choice.** It is 40.0, and it is
/// NOT Kite's own number. After the shift Kite↔Brolga measures 49.3, but the
/// roster's true closest pair is **Tawny↔Bowerbird at 40.6**, with
/// Magpie↔Brolga at 40.8 immediately behind it — two pairs that have nothing to
/// do with Kite and that nobody had pinned. Setting the floor at what Kite
/// happens to measure would fail on the day;
/// setting it at a round 35 or 30 would bless pairs looser than anything that
/// ships. 40.0 is the largest whole point below the roster's own measured
/// minimum, so this law is as tight as the roster actually is and a NEW world
/// cannot arrive closer to an existing one than the closest pair already
/// standing. It is written to sweep every pair because the true closest pair
/// is reliably the one nobody pins, and hand-picked pairs are how that
/// happens.
///
/// It sits BELOW the two pairwise floors this repo already defends (Firetail
/// ≥70, Tawny↔Mopoke ≥60) and that is deliberate. Those are specific defences
/// of specific decisions — a world's whole identity, and a pair that once
/// shipped byte-identical carets. This is a floor under the whole roster, not a
/// target for any pair, and a floor that could not be met by the roster it
/// governs would not be a law.
#[test]
fn every_pair_of_worlds_clears_the_roster_wide_distinctness_floor() {
    fn redmean(a: Srgb, b: Srgb) -> f32 {
        let rbar = (a.r as f32 + b.r as f32) * 0.5;
        let dr = a.r as f32 - b.r as f32;
        let dg = a.g as f32 - b.g as f32;
        let db = a.b as f32 - b.b as f32;
        ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
            .sqrt()
    }
    /// The SAME ten authored tokens the two pairwise laws above compare, so the
    /// three are one measurement with three scopes rather than three metrics.
    fn tokens(t: &Theme) -> [Srgb; 10] {
        [
            t.base_100,
            t.base_200,
            t.base_300,
            t.base_content,
            t.muted,
            t.faint,
            t.primary,
            t.primary_content,
            t.error,
            Srgb::rgb(
                t.selection_document.r,
                t.selection_document.g,
                t.selection_document.b,
            ),
        ]
    }
    const ROSTER_FLOOR: f32 = 40.0;

    let mut pairs: Vec<(f32, &str, &str)> = Vec::new();
    for (i, a) in THEMES.iter().enumerate() {
        for b in THEMES.iter().skip(i + 1) {
            let (ta, tb) = (tokens(a), tokens(b));
            let rms = (ta
                .iter()
                .zip(tb)
                .map(|(&x, y)| redmean(x, y).powi(2))
                .sum::<f32>()
                / ta.len() as f32)
                .sqrt();
            pairs.push((rms, a.name, b.name));
        }
    }
    pairs.sort_by(|x, y| x.0.total_cmp(&y.0));
    assert!(
        pairs.len() >= 100,
        "the sweep must be every PAIR of the whole roster, got {}",
        pairs.len()
    );
    let closest: Vec<String> = pairs
        .iter()
        .take(5)
        .map(|(v, a, b)| format!("{a}↔{b} {v:.1}"))
        .collect();
    let (worst, a, b) = pairs[0];
    assert!(
        worst >= ROSTER_FLOOR,
        "the roster's closest pair is {a}↔{b} at {worst:.1} RMS redmean, under the \
         {ROSTER_FLOOR} floor. The five closest are {closest:?} — the pair that trips \
         this floor is rarely the pair you were last adjusting, which is why the \
         sweep covers the whole roster instead of a hand-picked pair."
    );
    // The floor is a floor, not a description of one pair: the pairs the two
    // specific laws above defend must still be far clear of it, or this law has
    // quietly become the only thing holding them.
    for (v, x, y) in &pairs {
        if [*x, *y].contains(&FIRETAIL.name) {
            assert!(
                *v >= 70.0,
                "{x}↔{y} at {v:.1} — Firetail's own ≥70 law and this one disagree, \
                 which means one of them has stopped being maintained"
            );
        }
    }
}
