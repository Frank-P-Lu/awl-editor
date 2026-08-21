//! THE THEMES.md DEVIATION-TABLE LAWS. That table's last column answers "which
//! worlds carry this `RenderCaps` field away from its default" — a question the
//! code answers exactly, for every field, every time it is asked. It was
//! answered by hand, and by the time this law was written **eight of its
//! fourteen rows were wrong**: `elevation` named four worlds when fifteen
//! deviate and omitted `Recessed` entirely, `page_frame` named a world that no
//! longer carries a frame while missing the one that does, `list_style` called
//! two `Diagonal` worlds `Bars` and missed two more deviants, and
//! `title_style`, `card_anchor`, `chrome_face`, `facet_style` and
//! `caret_block_style` each omitted worlds. The table is one of the first things
//! a reader consults to learn which world does what, so a wrong row sends them
//! to the wrong world.
//!
//! TWO ARMS, over the same rows:
//!
//! * [`themes_md_deviation_rows_match_the_roster`] — the last column's
//!   `` `Variant` — World, World `` groups are exactly the roster's deviants,
//!   grouped by variant NAME. Grouping by name rather than by full value is
//!   deliberate: `Placard { corner: BL, scale: 4.5, … }` and
//!   `Placard { corner: Auto, scale: 3.0, … }` are one documented treatment with
//!   two tunings, and the prose column is where a tuning gets described.
//! * [`themes_md_values_column_names_every_shipped_variant`] — the Values column
//!   names every variant a world actually carries. This is the arm that catches
//!   the `Recessed`/`Filled`/`Diagonal`/`Rules` class: a row whose deviation list
//!   is right can still describe a two-variant field that has grown a third.
//!
//! ENROLMENT is the table's own rows, read out of the document, each keyed to
//! its field by a WILDCARD-FREE match ([`field_of`]) that panics on a row it does
//! not know — so a new row cannot be silently skipped. The reverse gap (a new
//! `RenderCaps` field with no row) is closed by
//! [`every_render_caps_field_has_a_themes_md_row_or_is_excused`], which reads the
//! field list out of `RenderCaps::DEFAULT`'s own literal.
//!
//! THE DOC CONVENTION the first arm reads: the deviation cell is `none`, or a
//! `;`-separated list of `` `Variant` — Name, Name `` groups. World names only —
//! any commentary belongs in the Governs column, because a parenthetical naming
//! another world would be harvested as a member of the wrong group.

use crate::theme::{RenderCaps, THEMES, Theme};
use std::collections::{BTreeMap, BTreeSet};

/// A `RenderCaps` field's value on a theme and on the default, as the variant
/// NAME alone — `Placard { … }` and `Placard { … }` with different tunings are
/// one name, `Named("Figtree")` is `Named`.
type Reader = fn(&RenderCaps) -> String;

/// The leading identifier of a `Debug` rendering: everything before the first
/// space, `{` or `(`.
fn variant_name(debug: &str) -> String {
    debug
        .split([' ', '{', '('])
        .next()
        .unwrap_or(debug)
        .to_string()
}

/// The reader for a deviation-table row's field. WILDCARD-FREE: an unrecognised
/// row name panics by name rather than being skipped, so a row added to the
/// document without a reader here fails loudly instead of going unchecked.
fn field_of(row: &str) -> Reader {
    macro_rules! r {
        ($f:ident) => {{
            fn g(c: &RenderCaps) -> String {
                variant_name(&format!("{:?}", c.$f))
            }
            g as Reader
        }};
    }
    match row {
        "selection_style" => r!(selection_style),
        "caret_block_style" => r!(caret_block_style),
        "backdrop" => r!(backdrop),
        "elevation" => r!(elevation),
        "decorative_wash" => r!(decorative_wash),
        "image_reveal" => r!(image_reveal),
        "highlight_texture" => r!(highlight_texture),
        "title_style" => r!(title_style),
        "placard_placement" => r!(placard_placement),
        "summoned_material" => r!(summoned_material),
        "page_frame" => r!(page_frame),
        "card_anchor" => r!(card_anchor),
        "chrome_face" => r!(chrome_face),
        "list_style" => r!(list_style),
        "pane_split" => r!(pane_split),
        "facet_style" => r!(facet_style),
        "location_style" => r!(location_style),
        other => panic!(
            "THEMES.md's deviation table has a `{other}` row this law cannot \
             read — add it to `field_of` (or explain why the row is not a \
             `RenderCaps` field)"
        ),
    }
}

/// `RenderCaps` fields with no deviation row, each excused with its reason. The
/// table documents the WORLD-FACING treatments; a field that is a tuning
/// constant rather than a treatment a reader picks a world for is out of its
/// scope, and saying so here is what keeps the omission examined rather than
/// forgotten.
const FIELDS_WITH_NO_ROW: &[&str] = &[
    // Numeric tunings, not treatments: no variant a reader could name.
    "spell_underline_gap",
    "fold_afford",
    // Documented in their own sections of THEMES.md rather than this table.
    "ambient",
    "card_texture",
    "card_shape",
];

/// The deviation table's own text: the `## …` section holding the row whose
/// first cell is `` `selection_style` ``, isolated from the tables that follow.
fn deviation_table() -> &'static str {
    let doc = crate::embedded_docs::THEMES_MD;
    let at = doc
        .find("| Field | Values | Governs | Deviates from default |")
        .expect("THEMES.md carries its RenderCaps deviation table, by that header");
    let rest = &doc[at..];
    rest.split_once("\n\n").map_or(rest, |(a, _)| a)
}

/// Every data row as `(field name, values cell, deviation cell)`.
fn rows() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for line in deviation_table().lines() {
        if !line.starts_with("| `") {
            continue;
        }
        // `\\|` is an ESCAPED pipe inside a cell (the Values column is full of
        // them), not a cell boundary. Splitting naively yields the wrong cell
        // count on thirteen of the fourteen rows, and the row is then skipped —
        // which is how a first draft of this law silently swept ONE row and
        // reported clean. Hidden before the split, restored after.
        const ESC: char = '\u{0}';
        let flat = line.replace("\\|", "\u{0}");
        let cells: Vec<String> = flat
            .trim()
            .trim_matches('|')
            .split('|')
            .map(|c| c.replace(ESC, "\\|"))
            .collect();
        if cells.len() != 4 {
            continue;
        }
        let name = cells[0].trim().trim_matches('`');
        out.push((
            name.to_string(),
            cells[1].trim().to_string(),
            cells[3].trim().to_string(),
        ));
    }
    out
}

/// A deviation cell's `` `Variant` — Name, Name `` groups. `none` yields an empty
/// map.
fn documented_groups(cell: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::new();
    if cell.trim() == "none" {
        return out;
    }
    for group in cell.split(';') {
        let group = group.trim();
        let Some((variant, worlds)) = group.split_once('—') else {
            continue;
        };
        let variant = variant.trim().trim_matches('`').to_string();
        let members: BTreeSet<String> = worlds
            .split(',')
            .map(|w| w.trim().to_string())
            .filter(|w| !w.is_empty())
            .collect();
        out.insert(variant, members);
    }
    out
}

/// The roster's own answer for one field: deviant variant name -> the worlds
/// carrying it, with the default's own variant excluded.
fn roster_groups(read: Reader) -> BTreeMap<String, BTreeSet<String>> {
    let default = read(&RenderCaps::DEFAULT);
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for t in THEMES.iter() {
        let v = read(&t.render_caps);
        if v != default {
            out.entry(v).or_default().insert(t.name.to_string());
        }
    }
    out
}

/// Whether `t` is in the roster at all — the harvest must not accept a name the
/// product does not have.
fn is_world(name: &str) -> bool {
    THEMES.iter().any(|t: &Theme| t.name == name)
}

/// THE DEVIATION LAW.
#[test]
fn themes_md_deviation_rows_match_the_roster() {
    let _g = crate::testlock::serial();
    let rows = rows();
    assert!(
        rows.len() >= 10,
        "THEMES.md's deviation table yielded only {} rows — the table's row \
         spelling changed and this law is now checking almost nothing",
        rows.len()
    );
    let mut problems: Vec<String> = Vec::new();
    let mut deviants_seen = 0usize;
    for (field, _values, cell) in &rows {
        let read = field_of(field);
        let want = roster_groups(read);
        let have = documented_groups(cell);
        deviants_seen += want.values().map(BTreeSet::len).sum::<usize>();
        for name in have.values().flatten() {
            if !is_world(name) {
                problems.push(format!(
                    "`{field}`'s row names `{name}`, which is not a world in \
                     `theme::THEMES`"
                ));
            }
        }
        if have != want {
            problems.push(format!(
                "`{field}`: doc says {have:?}, roster says {want:?}"
            ));
        }
    }
    assert!(
        deviants_seen > 0,
        "no world deviates from `RenderCaps::DEFAULT` on any documented field — \
         this law would then be satisfied by an empty table, which is the shape \
         it exists to rule out"
    );
    problems.sort();
    assert!(
        problems.is_empty(),
        "THEMES.md's `Deviates from default` column has drifted from the \
         roster ({} of {} rows; cell format is `Variant` — World, World, \
         `;`-separated, or `none`):\n{}",
        problems.len(),
        rows.len(),
        problems.join("\n")
    );
}

/// THE VALUES-COLUMN LAW — the arm that catches a field which has grown a
/// variant the row never mentions, even when its deviation list is right.
#[test]
fn themes_md_values_column_names_every_shipped_variant() {
    let _g = crate::testlock::serial();
    let mut problems: Vec<String> = Vec::new();
    for (field, values, _cell) in rows() {
        let read = field_of(&field);
        let mut shipped: BTreeSet<String> = THEMES
            .iter()
            .map(|t| read(&t.render_caps))
            .collect::<BTreeSet<_>>();
        shipped.insert(read(&RenderCaps::DEFAULT));
        // The Values column ticks the variant WITH its payload shape
        // (`Named(family)`, `Bars { radius, … }`), so compare variant NAMES on
        // both sides rather than looking for a bare-name tick — a first draft
        // did the latter and reported nine false misses.
        let named: BTreeSet<String> = values
            .split('`')
            .skip(1)
            .step_by(2)
            .map(variant_name)
            .collect();
        for variant in &shipped {
            if !named.contains(variant) {
                problems.push(format!(
                    "`{field}`'s Values column does not name `{variant}`, which \
                     a shipped world carries — it names {named:?}: {values}"
                ));
            }
        }
    }
    problems.sort();
    assert!(
        problems.is_empty(),
        "THEMES.md's Values column omits variants the roster ships:\n{}",
        problems.join("\n")
    );
}

/// THE COMPLETENESS ARM. Every `RenderCaps` field is either documented by a
/// deviation row or excused in [`FIELDS_WITH_NO_ROW`] with a reason — read out of
/// `RenderCaps::DEFAULT`'s own literal, so a field added to the struct cannot sit
/// permanently invisible to the two laws above (which only walk rows that
/// already exist).
#[test]
fn every_render_caps_field_has_a_themes_md_row_or_is_excused() {
    let _g = crate::testlock::serial();
    let src = include_str!("../model.rs");
    let literal = src
        .split_once("pub const DEFAULT: RenderCaps = RenderCaps {")
        .expect("`RenderCaps::DEFAULT`'s literal, by that spelling")
        .1;
    let literal = literal.split_once("\n    };").expect("its closing brace").0;
    let fields: Vec<&str> = literal
        .lines()
        .filter_map(|l| l.trim().split_once(':').map(|(n, _)| n.trim()))
        .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
        .collect();
    assert!(
        fields.len() > 15,
        "parsed only {} fields out of `RenderCaps::DEFAULT` — the literal's \
         spelling changed and this arm is checking almost nothing: {fields:?}",
        fields.len()
    );
    let rowed: BTreeSet<String> = rows().into_iter().map(|(f, _, _)| f).collect();
    for f in &fields {
        assert!(
            rowed.contains(*f) || FIELDS_WITH_NO_ROW.contains(f),
            "`RenderCaps::{f}` has no THEMES.md deviation row and is not \
             excused in `FIELDS_WITH_NO_ROW` — add a row, or excuse it with a \
             reason"
        );
    }
    for excused in FIELDS_WITH_NO_ROW {
        assert!(
            !rowed.contains(*excused),
            "`{excused}` is excused as having no THEMES.md row, but a row now \
             documents it — shrink the excuse list rather than leave it stale"
        );
        assert!(
            fields.contains(excused),
            "`{excused}` is excused as a `RenderCaps` field with no row, but \
             `RenderCaps::DEFAULT` has no such field any more — shrink the list"
        );
    }
}

/// THE CJK ASSIGNMENT-TABLE LAW. THEMES.md's zh-Hans / ko table gives one row
/// per world; it had rows for thirteen of twenty, so seven worlds' Chinese and
/// Korean faces were undocumented — including every world added after the
/// Chinese round.
///
/// Two facts, both derived: every world has a row, and the row's `ko` cell names
/// `CJK_KO_SERIF` exactly when the world is actually on that list. The second is
/// what makes the law more than a membership check — it asks the serif question
/// on BOTH sides and requires the answers to part, so a row copied from its
/// neighbour is caught rather than counted.
#[test]
fn themes_md_cjk_table_rows_every_world_with_its_ko_family() {
    let _g = crate::testlock::serial();
    let doc = crate::embedded_docs::THEMES_MD;
    let at = doc
        .find("| World       | Character  | `cjk` (ja)")
        .expect("THEMES.md carries its zh-Hans / ko assignment table, by that header");
    let table = {
        let rest = &doc[at..];
        rest.split_once("\n\n").map_or(rest, |(a, _)| a)
    };
    let mut rows: BTreeMap<String, String> = BTreeMap::new();
    for line in table.lines().skip(2) {
        let cells: Vec<&str> = line.trim().trim_matches('|').split('|').collect();
        if cells.len() != 5 {
            continue;
        }
        let world = cells[0].trim().trim_matches('*').to_string();
        if is_world(&world) {
            rows.insert(world, cells[4].trim().to_string());
        }
    }
    let mut serif_rows = 0usize;
    let mut sans_rows = 0usize;
    for t in THEMES.iter() {
        let ko_cell = rows.get(t.name).unwrap_or_else(|| {
            panic!(
                "THEMES.md's zh-Hans / ko table has no row for `{}` — a new \
                 world must arrive with its Chinese and Korean faces (the table \
                 lists {:?})",
                t.name,
                rows.keys().collect::<Vec<_>>()
            )
        });
        let is_serif = t.ko == crate::theme::cjk::CJK_KO_SERIF;
        let says_serif = ko_cell.contains("CJK_KO_SERIF");
        assert_eq!(
            says_serif,
            is_serif,
            "THEMES.md's `{}` row says ko = {ko_cell}, but the roster puts it \
             on {}",
            t.name,
            if is_serif { "CJK_KO_SERIF" } else { "CJK_KO" }
        );
        if is_serif {
            serif_rows += 1;
        } else {
            sans_rows += 1;
        }
    }
    assert!(
        serif_rows > 0 && sans_rows > 0,
        "the ko-family check saw only one side ({serif_rows} serif, \
         {sans_rows} sans) — with every world on one list the assertion above \
         cannot tell a right row from a copied one"
    );
}
