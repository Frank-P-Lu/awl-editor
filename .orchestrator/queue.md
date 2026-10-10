# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**13 open numbered tasks:** 677–679 await memory evidence, the authorized bullet-layout
candidate, and nested-code candidate integration; 582 and 666 await live motion
review; 657–659 and 661–665 retain their remaining journey or release obligations.
The user accepted Paperbark caret proportions (666), Japanese punctuation caret
spacing (669), Brolga decorative list markers (588), and uniform Gumtree ornaments
(561/618) on 2026-10-07 from current native captures at `7d2aa7c2`:
“So the four visual decisions, I like all of them. They look good. Thank you.”
This closes those visual decisions; physical input and live motion remain open.
Items 674 (rounded caret and tighter top clearance), 675 (Files parent refresh
and pointer activation) and 676 (opaque controls workspace) landed on main at
`03f5c22f`. The frozen native, wasm runtime and eight-family profile-parity gates
passed; exact-SHA required hosted CI will be checked after the authorized push.
Settings appearance was accepted; physical Mac/IME and human comfort remain open.

Dots appearance is explicitly
accepted by the user on 2026-10-02.
Items 668 (approved Japanese dots), 672 (selected close-hover accent) and 673
(adaptive caret plus previous-character preference) are complete on main at
`7f554e63`, with the frozen local gate and all four exact-SHA required CI jobs
passed. Their implementation claims are closed. Native caret captures are
complete; the supplied current proportions are accepted, while live desktop
motion remains owed below.
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

### 677 — investigate long-duration desktop memory growth

🔵 OWED — cap-stop diagnosis and long-duration evidence. Isolated source
`codex/677-memory-soak` at `e44d39c6` adds RSS/physical-footprint/Metal checkpoints,
first/last-decile change and late-run slope; nine soak laws and the targeted fast
slice passed. No growth cause or week-long fix is established.

After execution transport recovery on 2026-10-08, the prior supervisor was
absent with no rendered attempt recorded. Retained tool session 74149 now owns
restored supervisor PID 54082, retaining the original cutoff. Before the final
bullet gate ended it waited for the locked local display and serialized resource
gate, with no rendered baseline or soak interval. Its resumed attempt is below. Attempts are 24 hours with a
1.5 GiB RSS/footprint cap and a zero-presents stop. The original 48-hour supervisor
deadline remains 2026-10-10 01:51:14 UTC; restarts do not extend it. Caffeinate runs
only during an owned attempt. Exact binary/source and the stop procedure remain
under that worktree's `target/queue-677-memory-soak/e44d39c6`; retain these assets
and the active supervisor. The synthetic workload is separate from hosted GPU
failures. Local-thread follow-up scheduling is still owed: the automation service
rejected this delegated thread as non-local.
Root released only its owned `gate.pause` after terminal bullet full verification.
Supervisor 54082 started attempt 1 with owned app PID 71519 at 10:16:36 UTC.
The first baseline below precedes the material cap-stop recorded afterward.
At elapsed 61.746 seconds: 3,561 presents, RSS 232,996,864 bytes, physical
footprint 119,800,912 bytes and Metal allocation 12,402,688 bytes. This is an
early rendered baseline; warm-cache stabilization and long-duration growth
remain unestablished. Original cutoff is unchanged. Live logs:
`target/queue-677-memory-soak/e44d39c6/attempt-1.app.log` and monitor CSV.

Read-only observer PID 72984 / tool session 39114 now preserves hourly JSON
checkpoints and separate per-attempt trends until the same original cutoff.
It never starts, restarts, signals or stops the existing supervisor/app.
Five observer laws passed, including warm-up versus continuing growth and final
artifact generation at the cutoff. Observer/report directory:
`local evidence: memory-monitor-677`.
Final local outputs will be `final-cutoff.json` and `final-cutoff-report.txt`
at the cutoff, or first execution after wake if the host sleeps. Thread
notification remains blocked: a heartbeat requested from this delegated thread
was rejected as non-local, even targeting its source thread. The parent local
thread must retrieve and deliver the final interpretation; no cron workaround
or new Linux job was started.

Material cap-stop finding, 2026-10-08: attempt 1 stopped at 10:49:45 UTC
after 33m09s. External RSS rose 469,778,432 → 9,569,239,040 bytes in 30s;
supervisor SIGTERM and child exit 143 are preserved. No restart or cap increase.
Historical kernel logs independently warn of possible IOGPUResource leakage
for PID 71519: 400,000 at 10:49:17.753 → 1,200,000 at 10:49:41.971. Native
TextInputUIMacHelper cursor-update logs simultaneously accelerate 60/s → 1740/s
after a native interaction/state transition. This is a corroborated native
resource surge, not an established allocating call path or week-long reproduction.
App footprint/Metal values predate the surge; observer app-checkpoint peaks
exclude external RSS, so final interpretation must include the CSV/kernel evidence.
Cumulative swapins/out are 0; historical pressure/competing load remains unknown.
Separate diagnosis/report and original evidence checksums:
`local evidence: memory-analysis-677-cap-stop`.
Observer 72984 remains alive; supervisor 54082 and app 71519 have exited. Original
cutoff retained. Proposed next step is a separately assessed ≤35min synthetic
run with 1s external telemetry, lower/equal cap and rapid-growth stop, native
transition traces and safe bounded allocation capture. No new run launched.

🔵 BLOCKED — codex-memory-diagnostic (codex), branch
`codex/677-memory-spike-diagnostic`, clean at preserved e44d39c6. Authorized
single ≤35min passive synthetic diagnostic has NOT launched. Required worker
preflight session18145 exited1: free15,993,393,152bytes versus required
25,769,803,776bytes. Own empty-worktree sweep reclaimed0; no original/sibling
artifacts removed. Need≥9.1GiB additional free space through existing cleanup
ownership before builds; no floor bypass. Passive safeguard APIs inspected,
variant unimplemented/unverified because admission stopped before formatting
or compilation. Original active-window binary cannot substitute safely.
Planned512MiB cap is lower than original1.5GiB;1s RSS/footprint and independent
rapid-growth stop; nonactivating/background/pass-through window; retain24h
stimulus pacing to avoid planned faults within35min. No automatic stress retry.
Prelaunch receipt and assessment:
`local evidence: memory-diagnostic-677-preflight`.
Original evidence hashes, source, limits and cutoff remain unchanged; observer
72984 remains independent. No new app/Linux job, merge/push/release or security
change. Resume source and safeguard verification only after resource admission.

---

### 678 — review bullet markers in every world

🔵 OWED — integration approval. Branch `codex/678-theme-bullet-review` is frozen
clean at `93946586bc54370ba1c30c84fc31bf2c8ea69e65`, independently reviewed
without findings and registered awaiting-review. Authorized direction is
implemented: fourteen ordinary worlds; distinctive Gumtree, Quokka, Mangrove,
Brolga, Firetail and Paperbark. Quokka/Paperbark scale 0.72; shared measured
clearance, optical baseline seating, nested body starts and hanging wraps.

Required `scripts/verify.sh full` ended with exit 0 in root session 3048 on the
unchanged SHA. Native receipt: health 311 seconds, 5,365 unit tests in six shards,
both Mac/Linux input conventions, full-menu arm and 18 integration targets.
All 16 WebAssembly runtime tests and all eight debug/release profile-parity
families passed. Log: `target/queue-678-full-93946586/verify.log`.

Prior gates are retained: e9218a59/session94889 failed six native tests;
716d8b59/session40590 failed a 101-column assertion message before test suites.
Bounded repairs fixed concealed prefix source-column hit mapping, stale theme
and Awl Marks role oracles, and cached font metrics bypassing the document's
IBM Plex Mono Light weight-300 resolver. The default weight could select a
proportional system fallback. Font binary/cmap is unchanged; 21 live list-role
glyphs and retained legacy glyphs are explicit metadata. Compiled mapping and
weight omission mutations failed; restored positives passed. Targeted 25/25,
mono 2/2 and extraction 4/4 laws passed. Cohesive metrics extraction clears the
500-line ceiling without new health exceptions; final assertion wrap changes
no behavior. No generated vendor targets or release artifacts are tracked.

All forty after captures were refreshed on actual Mac Metal at 1x/2x using
clean aadabcd4 binary SHA256 `0b2c194e…`. Subsequent extraction/message wrap is
behavior-identical. Only Tawny changed; other 38 PNGs are byte-identical.
Independent sidecars: 656 continuation starts, zero failures, max 0.04959 logical
px. Tawny depth gap is 12/12/12 px and ordinary/task body difference 0.024 px.
Twenty labeled pairs, contact sheet and notes are confirmed in Library under
existing IDs, with all receipt xattrs applied. Contact version 1:
`libfile_6c337b8dc2e08191b62dab75e0d57f44` /
`file_0000000084548206bbb37d70abe7e3b7`.
Notes version 3 (all screenshot IDs, final gate and rendered memory baseline):
`libfile_0f19be178f488191824b278a0e782cb4` /
`file_00000000d07c8206ab843160ed0cbda2`.
Root receipt: `bullet-layout-678/checks/final-93946586-verification-receipt.md`
in the delegated workspace. Prior captures and failed receipts remain preserved.

Actual Linux OS/GPU candidate pixels remain unverified; Linux input conventions
on the Mac do not establish that parity. Existing pure-RTL native hit mapping
is preserved. Scrolling scope is closed without further native-feel changes.
Paperbark caret approval, code-block candidate and separate release/resource
ownership are preserved. Memory resumed at its original cutoff. No merge,
push, tag, publication, security-setting or work-computer action occurred.

---

### 679 — preserve list ownership when inserting fenced code

🔵 OWED — integration approval. Verified frozen
candidate `codex/679-list-code-insertion` at `fb6566fd` emits parser-checked fences
inside list items, preserves forward/reversed selection, source bytes and one-step
undo, and safely refuses ambiguous contexts. Independent review found no remaining
issues, including parity review of the source-health-only refactor.

The targeted fast workflow passed Clippy/source audits, 11 audit laws and 108
formatting laws. Compiled mutation restoring column-zero wrapping failed the
headline list-preservation law; restored source passed again. Quoted/tabbed/lazy
list contexts, exact no-separator empty tasks and invalid fences are named
lossless refusals. Two full-gate attempts stopped in source health: ordinary lint
repairs and cohesive extraction cleared those findings; the orchestrator lowered
the formatter mark from 367 to its actual 337 lines. Full composed verification
passed clean `fb6566fd57a36d0c22c5f4271762723f96aca417`: native all-targets receipt
covers both conventions, full menu-bar arm, 5,370 unit tests across six shards and
18 integration targets; all 16 wasm runtime tests passed; debug/release sidecars
match for all eight action families. Retained gate session 81209 ended with rc=0.
Source and logs remain preserved for review. No merge, push, tag or publication
was performed.

---

### 582 — review the landed Kite depth and motion

🔵 OWED — live motion review. Static appearance was already accepted by the user.
The session
“Find the right way to run awl” completed the depth fade at `4f9fd3fb`, following
`0d5388d6` and `5d3d4249`, with native/wasm verification, regression mutation
proof and five native captures. Later integrated main and the test-pipeline
candidate also passed their required gates; no verification worker is still owed.

Present the current bundled app for several-minute motion comfort,
dwell/transit/settle, pause/focus freeze and Reduce Motion. Static density near
page edges and the far core needs no repeated approval.
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

### 666 — finish live Paperbark caret comfort review

🔵 OWED — live caret motion and comfort only. The user explicitly accepted the
current Paperbark caret proportions in native captures at `7d2aa7c2` on 2026-10-07.
The shared grapheme-ink adaptive caret (`bee5ec68`, integrated at `7f554e63`),
rounded padding and tighter top clearance are implemented; automated ink bounds,
rendered-pixel bounds and mutation evidence remain valid within their receipts.
Source `codex/666-paperbark-caret` at `f548f9b8` remains a recovery reference.

Finish physical caret movement and comfort in an identified bundled release app,
including spaces, empty lines, headings, zoom and 1x/2x DPI. Preserve stable row
height, glyph visibility and shared metrics ownership. The accepted visual
proportions do not establish physical keyboard/IME acceptance or live motion.

---

Use ACCESSIBILITY.md for current acceptance and deferred work; resolved 584/626
needs no further confirmation sitting.

## Latest recorded verification

- **Current product code `03f5c22fe6ee191bbedca3b1c32f749a5669bb5d`:** frozen
  `verify.sh full` passed native health and the complete 5,300-unit roster across
  six shards (5,275 passed/25 ignored in each convention), the full opposite-menu
  roster, 18 integration targets, wasm
  runtime and all eight debug/release parity families. Files cloud transport has
  identical stable patch IDs; native caret captures match the reviewed pixels.
  Integration repairs retained typed pixel ownership and adapted the Diagonal
  backing law; the punctuation ink oracle now uses a matched blank background,
  preserving its half-ink floor and swallowed-glyph control. Required hosted CI
  is pending the authorized publication. Physical input/IME and human motion
  acceptance remain open. The queue-only reconciliation changes no product code.
- **Earlier product code `7f554e63e5fa318370f0ab89b4cb171b09df1f19`:** the
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
continue to name their actual tested commits; the new receipt names `03f5c22f`.

## Remaining execution and acceptance

Assessment only: no additional journey suite or release rehearsal was launched.
The code gate above is complete; do not repeat it merely to assess this board.

| Remaining work | Available executor evidence | What still prevents closure |
| --- | --- | --- |
| 657 Files; 659 Settings/Themes | Seeded headless-App/release captures can check composition, geometry, focus state and cancellation with the existing built candidate | An awake, unlocked Mac for actual browsing, chooser, focus and theme-switching journeys; human appearance judgment |
| 582 Kite; 666 Paperbark | Static appearance is accepted, including current Paperbark/CJK captures and Brolga/Gumtree visual decisions | An awake, unlocked Mac for several-minute motion comfort, pause/focus/Reduce Motion and physical caret movement |
| 658 physical query selection; 664 keyboard ingress; 665 Japanese IME | Existing route/App laws and diagnostic instrumentation are complete; disposable fixtures can be prepared | Physical Cmd-A/Backspace and Japanese composition in the focused Files query; unlocked desktop and supported automation permissions for comparing injected versus physical delivery |
| 661 external browser paste | Static release fixtures and bulk-insertion receipts can be inspected | Genuine external OS clipboard delivery in Chrome, Safari and Firefox on an interactive desktop; record versions and keep field/document ownership distinct |
| 662 macOS packaging | GitHub can run a credentialed nonpublishing rehearsal and inspect universal/signature/notarization/staple/Gatekeeper, compressed sizes and checksums; this is not blocked by the local screen lock | Actual hosted credential availability/operation and final DMG results remain unverified; a real unlocked Mac must launch the mounted app |
| Linux desktop obligations | Hosted Linux CI is green and nonpublishing artifact checks can run remotely | A real x86_64 Linux desktop/operator for both package forms, launcher/FUSE, drawn-menu Export, and AT-SPI/Orca with audio and a person |
| 663 release cut | The checklist and final public payload can be prepared after the preceding proof | Remaining acceptance and packaging results, then the user's explicit instruction for a specific tag/release; website deployment requires its own explicit instruction |

The smallest next user steps are to unlock and keep the Mac awake for one seeded
review session and physically try Files Cmd-A/Backspace and Japanese input.
That same session can cover browser paste and motion. The four supplied visual
decisions are accepted; fresh captures and checks are still required for the new
Choose theme heading and Filter themes… placeholder before closing their review. Linux acceptance needs a Linux desktop/operator; it
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
