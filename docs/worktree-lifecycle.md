# Worktree locations and retirement

Create all manually controlled checkouts, including detached performance baselines,
through `python3 scripts/worktree.py create <task>-<purpose> --owner <owner>
--purpose <purpose>`. The default is detached at main; pass `--ref <ref>` for a
baseline or `--branch codex/<name>` to create a development branch. The helper
resolves the main checkout and places work under its `.worktrees/` directory.
Do not create ad hoc checkouts in temporary directories. Disposable test fixtures
that remove themselves on exit are exempt. Tool-managed locations are exceptions:
register them using `state`; do not move live checkouts to satisfy the convention.

The local registry lives in Git's common directory and is shared across worktrees;
it records resource lifecycle, not task status (the board remains that owner).
Use `python3 scripts/worktree.py state <path> <state> --owner <owner> --reason <why>`.
States are active, awaiting-review, reusable, and retired. Awaiting-review requires
`--review <location>` identifying the result and any runnable dependencies. Preserve
those dependencies until review is resolved. Unknown or abandoned ownership stays
unknown until investigated; neither age nor a merged branch proves retirement.

Mark retired only after confirming the owning agent and all children/builds have
stopped and required source and review evidence are preserved outside the disposable
checkout. Supply `--stopped --preserved` to attest those checks. Do not restart a
retired lane: change it back to active before using it. Registry mutations and
retirement are serialized; these attestations are still required because direct
shell commands and external agents do not participate in that lock.

`scripts/sweep.sh --list` inventories registered worktrees wherever they live,
including legacy temporary locations, with size, state, location and keep/removal
reason. `scripts/sweep.sh --retired` previews retirement regardless of age;
`--retired --apply` performs it after the existing fleet-idle guard passes.
`DRY_RUN=1` always previews. Process inspection failure refuses application.
Unknown, active, reusable and awaiting-review worktrees are kept. Retired worktrees
must be unlocked, clean, have HEAD preserved on main, and contain no ignored files
outside a conventional non-symlink target directory. The sweep removes that build
output and then uses ordinary Git worktree removal, never forced removal; branches
remain. Unmerged experimental commits must first be preserved on main or kept for
manual review. Legacy worktrees are inventoried, never automatically reclassified.

The existing numeric-day mode remains a cache-trimming tool for retained checkouts,
not a lifecycle decision. Automatic disk preflight remains limited to its caller;
it never retires siblings. Do not run age-based sweeping on a checkout whose build
outputs are pinned for review. Review accumulated inventory between dispatch waves
and at handoff; report substantial leftovers and why they remain.
