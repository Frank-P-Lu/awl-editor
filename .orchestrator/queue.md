# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**14 open numbered tasks:** 582 is active; 668 awaits a design decision;
651, 657–659, 661–666, 669 and 672 are queued. The user confirmed on
2026-09-27 that only Kite background work is running. Previous worker claims
are released; their committed work and worktrees remain preserved.

**Integration pending:** `codex/merge-train-657-659-661` at `f0f68dfc`
contains the earlier Files, panels, search budgets, paste, release, IME, input,
caret and punctuation work. It is not on main. Its `05916301` candidate has a
recorded native/wasm pass; reconcile subsequent main changes and documentation,
inspect the combined result and gate the new frozen candidate before landing.
Resume from the train's integrated changes and preserve its conflict resolutions.
The source branches below are recovery/reference points.

**Integration review (2026-10-01):** 🟡 IN PROGRESS — local-integration (codex),
branch `codex/20261001-cloud-ui-integration`. The user authorized review,
reconciliation, main publication and subsequent safe worktree cleanup. Review the
preserved train and dirty edits, retain the newer main Find/Replace design, and
combine the cloud UI fixes. Unchosen prototypes, private data, tags and deployments
remain outside this publication candidate. Freeze and gate before landing.

**Design approved:** the shared-control candidate aligns Match case and Replace
at the field edge, uses larger category-sized checkbox/disclosure marks, and
shares cached tight ink placement across UI, document task/list and fold marks.
The final native release previews were approved. The full native gate exposed
integration defects in rail room/color, compact workspace/Files height, fractional
caption origins and two structural test audits. Bounded repairs preserve ordinary
layouts and all appearance floors; their affected suites pass across both menu-bar
arms, with independent regression mutations rejected. Freeze the repaired combined
candidate for full native, wasm and debug/release parity before main publication;
then verify exact-SHA hosted CI and safely retire only stopped, preserved worktrees.

**Test pipeline layers (2026-10-01):** 🟡 IN PROGRESS — test-pipeline (codex),
branch `codex/20261001-test-pipeline-layers`. Implement the user-approved fast
local checks, one complete frozen-candidate gate, and scheduled overnight plus
pre-release journeys and built-artifact launch validation. Remove duplicate Linux
code health while retaining all currently required coverage. Do not weaken caret
pixel laws, expand signing access, tag, deploy or publish a release.

**Release constraint:** every public installable app download must be strictly
under 50,000,000 bytes, measured after compression. This covers macOS DMG/app zip
and Linux AppImage/tarball. Browser deployment and source archives are separate.
Keep real Japanese bold; return any size tradeoff to the user.

### 582 — refine Kite background depth and finish live review

🟡 IN PROGRESS — existing Kite session (codex), branch `main`; chat
“Find the right way to run awl”. This is the only active product task.

Latest depth fade is committed at `4f9fd3fb`, following `0d5388d6` and
`5d3d4249`. Targeted Metal checks, mutation evidence and five captures were
reported in that session. Full native/wasm verification of the latest change
remains with that owner; the first isolated gate stopped on a worktree-cleanup
self-test, with its build-cache configuration under investigation.

Finish the current verification and present real awl captures. User review still
covers density near page edges and the far core, several-minute motion comfort,
dwell/transit/settle, pause/focus freeze and Reduce Motion in the bundled app.
Preserve the projected tunnel geometry and shared renderer ownership. The earlier
582 projection evidence remains in Git; there is no separate 564 task.

---

### 657 — finish the approved Files composition

⚪ QUEUED — review and integration. Source `codex/657-files-affordances`
ends at `fd1ec8df`; its corrections are in the pending integration train.
The prior main landing `ef344429` did not satisfy the live composition review.

Review the corrected folder heading, Change folder control, bounded labelled
search field and visible New document footer against
[approved study 05](https://awl-files-reconsidered.s84fzrm6tq.chatgpt.site/). Keep Files/Recent,
breadcrumbs/Up and choices coherent; preserve opaque chrome, distinct focus and
selection, content-free listings and existing file I/O boundaries.

Verify browsing, search, empty folders and narrow layouts in a seeded native
release session with explicit root/config. Find the actual controls in fresh
captures; accessibility presence alone does not establish visual fidelity.

---

### 658 — finish Files query selection and input review

⚪ QUEUED — review and integration. Source
`codex/658-files-select-all-physical` ends at `8f0adee7`; key-route laws and
pointer-focus repair are in the pending train. The user's physical Cmd-A failure
on 2026-09-24 remains the defect evidence; do not infer a complete repair from
a passing direct-action or menu test.

Recheck physical Cmd-A then Backspace in the Files query, alongside Edit ▸ Select
all, pointer editing, Tab/Enter, matches/no matches and accessible result opening.
Selection must belong to the focused query and never edit the document behind it.
Preserve the already-confirmed Change folder cancellation behavior. Use fresh
state/captures in a disposable release session; coordinate automated-input evidence
with 664 and Japanese composition with 665 without conflating the defects.

---

### 659 — finish Settings, Themes and Find/Replace composition review

⚪ QUEUED — review and integration. Source `codex/659-panel-composition`
ends at `d1b1988e`; composition and focus repairs are in the pending train.
Prior main commits `2dcdb1779`, `210c60e36`, `be399d5f3` and `0e5e5d132`
establish the earlier implementation, not acceptance of the final appearance.

Review Settings' recognizable search field, bounded label/value columns and
separate category selection, control selection and keyboard focus. Review Themes
as one coherent panel with heading, effective Switch/Cancel actions and a clear
current-versus-previewed choice. Review Find/Replace field, navigation/count,
Match case and replacement actions as readable groups without redundant labels.
Use approved study 05 (657), preserving theme identities and quieter opaque chrome.

Verify forward/reverse focus, matching/no-match search, nested pickers, cancellation
and immediate settings in an identified native release build. Include narrow
geometry, Replace mode and opening/preview worlds. Keep theme preview geometry
stable and restore the invoking Settings control on Escape.

---

### 651 — land bounded Search in folder work

⚪ QUEUED — review and integration. Source `codex/651-browser-fail-closed`
ends at `266bfa90`; bounded native reads, browser refusal and incomplete-coverage
reporting are in the pending train. Original bounded-read work was `7764e7ea`.

The user chose browser refusal on 2026-09-24: synchronous localStorage cannot bound
a cross-tab replacement before materializing it. Preserve an explicit notice and
avoid loading a partial browser corpus or adding transactional storage this round.

Verify attempts and bytes at the FS seam, including exact caps/cap+1, rejected
files, growing/unknown sizes, binary data and errors. Preserve admitted-file search
correctness and honest incomplete coverage. Retain the mutation proof against the
old post-read-only checks and run the integrated native/wasm gates.

---

### 661 — finish external plain-text browser paste

⚪ QUEUED — review and integration. Source `codex/661-browser-paste`
ends at `0aaae8a9`; the pending train includes trusted paste, bulk insertion,
composition/keybinding-capture protection and focus repairs. The board's old
“in repair” claim predates those commits; review the repaired result.

Verify genuine external paste in Chrome, Safari and Firefox on a static release
build: document, summoned fields and Table Dimensions; focus changes, replacement,
multiline/Unicode, single undo, repeated/empty/non-text paste, custom yank bindings
and palette actions. Prevent duplicate insertion and background-document edits;
unsupported external attempts must not silently paste stale internal text.

Retain work-count and release evidence for one field-owned bulk insertion and
recompute. The seven-TextField bulk measurement does not cover Table Dimensions
or prove OS clipboard delivery. Record actual browser versions and missing live
coverage; update WEB.md only to the behavior established. Styled/image paste is
outside scope. Reference:
[Clipboard events specification](https://www.w3.org/TR/clipboard-apis/#clipboard-event-paste).

---

### 662 — finish signed, notarized macOS release artifacts

⚪ QUEUED — review, integration and hosted proof. Source
`codex/662-signed-macos` ends at `b45e312d`; signing/notarization and dry-run
payload preparation are in the pending train. The user confirmed Apple setup is
complete; verify its operation without printing secrets or reopening setup by assumption.

Run a nonpublishing credentialed rehearsal. Inspect both architectures, bundle
identity, Developer ID signature, notarization/staple, Gatekeeper result, mounted
DMG contents, versioned names, checksums and actual compressed size. Missing or
partial credentials must fail before publication. The branch makes the DMG public
and the app zip diagnostic-only; check that final payload layout and document it.

Hosted rehearsal, final DMG size and a real Mac launch remain owed. Preserve the
under-50,000,000-byte public-download limit and return a measured packaging tradeoff
if needed. Do not tag or publish here; 663 owns the release cut.

---

### 663 — release one version with Linux and macOS downloads

⚪ QUEUED — depends on 662 and a frozen integrated candidate. Choose the version
from that candidate. Publish Linux x86_64 AppImage/tarball and signed, notarized
macOS DMG together; keep the browser demo on its separate deployment path.

Complete RELEASING.md's exact-commit checklist: native/wasm gates, outcome audit,
profile parity, nonpublishing dry run, final artifact size/checksums, release notes,
current CI and hardware checks below. Real Japanese bold is on main (`2dfb6a9e`);
a compressed local binary is not evidence of packaged-download size.

Inspect the final public file list and instructions, then obtain the user's
explicit go to tag and publish. Verify both platform downloads and checksums after
publication. This queued request does not authorize a tag or site redeployment.

---

### 664 — finish live macOS keyboard-ingress diagnosis

⚪ QUEUED — review and investigation. Source `codex/664-live-key-ingress`
ends at `d9cf10c6`; delivery/focused-field diagnostics are in the pending train.
Prior injected-key failures and stale captures did not establish where input was
lost; some earlier logs did contain successful keymap and App action receipts.

In a disposable named release session, record each injected key, timing, focused
app/window/control, native delivery, keymap action, App action and resulting field.
Compare Cmd-A, deletion and typing with physical input and direct App actions.
Identify the first divergent boundary and prove repeatability before claiming a
repair. If the fault belongs to automation or macOS permissions, provide a minimal
reproducer and use a supported driver for missing evidence. Keep private text out
of logs; do not use this task to delay or explain away 658's physical defect.

---

### 665 — finish Japanese IME ownership in Files and summoned fields

⚪ QUEUED — review, integration and physical input check. Source
`codex/665-ime-fields` ends at `45c38256`; focused preedit/commit, candidate
ownership and restart after cancelled preedit are in the pending train.

The user reported Japanese input failing to reach Files search after Cmd-O.
Verify Japanese and direct Latin input, opening with IME already active, pointer
focus, conversion, Enter and Escape in a seeded native release session. Record
focused surface, preedit/commit delivery, candidate geometry, query and unchanged
background-document bytes. Cover neighboring text fields and non-text surfaces.

Retain one focused-surface owner and regression laws at the App/IME seam. Preserve
selection, undo, cancellation, native menus and ordinary Cmd-O. Report physical
Japanese typing evidence separately from replay and unreliable injected keys (664).

---

### 666 — review the shorter Paperbark caret

⚪ QUEUED — review and integration. Source `codex/666-paperbark-caret`
ends at `f548f9b8`; shared full-ink block-padding reduction (`c907f9e2`,
train equivalent `6b7efadb`) is in the pending train, not main.

Review matched before/after native release captures against the user's oversized
caret report. Check actual ink and line bounds across Paperbark and nearby serif
worlds, zoom, 1x/2x DPI, lowercase/ascenders/descenders, spaces, empty lines and
headings. Preserve stable row height, glyph visibility and shared metrics ownership.
Retain the rendered-pixel bound and mutation evidence; finish visual smoke and
ask the user to judge live proportions where the remaining choice is taste.

---

### 668 — choose visible Japanese Markdown emphasis

🔵 OWED — prototype `9f28df85` on `codex/668-japanese-emphasis-study`;
no default chosen. This branch includes pending integration work and an experiment;
it is not a releasable default. Regular upright Japanese currently hides *emphasis*.

Present matched native captures of dots above kana/kanji, a real alternate face or
weight, quiet underline/bousen and optional ink treatment. Keep synthetic slant and
invisible Regular as rejected baselines. Existing six-world/stress captures favor
restrained dots: visible in one-bit Wagtail and less link-like than underline.

Decision: should restrained dots become the default for Japanese emphasis?
Include punctuation exclusions, mixed Japanese/Latin, wrapping, headings, `*` versus
`**` roles, zoom/DPI, collision and caret-reveal evidence, and bundle-size/implementation
cost. Preserve real Japanese bold and the download cap. The user must choose before
shipping; private content and HTML mockups are not prototype evidence.

---

### 669 — land CJK punctuation fixes and resolve the remaining caret taste choice

⚪ QUEUED — integration, with a separate taste decision still owed. Source
`codex/669-cjk-punctuation` at `70eb217c` contains font routing (`46be9e14`)
and document-evidence invalidation fixes. They are in train `05916301`, whose
native/wasm pass is recorded, but are absent from main.

Reconcile and land through the combined candidate. Preserve source, insertion
position, hit testing, selection, IME geometry and stable caret motion. Retain
coverage for opening/closing punctuation at line starts, within lines and wraps
across Japanese faces, zoom/DPI and caret modes.

The remaining 15 px Paperbark kana-to-caret gap was measured as a font-sidebearing
and caret-width taste choice. Keep the current ink-hugging fill until the user
judges matched captures of including leading punctuation-cell space versus the
current treatment. Do not conflate that choice with the repaired font/cache defects.

---

### 672 — keep the selected document’s × accented on hover

⚪ QUEUED — user decision, 2026-09-27. The selected document's close mark should
receive the theme accent on hover too. Keep the selected plate present; this is
a change to its × feedback, not a request to show the plate only on hover.

`render/chrome/gutter_stack.rs::close_mark_hover_ink` already asks for
`theme::accent_ink`, but the selected plate's contrast substitution can replace
the accent with ordinary ink. Reproduce the selected-row case before choosing a
fix; the reported screenshot alone does not prove that fallback was taken.
Use one shared treatment for the lone document and working-set stack. Keep the
mark readable and visibly responsive without shifting the filename or plate.

Verify rest/hover on selected and unselected rows across the theme roster,
including monochrome Wagtail and low-contrast plates, with native pixel evidence
and the required visual smoke. Preserve click-to-close and file contents. Return
any conflict between the authored accent and legibility as a concrete taste choice.

---

## Outstanding review of landed work

These are user judgments, not active implementation claims. Historical evidence
remains in Git; Kite review is part of active item 582 above.

- **588 — list-marker taste:** `•◦▪` passed the 20-world gallery and focused Metal
  laws; the Brolga depths are distinct, aligned, unclipped and legible. Keep the
  current glyphs unless the user chooses a different treatment. Evidence:
  `gallery/landed-visual-review/588-bullets/` (ignored).
- **561 / 618 — Gumtree proportions:** reduction `760f4f43`, merged at `b3e8d2aa`,
  measured 15.02%; snake/fish/snail ink heights are 3.41/5.84/3.87em. Native and
  headless-App captures establish presence and legibility. Live proportion judgment
  remains owed; per-glyph scaling requires a new user decision.

Use ACCESSIBILITY.md for current acceptance and deferred work; resolved 584/626
needs no further confirmation sitting.

## Latest recorded verification

- **Main baseline `63d96042040397cddb51993cc9d3c34400c22bb0`:** the dev-launch
  session recorded successful `scripts/native-gate.sh` and `scripts/web-smoke.sh`
  runs in its isolated checkout. The subsequent `e5d30662` edit was queue-only.
- **Pending integration candidate `05916301`:** native gate and browser smoke
  passes recorded by `831a8e3e`. That candidate is not main; its receipt does not
  validate a future merge with current main.
- **Kite `4f9fd3fb`:** full verification remains with active item 582.

This queue cleanup uses diff, heading, reference and status checks under
`docs/verification.md`; it does not relabel a prior receipt. Earlier detailed
receipts remain in Git. Local Metal/headless checks do not prove hosted GPU,
physical-input journeys or Linux desktop behavior.

## Needs specific hardware

🔴 BLOCKED on this Apple-silicon macOS host. Headless captures cannot discharge:

1. **Linux AT-SPI/Orca:** document reading, caret/selection, overlays and editing
   with a real AT-SPI2/D-Bus session, audio and a person.
2. **Linux drawn-menu Export:** a genuine click in a real window/compositor.
3. **Linux release launch:** current x86_64 tarball and AppImage, launcher metadata,
   FUSE and `--appimage-extract-and-run` on a real Linux desktop.

Use ACCESSIBILITY.md and RELEASING.md for scope. No new tag is authorized.

## Release authority

Signing/notarisation setup is complete; it is not an open setup task. Every new
tag/release still requires the user's explicit instruction per `RELEASING.md`.
