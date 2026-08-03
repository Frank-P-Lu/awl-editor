//! The ONE owner of what a summoned card SAYS.
//!
//! The About / Lifetime / Streaks / stats-HUD / peek cards used to compose
//! their text inside `render/chrome/hud.rs`, as a local `Vec<(String, u8)>` of
//! text-plus-style-role. That made the renderer the only description of the
//! content, so a second consumer — the semantic tree an assistive technology
//! reads — could only be built by re-deriving the same captions and figures
//! somewhere else. Two parallel descriptions of one card is exactly the drift
//! this module exists to prevent, so the content moved here and the renderer
//! kept what is genuinely its own: style, metrics and geometry.
//!
//! The split is: [`CardInputs`] carries every figure a card can show — the
//! three DOCUMENT ones from [`crate::card::figures`], the live-only ones in
//! [`CardLive`] — [`open_card`] decides which card is up and composes it, and
//! [`CardContent::spans`] flattens the composed card into the exact
//! `(text, style)` pairs the glyphon rich-text call wants. A semantic consumer
//! reads the same [`CardContent`] structurally and never sees the newlines.
//!
//! Because the figures are derivable without a render pipeline, the semantic
//! fold composes its own card rather than being handed one. That is what lets a
//! `--screenshot-app` capture — which has no pipeline of its own — announce the
//! card its PNG draws.

use super::CardLive;
use crate::streaks::CardView;

enum_with_all! {
    /// Every passive card awl can summon. A new one must be placed here to be
    /// swept by the roster laws — there is no wildcard arm anywhere over this.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CardKind {
        About,
        Lifetime,
        Streaks,
        Hud,
        Peek,
    }
}

/// Read only by the semantic fold, which is native-only.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
impl CardKind {
    /// The stable semantic id prefix for this card's nodes.
    pub fn id(self) -> &'static str {
        match self {
            CardKind::About => "card.about",
            CardKind::Lifetime => "card.lifetime",
            CardKind::Streaks => "card.streaks",
            CardKind::Hud => "card.hud",
            CardKind::Peek => "card.peek",
        }
    }

    /// The card's accessible name. Cards carry no visible title bar — the
    /// About card's own "Awl" line is content, not a heading — so the name is
    /// stated here rather than lifted out of the spans.
    pub fn title(self) -> &'static str {
        match self {
            CardKind::About => "About awl",
            CardKind::Lifetime => "Lifetime statistics",
            CardKind::Streaks => "Writing streaks",
            CardKind::Hud => "Document statistics",
            CardKind::Peek => "Shortcuts",
        }
    }
}

/// How a span reads. The renderer maps these to metrics + ink; the semantic
/// tree maps them to a role. Nothing else may interpret them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardStyle {
    /// A small faint caption above a figure ("WORD COUNT").
    Caption,
    /// The figure itself, in body ink.
    Body,
    /// The About card's wordmark, at section scale.
    Title,
    /// The About card's end-mark, drawn in the world's ornament face.
    Ornament,
}

impl CardStyle {
    /// The renderer's style index. `prepare_hud` matches on this to pick
    /// metrics and colour; the numbering is the renderer's private business
    /// and is kept here only so both card paths agree on one mapping.
    pub(crate) fn index(self) -> u8 {
        match self {
            CardStyle::Caption => 0,
            CardStyle::Body => 1,
            CardStyle::Title => 2,
            CardStyle::Ornament => 3,
        }
    }
}

/// One line of a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSpan {
    pub text: String,
    pub style: CardStyle,
    /// A blank line follows this one — the gap that groups a card's figures.
    pub gap_after: bool,
}

impl CardSpan {
    fn new(text: impl Into<String>, style: CardStyle, gap_after: bool) -> Self {
        Self {
            text: text.into(),
            style,
            gap_after,
        }
    }
}

/// A composed card: which one, what it is called, and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardContent {
    pub kind: CardKind,
    pub spans: Vec<CardSpan>,
}

impl CardContent {
    /// The renderer's flattened rich-text runs: each span's text with its own
    /// line break and grouping gap already in it, paired with a style index.
    /// The LAST span gets no trailing newline — a card never ends on an empty
    /// line, which is what keeps the measured block height equal to the ink.
    pub(crate) fn spans(&self) -> Vec<(String, u8)> {
        let last = self.spans.len().saturating_sub(1);
        self.spans
            .iter()
            .enumerate()
            .map(|(i, span)| {
                let text = if i == last {
                    span.text.clone()
                } else if span.gap_after {
                    format!("{}\n\n", span.text)
                } else {
                    format!("{}\n", span.text)
                };
                (text, span.style.index())
            })
            .collect()
    }

    /// The card's lines as an assistive technology should hear them: the text
    /// alone, in reading order, with no layout characters.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub fn lines(&self) -> Vec<&str> {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

/// Every figure a card can show. The three DOCUMENT figures come from
/// [`crate::card::figures`], the one owner both the renderer and the semantic
/// fold derive them through; the live-only figures ride in [`CardLive`].
/// Nothing in this struct is a renderer type, so the composition below is
/// testable with no GPU.
#[derive(Debug, Clone)]
pub struct CardInputs {
    /// The stats HUD's hold is live AND not yielding to a summoned overlay.
    pub hud_held: bool,
    /// The shortcut peek's hold is live AND not yielding to a summoned overlay.
    pub peek_shown: bool,
    /// Which streaks page is showing.
    pub streaks_page: CardView,
    /// The word count, frontmatter language and through-doc percent, derived
    /// from the document by their one owner.
    pub doc: crate::card::figures::DocFigures,
    /// The document's line-ending convention.
    pub eol: crate::buffer::Eol,
    /// The figures only a running App can gather.
    pub live: CardLive,
}

/// The OFF-the-live-App reading: nothing summoned and every live-only figure
/// absent, so each card composes its documented placeholder. Written out rather
/// than derived because `CardView`'s default page is the streaks card's design
/// statement, not a language default, and it is owned there.
impl Default for CardInputs {
    fn default() -> Self {
        Self {
            hud_held: false,
            peek_shown: false,
            streaks_page: CardView::Heatmap,
            doc: crate::card::figures::DocFigures::default(),
            eol: crate::buffer::Eol::Lf,
            live: CardLive::default(),
        }
    }
}

/// Which card is up, and what it says — `None` when the room is calm.
///
/// The precedence is the renderer's own draw order, stated once: the streaks
/// card wins outright (it has its own geometry), then About, then Lifetime,
/// then the shortcut peek, and the stats HUD is what a bare hold shows.
pub fn open_card(inputs: &CardInputs) -> Option<CardContent> {
    let kind = if crate::streaks::streaks_open() {
        CardKind::Streaks
    } else if crate::about::about_open() {
        CardKind::About
    } else if crate::lifetime::lifetime_open() {
        CardKind::Lifetime
    } else if inputs.peek_shown {
        CardKind::Peek
    } else if inputs.hud_held {
        CardKind::Hud
    } else {
        return None;
    };
    Some(card(kind, inputs))
}

/// Compose one card by name. Public so a law can sweep [`CardKind::ALL`]
/// without having to reach every card's process-global open flag.
pub fn card(kind: CardKind, inputs: &CardInputs) -> CardContent {
    let spans = match kind {
        CardKind::About => about_spans(inputs),
        CardKind::Lifetime => lifetime_spans(inputs),
        CardKind::Streaks => streaks_spans(inputs),
        CardKind::Hud => hud_spans(inputs),
        CardKind::Peek => peek_spans(inputs),
    };
    CardContent { kind, spans }
}

fn about_spans(inputs: &CardInputs) -> Vec<CardSpan> {
    let world = crate::theme::active();
    let mut spans = vec![
        CardSpan::new("Awl", CardStyle::Title, true),
        CardSpan::new(
            format!("v{}", env!("CARGO_PKG_VERSION")),
            CardStyle::Body,
            false,
        ),
        CardSpan::new("by Frank Lu · GPL-3.0", CardStyle::Caption, true),
        CardSpan::new(world.name, CardStyle::Caption, false),
    ];
    if let Some(line) = crate::updates::checked_line(inputs.live.update_checked) {
        spans.push(CardSpan::new(line, CardStyle::Caption, false));
    }
    if inputs.live.pending_crash {
        spans.push(CardSpan::new(
            "previous crash log available · Settings → Report a Problem",
            CardStyle::Caption,
            false,
        ));
    }
    spans.push(CardSpan::new("⌘P → Credits", CardStyle::Caption, true));
    spans.push(CardSpan::new(
        world.ornaments.dash.to_string(),
        CardStyle::Ornament,
        false,
    ));
    spans
}

fn lifetime_spans(inputs: &CardInputs) -> Vec<CardSpan> {
    figure_spans(
        crate::hud::odometer_rows(inputs.live.stats.as_ref())
            .into_iter()
            .map(|(caption, value)| (caption.to_string(), value)),
    )
}

fn hud_spans(inputs: &CardInputs) -> Vec<CardSpan> {
    let mut rows: Vec<(String, String)> = vec![(
        "SAVED".to_string(),
        crate::hud::saved_readout(inputs.live.saved),
    )];
    if !inputs.doc.words.is_empty() {
        rows.push(("WORD COUNT".to_string(), inputs.doc.words.clone()));
    }
    if let Some(lang) = inputs.doc.lang {
        rows.push(("LANGUAGE".to_string(), lang.code().to_string()));
    }
    rows.push((
        "THROUGH DOC".to_string(),
        format!("{}%", inputs.doc.percent),
    ));
    rows.push(("LINE ENDINGS".to_string(), inputs.eol.label().to_string()));
    figure_spans(rows.into_iter())
}

fn streaks_spans(inputs: &CardInputs) -> Vec<CardSpan> {
    let view = inputs
        .live
        .streaks
        .clone()
        .unwrap_or_else(crate::streaks::placeholder);
    let streak = if view.streak == 1 {
        "1 day".to_string()
    } else {
        format!("{} days", crate::hud::group_thousands(view.streak))
    };
    let (caption, value) = match inputs.streaks_page {
        CardView::Heatmap => (
            "WRITTEN TODAY",
            format!("{} words", crate::hud::group_thousands(view.today_words)),
        ),
        CardView::Cumulative => (
            "PAST YEAR",
            format!(
                "{} words",
                crate::hud::group_thousands(view.cumulative.last().copied().unwrap_or(0))
            ),
        ),
    };
    figure_spans(
        [
            ("CURRENT STREAK".to_string(), streak),
            (caption.to_string(), value),
        ]
        .into_iter(),
    )
}

fn peek_spans(inputs: &CardInputs) -> Vec<CardSpan> {
    // The peek reads the other way round: the chord is the figure and the
    // command name is its caption underneath.
    let rows = crate::peek::rows_or_starter(&inputs.live.peek_rows);
    let mut spans = Vec::with_capacity(rows.len() * 2);
    for row in rows {
        spans.push(CardSpan::new(row.chord, CardStyle::Body, false));
        spans.push(CardSpan::new(row.name, CardStyle::Caption, true));
    }
    spans
}

/// The shared caption-over-figure shape: a faint caption, its figure, and a
/// blank line before the next pair.
fn figure_spans(rows: impl Iterator<Item = (String, String)>) -> Vec<CardSpan> {
    let mut spans = Vec::new();
    for (caption, value) in rows {
        spans.push(CardSpan::new(caption, CardStyle::Caption, false));
        spans.push(CardSpan::new(value, CardStyle::Body, true));
    }
    spans
}

#[cfg(test)]
mod tests;
