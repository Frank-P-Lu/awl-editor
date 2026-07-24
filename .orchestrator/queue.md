# awl — live build queue

> Live execution state only. Completed and superseded work is in git history
> (`git log -p .orchestrator/queue.md`). Protocol, claiming, worktrees, and
> execution hygiene live in `.orchestrator/README.md`.

## Ready — current user-visible wave

76. **One active folder; ordinary documents; retire the Notes desk.** Replace the notes-specific navigation/document model with one coherent active-folder context. Launch precedence is one law: an explicit file/folder/`--root` (including `awl .` and an OS open-file request) wins; an argument-free terminal or desktop launch restores the last active folder + document; a first launch with nothing remembered opens the configured `default_folder` (`~/notes` initially). Bare `awl` therefore resumes, while terminal “use this directory” is explicitly `awl .`; any successful context change, regardless of entry door, becomes the next remembered context. Persist that context through one owner so sticky project state and session restore cannot disagree. Rename the user/config concept from `notes_root` to `default_folder`; this is pre-release, so retire the old key rather than carrying a compatibility layer. Remove the `Notes`/`NotesFlip` command, two-desk return memory, and lasting note identity. Rename `New note` to **New document**: Cmd-N creates a fresh Markdown document in the active folder, the first meaningful line supplies its filename once on first material save, and later title edits never rename it automatically. Thereafter it is an ordinary file; Rename/Duplicate/Move are generic file verbs. Make `Last file` restore the complete previous location (folder + buffer/view), not a notes-specific return path. Do not add a Library/sidebar, per-folder desk stack, or terminal/desktop mode. **High-risk ownership round: Opus plans the state/launch migration; Sonnet implements; targeted Opus verification attacks explicit-vs-restored launch precedence, daemon handoff, first-run fallback, A→B→A folder/file/view restoration, one-shot naming, generic file verbs, and the absence of Notes rows/state; full native + wasm gates.**

## Ready — shared ownership and performance

## Timed — not blocked

20. **Pre-tag taste pass.** At the user’s explicit tag/release start, the implementation/release owner generates one current world screenshot export, then Fable judges only those images for per-world bullets, squiggle size/baseline including Bilby, dash padding, and Saltpan font outcomes; Fable never implements or edits. Ordinary pushes do not trigger it.

24. **Release-adjacent user-facing docs refresh.** After the current user-visible wave settles and before release preparation, update GUIDE, welcome/tour, and site guide for the current product, chords, and features. Matter-of-fact voice; facts verified. Site copy may change; deployment remains separately user-gated.

## Parked — explicit gate or future design

- **Export save-dialog scope:** macOS + Linux, one live-only cross-platform seam; capture uses an explicit path. Decided, not scheduled.
- **Per-world living-band choreography:** audition TwoShape/Slam/Soft against Morph; live feel is the oracle. Needs a design session.
- **Per-world copy-pulse differentiation:** possible future motion tweak; needs a design session.
- **Site deployment:** only on the user’s explicit word.

## Monitoring — non-blocking

- **Hands-on checks still useful:** Dawn/Bilby world feel; writer-diff panel/Tab + zoom readout; phantom image resize handle; upward scrolling past images in release; right-click Add-to-dictionary summon; 2px Wagtail stipple taste.
- **GPU memory:** no action unless the 6 GB symptom recurs; then probe the live surface with the window foregrounded.

## Release blockers and reminders

- App icon.
- Dictionary/font/license notices plus code copyright/NOTICE review.
- Apple signing secrets and Fly deployment token; see `RELEASING.md`.
- Tags and releases require the user’s explicit word. A dry run may precede them.
