//! The checked-in Awl Marks adoption roster, shared by chrome routing and laws.

use std::sync::LazyLock;

pub(crate) const RAW: &str = include_str!("../../assets/fonts/AwlMarks.roster.tsv");

#[derive(Debug)]
pub(crate) struct AdoptedMark {
    pub codepoint: u32,
    #[cfg(test)]
    pub name: &'static str,
    pub roles: Vec<&'static str>,
    #[cfg(test)]
    pub source_range: &'static str,
}

static ROSTER: LazyLock<Vec<AdoptedMark>> = LazyLock::new(|| {
    let mut marks = Vec::new();
    let mut saw_header = false;
    for (line_index, line) in RAW.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !saw_header {
            assert_eq!(
                line,
                "codepoint\tname\troles\tsource_range",
                "AwlMarks.roster.tsv:{}: malformed header",
                line_index + 1
            );
            saw_header = true;
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            4,
            "AwlMarks.roster.tsv:{}: expected four TSV fields",
            line_index + 1
        );
        let codepoint = u32::from_str_radix(
            fields[0]
                .strip_prefix("U+")
                .unwrap_or_else(|| panic!("AwlMarks.roster.tsv:{}: bad codepoint", line_index + 1)),
            16,
        )
        .unwrap_or_else(|_| panic!("AwlMarks.roster.tsv:{}: bad codepoint", line_index + 1));
        assert!(
            char::from_u32(codepoint).is_some(),
            "AwlMarks.roster.tsv:{}: codepoint is not a Unicode scalar",
            line_index + 1
        );
        assert!(
            fields[1..].iter().all(|field| !field.is_empty()),
            "AwlMarks.roster.tsv:{}: name, roles, and source range are required",
            line_index + 1
        );
        marks.push(AdoptedMark {
            codepoint,
            #[cfg(test)]
            name: fields[1],
            roles: fields[2].split(',').collect(),
            #[cfg(test)]
            source_range: fields[3],
        });
    }
    assert!(
        saw_header && !marks.is_empty(),
        "AwlMarks.roster.tsv is empty"
    );
    assert!(
        marks
            .windows(2)
            .all(|pair| pair[0].codepoint < pair[1].codepoint),
        "AwlMarks.roster.tsv must be strictly sorted with no duplicates"
    );
    marks
});

pub(crate) fn has_role(ch: char, role: &str) -> bool {
    ROSTER
        .binary_search_by_key(&(ch as u32), |mark| mark.codepoint)
        .ok()
        .is_some_and(|index| ROSTER[index].roles.contains(&role))
}

#[cfg(test)]
pub(crate) fn roster() -> &'static [AdoptedMark] {
    &ROSTER
}
