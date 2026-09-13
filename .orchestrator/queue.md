# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**5 open numbered tasks.** In implementation or review: 657, 659 and 661.
Blocked: 651 and 658.
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

🔴 BLOCKED on trustworthy physical Cmd-A confirmation. A fresh release bundle
built and assembled, but the bounded automation attempt delivered no keyboard or
`App::apply` trace events; the app-selection call hung until interrupted. The CUA
injector also produced inconsistent deletions (`star` → `sta`, `shared` → `hare`).
Existing direct-action and real menu-click laws both pass, so product code must not
change until a physical key-equivalent run distinguishes an awl defect from the
automation door. Earlier implementation and audit landed with 657 at `ef344429`.

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

🔴 BLOCKED on browser-backend direction after independent audit. Native/loading/UI
work is committed at `7764e7ea`, but is not mergeable as complete: synchronous
`localStorage.getItem` cannot bound a cross-tab replacement before materializing it.
Transactional IndexedDB chunks preserve parity but expand the storage architecture;
failing browser folder search closed is safe but a user-visible degradation.

Finding: `App::gather_overlay_inputs(OpenSearchFolder)` calls
`search_folder::load_corpus` synchronously with whole-file `read_to_string` before
showing the picker. `load_corpus` checks `max_file_bytes` only after reading and
counts only retained files/bytes. Oversized or failed reads consume neither budget;
the total retained-byte limit can also be exceeded by the last accepted file.

Decision required: either move browser folder storage to transactional IndexedDB/
OPFS chunks so the budget is enforceable, or make browser Search in folder fail
closed with an explicit notice. There is no safe third option under the current
synchronous localStorage backend.

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
