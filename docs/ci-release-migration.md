# CI and cloud release migration

Baseline: `c69f469f50a8d633e7c3c526071a26d5ffd92ba1`. No editor or icon changes.

## Coverage placement

| Existing check / dependency | Consolidated owner | Gate and migration effect |
| --- | --- | --- |
| Linux complete native gate: every target, both conventions, canary, menu-bar arms, partition audit and frozen-source receipt | Nightly and pre-release `linux` | Still blocking and unfiltered; moved out of per-change CI |
| Linux editing, Unicode, undo, persistence and core integration correctness | Parallel per-change `linux-correctness` lanes, both conventions | All native targets compile; complete non-render unit suites and every integration target except the three explicit deep matrices execute |
| Formatting, strict Clippy, source/ownership audits, tooling laws and unused dependencies | Per-change `linux-health`; full gate also owns health | Blocking; no duplicate standalone health before full gate |
| Dependency license/advisory/source policy | Per-change Linux health and nightly full Linux | Blocking, same policy owner and pinned checker |
| Exhaustive caret pixel matrices and full alternate menu-off unit suite | Nightly/pre-release Linux | Blocking; two complete headline pixel regressions and focused menu laws stay per change |
| Mac native cfg Clippy and every non-render test, both conventions | Per-change `mac` | Blocking; unchanged platform coverage |
| Hosted Mac render family and wedge watchdog | Nightly `mac-render-tests` | Existing tolerated failure remains explicit; no claim to solve virtualized Metal |
| AT-SPI wire structure/liveness and watchdog | Nightly `atspi` | Existing tolerated failure remains explicit; no human screen-reader claim |
| Wasm build, test compilation and Node execution | Per-change `web` | Blocking; missing runtime runner fails rather than silently skipping |
| Trunk release web distribution and artifact | Nightly `web-bundle` | Blocking; removed from fast CI; existing release-rehearsal web zip retained |
| Eight-family debug/release state parity | Main push; nightly; release Linux packages | Blocking; existing push behavior preserved |
| Short hosted Mac live GPU contract | Per-change `mac-live-probe` | Main/manual and reusable release calls; PR skip preserved |
| All worlds, both DPIs and 120-second live launches on Linux and Mac | Nightly/pre-release `journeys` | Blocking; no narrowed world roster or assertion changes |
| Mac arm64 and Intel build, signing/notary/staple, pinned signature/team/source/version/architecture, mounted final-app launches, receipts, size and hashes | Release `mac` | Hosted path for manual rehearsal/publication; no personal Mac dependency |
| Exact staged Mac downloads and independent validation | Release `mac-staged` | Explicit manual fallback rehearsal and legacy tag release retained |
| Ubuntu 22.04 glibc floor, Linux tarball and AppImage, licenses, final-package live launches | Release `linux` | Blocking; unchanged package builders and actual entry points |
| Producer hashes, strict public size cap, exactly four downloads, SHA256SUMS | `prepare-release`, then `publish` | Publication downloads the prepared artifact, rechecks hashes, uploads unchanged files |
| Web/site deployment | Existing manual `deploy-web.yml` | Separate user decision; no deploy run by this migration |

The release workflow calls reusable CI and the deeper suite for its own source
SHA. It does not infer readiness from another branch's green status or depend on
branch protection. Package jobs wait for both suites. There is no new tolerated
failure. The original two diagnostic jobs keep their existing tolerances.

The Linux fast suite has independent health and convention lanes, followed by a
blocking aggregate. Both convention lanes run all unit tests outside `render::tests`
and all integration targets except three explicitly deferred exhaustive matrices.
The existing Bombora asterisk and Bowerbird colour-flip pixel tests still execute in
both lanes, with their full assertions, punctuation roster, scales and caret forms.
Focused menu-off laws run per change; the full alternate menu-off unit pass runs in
nightly/prerelease. No assertions or tests are deleted or newly ignored. New native
integration targets join fast CI by default, and required target/smoke absence fails.

The baseline hosted run [39131961640](https://github.com/Frank-P-Lu/awl-editor/actions/runs/39131961640)
used an exact Rust-cache hit but took 39m18s on Linux: three serial unit passes
18m14s, the three caret matrices across both conventions 11m16s, health 4m46s, and
initial test compilation 2m38s. The revised design targets a warm Linux critical
path below 15 minutes by using separate runners for both conventions and health,
while keeping meaningful per-change correctness and moving only explicit deep
coverage. This is a target, not proof of cold-cache or queue-time performance.

Measure the final exact PR head after every change to the candidate. Record workflow
creation/start, each lane start/end, cache hit/miss, compile and execution phases,
and aggregate completion. Report both elapsed Linux feedback time (including queue
and aggregation) and execution time. A green run over 15 minutes misses the latency
target and needs investigation. Do not replace measured time with a parallelization
estimate. Mac remains an independent platform gate.

All Linux lanes restore the prior `linux` dependency cache. Only the Linux-convention
lane writes it, preventing health-only metadata from winning a cold-cache save race.
Policy tools have a separate versioned cache. This reuses caches without treating
warm-cache evidence as a cold-build guarantee.

The three exhaustive caret matrices and the full menu-off cross-suite remain
mandatory in nightly and every prerelease/release invocation of extended verification.
The tradeoff is that a regression unique to an unsampled world/DPI/menu-off
combination can be found later than PR CI; packaging still waits for deep coverage.

## Release intent and immutable bytes

`dry_run=true` is the default and cannot publish, even when signing is requested.
Manual publication requires `dry_run=false`, runs from `main`, and selects
`channel=preview` (prerelease) or `channel=stable` (full/latest). Both use identical
hosted package machinery. A version already released cannot be overwritten, and
an existing tag must resolve to the tested source SHA. Runs serialize globally.

A publication run builds once, finalizes signatures/notarization before packaging,
launches the actual packages, and binds those final bytes to producer checksums.
Payload preparation verifies them, copies them without transformation, and creates
one checked manifest. Publication rechecks the downloaded payload and performs no
build/sign/staple/repack operation. Mac receipt and signed presentation/hash
checks are unchanged. Unsigned rehearsal success cannot establish signed readiness.

Legacy `v*` publishing still uses staged signed DMGs during migration. It now also
checks tag/Cargo version agreement and exact-source CI. Its historical channel
mapping (0.x preview, 1.x+ stable) remains; new manual publishing uses explicit
selection. Do not remove this fallback until hosted signing is demonstrated.

## Evidence and setup boundaries

Read-only GitHub evidence confirms baseline unsigned rehearsal
[39109414428](https://github.com/Frank-P-Lu/awl-editor/actions/runs/39109414428)
completed successfully. Its signing/notarization steps and publication were skipped.
That run predates this consolidation and is not evidence for the changed workflow.

Current GitHub secret inventory could not be verified in this cloud session:
`gh secret list --repo Frank-P-Lu/awl-editor` returned `Forbidden`. The available
GitHub connector excludes secrets endpoints. The prior report of only
`FLY_API_TOKEN` is not a verified current inventory. No secret values were read.

An authorized owner must separately approve and provision/confirm these existing
workflow inputs before hosted signed rehearsal: `MACOS_CERT_P12`,
`MACOS_CERT_PASSWORD`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER`, `APPLE_API_KEY_B64`.
A valid Developer ID certificate/private key for the pinned team and notarization
API authorization are needed. Local-only keys do not satisfy hosted readiness.
No keys, secrets, access, environments or branch protection were configured here.

## Acceptance and retirement

1. Review the draft diff, coverage map, regression laws and local verification.
2. Run the exact branch's CI and unsigned nonpublishing hosted rehearsal. Record
   durations, both final DMG launches, Linux package launches and payload hashes.
3. Obtain separate action-time approval for any credential transfer, creation,
   configuration or security-setting change. Resolve the inventory uncertainty.
4. Run `dry_run=true` with `credentialed_macos_rehearsal=true`. Require both native
   signed DMG checks, receipts, presentation, hashes and the complete payload.
5. Finish the real-device/human journeys in RELEASING.md, WEB.md and ACCESSIBILITY.md.
   Hosted correctness does not establish physical IME or interactive performance.
6. Obtain explicit publication authorization for a named version/channel. Publish
   via the hosted workflow and verify downloaded assets against its manifest.
7. Only after the replacement succeeds, separately retire the legacy tag/local
   staging route and obsolete documentation. No fallback removal in this change.

Branch protection/rulesets or protected signing-environment policy are separate
security decisions; this change does not alter them. The release's explicit
correctness dependencies work without such settings. Never infer publication or
credential approval from a green rehearsal.
