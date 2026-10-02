# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**14 open numbered tasks:** 582, 666 and 669 await user review;
657–659 and 661–665 are queued for the remaining journeys or release proof.
674's newest top-clearance candidate is ready; 675 owns the active Files
parent-listing refresh repair; 676 is accepted and ready for integration.
No new product candidate is pushed or integrated.
Dots appearance is explicitly
accepted by the user on 2026-10-02.
Items 668 (approved Japanese dots), 672 (selected close-hover accent) and 673
(adaptive caret plus previous-character preference) are complete on main at
`7f554e63`, with the frozen local gate and all four exact-SHA required CI jobs
passed. Their implementation claims are closed. Native caret captures are
complete; user proportion judgment and live desktop motion remain owed below.
The latest display check still reports locked, asleep and inactive.
The prior Kite and test-pipeline sessions completed. Kite live review remains
below; the test-pipeline claim is closed. Previous worker claims are released.

The Files, panel, bounded-search, paste, release, IME, input, caret and punctuation
implementations are on main through verified integration `69d67a39`. Find/Replace
and the shared-control appearance were approved. Source branches remain recovery
points; removed source worktrees have verified local archives. Retiring a checkout
does not close a physical-input, hardware, release or taste obligation.

**Release constraint:** every public installable app download must be strictly
under 50,000,000 bytes, measured after compression. This covers macOS DMG/app zip
and Linux AppImage/tarball. Browser deployment and source archives are separate.
Keep real Japanese bold; return any size tradeoff to the user.

### 582 — review the landed Kite depth and motion

🔵 OWED — live motion and appearance review. The session
“Find the right way to run awl” completed the depth fade at `4f9fd3fb`, following
`0d5388d6` and `5d3d4249`, with native/wasm verification, regression mutation
proof and five native captures. Later integrated main and the test-pipeline
candidate also passed their required gates; no verification worker is still owed.

Present the preserved native captures and current bundled app. User review still
covers density near page edges and the far core, several-minute motion comfort,
dwell/transit/settle, pause/focus freeze and Reduce Motion in the bundled app.
Preserve the projected tunnel geometry and shared renderer ownership. The earlier
582 projection evidence remains in Git; there is no separate 564 task.

---

### 657 — finish the approved Files composition

⚪ QUEUED — finish the native composition journeys. Source
`codex/657-files-affordances` at `fd1ec8df` is integrated on main. Earlier source
landing did not establish acceptance of every live browsing journey.

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

⚪ QUEUED — physical input review. Source
`codex/658-files-select-all-physical` at `8f0adee7` is integrated on main with
key-route laws and pointer-focus repair. The user's physical Cmd-A failure
on 2026-09-24 remains the defect evidence; do not infer a complete repair from
a passing direct-action or menu test.

Recheck physical Cmd-A then Backspace in the Files query, alongside Edit ▸ Select
all, pointer editing, Tab/Enter, matches/no matches and accessible result opening.
Selection must belong to the focused query and never edit the document behind it.
Preserve the already-confirmed Change folder cancellation behavior. Use fresh
state/captures in a disposable release session; coordinate automated-input evidence
with 664 and Japanese composition with 665 without conflating the defects.

---

### 659 — finish Settings and Themes composition review

⚪ QUEUED — remaining Settings and Themes journeys. Source
`codex/659-panel-composition` at `d1b1988e` is integrated, with newer main
composition and bounded repairs retained. The final Find/Replace and shared-mark
release previews were approved; Settings/Themes live journey review remains owed.

Review Settings' recognizable search field, bounded label/value columns and
separate category selection, control selection and keyboard focus. Review Themes
as one coherent panel with heading, effective Switch/Cancel actions and a clear
current-versus-previewed choice. Retain the approved Find/Replace grouping and
shared control marks. Use approved study 05 (657), preserving theme identities and quieter
opaque chrome.

Verify forward/reverse focus, matching/no-match search, nested pickers, cancellation
and immediate settings in an identified native release build. Include narrow
geometry, Replace mode and opening/preview worlds. Keep theme preview geometry
stable and restore the invoking Settings control on Escape.

---

### 661 — finish external plain-text browser paste

⚪ QUEUED — genuine external clipboard journeys. Source
`codex/661-browser-paste` at `0aaae8a9` is integrated on main: trusted paste,
bulk insertion, composition/keybinding-capture protection and focus repairs.
Automated laws do not establish OS clipboard delivery in the three browsers.

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

⚪ QUEUED — credentialed nonpublishing rehearsal and real launch. Source
`codex/662-signed-macos` at `b45e312d` is integrated on main, including
signing/notarization, dry-run payload preparation and strict compressed-size checks.
The user confirmed Apple setup is complete; verify its operation without printing secrets or reopening setup by assumption.

Run a nonpublishing credentialed rehearsal. Inspect both architectures, bundle
identity, Developer ID signature, notarization/staple, Gatekeeper result, mounted
DMG contents, versioned names, checksums and actual compressed size. Missing or
partial credentials must fail before publication. The integrated workflow makes
the DMG public and the app zip diagnostic-only; check that final payload layout and document it.

Hosted rehearsal, final DMG size and a real Mac launch remain owed. Preserve the
under-50,000,000-byte public-download limit and return a measured packaging tradeoff
if needed. Do not tag or publish here; 663 owns the release cut.

---

### 663 — release one version with Linux and macOS downloads

⚪ QUEUED — depends on 662, the remaining release/hardware journeys and the
user's explicit release instruction. Choose the version from a frozen verified
candidate. Publish Linux x86_64 AppImage/tarball and signed, notarized
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
at `d9cf10c6` is integrated on main with delivery/focused-field diagnostics.
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

⚪ QUEUED — physical Japanese input check. Source
`codex/665-ime-fields` at `45c38256` is integrated on main: focused preedit/commit,
candidate ownership and restart after cancelled preedit. Integration also repaired
the single-line field path; physical OS input remains separate evidence.

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

🔵 OWED — live caret proportion judgment. The earlier shared block-padding
reduction (`6b7efadb`, source `c907f9e2`) is superseded by the approved shared
grapheme-ink adaptive caret (`bee5ec68`, integrated and verified at `7f554e63`).
Matched Paperbark x/a/H/g/Å/W before/after and four-world Latin/CJK native captures
are saved privately for review. Their automated bounds and mutation proof passed;
they do not close the user's visual judgment or live motion review.
Source `codex/666-paperbark-caret` at `f548f9b8` remains a recovery reference.

Review matched before/after native release captures against the user's oversized
caret report. Check actual ink and line bounds across Paperbark and nearby serif
worlds, zoom, 1x/2x DPI, lowercase/ascenders/descenders, spaces, empty lines and
headings. Preserve stable row height, glyph visibility and shared metrics ownership.
Retain the rendered-pixel bound and mutation evidence; finish visual smoke and
ask the user to judge live proportions where the remaining choice is taste.

---


### 669 — resolve the remaining CJK caret taste choice

🔵 OWED — caret/sidebearing judgment. Source `codex/669-cjk-punctuation`
at `70eb217c`, including font routing (`46be9e14`) and document-evidence
invalidation, is already on main. Integrated native/wasm and punctuation laws
cover those implementation fixes; no source integration remains.

The earlier 15 px Paperbark kana-to-caret gap was a measured font-sidebearing
and caret-width taste choice before the adaptive caret landed at `7f554e63`.
Do not present that old measurement as current geometry. Review the current
matched native Latin/CJK captures and remeasure the relevant punctuation case
before asking whether to include leading punctuation-cell space. Preserve the
current shared ink-hugging treatment pending judgment; the font/cache fixes and
adaptive implementation are complete, while this visual choice remains open.

---

### 674 — soften the adaptive caret corners

🔵 READY FOR INTEGRATION — local candidate `80bfe80d2b1ee5f8a8cdcf9507005f8bd641e43f`
on `codex/674-rounded-caret`; shared top clearance is tighter, with width,
bottom and rounding preserved, and 273 caret checks/1 ignored, eight punctuation
integrations, named mutations, native/Linux Clippy and structural ratchets passed.
Eighteen final native frames and a private Library three-way Å comparison are
ready; the actual body is two device pixels shorter at the top than prior rounded
at 2x DPI with ring pixels unchanged, and parent product integration/gates remain.
The user asked on 2026-10-02 for a bit more rounding, closer to the former look,
while retaining character-sized fit. Tune shared corner radius and ink padding
together; keep document and picker geometry on one owner. Preserve x/a/H/g/Å/W,
combining marks, serifs, resolved CJK coverage, visibility floors, glyphless and
ligature fallbacks, motion, and the previous-character preference unchanged.

Verify actual rounded-boundary containment and native pixels at 1x/2x DPI,
focused caret/preview/input laws and a named regression mutation. Prepare actual
matched local before/after captures for judgment. Private Library comparison uploads are approved; hold product integration
until the newest caret and separate Files repair are ready. Inspect the newly reported close-×
alignment read-only and return a diagnosis; do not mix its implementation here.

---

### 675 — refresh Files after returning from an empty subfolder

🟡 IN PROGRESS — cloud worker `/root/fix_files_parent_refresh` (codex),
branch `codex/675-files-parent-refresh`, isolated `.worktrees/675-files-parent-refresh/`.
Base product `c864b144`; create the isolated checkout through `scripts/worktree.py`.

Live cloud QA twice reproduced the failure with real X11 pointer clicks:
seeded notes contains alpha.md, beta.md and empty-folder. Open empty-folder,
then click Up. The heading returns to notes and subfolders remain, but Markdown
files disappear and the panel says no supported files. Escape and Ctrl-O restore
the files; the filesystem remains intact. Reproduce and repair the Files parent
navigation refresh under 657's remaining browsing acceptance, preserving query,
focus, root, supported-file filtering and ordinary folder selection boundaries.

Identify the first stale owner and add a regression law at the real App/Files
refresh seam plus the same live pointer journey. The parent will supply detailed
worker context. Do not mix caret or close-× changes here; no push is authorized
for these new refinements until the current approval is handled.

---

### 676 — keep Settings workspace controls clear of document ink

🔵 READY FOR INTEGRATION — user accepted the native before/after for local
candidate `2603843c6dff4959c09b8a19ca7acd8f67948734`
on `codex/676-settings-workspace`; shared controls-workspace opacity passes all
160 pixel cells, fails the original policy by name, and passes 109 workspace
checks/1 ignored plus ruled/comparison/audition laws and full source-health.
Four matched native App frames per phase and two local contacts are ready;
integrated product gates remain owed.
Live Linux QA on `c864b144` reproduced sharp document text overlapping Settings
labels and controls in Paperbark at 641×800 and 1180×812. Reproduce with a heading
and dense prose behind Settings. The workspace currently stays crisp while its
backing can inherit Ruled's bare surface: those policies disagree.

Repair shared workspace/backdrop composition with one ownership rule, preserving
world identities, the Settings rail/controls, theme/caret live auditions and ordinary
list composition. Verify native pixels against different documents underneath,
cover narrow/wide dimensions and affected backing styles, and prove the regression
law fails when the defect is restored. Keep this work separate from caret 674 and
Files 675. Return a local commit and captures; no push or Library upload yet.

---

## Outstanding review of landed work

These are user judgments, not active implementation claims. Historical evidence
remains in Git and preserved local galleries; Kite review is item 582 above.

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

- **Current product code `7f554e63e5fa318370f0ab89b4cb171b09df1f19`:** the
  frozen local `verify.sh full` passed native, wasm runtime and all eight
  debug/release parity families. [Exact-SHA required CI](https://github.com/Frank-P-Lu/awl-editor/actions/runs/36963886854)
  completed successfully: Linux, Mac non-render, wasm and Mac live-probe all
  passed. This closes implementation claims 668/672/673, including the Settings
  caption-fit and native caret-contract integration repairs. Headless native
  captures and automated input laws do not establish live motion, physical input,
  external clipboard delivery or human accessibility acceptance.
- **Earlier product baseline `3df4dc9a64f940f466efa76b962e0306b14c61b1`:** the local frozen
  composed gate passed. [Required CI](https://github.com/Frank-P-Lu/awl-editor/actions/runs/36885217287)
  completed with all four gating jobs successful. [Extended verification](https://github.com/Frank-P-Lu/awl-editor/actions/runs/36885216205)
  passed on Linux and macOS: 640 captures per platform, zero findings, and both
  actual 120-second built-artifact launches. The three-layer pipeline and scoped
  Rust 1.99/tool runtime selection are implemented.
- **Queue-only closure `b1330874c2b562674141533ce7b19a350f33597c`:** its
  [required CI](https://github.com/Frank-P-Lu/awl-editor/actions/runs/36892748842)
  completed successfully. It changes no code from `3df4dc9a`.
- **Reviewed integration `69d67a39`:** native, wasm, profile parity and exact-SHA
  required hosted CI passed; final Find/Replace/shared-control previews approved.
  Bounded Search in folder (651), chosen browser refusal and its work-cap laws
  landed. The excluded transactional browser-storage experiment stays preserved.

Tolerated hosted Mac-render GPU out-of-memory/atlas failures and the AT-SPI
probe/dependency gaps remain separate; required or extended success does not close
human accessibility or physical-input obligations. This queue reconciliation uses
diff, heading, reference and status checks under `docs/verification.md`; receipts
continue to name their actual tested commits. No new product gate is claimed.

## Remaining execution and acceptance

Assessment only: no additional journey suite or release rehearsal was launched.
The code gate above is complete; do not repeat it merely to assess this board.

| Remaining work | Available executor evidence | What still prevents closure |
| --- | --- | --- |
| 657 Files; 659 Settings/Themes | Seeded headless-App/release captures can check composition, geometry, focus state and cancellation with the existing built candidate | An awake, unlocked Mac for actual browsing, chooser, focus and theme-switching journeys; human appearance judgment |
| 582 Kite; 666 Paperbark; 669 CJK; landed list-marker/Gumtree review | Preserved actual native images are ready; current CJK punctuation geometry can be measured headlessly | User taste answers; an awake, unlocked Mac for several-minute motion comfort, pause/focus/Reduce Motion and caret movement |
| 658 physical query selection; 664 keyboard ingress; 665 Japanese IME | Existing route/App laws and diagnostic instrumentation are complete; disposable fixtures can be prepared | Physical Cmd-A/Backspace and Japanese composition in the focused Files query; unlocked desktop and supported automation permissions for comparing injected versus physical delivery |
| 661 external browser paste | Static release fixtures and bulk-insertion receipts can be inspected | Genuine external OS clipboard delivery in Chrome, Safari and Firefox on an interactive desktop; record versions and keep field/document ownership distinct |
| 662 macOS packaging | GitHub can run a credentialed nonpublishing rehearsal and inspect universal/signature/notarization/staple/Gatekeeper, compressed sizes and checksums; this is not blocked by the local screen lock | Actual hosted credential availability/operation and final DMG results remain unverified; a real unlocked Mac must launch the mounted app |
| Linux desktop obligations | Hosted Linux CI is green and nonpublishing artifact checks can run remotely | A real x86_64 Linux desktop/operator for both package forms, launcher/FUSE, drawn-menu Export, and AT-SPI/Orca with audio and a person |
| 663 release cut | The checklist and final public payload can be prepared after the preceding proof | Remaining acceptance and packaging results, then the user's explicit instruction for a specific tag/release; website deployment requires its own explicit instruction |

The smallest next user steps are to unlock and keep the Mac awake for one seeded
review session, physically try Files Cmd-A/Backspace and Japanese input, and judge
the supplied caret/CJK and remaining theme images. That same session can cover
browser paste and motion. Linux acceptance needs a Linux desktop/operator; it
cannot be substituted by this Mac's captures. No signing setup question is owed;
verify the confirmed setup by nonpublishing rehearsal. Ask for a release instruction
only after the final artifacts and remaining acceptance are reviewable.

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
