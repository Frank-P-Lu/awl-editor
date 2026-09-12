# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**3 open numbered tasks.** Ready: 646, 645 and 644.
Outstanding review of landed work and hardware checks are listed separately below.

### 646 — shorten historical commentary in tests and production source (user request, 2026-09-12)

🟢 READY — queued cleanup only; implementation is not dispatched.

Build: shorten comments in tests and live code that recount previous bugs,
implementation rounds, queue items, superseded designs or repeated narratives.
Keep concise, present-tense explanations of non-obvious behavior, ownership,
correctness constraints and each test's purpose, defect, coverage and oracle.
Preserve calibration evidence and mutation rationale needed to maintain a law.
Git retains history; do not move the removed narrative into another large document.

Scope: inventory `src/` and work in bounded batches. Starting samples are
`src/render/tests/chrome_panels.rs`, `src/render/tests/list_surfaces.rs`,
`src/capture/tests/panels.rs`, `src/app_icon/tests.rs`,
`src/render/tests/caret_transition.rs`, `src/actions/workspace_nav.rs`,
`src/app/apply.rs` and `src/main/args/flags.rs`. Keep executable code, test names,
attributes, assertions, fixtures, thresholds, enrollment and skips unchanged.
The shared refactors are already landed; measure only this cleanup so commentary
savings remain independently reproducible. There is no deletion quota.

Done/Verify: follow docs/verification.md. Prove executable Rust tokens and test
enrollment unchanged with syntax-aware comparison; preserve doc-test code,
license notices and source-audit-sensitive comments. Review the diff, relevant
source audits, formatting and documentation links; use the required integrated
native/wasm gate for source changes. Never weaken an audit to delete commentary.

Required user report: name the before/after commits, counting tool/method and exact
file scope. Give physical lines before and after, gross lines removed and added,
and net lines removed, both per batch and cumulatively. Separate comment-only,
blank and executable-code changes, and test-only versus production sections;
executable-code changes must be zero. Report mixed code/comment lines separately
rather than counting them twice. Include a plain headline, “Removed N net lines
of historical commentary,” with blank-line savings separate. Count only this
cleanup, excluding unrelated concurrent edits and refactors. Useful rationale is
retained even if savings are small; do not present source reduction as binary-size
or performance improvement.

---

### 645 — distinct bullet, divider and task vocabularies (user direction, 2026-09-12)

🟢 READY — queue only. User approved the revised Site symbol sets on 2026-09-12.
Implement alongside 644; routine fitting does not require another taste approval.

Finding: Gumtree's depth-zero bullet is U+F591, named `SNAKE-4 tail` in
`assets/fonts/AwlMarks.roster.tsv`; the nearby world comment incorrectly calls
it the head. The dash divider intentionally joins tail, two trunk pieces and
head. The task-marker Site copied those assignments, but displayed the divider
trios without syntax labels and at comparison sizes, not awl's actual sizes.
`Ornaments::of` and `pick` map dash/star/underscore explicitly; different glyph
strings alone do not establish perceptual distinction.

Build: implement the approved bullet assignments below, preserving current divider
assignments and the approved Nishiki checkbox
mapping; its explicit checks supersede the Cabinet's older hollow/solid animal,
flower, rook and other task proposals. Cross-world reuse is acceptable; within
each world bullets, divider glyphs (including joining components) and task marks
must be disjoint. Audit visual near-collisions with folds, quotes, footnote marks
and other chrome as well as exact codepoint overlap. Bullet siblings must remain
distinct at actual size and carry comparable ink weight.

The current `assert_bullet_pair_law` in `src/theme/tests/ornament.rs` REQUIRES
membership in the divider set. Replace that obsolete product rule with the new
separation law, preserving font coverage, depth, geometry and scale guarantees;
update matching contracts and exception handling. Do not simply remove the audit.
Keep Gumtree's complete joined snake as a divider; no detached joining fragment
is a suitable bullet. Include Bombora in that neighborhood review.

Approved reference: [Ornament Cabinet](https://awl-task-marker-study.s84fzrm6tq.chatgpt.site/),
version 7, source `e80a8c126adf799fb29a1f9c3557dcf5fb8259a0`.
These are Nishiki-teki 4.0.5 drawings, never system emoji fallback. Adopt the
needed glyphs into the verified bundled subset. Shared complete sets are deliberate.

| World | Bullets, outer to inner depth (Unicode) |
| --- | --- |
| Potoroo | U+1F330, U+1F331, U+1F98B |
| Tawny | U+1F330, U+1F331, U+1F98B |
| Mopoke | U+2606, U+2601, U+2604 |
| Currawong | U+2657, U+2654, U+2656 |
| Gumtree | U+1F426, U+1F98B, U+1F343 |
| Bilby | U+2606, U+2601, U+2604 |
| Saltpan | U+25B3, U+25C7, U+25CB |
| Quokka | U+1F377, U+2615, U+2694 |
| Bombora | U+2693, U+26F5, U+2638 |
| Bowerbird | U+2606, U+2601, U+2604 |
| Mulga | U+2160, U+2161, U+2162 |
| Mangrove | U+2693, U+26F5, U+2638 |
| Galah | U+2680, U+2681, U+2682 |
| Magpie | U+203B, U+2301, U+2234 |
| Brolga | U+273E, U+2742, U+273A |
| Wagtail | U+266D, U+266E, U+266F |
| Firetail | U+2604, U+2607, U+2739 |
| Cassowary | U+2607, U+2301, U+2733 |
| Paperbark | U+270E, U+2701, U+2709 |
| Kite | U+2606, U+2601, U+2604 |

The nautical U+2638 wheel needs optical enlargement beside anchor and sailboat:
the Site uses 30px versus 24px (1.25×), shared by Bombora and Mangrove. Fit this
same intent in awl without changing indentation or row height. The user accepted
the corrected monochrome rendering; earlier colour-emoji appearances were not
the approved drawings. Preserve Mulga's approved stroke trio.

Historical candidate notes live in `.orchestrator/ornament-cabinet-review.md`;
the assignments above supersede them. Compare the three-depth bullets with
syntax-labelled `---`, `***`, `___` dividers and the inline approved task rows
in actual awl. Do not revive older Cabinet divider swaps.

Verify: read docs/render.md, docs/markdown.md, docs/harness-reach.md and
docs/verification.md. Check the glyph drawings and joining behavior, ordinary and
nested bullets, divider syntax mapping and caret/selection reveal across worlds
and DPI 1/2. Add a regression-sensitive law for the detached-piece defect and
missing outcome coverage found by the neighborhood audit. Use real awl pixel
evidence and the standing vision smoke; a browser specimen is a taste aid only.

---

### 644 — add visible open/completed task markers (design approved, 2026-09-12)

🟢 READY — per-theme Nishiki picks approved by the user; enhancement, not an
established regression. Implement the approved mapping below.

Finding: a direct pulldown-cmark 0.13 probe with task lists enabled recognises
`- [ ] task` and `- [x] task completed`, but not `- [] task`. Current rendering
styles literal brackets (`src/render/spans/attrs.rs`) beside the world's list
bullet; no dedicated themed checkbox renderer was found. Checked bodies dim.
History inspected back to initial task styling (`a43b79c47`) yielded no evidence
of a removed per-world checkbox design. Two existing parser/style unit tests
passed in the prebuilt test binary; no fresh-build claim. The pixel test skipped
and capture attempts failed because no GPU adapter was available to the agent.

Build: give valid tasks one recognisable open/completed marker in preview,
replacing the decorative bullet plus literal brackets. Use one shared mechanism
with world data: quiet empty outline and unmistakable completion, no additional
accent or motion. Preserve completed-body dimming, raw source on caret/selection
reveal, indentation, hit geometry and unchanged file bytes. Keep `[]` ordinary
text; neither parser shorthand nor automatic rewriting belongs to this change.
Verify the approved treatment with real awl captures; routine fitting and
verification do not require another design approval.

Approved design: the Site's inline **Approved task pair** examples beside actual
task sentences, reconfirmed 2026-09-12. The lower “Open / Completed” legend has
inconsistent oversized boxes and a tiny tick; it is NOT the implementation
reference. Preserve the mapping below, also shown in Site version 7 referenced
in 645. User confirmed standard `- [ ] task` / `- [x] task` syntax, including the
space inside an empty marker; `- [] task` remains ordinary text.

| Pair | Worlds |
| --- | --- |
| Native: U+2610 open / U+2611 checked | Tawny, Currawong, Saltpan, Bombora, Mulga, Magpie, Brolga, Paperbark, Kite |
| Bold: U+2610 open / U+1F5F9 checked | Quokka, Mangrove, Firetail, Cassowary |
| Rounded: U+25A2 open / U+25A2 with U+2713 overlaid | Potoroo, Mopoke, Gumtree, Bilby, Bowerbird, Galah, Wagtail |

The rounded check is an explicit two-glyph composition, not a precomposed font
character. Share its fitting owner across the seven worlds. Add the five selected
codepoints to the existing Awl Marks adoption roster and regenerate from the
verified upstream font; do not substitute fallback glyphs. This approval covers
task-marker pairs only, not the rejected snake bullet or unresolved divider
choices in 645. Compare beside existing symbols during verification.

Upstream inspection (2026-09-12): downloaded the author's full 4.0.5 font and
verified its SHA-256 equals the pinned source in the adoption roster. The cmap
contains U+2610/U+2611 (native empty/checked boxes), U+1F5F9 (boxed bold check),
U+25A2 (rounded square), U+2713/U+2714 and U+1F5F8 (check weights). Embedded and
archive OFL grants are present. Direct font specimens at 22px and larger favor
the native U+2610/U+2611 pair as the starting point; U+2610/U+1F5F9 is the bolder
alternative. U+25A2 plus U+2713 is a promising rounded COMPOSITION, not an existing
single checked glyph. The bold-square composition crowds its tick; flower/star
fill pairs blur task state with existing ornaments. These are taste findings,
not awl render verification. Prefer a small expanded Awl Marks subset over
shipping the full font; fit any selected pair in the real renderer before adoption.

Done/Verify: read docs/markdown.md, docs/render.md, docs/harness-reach.md and
docs/verification.md. Cover valid/invalid syntax, both task states, caret/selection,
nested/wrapped tasks and ordinary lists across worlds and DPI 1/2. Require pixel
presence, state distinction and absence of duplicate bullets, plus sidecar/source
fidelity. Add regression-sensitive laws and the standing neighborhood audit and
vision smoke. Rendering evidence remains outstanding; use an available GPU and
the single integrated native/wasm gate under the verification policy.

## Outstanding review of landed work

These are follow-ups, not additional unimplemented build tasks. Completed work and
past verification reports remain in `git log -p -- .orchestrator/queue.md`.

- **582 — projected Wagtail tunnel landed; blocked on live motion and user taste.**
  The replacement renders one bounded projected mesh with depth-dependent bend,
  full 42–45%-class folded sections, independent roll and curved fixed-theta rails.
  An independent judge accepted the reference match mechanically after checking
  16 native views across pose, viewport and DPI. The focused 31-law render suite,
  compiling concentric/page-mask mutations, WebGL2 validation and release frame
  benchmark are clean; the measured median change was +0.007 ms at 2400×1600 @2x.
  Remaining judgment is live dwell/transit/settle comfort, pause/focus freeze,
  Reduce Motion in the bundled app, and whether the intentionally quieter far core
  has the right density. Keep the repaired projection is recommended; the merge is
  a single reversible unit if live review rejects it. The ignored scratch gallery
  used by the judge remains review evidence rather than a product input.
- **588 — mechanically complete; blocked on user taste.** The current plain
  `•◦▪` fallback passed a fresh 20-theme release gallery and focused Metal laws:
  all three Brolga depths are present, distinct, aligned, contained, unclipped and
  legible (sampled contrast 13.46:1, 5.23:1, 13.46:1). Evidence is in the ignored
  `gallery/landed-visual-review/588-bullets/`. Keeping it is recommended. A
  different glyph or wider box is a separate mechanism decision only the user
  can authorize; mechanical checks do not supply that taste decision.
- **553 — folder-search highlight review.** Real-pixel match-highlight legibility
  remains unverified. Retain the known boundaries: results use summon-time disk
  contents, grouping differs from lens headers, and CRLF matches can retain a
  cosmetic trailing carriage return. Review with 639/640 rather than treating
  the original implementation as unfinished.
- **561 / 618 — mechanically complete; blocked on user taste.** The requested
  reduction is merged (`760f4f43`, merge `b3e8d2aa`) and remeasured at 15.02%.
  Fresh release and live-headless-App captures show every Gumtree ornament present
  and legible; measured ink heights are snake 3.41em, fish 5.84em and snail 3.87em.
  Their distinct shapes are not independently ink-equalized and the existing law
  deliberately measures the shared dash scale. Accepting the fish-forward character
  is recommended; a per-glyph scale is a new mechanism requiring the user's taste
  decision. True-window live proportion judgment remains part of that decision.

Kite's unresolved appearance and live motion review belong to **582** above;
there is no separate 564 build item. Its review must include convergence near
page edges at common window sizes as well as several-minute motion comfort.
For current accessibility acceptance and deferred work, use `ACCESSIBILITY.md`;
the resolved 584/626 investigation does not require another confirmation sitting.

## Latest recorded verification

The latest recorded native/wasm baseline is **`0c49d174`** (647–649 integrated
refactor candidate), already on main:

```text
native-gate-receipt commit=0c49d1748852bcc36595ea5ff22d7e7438d0139a health=pass:284s
  conventions=mac,linux scope=all-targets menubar=full:on unit_tests=5183
  unit_shards=6 integration_targets=18
web-smoke: OK
```

Subsequent queue/policy-only commits use diff/link checks under `docs/verification.md`.
Older receipts, resolved CI investigations and completed train summaries are in Git
history. Local hardware receipts do not establish hosted-GPU or live-journey results;
check remote status before a future push rather than inheriting old push warnings.

## Needs specific hardware

🔴 BLOCKED on the orchestration host (audited 2026-09-11). It is Apple-silicon
macOS with Metal: there is no Linux graphical/AT-SPI/D-Bus session, Orca, X11 or
Wayland compositor, Linux awl executable, AppImage/tarball, or FUSE runtime.
Headless capture cannot substitute for any missing live door. Honor the current
scope and release policy in `ACCESSIBILITY.md` and `RELEASING.md`.

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst (post-v1 per
   `ACCESSIBILITY.md`). **Blocked by external Linux UI hardware/session and
   intrinsic user listening:** needs Orca, an active AT-SPI2/D-Bus bus, audio,
   current native Linux awl, and a person; current mechanical CI cannot close it.
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm
   the rendered menu's Export action reaches its destination. **Blocked by an
   external Linux compositor and real pointer door:** needs current native awl on
   X11 or Wayland, then a genuine File → Export click and destination/file check.
3. **Current Linux release artifacts** — launch both the tarball and AppImage
   on a real desktop; check launcher name/icon and the AppImage FUSE fallback.
   **Blocked twice:** no current artifacts are present, and this arm64 Mac cannot
   run or judge x86_64 Linux launcher/FUSE behavior. A dry-run/download can supply
   artifacts without release authority; acceptance still needs an x86_64 Linux
   desktop, supported DE, real Vulkan driver, runtime libraries, and FUSE plus
   `--appimage-extract-and-run` fallback checks. A new tag remains unauthorized.

## Release authority

Signing/notarisation setup is complete; it is not an open setup task. Every new
tag/release still requires the user's explicit instruction per `RELEASING.md`.
