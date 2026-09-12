# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**8 open numbered tasks.** Ready fixes: 654, 653, 650, 651, 646, 645 and 644.
Read-only follow-up investigation: 652.
Outstanding review of landed work and hardware checks are listed separately below.

### 654 — remove retired floating-card shadow machinery (user request, 2026-09-12)

🟢 READY TO MERGE — branch `codex/654-remove-dead-shadow-pipelines`, commit
`4e8668a0` (based on `c4f9496a0`).

Finding: the July 22 visual fix stopped emitting drop-shadow geometry but deliberately
left five complete `SelectionPipeline` instances wired through construction, theme
sync, per-frame preparation, painter order, benchmarks and tests. They can produce no
pixels: every preparation supplies an empty rectangle slice and `draw` returns at zero
instances. The retained objects nevertheless create ten explicit GPU buffers (13,200
requested bytes in total), five bind groups and the associated live-app pipeline/layout
objects, plus five empty 80-byte uniform writes during ordinary chrome preparation.

Build complete: delete `panel_shadow`, `float_shadow`, `hud_shadow`, `wk_shadow` and
`menu_drop_shadow`; remove their constructors, tint writes, empty prepares, draw calls
and structural tests; remove `float_shadow_srgba`; update live comments and theme/render
contracts from shadow/border/card trios to border/card pairs. Preserve `FloatElevation`
because it still selects rimmed versus flat cards. Preserve the two pixel laws proving
dark worlds have no bright slab and light cards remain visibly elevated by their rim.
The branch changes 24 files, removing 186 lines and adding 48 (138 net removed), with
no remaining source or contract-doc reference to the retired identifiers.

Verification: formatting and diff checks pass; `cargo check --tests` passes; the native
source ownership test `float_surface_primitive_has_no_bypass_among_the_unified_family`
passes; `cargo check --target wasm32-unknown-unknown` passes with the base branch's
existing Wasm warnings. Both retained appearance tests compile but skip because the
worktree environment exposes no GPU, so the integrated candidate still needs the real
GPU native gate and `scripts/web-smoke.sh`. `scripts/preflight.sh` currently stops on
the inherited `clippy::obfuscated_if_else` at `src/overlay/build.rs:353`, outside this
branch's diff; resolve that integrated-base blocker before treating the gate as green.

---

### 653 — remove the checked-in legacy web-editor bundle (user request, 2026-09-12)

🟡 IN PROGRESS — `/root/legacy_web_bundle` (Codex), branch
`codex/653-remove-legacy-editor-bundle`.

Finding: `site/editor/` is generated Trunk/wasm-bindgen output, not authored product
source. The only tracked files are a 137,136-byte hashed JavaScript glue file and its
generated `index.html`; that page references an `_bg.wasm` file which is not tracked,
so the checked-in bundle is incomplete. The current `deploy-web.yml` already ignores
this directory: it builds a fresh release bundle into `dist/`, copies `site/` to a
scratch deploy directory, removes the copied `editor/`, and installs the fresh build
there. The workflow itself labels the checked-in files legacy. Git history preserves
the old artifact if it is ever needed.

Build/Scope: remove the tracked `site/editor/` files and prevent generated editor
bundles from being recommitted. Preserve `/editor/` in the deployed website by keeping
one fresh Trunk build as the source of that route. Update `site/README.md`,
`.github/workflows/deploy-web.yml`, `scripts/with-remap.sh`, ignore rules and any other
live references so they describe the current scratch-assembly path rather than a
committed deployable bundle. Preserve a documented local way to preview the landing
page and freshly built editor together without restoring generated output to Git; reuse
the existing assembly mechanism rather than introducing a parallel build pipeline.
Do not change editor behavior, analytics coverage, hosting, release policy or the
generated bundle's public URL. The separate code-growth dashboard is out of scope.

Done/Verify: follow `docs/verification.md`. Require `git ls-files site/editor` to be
empty and add a structural audit that fails if a generated editor bundle is tracked
there again. Exercise the deploy assembly against a fresh
`scripts/with-remap.sh trunk build --release --public-url /editor/`; require the staged
`editor/index.html`, its hashed JavaScript and wasm references to exist, remain rooted
at `/editor/`, carry the analytics beacon, and contain no builder-home path. Run the
relevant site/link checks and `scripts/web-smoke.sh`. Inspect the deletion and updated
documentation/workflow diff; this item does not authorize a deploy, push, tag or
release.

---

### 650 — Files listings use names, never file contents (user-approved scope, 2026-09-12)

🟠 INTEGRATED — implementation + targeted/mutation evidence landed on local
`main` at merge `1bb70883`; seeded live release smoke and the combined gate remain.

Finding: a three-second sample of the frozen live macOS app put every sampled main-thread
stack in `App::apply` → `FilesOverlayBuilder::attach_level` →
`overlay::files::unsupported_level_files` → `openable::classify` → `std::fs::read`.
The open descriptor named a 778,658,385-byte MP4 in the user's Documents folder.
No Files list had appeared; the app beachballed. This is a synchronous content read
before presentation, not a GPU failure or observed process crash. Keep private paths
and the raw process sample out of tracked artifacts.

Origin: `22872ed0` (2026-09-11, “fix: preserve Files unsupported-only outcome”)
added whole-corpus content classification to Files. `26e7758e` seven minutes later
restricted that to the displayed directory, retaining full reads per file.
`b4c3f92f` (2026-09-12) added selected deep-result classification, with a 256 KiB
metadata threshold. Browse's corresponding whole-file listing reads date to
`9d888a65f` (2026-07-25). These origins are verified from diffs; historical binaries
were not replayed. The live sample confirms the current directory-level path.

Build/Scope: make Files and Browse listing/filtering use filenames, extensions and
entry kinds only. Use one shared, pure presentation-hint owner. Unknown, extensionless
and misleading extensions may be offered and then refused on open; that tradeoff is
user-approved. Remove preaccept content reads from directory summon/relevel AND
selected deep results, including query, wheel, pointer, accessibility and replay doors.
Do not claim “text · ready” or a definitive content verdict from a filename. Preserve
folders, hidden-file policy, Text/All semantics, recent/open identities and selection.
Keep full content validation at the actual open gate before any document/path mutation.
This item does not introduce asynchronous folder scanning or claim immunity to slow
metadata/directory reads; that wider work is investigated in 652.

Done/Verify: follow docs/verification.md. Extend the existing CountingFs laws to require
zero content reads before acceptance across Files summon/relevel, Browse Text/All,
and deep selection input doors. Sweep ordinary text, large video/archive, empty,
extensionless, unknown-extension and binary-under-.md cases. Assert actual listing
outcomes as well as zero reads. Acceptance must still refuse binary content without
changing the active document, root or disk bytes. Update the tests that currently
REQUIRE preaccept reads/readiness, including `app/tests/files.rs` and the replay law
in `main/tests/capture_scenarios.rs`; preserve their input-door coverage. Prove the
zero-read regression law fails when a real old classify call is restored. Update
`openable.rs`/`file_visibility.rs` documentation. Use seeded roots/configs for any
capture and a live release smoke for the reported route; one integrated native/wasm
gate after the candidate is committed and frozen.

---

### 651 — Search in folder budgets must bound reading work (audit finding, 2026-09-12)

🟡 IN PROGRESS — `/root/search_folder_budgets` (Codex), branch
`codex/651-search-folder-budgets`. Separate from 650 and based on its landed owner.

Finding: `App::gather_overlay_inputs(OpenSearchFolder)` calls
`search_folder::load_corpus` synchronously with whole-file `read_to_string` before
showing the picker. `load_corpus` checks `max_file_bytes` only AFTER reading, and
counts only retained files/bytes. Oversized or failed reads consume neither budget;
the total retained-byte limit can also be exceeded by the last accepted file.
A standalone probe compiled the unchanged SearchBudget/load_corpus source: with
max_files=3, max_file_bytes=10 and max_total_bytes=25, 1,000 oversized candidates
caused 1,000 callbacks returning 100,000 bytes and zero retained files; 1,000 failed
reads also caused 1,000 callbacks. Ten-byte inputs retained 30 bytes against 25.
This demonstrates the budget defect, not a reproduced live UI hang. Origin:
`cca5b34e` introduced the loader on 2026-09-02; `054cdc90` wired the live feature.
Existing tests primarily assert the retained corpus, not attempted I/O.

Build/Scope: give attempted reads and actual bytes explicit enforceable budgets at
the read owner, including oversized, invalid-text, failed and changing-size files.
Avoid reading a complete giant file merely to reject its size. Preserve search
correctness for admitted files and make incomplete coverage honest. A size/byte cap
is not a time guarantee: assess background/cancelable loading with 652 before claiming
responsiveness on slow storage. Coordinate shared loading design sequentially, not
as overlapping worker changes. Do not turn this into the names-only policy from 650:
full-text search legitimately needs contents.

Done/Verify: read docs/verification.md. Count attempts and bytes at the actual FS seam;
probe exact limits, cap+1, all-rejected corpora, unknown/growing sizes, binary data and
read errors. Mutation-prove that the old post-read-only limits fail. Check live and
replay consumers and incomplete-result communication. Use one integrated native/wasm
gate for a final code candidate, never describe retained-corpus tests as I/O bounds.

---

### 652 — investigate neighboring synchronous I/O stalls (user request, 2026-09-12)

✅ INVESTIGATION COMPLETE — measured on `32cad5c6`, independently audited on
`76d3a151`; no application changes. Implementation/design remains out of scope.
The initial source audit is complete. The items below are source-confirmed blocking
paths, not independently reproduced freezes. Measure with controlled fixtures and
blocked/slow readers before choosing changes; no probing by reading large private files.

- Folder indexing: `gather_goto_inputs` → `rescan_file_index` → `index::build_index`
  runs before Files, projects, Asset Cleaner and folder-search actions. Recursive
  non-git walks and synchronous git commands remain after 650; the git path also
  walks for .env files. Symlink directories and named junk directories are skipped.
  Measure large and slow trees; background loading needs stale-result suppression
  when root/query changes, bounded workers and responsive cancellation.
- History: `gather_overlay_inputs` → `history::timeline_rows` loads every snapshot
  before composing rows. Loose-file `load` rereads/parses the full history log for
  each version; Git history runs synchronous `git log` and `git show` per version,
  then computes row diffs. Inspect batching/lazy previews before adding machinery.
  The timeline content-loading shape dates to `9b439f406` (2026-07-01), with later
  module extraction; do not blame the September Files change for this older path.
- Asset Cleaner: `assets::scan` reads every non-hidden Markdown candidate in full
  on summon, then stats asset candidates. The scan originated in `9cfb65294`
  (2026-07-09). Failed/incomplete reference scans must never become authoritative
  evidence that a used asset is orphaned. The preview path
  `render/chrome/asset_preview.rs` → `render/image_cache.rs::ensure` synchronously
  opens/decodes on a cache miss; verify compressed size versus decoded pixel bounds
  and stale-mtime behavior before proposing background decode.
- External changes: `external::Seen::at` unconditionally reads/hashes the file at
  focus/persistence/buffer-identity boundaries. This shape dates to `1127673d`
  (2026-08-03). It detects same-size/same-mtime changes and MUST NOT be replaced by
  a stat-only shortcut. Measure slow-file behavior and preserve save/conflict/data
  guarantees if moving observation off the UI thread. Include explicit file open
  and image-drop full reads in the neighboring inventory.

Done/Verify: publish a small trigger × owner × blocking operation × existing guard
matrix with measured cases, reproduction status and prioritized bounded proposals.
For delayed work, prove Escape/navigation remains serviceable and late results cannot
change the wrong root/document or authorize writes/deletions. Read docs/platform.md,
docs/render.md and docs/harness-reach.md for the relevant surface before promising
verification. No application source edits are authorized by this investigation item.

---

### 646 — shorten historical commentary, starting with `src/render.rs` (user request, 2026-09-12)

🟠 INTEGRATED — the original production/test batches landed through `76d3a151`;
the `src/render.rs` batch landed at merge `c4f9496a` and its source-size guard at
`83323622`. The combined native/web gate remains.

Build: shorten comments in tests and live code that recount previous bugs,
implementation rounds, queue items, superseded designs or repeated narratives.
Keep concise, present-tense explanations of non-obvious behavior, ownership,
correctness constraints and each test's purpose, defect, coverage and oracle.
Preserve calibration evidence and mutation rationale needed to maintain a law.
Git retains history; do not move the removed narrative into another large document.

Scope: the first bounded batch is specifically `src/render.rs`, before the wider
`src/` inventory. It is currently 3,198 physical lines: cloc classifies 1,432 as
code, 1,586 as comments and 180 as blank. Its 1,061-line `TextPipeline` declaration
alone contains about 726 comment lines around 334 declaration/nonblank code lines.
Shorten historical narratives, repeated per-field mechanism tours, retired-option
comparisons and commentary that merely restates adjacent types or modules. Preserve
concise present-tense ownership, ordering, cache invalidation, units, safety and
cross-platform constraints; several long comments encode real rendering laws, so the
raw 1,586-comment count is not a deletion target. Keep every executable Rust token,
attribute and public API unchanged in this first batch. Measure `src/render.rs`
independently before expanding to starting samples such as
`src/render/tests/chrome_panels.rs`, `src/render/tests/list_surfaces.rs`,
`src/capture/tests/panels.rs`, `src/app_icon/tests.rs`,
`src/render/tests/caret_transition.rs`, `src/actions/workspace_nav.rs`,
`src/app/apply.rs` and `src/main/args/flags.rs`. The shared refactors are already
landed; measure only this cleanup so commentary savings remain independently
reproducible. There is no deletion quota.

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

🟠 INTEGRATED — landed with 644 at merge `30c4d1a4`; combined gate remains.
User approved the revised Site symbol sets on 2026-09-12.
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

🟠 INTEGRATED — landed with 645 at merge `30c4d1a4`; combined gate remains.
Per-theme Nishiki picks were approved by the user; enhancement, not an
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
