# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Ready to build

---
### 525 — start screen: equal ink + chord hints now; per-world dress later (user decision, 2026-08-29)

🟡 IN PROGRESS — claude, branch item-525

Today's start screen draws its two actions in different inks —
`New document` in `base_content`, `Go to` in `theme::muted()`
(`render/chrome/start.rs::prepare_start_surface`) — and the muted one
wears the universal disabled costume ("why are the two buttons
differently coloured?"). DECIDED: the minimal repair — both actions in
the SAME full ink, hierarchy carried by order alone, each with its chord
beside it in the established footer-hint grammar (`↵ New document ·
⌘O Go to` — quiet chord, full-ink verb) so they read as commands, not
buttons. Verify the drawn hit rects still match (`start_rows` is the
shared geometry). Cheap to revert (one function's ink + label shaping):
land on main per the standing taste policy.

FLAGGED for a later design session, deliberately not scoped here: the
user wants per-world start screens eventually ("each theme can have a
starting screen that suits them... stylise it later"). Constraint to
carry into that session: **no theme may need its own code path** — any
per-world start expression is authored RenderCaps-style DATA through the
one start renderer (the backgrounds already prove the pattern), never a
per-world start module.

---
### 526 — a concealed thematic break still draws its trailing-whitespace nit tick (user-reported, 2026-08-29)

Typing `--- ` (a rule plus one trailing space) conceals to the rule
ornament once the caret leaves the line, but the trailing-whitespace nit
still renders — a stray underscore-looking tick beside the glyph
("the special glyph thing renders, BUT ... the _ gets rendered"). Root:
`nit_underlines` (`render/rects/underlines.rs`) has read-time
suppressions for the caret line, the bullet-glyph mask and off-screen
rows, but no membership check against the concealed rule-line set; the
image-conceal advance guard (`rects.rs`,
`IMAGE_CONCEAL_UNDERLINE_MIN_ADVANCE`) is deliberately gated on
`line_is_inline_image` — its own doc says it must never suppress the
ordinary trailing-whitespace tick — so it correctly does not fire here.

Shape: suppress nit underlines on lines in the cached rule-line set,
caret line excepted — reveal-on-cursor keeps the nit visible while the
raw `---` text shows, the same semantics as the ornament itself. The
detector (`nits.rs`) stays a pure per-line function of text; this is a
third renderer-side scope refinement alongside the two its module doc
already names. Cheap: `ornament_cache.rule_lines` is already built and
the filter loop is the seam.

Neighborhood probe (bugs cluster): a concealed TABLE source row with
trailing whitespace after the final `|` — the wash builder already skips
table-overlapping spans for the sliver reason, and nits may have the
same gap; and confirm spell squiggles are structurally unreachable on
rule lines (no word chars in `---`/`***`/`___`) rather than assuming it.
Law shape: off-caret `--- ` / `*** ` / `___ ` (and a tab-trailing
variant) draw the ornament and zero nit protos; the caret-on line draws
raw text plus the nit; non-vacuity is the current code failing red.

---
### 527 — label truth is one-directional: seeded Meta-layer chords dispatch but are advertised nowhere (user-reported, 2026-08-29)

On Linux under `keymap = "emacs"`, `M-x` opens the command palette
(`LINUX_EMACS_META_SEED`, dispatch-layer only) — the user confirmed it
fires — but no label surface shows it: the drawn menu's chord column
(`commands::menu_native_label`) prints an empty cell (native Ctrl-P is
correctly suppressed as a kept letter, the emacs slot is empty), the
palette's own chord join (`join_slots_truthful`) shows nothing, and the
generated GUIDE keys table can't see it either. "i just dont see it in
the file tab though.. it didn't show this binding." Every label surface
reads the two catalog slots; the Meta seed feeds `KeymapState` alone —
so the codebase's own label-truth rule ("never a chord the resolver
would not actually dispatch") holds in one direction only, and a chord
the resolver DOES dispatch goes unadvertised.

Shape: the seed roster becomes readable by the label layer — one owner
(`LINUX_EMACS_META_SEED` already is the data; labels need a "what seeded
chords target this action, under this convention/flavor" query off the
same table dispatch consumes), never a second hand-kept list. Applies
identically to any future seeded layer (item 528's C-x seeds inherit
whatever mechanism lands here — sequence 527 first). Law shape: a
label↔dispatch agreement sweep — for every catalog command, every chord
any surface advertises resolves to that command's action, AND every
seeded default chord is advertised on at least one surface — swept over
convention × flavor, non-vacuous today (M-x fails the second half).

---
### 528 — Linux emacs flavor: re-seed the classic chords the displaced letters orphan (user decision, 2026-08-29)

Under Linux `keymap = "emacs"`, keeping the displaced letters as their
emacs meanings leaves common commands with NO chord at all: Save
(C-s → isearch; the C-x C-s default was retired in the identity round),
New document (C-n), Finish file (C-w), Select all (C-a), Bold (C-b),
Inline code (C-e), Find & replace (C-r). DECIDED (user-confirmed
2026-08-29): seed the classic emacs chords back, **emacs flavor on Linux
only** — `C-x C-s` Save (the user's explicit ask: "bring it back for
save... on the emacs binding only"), `C-x C-f` Go to, `C-x k` Finish
file, `C-x h` Select all, and `M-%` Find & replace joining the Meta
seed. The prefix machinery, which-key panel and `c_x` override map all
survived the retirement and sit unused; the retirement itself stays
intact for the native flavor and for Mac — these are flavor-gated seeds,
the same gate as `LINUX_EMACS_META_SEED`, never a return of the static
arms. A `[keys]` override still outranks a seed, same as every default.

Sequenced AFTER 527: the new seeds must be advertised by whatever
label mechanism 527 lands, and the which-key panel should list the C-x
continuations (verify it reads the seeded map, not just overrides).

Noted, deliberately not scoped: Bold/Italic/Inline code stay
palette-only on Linux emacs (their emacs home is the C-c prefix, which
stays native Copy by the clipboard carve-out); and the whole C-c
emacs-slot layer (`follow_link` C-c C-o, `fold_section` C-c C-f,
`collapse_other_sections` C-c C-t, `insert_date` C-c .) is dead on
Linux under BOTH flavors for the same reason — the lane confirms and
documents that fact (GUIDE Linux column must not advertise them) but
any replacement chords are a separate taste session.

---
### 529 — Nishiki-teki: audition a Japanese symbol cabinet, then give each adopted mark one honest purpose (user decision, 2026-08-29)

"It's beautiful"; DECIDED: **Nishiki-teki is the first symbol face awl
should pursue. Character comes before byte count.** This is not a request
for a generic emoji font: the appeal is the Japanese-authored cabinet the
official 4.0.5 release carries — Genjikō/incense patterns, ARIB and Biblos
compatibility marks, lunar/Go/technical notation, early emoji compatibility,
and the stranger pictographs beyond Unicode. The upstream TTF is about
12.3 MB; audition the complete face rather than choosing a subset from names
alone. The downloaded upstream distribution and the font's own metadata both
declare SIL OFL 1.1 (its packaged OFL names no Reserved Font Name); record the
artifact and licence in the existing font ledger before any asset lands.
Do not claim editable design sources: none have been found yet.

The lane owns a GLYPH AUDIT before integration, rendered through awl's real
text pipeline rather than judged from a Unicode chart. Build a review gallery
across representative worlds, light/dark grounds, actual ornament/UI sizes,
and 1x/2x scale. Sample the WHOLE relevant cabinet by range, not a hand-picked
page of known beauties, and identify every sample by code point, glyph/range
name, provenance family, and whether it is standard Unicode or PUA. The
gallery is owed to the user for the taste call; report exact enrolment so a
missing range cannot make the audit vacuously pretty. DELIVERY (user ask,
2026-08-29): publish the gallery as a Claude Artifact page — the captures
embedded with their code-point/name/provenance labels — so the user can
flip through it in a browser rather than opening PNGs by hand. This does
not touch the no-web-artifacts convention: the glyphs themselves are still
rendered by awl's real pipeline via headless capture (never an HTML
re-rendering of the font); the artifact is only the viewing surface for
the taste review.

Classify the findings by an intended product role, with the present hypotheses
treated as questions the rendered evidence may overturn:

- **Thematic-break ornaments — strongest fit:** Genjikō and restrained
  geometric/lunar/technical marks can extend the existing ornament roster.
  They must remain legible at prose size, preserve the calm figure/ground
  hierarchy, and never impersonate a semantic control.
- **Per-world start dress — promising, separate application:** nominate marks
  that could serve item 525's later data-driven start-screen expression. This
  item audits and records them; it does not smuggle in a per-world renderer or
  pre-empt that design session.
- **Document fallback — standard Unicode only:** measure whether Nishiki fills
  real holes in the existing never-tofu ladder without disturbing ordinary
  Japanese text. PUA is never an ambient fallback and never silently changes
  the meaning of a user's file.
- **Semantic chrome — presumption against:** an unfamiliar mark is decoration,
  not a Save/Close/Warning icon. Admit one only if its meaning survives without
  a legend and it is clearer than AwlMarks' existing owner.
- **Insertion into documents — out by default:** inserting Nishiki PUA would
  make nominally plain text depend on awl's private font mapping. Standard
  Unicode symbols may motivate a later symbol-palette decision, not an
  unasked-for feature in this item.
- **Museum only:** vendor/service logos, historical compatibility brands,
  culturally specific religious/occult signs used as generic UI, and the
  elaborate shell/character surfaces may be fascinating gallery material but
  do not become product furniture merely because the font licence permits it.

Deliver a small, named adopted roster and a larger recorded reject/reserve
roster with reasons; "the font has thousands" is not a design system. Only
after that taste review does the lane bundle the upstream face (or an audited
subset if the selected roster makes that obviously better), route it through
one explicit symbol/ornament family rather than the general prose stack,
update `docs/fonts.md` and `assets/fonts/LICENSES.md`, and add laws for licence
enrolment, glyph presence/no-tofu, explicit-family routing, and the rendered
size/contrast of each actual use. Because adding a binary permanently grows
Git history even if reverted, the gallery/taste checkpoint precedes the asset
commit despite the standing land-easy-taste-changes policy.

---
### 530 — smart punctuation, phase one: dashes and ellipsis as display-only conceal (user decision, 2026-08-29)

DECIDED (user-confirmed 2026-08-29): render `--` as an en dash (–),
`---` as an em dash (—) and `...` as an ellipsis (…) — **display-only
conceal through the existing markdown machinery, never an as-you-type
buffer mutation.** The file keeps the literal bytes; the caret's own
line reveals raw source and moving off it renders the substitute, the
user's stated semantics ("when you're on the line, nothing changes.
when you move your cursor off, then it renders live") — the same
reveal-on-cursor contract as bold/italic conceal, with the bare-URL
ellipsis slot as the painted-substitute precedent.

Mapping is CommonMark smart punctuation (`--` en, `---` em), chosen so
the Word/HTML/PDF exporters can adopt the same standard mapping —
ONE mapping owner read by both display and export, so what the page
shows and what an export emits can never disagree. Runs of exactly two
and exactly three dashes only; four or more stay literal (ASCII
dividers keep their shape).

SCOPE — where the real work is; each region law-tested: inline prose
text runs ONLY. Never inside inline code or fenced blocks (prose about
CLI flags — `--keys`, `--release` — is this repo's own daily bread),
never in frontmatter, and never on a line that block-parses as a
thematic break: `---` alone on a line IS the section break and stays
the rule ornament, byte-identically (user raised the collision
explicitly; block-parse precedence resolves it by construction — the
inline arm only ever sees runs with other content on the line).
Conceal changes glyph advances (the `refresh_rule_conceal` tripwire):
the reveal must invalidate `row_geom`, and wrap positions legitimately
shift between raw and rendered — a law asserts the reshape actually
happens rather than a stale layout surviving the toggle.

Law shape: state × surface sweep — off-caret conceal renders the
substitute and the sidecar still carries the raw text; caret-on is
byte-identical to today; code-span/fence/frontmatter/thematic-break
exemptions each pinned with a would-be-hit fixture; export/display
mapping agreement via the shared owner. Non-vacuity: break each
exemption and watch its law go red.

Deferred, deliberately: straight → curly quotes. Pairing heuristics
(apostrophes, '90s, nested quotes) are where smart punctuation earns
its bad name — a separate taste session decides if that half ever
ships.

---
## Needs specific hardware

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst.
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm
   the rendered menu's Export action reaches its destination.
3. **Current Linux release artifacts** — launch both the tarball and AppImage
   on a real desktop; check launcher name/icon and the AppImage FUSE fallback.

## Needs release authority

1. **macOS release signing** — supply the Apple secrets required by
   `RELEASING.md` §1 before the macOS release arm can run.
