//! Differential laws for the one-value-per-line splice shared by the nit and
//! Han-evidence projections. Their analysis stays feature-owned; these laws
//! cover the common retention, validation, and work-count contract.

use super::super::rects::{HanEvidenceProjection, NitProjection};

fn changed_band(old: &[&str], new: &[&str]) -> (usize, usize, usize) {
    let mut prefix = 0;
    while prefix < old.len() && prefix < new.len() && old[prefix] == new[prefix] {
        prefix += 1;
    }
    let mut suffix = 0;
    while suffix < old.len() - prefix
        && suffix < new.len() - prefix
        && old[old.len() - 1 - suffix] == new[new.len() - 1 - suffix]
    {
        suffix += 1;
    }
    (prefix, old.len() - suffix, new.len() - suffix)
}

fn assert_current_oracles(nits: &NitProjection, han: &HanEvidenceProjection, lines: &[&str]) {
    assert!(nits.eligible());
    assert_eq!(nits.line_count(), lines.len());
    for (index, &line) in lines.iter().enumerate() {
        assert_eq!(
            nits.spans_for(index),
            crate::nits::line_nits(line).as_slice(),
            "nit slot {index} must match a fresh analysis of {line:?}"
        );
    }

    let expected_han: Vec<_> = lines
        .iter()
        .map(|&line| crate::script::line_evidence(line))
        .collect();
    assert_eq!(han.line_evidence(), expected_han);
    assert_eq!(
        han.aggregate(),
        crate::script::cjk_evidence(&lines.join("\n")),
        "retained Han counts must resolve like a fresh document scan"
    );
}

#[test]
fn both_projections_match_fresh_analysis_across_every_edit_shape() {
    let states: [&[&str]; 6] = [
        &["head  gap", "anchor", "說 tail", "끝  row"],
        &["head  gap", "anchor café  東京", "說 tail", "끝  row"],
        &[
            "head  gap",
            "anchor café  東京",
            "かな  insert",
            "說 tail",
            "끝  row",
        ],
        &["head  gap", "anchor café  東京", "かな  insert", "끝  row"],
        &["head  gap", "replacement  这", "another  說", "끝  row"],
        &["unrelated  α", "brand new", "かな  final"],
    ];

    let mut nits = NitProjection::new();
    let mut han = HanEvidenceProjection::new();
    assert_eq!(nits.refresh(true, states[0], None), states[0].len() as u64);
    assert_eq!(han.refresh(states[0], None), states[0].len() as u64);
    assert_current_oracles(&nits, &han, states[0]);

    let mut observed = Vec::new();
    for pair in states.windows(2) {
        let band = changed_band(pair[0], pair[1]);
        let expected_work = (band.2 - band.0) as u64;
        observed.push((band, expected_work));
        assert_eq!(nits.refresh(true, pair[1], Some(band)), expected_work);
        assert_eq!(han.refresh(pair[1], Some(band)), expected_work);
        assert_current_oracles(&nits, &han, pair[1]);
    }

    assert_eq!(observed[0].0, (1, 2, 2), "Unicode replacement band");
    assert_eq!(observed[1].0, (2, 2, 3), "line insertion band");
    assert_eq!(observed[2].0, (3, 4, 3), "line deletion band");
    assert_eq!(observed[3].0, (1, 3, 3), "multiline replacement band");
    assert_eq!(observed[4].0, (0, 4, 3), "unrelated buffer swap band");
    assert_eq!(observed[2].1, 0, "a deletion analyzes no current line");
}

#[test]
fn invalid_or_unproven_bands_reseed_both_projections_without_panicking() {
    let old = ["old  head", "old 說", "old tail"];
    let current = ["fresh  α", "かな", "new 說"];
    let shorter = ["fresh  α", "new 說"];
    let invalid = [
        (current.as_slice(), Some((2, 1, 2))),
        (shorter.as_slice(), Some((2, 2, 1))),
        (current.as_slice(), Some((0, 3, 4))),
        (current.as_slice(), Some((1, usize::MAX, 2))),
        (current.as_slice(), Some((1, 1, 2))),
        (current.as_slice(), None),
    ];

    for (current, change) in invalid {
        let mut nits = NitProjection::new();
        let mut han = HanEvidenceProjection::new();
        nits.refresh(true, &old, None);
        han.refresh(&old, None);

        assert_eq!(
            nits.refresh(true, current, change),
            current.len() as u64,
            "unproven band {change:?} must fully reseed nits"
        );
        assert_eq!(
            han.refresh(current, change),
            current.len() as u64,
            "unproven band {change:?} must fully reseed Han evidence"
        );
        assert_current_oracles(&nits, &han, current);
    }
}

#[test]
fn nit_eligibility_clears_then_requires_a_full_reseed() {
    let old = ["old  head", "anchor", "old tail"];
    let current = ["old  head", "anchor", "fresh  說", "old tail"];
    let mut nits = NitProjection::new();

    assert_eq!(nits.refresh(true, &old, None), old.len() as u64);
    assert_eq!(nits.refresh(false, &current, Some((2, 2, 3))), 0);
    assert!(!nits.eligible());
    assert_eq!(nits.line_count(), 0);

    assert_eq!(
        nits.refresh(true, &current, Some((2, 2, 3))),
        current.len() as u64,
        "returning to eligibility cannot retain state that was deliberately cleared"
    );
    for (index, &line) in current.iter().enumerate() {
        assert_eq!(nits.spans_for(index), crate::nits::line_nits(line));
    }
}
