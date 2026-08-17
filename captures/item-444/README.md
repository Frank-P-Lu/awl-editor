# item 444 — working-set capture set

`sh captures/item-444/shoot.sh` from the repo root regenerates the shots.

Hermetic: the sandbox is seeded from `fixture/` alone, through `--seed-tree`,
with an explicit `--config` and `--root` — never the ambient project or the
ambient config — so nothing here photographs a real directory. The PNGs and
their sidecars are scratch and are not committed; the fixture and the script
are, so the set survives the worktree that produced it.

| shot | what it shows |
| --- | --- |
| `one-file.png` | one open file: no stack, `buffers.files == []`, `active_index: null` |
| `three-files.png` | three open, stable open order, the nested one reading `journal/field-notes.md`, `active_index: 2` |
| `three-files-switched.png` | the same three after switching back to the first: `files` unchanged, `active_index: 0` |
