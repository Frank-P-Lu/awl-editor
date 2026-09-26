# Capture reach

Read before promising capture evidence or writing a Verify clause. Choose the
smallest real driver that reaches the action and state in question.
[CAPTURE.md](../CAPTURE.md) gives commands and the checked sidecar field summary;
[verification policy](verification.md) defines evidence obligations.

## The three tiers

| Tier | What runs | Driver | Evidence |
| --- | --- | --- | --- |
| 1: shared core | Real keymap and `actions::apply_transition` | `--screenshot --keys`, strict replay, storyboards, `ReplaySession` | Sidecar state/layout and PNG pixels |
| 2: headless App | Real `App::apply` and live effect interpretation | `--screenshot-app`, `App::press_spec_headless`, `--semantic-json` | App assertions, sidecar/PNG, or semantic JSON without a GPU |
| 3: window, surface, event loop | Native callbacks and presentation | Real window, `--live-script`, `--soak-gpu`, platform smoke | Live probe evidence and human confirmation |

A tier-1 capture can skip App-owned work. Strict replay rejects Unsupported effects
and grants only its explicitly isolated filesystem capability; it does not turn
replay into an App. A tier-2 capture uses a hermetic scenario filesystem, executes
App effects, and reports `driver: "live-app"` with no replay skips. It still has no
window/surface/event loop. “No skips” does not establish tier-3 coverage.

## Choose by the actual interaction

| Subject | Required distinction |
| --- | --- |
| Buffer edits, selection, search, zoom, folds, overlay Journey | Shared-core replay exercises these owners. Confirm the chosen entry door is reachable too. |
| Keymap picks, rebinding, buffer registry | Use the headless App for App-owned state. A command twin's success does not prove the Settings-row door. |
| Setting toggle/value/path effects | Some promote under isolated filesystem replay; others, including keymap rebuilds, remain Unsupported. Read the effect table. |
| Project change | Applied replay routes must resync root, workspace, corpus, and later steps through shared location/fold owners. Files' native folder chooser is a separate live-only door. |
| History comparison or an existing external-change conflict | Use `--screenshot-app --seed-data DIR` with an explicit prepared store. Chords cannot create an outside writer mid-run. |
| Semantic/accessibility state | `--semantic-json` reads the App snapshot without a GPU; it does not prove native assistive-technology delivery or pixel appearance. |
| Native menu equivalents, OS input methods, pointer gestures | Key replay cannot establish those dispatch paths. Use the relevant App-level test seam or a real window as the input census specifies. |
| Live theme switching, redraw timing, resizing, compositor behavior | Rebuilt single frames are insufficient; use the runtime seam and live evidence. |

Use the checked effect table below and [effect details](capture/reach-effects.md)
for capability-dependent exceptions, command-versus-row behavior, and history fixtures.
Read [the live input census](capture/reach-live.md) for native menu, IME, drag,
chooser, recent-project, personal-dictionary, and CJK boundaries. Read
[geometry reach](capture/reach-geometry.md) for canvas/DPI and the exact picker and
find/replace geometry exposed by each driver.

## Fixtures and geometry

Ordinary capture reads ambient config and stores unless explicitly isolated.
Use explicit config and seeded roots; keep capture outputs outside any directory
the picker lists. `--seed-data` recursively seeds awl's data root, preserving
relative paths, sorting entries, skipping symlinks, and refusing more than 256
files or 4 MiB. It is accepted only by hermetic doors. This reaches recovery,
scratch/session records, and nested history stores without reading private state.

For a conflict, assert `gutter.changed`, `overlay.preview_view`, and the previewed
text; assert the affordance's appearance from PNG pixels. `notice` is also captured,
but its transient live lifetime does not replace the persistent conflict state.

Both ordinary and App captures honor canvas size and DPI. Record `font.zoom`:
persisted config or explicit ordinary-capture overrides can change it. Equal
canvas sizes do not make core replay an oracle for App-owned launch state.

Settings workspace entry (`OpenSettingsMenu`) and Journey transitions live in the
shared core; `OpenSettings` is the separate config-buffer action. Settings child
return is also reachable through the App driver. Enter the content
region before opening a child: Escape from the category region closes Settings,
whereas Escape from its child restores the parent's row and focus. Test the actual
chords rather than calling the effect interpreter directly; retain the existing
Settings roster and anti-vacuity laws.

## Remaining limits

`--screenshot-app` captures one final frame, not a per-step App storyboard. It
refuses replay-only state overrides such as `--sel`, `--scroll`, `--search`, and
`--preedit`, and `--default-folder`; drive App state through real input instead.
It is not a strict-replay mode. Storyboards remain core replay.

A seeded conflict can exist at launch; no capture chord creates an external disk
change during the run. A headless pipeline does not observe OS presentation
between frames, nor every live cache/invalidation path. Appearance needs pixels;
timing and taste retain live human confirmation.

Unsupported means the chosen driver lacks a capability, not that the behavior is
unobservable in principle. Extend the harness toward the real owner when needed.

## Replay effect table

These classifications are for ordinary core replay without an isolated filesystem
capability. The production-classifier law reads this table at this path. Applied
is not a claim about OS presentation; Intercepted handoffs are observed but not
performed. Unsupported effects require another driver or an explicit capability.

<!-- reach-table:begin -->
| Effect | Class |
| --- | --- |
| `add_to_dictionary` | Unsupported |
| `check_for_updates` | Intercepted |
| `clipboard_paste_image` | Intercepted |
| `clipboard_write` | Intercepted |
| `copy_pulse` | Applied |
| `daemon_notify_finished` | Intercepted |
| `delete_squash` | Applied |
| `download_file` | Intercepted |
| `duplicate_note` | Unsupported |
| `edit_streak` | Applied |
| `export` | Intercepted |
| `finish_buffer` | Unsupported |
| `finish_save` | Unsupported |
| `flush_writing_streaks` | Applied |
| `follow_link` | Intercepted |
| `forget_user_word` | Unsupported |
| `gulp` | Applied |
| `insert_date` | Applied |
| `jump_to_line` | Applied |
| `keep_version` | Unsupported |
| `last_buffer` | Unsupported |
| `line_land` | Applied |
| `new_document` | Applied |
| `none` | Applied |
| `notice_clear` | Applied |
| `notice_sticky` | Applied |
| `notice_toast` | Applied |
| `open_file_chooser` | Unsupported |
| `open_folder_chooser` | Unsupported |
| `open_path_at_line` | Applied |
| `open_settings` | Applied |
| `overlay_accept:Assets` | Unsupported |
| `overlay_accept:Browse` | Unsupported |
| `overlay_accept:Caret` | Applied |
| `overlay_accept:CjkLang` | Applied |
| `overlay_accept:Command` | Unsupported |
| `overlay_accept:Conflict` | Unsupported |
| `overlay_accept:Context` | Unsupported |
| `overlay_accept:Credits` | Unsupported |
| `overlay_accept:Date` | Applied |
| `overlay_accept:Dictionary` | Applied |
| `overlay_accept:ExportDest` | Unsupported |
| `overlay_accept:Goto` | Applied |
| `overlay_accept:History` | Applied |
| `overlay_accept:InsertLink` | Unsupported |
| `overlay_accept:KeepName` | Unsupported |
| `overlay_accept:Keybindings` | Unsupported |
| `overlay_accept:Keymap` | Unsupported |
| `overlay_accept:MoveDest` | Unsupported |
| `overlay_accept:Project` | Applied |
| `overlay_accept:ProjectBrowse` | Unsupported |
| `overlay_accept:Rename` | Unsupported |
| `overlay_accept:SearchFolder` | Unsupported |
| `overlay_accept:Settings` | Unsupported |
| `overlay_accept:Spell` | Unsupported |
| `overlay_accept:TableDims` | Unsupported |
| `overlay_accept:Theme` | Applied |
| `overlay_accept:UserWords` | Unsupported |
| `persist_caret_mode` | Applied |
| `persist_menu_bar` | Applied |
| `persist_outline` | Applied |
| `persist_page_mode` | Applied |
| `persist_page_reset` | Applied |
| `persist_page_width` | Applied |
| `persist_spellcheck` | Applied |
| `persist_typewriter` | Applied |
| `persist_writing_nits` | Applied |
| `quit` | Unsupported |
| `rebind_commit` | Unsupported |
| `rebind_reset` | Unsupported |
| `recoil` | Applied |
| `redraw` | Applied |
| `rename_note_commit` | Unsupported |
| `report_problem` | Intercepted |
| `reshape` | Applied |
| `resolve_keep_mine` | Unsupported |
| `resolve_take_theirs` | Unsupported |
| `reveal_in_file_manager` | Intercepted |
| `review_external_change` | Unsupported |
| `run_action` | Applied |
| `save` | Unsupported |
| `setting_path_pick` | Unsupported |
| `setting_range_step` | Applied |
| `setting_toggle` | Unsupported |
| `setting_value_commit` | Unsupported |
| `show_about` | Applied |
| `sync_view` | Applied |
| `trash_asset` | Intercepted |
| `type_impact` | Applied |
| `zoom_changed` | Applied |
<!-- reach-table:end -->
