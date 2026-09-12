# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**4 open numbered tasks.** Ready: none. Integrated work awaiting visible native
review: 659, 658 and 657. Blocked direction: 651.
Outstanding review of landed work and hardware checks are listed separately below.

### 657 — restore the approved Files composition (user report, 2026-09-12)

🟠 INTEGRATED — implementation, focused native/wasm evidence and independent
audit landed on local `main` at merge `ef344429`; visible native review remains.
Corrective follow-up to 637/628/640; prior completion did not establish fidelity.

Evidence: the user's native screenshot shows a detached search strip, Change
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

🟠 INTEGRATED — implementation, focused native/wasm evidence and independent
audit landed with 657 on local `main` at merge `ef344429`; the required visible
native review remains.

Observed through the running app's accessibility state: Cmd-A then Backspace in a
nonempty Files query removed only its last character on two attempts. An unmatched
query left Change folder selected and advertised Enter as change-folder. A known
filename vanished from the exposed result list while Enter still opened its file;
that is an accessibility/result-publication hypothesis, not evidence search failed.
The screenshot feed was stale during this sitting, so visual absence is unverified.

Verify in a disposable seeded workspace with a recorded build: query select-all,
clear, matches/no matches, Tab and Enter, menu Select All, pointer and accessibility
results. A no-match state must not implicitly accept Change folder. Matching choices,
selection and focus must agree visually and semantically. Never type probes into the
user's working document or rely on a stale screenshot to establish focus.

---

### 659 — reconcile Settings focus flow and finish native panel review (user review, 2026-09-12)

🟠 INTEGRATED — implementation landed on local `main` at merge `2dcdb1779`,
followed by focus-owner, test-health and schema commits `210c60e36`, `be399d5f3`
and `0e5e5d132`. The explicitly owed visible Settings/Find-Replace/Themes review
remains.

Observed: Settings opened on Categories; Tab entered the setting list, typing
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
- **553 — folder-search highlight review.** Real-pixel match-highlight legibility
  remains unverified. Retain the known boundaries: results use summon-time disk
  contents, grouping differs from lens headers, and CRLF matches can retain a
  cosmetic trailing carriage return.
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
**`943981bf26117d14ddbf1420d3b4f60368e7897c`**:

```text
native-gate-receipt commit=943981bf26117d14ddbf1420d3b4f60368e7897c health=pass:281s
  conventions=mac,linux scope=all-targets menubar=full:on unit_tests=5215
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
