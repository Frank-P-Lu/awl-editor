//! The two emitters. One table model, built once from the live rosters
//! (`super::rows`), rendered as GFM markdown for `REFERENCE.md` and as HTML for
//! `site/reference.html`.
//!
//! There is deliberately no third path: the site copy is not a hand-mirror of
//! the markdown (the accepted-drift arrangement `site/guide.html` carries
//! against `GUIDE.md`), it is the SAME rows through a different emitter, so the
//! two documents cannot disagree about a fact even in principle.

/// One table cell. The distinction matters to the emitters, not to the rows:
/// [`Cell::Code`] becomes a backtick span in markdown and a `<code>` element in
/// HTML, [`Cell::Text`] stays prose in both, and an absent value renders as an
/// em dash rather than as a blank a reader could mistake for an omission.
pub(crate) enum Cell {
    Text(String),
    Code(String),
    Dash,
}

impl Cell {
    pub(crate) fn text(s: impl Into<String>) -> Cell {
        Cell::Text(s.into())
    }

    pub(crate) fn code(s: impl Into<String>) -> Cell {
        Cell::Code(s.into())
    }

    /// A [`Cell::Code`] for a non-empty value, an em dash for an empty one —
    /// the shape every "this roster member may have no such thing" column wants.
    pub(crate) fn code_or_dash(s: &str) -> Cell {
        if s.trim().is_empty() {
            Cell::Dash
        } else {
            Cell::Code(s.to_string())
        }
    }

    /// [`Self::code_or_dash`]'s PROSE sibling — a [`Cell::Text`] for `Some`, an
    /// em dash for `None`. The shape a "this roster member may have nothing to
    /// say" prose column wants, distinct from `code_or_dash` because a
    /// description is a sentence fragment, never a code span.
    pub(crate) fn text_or_dash(s: Option<&str>) -> Cell {
        match s {
            Some(s) => Cell::text(s),
            None => Cell::Dash,
        }
    }

    fn to_markdown(&self) -> String {
        match self {
            // A literal `|` inside a GFM table cell ends the cell — escaped
            // here (GFM honours `\|` inside a code span too, which is what the
            // markdown-table construct's own example row needs).
            Cell::Text(s) => escape_pipes(s),
            Cell::Code(s) => code_span(&escape_pipes(s)),
            Cell::Dash => "—".to_string(),
        }
    }

    fn to_html(&self) -> String {
        match self {
            Cell::Text(s) => escape_html(s),
            Cell::Code(s) => format!("<code>{}</code>", escape_html(s)),
            Cell::Dash => "—".to_string(),
        }
    }
}

/// A CommonMark code span whose fence is longer than any backtick run inside
/// it, padded with a space when the content itself starts or ends with a
/// backtick. A fixed single-backtick wrapper breaks on exactly the rows a
/// markdown reference must carry — `` `code` `` and a fence line.
fn code_span(s: &str) -> String {
    let mut run = 0usize;
    let mut longest = 0usize;
    for c in s.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat(longest + 1);
    let pad = if s.starts_with('`') || s.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{s}{pad}{fence}")
}

fn escape_pipes(s: &str) -> String {
    s.replace('|', "\\|")
}

pub(crate) fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) struct Table {
    headers: Vec<&'static str>,
    rows: Vec<Vec<Cell>>,
}

impl Table {
    pub(crate) fn new(headers: &[&'static str]) -> Table {
        Table {
            headers: headers.to_vec(),
            rows: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, cells: Vec<Cell>) {
        assert_eq!(
            cells.len(),
            self.headers.len(),
            "a reference table row must carry exactly one cell per header"
        );
        self.rows.push(cells);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub(crate) fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("| {} |\n", self.headers.join(" | ")));
        out.push_str(&format!(
            "|{}|\n",
            self.headers
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join("|")
        ));
        for row in &self.rows {
            let cells: Vec<String> = row.iter().map(Cell::to_markdown).collect();
            out.push_str(&format!("| {} |\n", cells.join(" | ")));
        }
        out
    }

    pub(crate) fn to_html(&self) -> String {
        let mut out = String::new();
        // The scroll wrapper is the page's job, not the row's: a wide table
        // scrolls inside its own box rather than pushing the page sideways.
        out.push_str("<div class=\"table-scroll\">\n<table>\n<thead>\n<tr>");
        for h in &self.headers {
            out.push_str(&format!("<th>{}</th>", escape_html(h)));
        }
        out.push_str("</tr>\n</thead>\n<tbody>\n");
        for row in &self.rows {
            out.push_str("<tr>");
            for c in row {
                out.push_str(&format!("<td>{}</td>", c.to_html()));
            }
            out.push_str("</tr>\n");
        }
        out.push_str("</tbody>\n</table>\n</div>\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pipe_in_a_cell_is_escaped_so_it_cannot_split_the_row() {
        let mut t = Table::new(&["A", "B"]);
        t.push(vec![Cell::code("| a | b |"), Cell::text("x")]);
        let md = t.to_markdown();
        let row = md.lines().last().expect("a row");
        // Three separators: the leading, the middle, and the trailing one.
        assert_eq!(
            row.matches(" | ").count(),
            1,
            "an unescaped pipe would split this row into extra cells: {row}"
        );
        assert!(row.contains("\\|"), "the literal pipes are escaped: {row}");
    }

    /// The axis a single-backtick wrapper fails on, and the exact rows a
    /// markdown reference carries: content that itself contains backticks.
    #[test]
    fn a_code_cell_containing_backticks_fences_longer_than_its_content() {
        for (content, want) in [
            ("code", "`code`"),
            ("`code`", "`` `code` ``"),
            ("```rust", "```` ```rust ````"),
            ("a ` b", "``a ` b``"),
        ] {
            assert_eq!(code_span(content), want, "fencing {content:?}");
        }
    }

    #[test]
    fn html_escapes_the_three_dangerous_characters() {
        let mut t = Table::new(&["A"]);
        t.push(vec![Cell::text("<b> & </b>")]);
        let html = t.to_html();
        assert!(html.contains("&lt;b&gt; &amp; &lt;/b&gt;"), "{html}");
        assert!(
            !html.contains("<b>"),
            "raw markup escaped into text: {html}"
        );
    }

    #[test]
    fn an_absent_value_renders_as_a_dash_in_both_emitters() {
        let mut t = Table::new(&["A"]);
        t.push(vec![Cell::code_or_dash("   ")]);
        assert!(t.to_markdown().contains('—'));
        assert!(t.to_html().contains('—'));
    }
}
