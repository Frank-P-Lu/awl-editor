# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**10 open numbered tasks.** In implementation or review: 651, 657, 658, 659 and 661.

Queued work: releases 662–663, live-input investigation 664 and Japanese-input
Files repair 665, and Paperbark caret sizing 666.

Outstanding review of landed work and hardware checks are listed separately below.

### 657 — restore the approved Files composition (user report, 2026-09-12)

🟡 IN PROGRESS — /root/files_affordances (codex), branch
`codex/657-files-affordances`. The prior implementation landed at `ef344429`;
the 2026-09-13 live review finds hierarchy still short of the approved composition.
Corrective follow-up to 637/628/640; prior completion did not establish fidelity.

Live review (2026-09-13, running dev-app in Bowerbird with a temporary fixture):
the unified Files panel and separate accessible actions are present, but folder
name, Change folder and Search read as one faint inline sentence. The search has
no recognizable field boundary, and New document reads like a footnote. Correct
these through a distinct folder heading, a visibly interactive Change folder,
a clearly bounded labelled search field and a legible footer action. Preserve
quiet chrome without making essential controls look disabled; distinguish keyboard
focus from selected view/row. Judge native screenshots by finding these affordances,
not by their mere presence in the accessibility tree. Background blur is absent
in the reviewed Files frame; preserve that improvement. Do not reimplement the
already-separated action ownership or reintroduce mixed action rows.

Original evidence (2026-09-12, before the latest correction): the user's screenshot shows a detached search strip, Change
folder and New document as list rows, ambiguous `root/` scope, repeated folder
markers and background blur. The live accessibility tree confirms the mixed rows.

Build: compose Files as one coherent surface: named folder and Change folder in
the header, labelled search, Files/Recent views, breadcrumbs/Up, file/folder choices,
and a separate New document footer action naming its destination. Use one clear
folder affordance rather than `/`, disclosure and “folder” together. Preserve the
bottom-left working set and the approved quieter opaque chrome; remove background
blur for the reviewed picker surfaces. Preserve native theme identities, bindings,
root ownership and buffer/save semantics. The accepted site is reference only:
https://awl-files-reconsidered.s84fzrm6tq.chatgpt.site/ (study 05).

Verify: use a disposable seeded native release build with explicit root/config.
Cover browsing, search, empty folders and narrow layout; compare hierarchy and
control placement against the approved composition. Preserve content-free listings
and the I/O boundaries established by the completed 650/652 work. No user-file
captures belong in the public repo.

---

### 658 — reproduce and repair Files query/action/accessibility gaps (live observations, 2026-09-12)

🟡 IN PROGRESS — /root/files_select_all (codex), branch
`codex/658-files-select-all-physical`. On 2026-09-24 the user physically confirmed that
Cmd-A in the Files search field does not select its query, and Backspace deletes
only the last character. Earlier implementation and audit landed with 657 at
`ef344429`. Prior automation delivered no trustworthy keyboard or `App::apply`
trace events, so the physical report is the defect evidence; distinguish the key
route from the already-passing direct-action and menu-click laws.

In a disposable Bowerbird release session, typing `shared` in Files and pressing
Cmd-A then Backspace left `hare`: the command did not select the query and the
delete removed only its last grapheme. Edit ▸ Select all followed by Backspace
cleared the whole query. This is an action-level discrepancy, not a general
selection failure. The same session established that an unmatched query kept
focus in Search and Enter did nothing; `notes/draft.md` was exposed, selected and
opened by Enter, and `shared.md` opened through its accessible pointer action.
The screenshot feed remained stale while the accessibility tree and interactions
advanced, so it is not appearance evidence.

Cancel retry (2026-09-13): starting in Recent, clicking Change folder opened the
macOS Open panel; clicking Cancel returned to the same root and Recent empty
state, with the same open document and focus on Change folder. Confirmed in both
the accessibility tree and an updated screenshot. The earlier tool timeouts did
not establish a cancellation bug. This bounded check does not discharge the
remaining query selection, no-match or result-publication checks below.

Build: route the Cmd-A menu-equivalent action to the focused Files query through
the same surface-level selection owner used by Edit ▸ Select all. Preserve the
confirmed no-match and result-publication behavior. Add an action-level law that
fails when Select all falls through to the background document, then recheck query
clear, matches/no matches, Tab and Enter, menu Select All, pointer and accessible
results in a disposable seeded workspace. Never type probes into the user's working
document or rely on a stale screenshot to establish focus.

---

### 659 — reconcile Settings focus flow and finish native panel review (user review, 2026-09-12)

🟡 IN PROGRESS — /root/panel_composition (codex), branch
`codex/659-panel-composition`. The earlier implementation landed at `2dcdb1779`
with follow-ups `210c60e36`, `be399d5f3` and `0e5e5d132`. Live review on 2026-09-13
confirms working transitions but remaining composition gaps; do not mark this as
only waiting for acceptance of the existing appearance.

Queued corrections from actual Bowerbird frames:

- Settings: make search a recognizable field rather than a faint breadcrumb.
  Preserve the bounded label/value column; give category selection, selected control
  and active keyboard focus distinct treatments. Compose the surrounding workspace
  deliberately instead of an oversized plate around a dense narrow column.
- Themes: unite the detached query strip and list into the approved coherent panel,
  with a clear Themes heading and visible clickable Switch/Cancel actions with
  effective shortcut hints. Distinguish the current theme from the previewed choice.
  Bowerbird → Mulga preview held panel geometry/colors steady and Esc restored
  Bowerbird and the invoking Settings control: preserve those verified behaviors.
- Find: separate field, count/previous/next and Match case into readable groups.
  Remove redundant Aa beside Match case and crowded inline shortcut clutter; extend
  the same composition to Replace mode, keeping replacement actions separate.

Use the approved study 05 as a composition reference, not a hardcoded color skin.
Prioritize recognizable fields/actions and focus before spacing-only retuning.
Settings search focus now works (Tab from Categories reaches the search field);
retain that improvement. Review matched native frames and actual input routes
under docs/verification.md; do not claim the full theme roster, narrow geometry,
Replace mode or release-build parity was established by this bounded live sitting.

Earlier observation (2026-09-12): Settings opened on Categories; Tab entered the setting list, typing
filtered while the setting row retained accessible focus, and Enter opened Themes.
Theme preview followed by Esc restored the original theme and the filtered Settings
page. Those transitions worked. The approved prototype instead teaches an explicit
search field followed by controls. Verify a discoverable complete forward/reverse
focus route, keeping search, categories, controls and value-editing states distinct.

Cover search → matching controls → categories/close and reverse traversal, no
matches, nested Theme/other pickers, cancellation and immediate-setting semantics.
Use disposable documents and inspect state after every transition before typing.
Recheck Settings, Find/Replace and Themes on one visible, identified release build,
including Themes placement across opening/preview worlds, Switch/Cancel, quieter
chrome and removed blur.

---

### 651 — Search in folder budgets must bound reading work (audit finding, 2026-09-12)

🟡 IN PROGRESS — /root/search_folder_fail_closed (codex), branch
`codex/651-browser-fail-closed`. The user chose the fail-closed browser path on
2026-09-24. Native/loading/UI work is committed at `7764e7ea`, but is not
mergeable as complete: synchronous `localStorage.getItem` cannot bound a cross-tab
replacement before materializing it. Browser folder search must refuse with a
clear notice; do not add transactional storage in this round.

Finding: `App::gather_overlay_inputs(OpenSearchFolder)` calls
`search_folder::load_corpus` synchronously with whole-file `read_to_string` before
showing the picker. `load_corpus` checks `max_file_bytes` only after reading and
counts only retained files/bytes. Oversized or failed reads consume neither budget;
the total retained-byte limit can also be exceeded by the last accepted file.

Decision: make browser Search in folder fail closed with an explicit notice.
There is no safe budgeted-read path under the current synchronous localStorage
backend. Preserve the native search path and avoid partially loading a browser
corpus before refusing.

Done/Verify after direction: count attempts and bytes at the actual FS seam; probe
exact limits, cap+1, all-rejected corpora, unknown/growing sizes, binary data and read
errors. Preserve search correctness for admitted files and make incomplete coverage
honest. Use the completed 652 investigation when designing background/cancellation
work. Mutation-prove that the old post-read-only limits fail, then run one integrated
native/wasm gate on the final frozen candidate.

### 661 — external plain-text paste in the browser

🟡 IN REPAIR — /root/browser_paste (codex), branch `codex/661-browser-paste`.
Independent review of source/tests commit `0ab25650` found browser-menu paste can
mutate keybinding capture, and per-scalar insertion makes large summoned-field
paste quadratic with repeated refilters. Repair with capture/composition gates,
one field-owned bulk insertion/recompute and work-count/release evidence. The
narrow WEB.md commit is `623eb83f`; the full final Chrome/Safari/Firefox interaction
matrix remains owed. This remains independent of browser-storage item 651.

Problem: the wasm clipboard reader in `src/app.rs` always returns unavailable;
`refresh_kill_from_clipboard` silently retains the internal kill ring. External
paste can therefore do nothing or insert stale internal text. The installed winit
web keydown handler cancels default behavior, which may suppress browser paste
events. These are source findings; a live browser reproduction remains required.

Build: investigate and implement trusted browser `paste` events carrying
`clipboardData` plain text, allowing the native browser paste gesture without
also executing an internal yank. Route the received text through the shared
Actions/transition and focused-surface owners, preserving selection replacement,
Unicode and undo. Preserve browser shortcut suppression elsewhere and existing
keymap/remapping semantics. Explicitly handle custom yank bindings and palette
paste, which do not inherently generate a browser paste event. For an observed
failed or unsupported external-paste attempt, show an actionable notice instead
of silently inserting stale internal text. Keep intentional internal yank behavior
explicit. Image paste and styled clipboard are outside this task. Update WEB.md
with verified behavior rather than treating async clipboard-read limitations as
proof that all external paste is impossible.

Verify: reproduce before fixing, then exercise genuine external copy/paste in
Chrome, Safari and Firefox on a static release build. Cover document and summoned
text fields, focus changes, selection replacement, multiline/Unicode, single undo,
repeated paste, empty/non-text data, custom bindings and palette invocation. Prove
no duplicate insertion or background-document edit. Synthetic clipboard payloads
can test routing but cannot prove OS clipboard access. Add targeted regression laws
that fail with the old behavior; record browser versions and unavailable coverage.
Follow the verification policy for integration gates. Reference:
[Clipboard events specification](https://www.w3.org/TR/clipboard-apis/#clipboard-event-paste).

---

### 662 — make the macOS download a signed, notarized release artifact

⚪ QUEUED — requested by the user 2026-09-24. Apple signing and notarization
setup are already complete, as the user confirmed. The current `release.yml`
builds an unsigned universal `Awl.app` and DMG only on dry runs; tag runs skip
the mac job and publish Linux alone. A workflow artifact is a short-lived build
receipt, not the public macOS download. The stale setup-state claims in
`RELEASING.md` were corrected when this work was queued. Never print secret values.

Build: make tag runs produce a universal macOS app using the existing Developer
ID signing and notarization setup, staple it, and attach a versioned DMG (and
app zip if useful) with checksum coverage to the same GitHub Release as Linux.
A missing or partial signing setup must fail the macOS release job before
publication; never attach an unsigned app as if it were ready for normal download.
Keep dry runs nonpublishing and exercise the packaging and publish-file layout
without making a tag. Update the release instructions and public download copy
to describe the verified path.

Verify: inspect both architectures in the universal binary; verify bundle
identity, signature, notarization/staple, DMG contents, checksums and artifact
names on the actual candidate. Record what the hosted macOS runner proves and
what still needs a launch on a real user's Mac. Do not tag or publish in this
item; item 663 owns the release cut.

---

### 663 — release one version with Linux and macOS downloads

⚪ QUEUED — requested by the user 2026-09-24; depends on 662 and a frozen,
integrated release candidate. `Cargo.toml` currently names 0.13.0, but choose
the tag from the candidate's actual version rather than assuming that number
will remain current. Publish the Linux x86_64 AppImage and tarball alongside the
signed, notarized macOS DMG from the same tag. Keep the browser demo on its
separate site deployment path; a zipped workflow artifact is not its main
distribution.

Work the exact-commit pre-tag checklist in `RELEASING.md`, including native and
wasm gates, audit, profile parity, dry run, checksums, release notes and current
CI status. Confirm the existing Linux hardware checks on a real desktop or
record the untested combinations honestly. Inspect the final file list and
download instructions before the tag push. The user's request here queues the
release; obtain the explicit go to cut the tag and publish at execution time.
After publication, verify both platform downloads and checksums from the public
Release. Treat any site redeploy as a separate explicit action.

---

### 664 — diagnose unreliable automated keyboard input in the live macOS app

⚪ QUEUED — requested by the user 2026-09-24. During the Files query review,
injected keys produced no keyboard or `App::apply` trace, app selection hung,
and injected deletions gave inconsistent text (`star` → `sta`, `shared` →
`hare`). Those observations do not identify where input was lost. The user's
physical Cmd-A test remains evidence for item 658's Files defect; investigate
the automation path separately so neither result is used to explain the other.

Reproduce with a disposable document and config in a named release-build app
session. Record the focused app/window/control, the injected key and timing,
native event delivery, keymap action and `App::apply` receipt at each attempt.
Compare with physical input and a direct App-level action on the same state;
check focus handoff, modifier state, event injection and screenshot freshness.
Identify the first boundary that diverges before changing code. Repair the
owned app/harness boundary if one is found; if the loss is in an external
automation service or macOS permission layer, report a minimal reproducer and
use a trustworthy supported driver for the missing evidence instead.

Verify repeatability across fresh sessions and several key sequences, including
Cmd-A, deletion and plain typing. Confirm a successful injected sequence reaches
the intended field exactly once, and a failed sequence is observable rather
than silently counted as a pass. Keep probe logs and captures free of private
paths and document text. Do not use this investigation to delay item 658's
direct fix or to claim live behavior from headless replay alone.

---

### 665 — restore Japanese input focus in Files after Cmd-O (user report, 2026-09-24)

⚪ QUEUED — the user reports that with Japanese input active, Cmd-O opens Files
but typing cannot reach its search field; the caret appears stuck in the
background document. The exact macOS input-source state, whether preedit or
commit is lost, and whether the document actually changes are unverified.
This is distinct from 658's confirmed Cmd-A selection failure and 664's
unreliable automated key injection; coordinate evidence without conflating them.

Reproduce in a disposable native release session with a seeded root/config and
both Japanese and direct Latin input. Record input-source and composition state,
focused surface, caret/IME candidate position, preedit and commit delivery, query
text and background-document bytes before and after Cmd-O, typing, conversion,
Enter and Escape. Compare a pointer-focused Files search and other summoned text
fields. Inspect the focus handoff, `WindowEvent::Ime` route and text ownership:
`src/app/input/ime.rs` currently sends committed text through the document-text
door, which is a lead to verify, not a diagnosed cause. Also check whether the
search field can accept a Japanese composition when opened while IME is already
active.

Build one focused-text-surface owner for IME preedit and commit: the document
receives text when it is focused, and a summoned text field receives text when
it is focused. Files search must receive Japanese composition and committed text
exactly once, show its active caret/preedit in the right place, and never edit
the document behind it. Non-text surfaces must not insert into the background
document. Preserve ordinary Cmd-O, Latin query input, cancellation, selection,
undo and native menu routing. Add regression laws at the real App/IME boundary
that fail under the old behavior across the focused text-field roster; audit
neighboring surfaces and both input-source states, then verify physical Japanese
typing in the bundled release app. Report any live-driver limits separately from
product behavior.

---

### 666 — shorten the oversized Paperbark caret (user screenshot, 2026-09-24)

⚪ QUEUED — the user reports that Paperbark's coral block caret is too tall. In
the supplied screenshot, its rounded body rises noticeably above and falls
below the adjacent lowercase text. The screenshot establishes the appearance,
but its zoom, DPI, caret mode and settled/moving state are not known; reproduce
those conditions before choosing a sizing change. Do not publish the private
attachment path or copy the screenshot into the public repository.

Inspect the shared vertical caret owner (`src/render/caret.rs` and its
`caret::vertical` helpers), Paperbark's resolved face and metrics, and the
existing one-height pixel laws. Measure the caret's rendered top, bottom and
height against the row's actual ink and line box in a seeded, content-safe
Paperbark capture. Compare nearby serif worlds and a smaller/larger zoom at 1x
and 2x DPI; check lowercase, ascenders, descenders, spaces, empty lines and
headings. Identify whether the excess comes from the face metrics, the shared
minimum/padding, a tall row, or the active caret treatment before changing it.

Make the resting caret read as proportionate to the text without losing its
presence or clipping glyphs, and keep its height stable while typing across a
row. Use the shared renderer and data-driven metrics rather than a Paperbark
identity branch. Add a rendered-pixel bound that fails on the oversized
appearance while preserving the existing stability law. Verify matching
before/after PNG measurements and visual smoke in Paperbark plus adjacent worlds;
ask the user to judge the final live feel if the measured fix still leaves a
taste choice.

---

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
  has the right density. Keeping the repaired projection is recommended.
- **588 — mechanically complete; blocked on user taste.** The current plain
  `•◦▪` fallback passed a fresh 20-theme release gallery and focused Metal laws:
  all three Brolga depths are present, distinct, aligned, contained, unclipped and
  legible (sampled contrast 13.46:1, 5.23:1, 13.46:1). Evidence is in the ignored
  `gallery/landed-visual-review/588-bullets/`. Keeping it is recommended; a different
  glyph or wider box is a separate mechanism decision only the user can authorize.
- **561 / 618 — mechanically complete; blocked on user taste.** The requested
  reduction is merged (`760f4f43`, merge `b3e8d2aa`) and remeasured at 15.02%.
  Fresh release and live-headless-App captures show every Gumtree ornament present
  and legible; measured ink heights are snake 3.41em, fish 5.84em and snail 3.87em.
  Accepting the fish-forward character is recommended; a per-glyph scale is a new
  mechanism requiring the user's taste decision. True-window live proportion
  judgment remains part of that decision.

Kite's unresolved appearance and live motion review belong to **582** above;
there is no separate 564 build item. Its review must include convergence near
page edges at common window sizes as well as several-minute motion comfort.
For current accessibility acceptance and deferred work, use `ACCESSIBILITY.md`;
the resolved 584/626 investigation does not require another confirmation sitting.

## Latest recorded verification

The latest recorded native/wasm baseline is
**`6e76d6b59ddfa8c2816a4ea61d88eb194ba7a601`**:

```text
native-gate-receipt commit=6e76d6b59ddfa8c2816a4ea61d88eb194ba7a601 health=pass:283s
  conventions=mac,linux scope=all-targets menubar=full:on unit_tests=5218
  unit_shards=6 integration_targets=18
web-smoke: OK
```

Fresh `/editor/` assembly from a release Trunk build passed: the staged page uses
rooted 16-hex hashed JavaScript and Wasm assets, retains analytics, contains no
builder-home path, and no generated editor bundle is tracked.

The dark-world no-brightening-slab and light-world rim-elevation panel laws each
ran individually with `--exact --nocapture`: 1/1 passed apiece with no adapter-skip
output on the built-in Apple M1 Max Metal adapter. Local Metal does not establish
hosted virtualised-Metal behavior.

Subsequent queue/policy-only commits use diff/link checks under `docs/verification.md`.
Older receipts and completed train summaries are in Git history. Local hardware
receipts do not establish hosted-GPU or live-journey results.

## Needs specific hardware

🔴 BLOCKED on the orchestration host (audited 2026-09-11). It is Apple-silicon
macOS with Metal: there is no Linux graphical/AT-SPI/D-Bus session, Orca, X11 or
Wayland compositor, Linux awl executable, AppImage/tarball, or FUSE runtime.
Headless capture cannot substitute for any missing live door. Honor the current
scope and release policy in `ACCESSIBILITY.md` and `RELEASING.md`.

1. **AT-SPI journey** — on a real Linux desktop with Orca, exercise document
   reading, caret/selection, overlays, and an editing burst. This needs Orca, an
   active AT-SPI2/D-Bus bus, audio, current native Linux awl, and a person.
2. **Linux drawn-menu Export click** — with a real window/compositor, confirm the
   rendered menu's Export action reaches its destination through a genuine click.
3. **Current Linux release artifacts** — launch both the tarball and AppImage on a
   real x86_64 Linux desktop; check launcher name/icon, FUSE, and
   `--appimage-extract-and-run`. No current artifacts are present and a new tag is
   not authorized.

## Release authority

Signing/notarisation setup is complete; it is not an open setup task. Every new
tag/release still requires the user's explicit instruction per `RELEASING.md`.
