# awl — live build queue

> Open work only. Remove an item when it lands; closed decisions and findings
> remain in `git log -p -- .orchestrator/queue.md`. Execution protocol lives in
> `.orchestrator/README.md`.

## Open build and design tasks

**2 open numbered tasks.** Ready: 640. User decision: 579.
Outstanding review of landed work and hardware checks are listed separately below.

### 640 — integrated navigation/coherence acceptance and documentation (user approval, 2026-09-10)

🟡 DEPENDS ON 637–639 and their relevant 628/589 integration — queue only, not dispatched.

Build: review the combined native experience as one journey, remove obsolete parallel
entry points/contradictory teaching, and update the existing contracts, GUIDE,
Welcome/tour, keybinding reference and accessibility documentation to verified behavior.
Keep stable user config compatibility deliberate; no silent binding migrations.
The approved study is a design reference, not a shipped-behavior or performance receipt.

Acceptance journey: no-document/Welcome → choose writing root → Files → enter
subfolder → search/clear → open/edit → switch open documents → reopen/cancel → New
document via button and actual shortcut → Move/Save a Copy/Export in hermetic paths →
folder-content search → Settings → world preview/cancel → History compare/cancel.
Include two roots, duplicate names, unsaved edits, unavailable/empty folders, narrow
windows, and focus moved away from a still-selected row. Verify that path ownership,
buffer identity and save behavior remain understandable throughout. Include a Mac
menu-key-equivalent journey and real Linux fallback coverage; headless keys alone
do not certify either. Preserve the screen-lock checks at both ends of live runs.

Verify: follow docs/verification.md: cheap/targeted checks in each owning item, outcome
audits and mutation proofs on the integrated candidate, then one full native gate
and web smoke after commit/freeze. Use seeded captures, world/geometry/DPI coverage,
pixel presence and legibility, and the standing five-shot vision smoke. Validate
supported web behavior without claiming desktop chooser parity. Distinguish live
feel/taste still owed to the user from mechanically proven state and geometry.
Do not dispatch or claim implementation merely because this acceptance work is queued.

---

### 579 — software rendering: profiled, and the answer is a product call (investigation complete, 2026-09-09)

🟢 **PROFILED — awaiting the user's decision. No code change is proposed and none should be
until this is answered.**

The profile settles the engineering question. On the cited configuration — arm64, Debian 12,
Mesa 22.3.6, `llvmpipe (LLVM 15.0.6, 128 bits)`, `PHYSICAL_DEVICE_TYPE_CPU`, reproduced in a
pre-existing rig rather than a new one — every sample at every contention level has the same
shape:

```
   queue.submit + device.poll |  81.296 ms | 98.9%
   22 other CPU-side stages   |   ~0.7 ms  | <1%
   TOTAL (median frame)       |  82.164 ms
```

**There is no hot spot in awl's code.** Text shaping, layout, ornaments, table grid, chrome,
spell squiggles and render encode together cost under 1ms per frame, under 1.5% of the frame
even in the cleanest run. Over 99% sits inside Mesa's own rasterisation of already-encoded draw
calls. Document size barely matters — 1943 lines and 124 lines cost nearly the same — which is
the O(visible) principle holding.

**The item's own per-world spread did NOT reproduce.** The cited 82→184ms range across worlds
became four worlds within 2ms of each other at ~83ms, including the originally cited fastest
and slowest. The lane could not tell whether the original spread is contention-sensitive or
whether its own best window (load ~45-60, never this host's ~5 idle) flattened real differences
toward a floor. Named as an open gap rather than resolved.

⚠️ The orchestrator told that lane it owned the measurement window and then ran a gate on top
of it. The order-of-magnitude finding survives that easily — the signal is 99% against 1% — but
the per-world question is exactly the kind a contended host destroys, and it should be re-asked
on a genuinely idle machine before anyone concludes the spread was imaginary.

**Not measurable from here:** CI's x86_64 lavapipe. This host is arm64, and a qemu-emulated
x86_64 container would add emulation overhead indistinguishable from driver cost. Every number
above is the arm64/Mesa-22.3.6 axis only.

**THE DECISION, which is the user's:**

1. **Documented non-target.** A line in RELEASING.md/WEB.md naming software rendering as
   unsupported, stating what a person actually sees — roughly 5-12 fps at this canvas size,
   usable for reading and light editing, visibly laggy while typing or scrolling — and pointing
   at a working GPU driver. No code.
2. **Supportable with named work.** A software-adapter-detected degraded mode: smaller internal
   canvas, simplified backgrounds, fewer glyphs shaped. New mechanism, scoped as future work.

The lane leans to (1) and so does this board: closing an 80-180ms gap needs a different render
strategy, not a fix, and "more machinery for one degraded case" is the direction PHILOSOPHY.md
leans away from. But a Linux user on a VM, a remote desktop, or a machine with no working
Vulkan driver lands here, so it is a product-shape question and not an engineering one.

Leftover: Docker volumes `awl579-cargo-registry` and `awl579-target` hold the built arm64 rig
for a clean re-measurement without repaying the build.

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

The latest recorded native/wasm baseline is **`4e225355`** (633 and the aggregate
perf documentation), already on main:

```text
native-gate-receipt commit=4e225355 health=pass:304s conventions=mac,linux scope=all-targets
  menubar=full:on unit_tests=5123 unit_shards=6 integration_targets=18
web-smoke: OK
```

This is the original baseline receipt, not verification of later code. Subsequent
queue/policy-only commits use diff/link checks under `docs/verification.md`.
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
