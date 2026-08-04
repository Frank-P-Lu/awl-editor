#!/usr/bin/env python3
"""AT-SPI2 bridge-liveness and structure probe (item 252, CI-only).

Grades the bridge itself, not something adjacent to it (the four vacuous laws
in items 237/244/247/248 all made that mistake). Launches the real awl binary
under a real X display, connects to the AT-SPI2 accessibility bus as an
ordinary assistive-technology client would, and asserts that the tree
`SemanticSnapshot` intends is actually there:

  - the application registers on the bus at all (a broken/disabled adapter
    means this alone never appears — that is the mutation-proof case);
  - the editable multiline document (ROLE_ENTRY, EDITABLE, MULTI_LINE,
    eventually FOCUSED) as a descendant of the application node;
  - item 218's STABLE LINE RUNS as its children, one per rope line plus the
    trailing empty run, each ROLE_STATIC with the exact expected text — a
    monolithic single-node document (the pre-218 shape) fails this exact
    check, which is the point: this confirms the RUN-BASED tree specifically,
    not merely that *a* tree exists;
  - a real caret/selection: the initial caret at offset 0, and — driven by an
    actual Shift+Right keypress delivered through X11, not synthesized — a
    live AT-SPI selection afterwards.

ACTIVATION IS LAZY, AND THIS PROBE ACTS AS THE AT CLIENT THAT TRIGGERS IT.
Read straight from the vendored `accesskit_unix` 0.22.1 source
(`src/context.rs`): the adapter starts `Inactive` and only builds a tree once
its background thread observes `org.a11y.Status`'s `IsEnabled` property (at
`org.a11y.Bus`, path `/org/a11y/bus`, the session-bus object every real AT
client — Orca included — sets on startup) read as true, via zbus's
`receive_is_enabled_changed()` stream. That stream self-primes: zbus's
`PropertyStream` fires once on the interface's very first cached `GetAll`
(`zbus-5.18.0/src/proxy/mod.rs`'s `init()`/`update_cache()`), carrying
whatever the CURRENT value is — so setting `IsEnabled=true` BEFORE awl even
launches is enough, with no race against exactly when awl's own thread
subscribes. A run of this probe (30886162170) hit exactly this: awl started
cleanly (no panic, nothing to fix in awl) and simply never saw an AT ask for
it, which is the correctly-lazy behavior, not a bridge defect — so
`set_bus_enabled` below is not a workaround, it is the probe finally doing
the one thing a real screen reader does that a passive tree-walk never did.

NO ROLE_FRAME EXISTS ANYWHERE IN AWL'S TREE, AND THAT IS NOT A TIMING BUG —
an earlier version of this probe required one and failed within ~0.65s of
launch, which read as ambiguous (a real gap, or just too fast a check?) until
traced structurally, the same way the lazy-activation question was settled.
Confirmed from source, not timing: `accesskit_atspi_common` 0.19.1's
`add_node` (`adapter.rs:62`) only fires its window-registration path when
`is_root() && role() == Role::Window`, and its AT-SPI role mapping
(`node.rs`) only ever produces `AtspiRole::Frame` from `accesskit::Role::
Window` — nothing else synthesizes one. Awl's own tree root is built once,
at `src/app/semantic/projection.rs:172`
(`SemanticNode::new(ROOT_ID, SemanticRole::Application, "awl")`), which
`src/semantic/native.rs:261` maps to `accesskit::Role::Application`, never
`Role::Window`. So the real, permanent AT-SPI shape is Desktop ->
Application (accesskit_atspi_common's own synthetic per-process object,
`node.rs`'s `PlatformRoot`, always role Application) -> awl's own root
(role Application again, awl's tree) -> Document -> runs — genuinely no
Frame at any depth, not something a longer wait would ever find.

THAT IS A FACT ABOUT THE TREE, NOT A CLAIM THAT NO FRAME IS NEEDED — those
are two different questions and this probe answers only the first. Whether
AT-SPI/Orca can navigate an application that publishes no Frame is left
explicitly UNKNOWN here: this item's scope is bridge liveness and tree
structure, not the screen-reader experience, and settling it would take a
real Orca session — item 251's job, not this probe's. Not asserting
ROLE_FRAME is the correct probe behavior regardless of that answer (it
should never fail on a node the product was never going to publish) — but
that correction must not be read as "so a missing Frame is fine". It is
recorded as open, not resolved, in ACCESSIBILITY.md and item 252's landing
report, precisely so item 251 has something to check rather than an
assumption nobody wrote down.

THIS IS NOT ITEM 251. It says nothing about what a screen reader user would
hear or how navigation feels — only that the bridge is live and shaped right.
Item 251 still needs a human at a real Linux desktop running Orca.

Exits non-zero, with a message naming exactly what was missing, on any
mismatch or timeout. Exit 0 only when every assertion below passed.
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import time

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi, GLib  # noqa: E402

APP_TIMEOUT_S = 30.0
DOCUMENT_TIMEOUT_S = 10.0
FOCUS_TIMEOUT_S = 10.0
SELECTION_TIMEOUT_S = 10.0
POLL_S = 0.5
MAX_DEPTH = 12
# Every external call this probe makes (dbus-send, xdotool) gets an explicit,
# generous timeout. Run 30887479906's job hit its 20-minute CANCEL because an
# earlier version of set_bus_enabled used `Gio.DBusProxy.new_for_bus_sync` —
# a synchronous GI call with no timeout parameter at all — and something in
# this sandboxed dbus-run-session/Xvfb stack made it block indefinitely. A
# probe's own job is to never do that: every blocking call below is a
# subprocess with a hard, catchable ceiling, not a library call whose
# internal timeout behavior this script does not control. (The outer CI job
# also wraps the whole probe in scripts/ci-atspi-budget.sh as a second,
# independent backstop — belt and suspenders, not a substitute for either.)
SUBPROCESS_TIMEOUT_S = 10.0

FIXTURE_LINES = ["line one", "line two", "line three"]
# item 218's shape: one run per rope line, PLUS a trailing empty run for the
# implied empty line after the fixture's final newline.
EXPECTED_RUN_TEXT = [line + "\n" for line in FIXTURE_LINES] + [""]


# Set once by main(), before anything can fail, so `fail()` can ALWAYS dump
# awl's own output regardless of which assertion tripped. A probe that reports
# failure without the failure's own output is undiagnosable by construction —
# the earlier per-call-site `dump_awl_log()` calls missed most of the fail()
# sites below, and the one helper that existed swallowed its own read error
# silently (a bare `except OSError: pass` around a second `open()` of the same
# path) instead of ever printing anything. This replaces both with one
# unconditional path.
_AWL_LOG_PATH: str | None = None


def _dump_awl_log() -> None:
    if _AWL_LOG_PATH is None:
        return
    try:
        with open(_AWL_LOG_PATH, "rb") as f:
            raw = f.read()
    except OSError as exc:
        # Loud, not swallowed: a failed read is itself diagnostic information,
        # not a reason to print nothing.
        print(f"ATSPI-PROBE: could not read {_AWL_LOG_PATH}: {exc}", file=sys.stderr)
        return
    text = raw.decode("utf-8", errors="replace")
    lines = text.splitlines()
    tail = lines[-80:]
    print(
        f"----- awl output ({len(tail)} of {len(lines)} lines"
        f", {len(raw)} bytes total) -----",
        file=sys.stderr,
    )
    if not lines:
        print("(awl produced no captured stdout/stderr output at all)", file=sys.stderr)
    for line in tail:
        print(line, file=sys.stderr)
    print("------------------------------", file=sys.stderr)


def fail(msg: str) -> None:
    _dump_awl_log()
    print(f"ATSPI-PROBE FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def set_bus_enabled(value: bool) -> None:
    """Set org.a11y.Status.IsEnabled on the session bus's org.a11y.Bus object.

    This IS the AT-registration handshake, not a workaround for one: it is
    the exact property accesskit_unix's background thread subscribes to
    (`context.rs::run_event_loop`) to decide whether to build a tree at all.
    Real assistive technology (Orca included) sets this on startup; a probe
    that only walks the tree afterward never performs the one call that
    makes a correctly-lazy adapter do anything. `org.a11y.Bus` is D-Bus
    service-activatable (at-spi2-core ships its `.service` file), so this
    call also transparently starts `at-spi-bus-launcher` on first use if
    nothing has touched the service yet.

    Uses the `dbus-send` CLI (part of the `dbus` package already installed
    for `dbus-run-session`) under an explicit subprocess timeout, not
    `Gio.DBusProxy` — a first version used the synchronous GI proxy call,
    which takes no timeout parameter at all, and something in this sandboxed
    dbus-run-session/Xvfb stack made it block indefinitely (run 30887479906
    hit its job's 20-minute CANCEL this way). A bare `subprocess` call with a
    hard ceiling is something this script fully controls; a GI library call's
    internal blocking behavior is not.
    """
    variant = "true" if value else "false"
    try:
        subprocess.run(
            [
                "dbus-send",
                "--session",
                "--type=method_call",
                "--print-reply",
                "--dest=org.a11y.Bus",
                "/org/a11y/bus",
                "org.freedesktop.DBus.Properties.Set",
                "string:org.a11y.Status",
                "string:IsEnabled",
                f"variant:boolean:{variant}",
            ],
            capture_output=True,
            text=True,
            timeout=SUBPROCESS_TIMEOUT_S,
            check=True,
        )
    except subprocess.TimeoutExpired:
        fail(
            f"dbus-send Set IsEnabled={value} did not return within "
            f"{SUBPROCESS_TIMEOUT_S}s — the session bus or org.a11y.Bus "
            "service activation itself is wedged, not the AT-SPI bridge"
        )
    except subprocess.CalledProcessError as exc:
        fail(
            f"dbus-send Set IsEnabled={value} failed (exit {exc.returncode}): "
            f"{exc.stderr.strip()!r}"
        )


def run_xdotool(args: list[str], check: bool) -> subprocess.CompletedProcess:
    """A timeout-bounded xdotool call — see SUBPROCESS_TIMEOUT_S's doc."""
    try:
        return subprocess.run(
            ["xdotool", *args],
            capture_output=True,
            text=True,
            timeout=SUBPROCESS_TIMEOUT_S,
            check=check,
        )
    except subprocess.TimeoutExpired:
        fail(f"xdotool {' '.join(args)} did not return within {SUBPROCESS_TIMEOUT_S}s")
    except subprocess.CalledProcessError as exc:
        fail(f"xdotool {' '.join(args)} failed (exit {exc.returncode}): {exc.stderr.strip()!r}")


def bump_bus_enabled() -> None:
    """Force a genuine false->true edge, for the retry path.

    zbus's property-changed stream also fires on the first cached value
    (see the module docstring), so setting `True` once before awl launches
    is normally enough — but a plain `Set(True)` when the value is ALREADY
    true may not emit a `PropertiesChanged` signal at all (server-dependent
    on whether it compares old/new). A real edge is unambiguous either way.
    """
    set_bus_enabled(False)
    time.sleep(0.1)
    set_bus_enabled(True)


def find_by_pid(node, pid, depth=0):
    try:
        if node.get_process_id() == pid:
            return node
    except GLib.Error:
        pass
    if depth >= MAX_DEPTH:
        return None
    try:
        count = node.get_child_count()
    except GLib.Error:
        return None
    for i in range(count):
        try:
            child = node.get_child_at_index(i)
        except GLib.Error:
            continue
        if child is None:
            continue
        found = find_by_pid(child, pid, depth + 1)
        if found is not None:
            return found
    return None


def find_role(node, role, depth=0):
    if node.get_role() == role:
        return node
    if depth >= MAX_DEPTH:
        return None
    try:
        count = node.get_child_count()
    except GLib.Error:
        return None
    for i in range(count):
        try:
            child = node.get_child_at_index(i)
        except GLib.Error:
            continue
        if child is None:
            continue
        found = find_role(child, role, depth + 1)
        if found is not None:
            return found
    return None


def text_of(node) -> str:
    """The run's text, across whichever GI Text-interface shape is live."""
    try:
        count = node.get_character_count()
        return node.get_text(0, count)
    except (AttributeError, GLib.Error):
        pass
    iface = node.get_text_iface()
    count = iface.get_character_count()
    return iface.get_text(0, count)


def caret_offset_of(node) -> int:
    try:
        return node.get_caret_offset()
    except (AttributeError, GLib.Error):
        return node.get_text_iface().get_caret_offset()


def selection_of(node):
    """(n_selections, (start, end) | None) across whichever GI shape is live."""
    try:
        n = node.get_n_selections()
        rng = node.get_selection(0) if n > 0 else None
    except (AttributeError, GLib.Error):
        iface = node.get_text_iface()
        n = iface.get_n_selections()
        rng = iface.get_selection(0) if n > 0 else None
    if rng is None:
        return n, None
    if isinstance(rng, tuple):
        return n, rng
    return n, (rng.start_offset, rng.end_offset)


def main() -> None:
    if len(sys.argv) != 2:
        fail("usage: ci-atspi-probe.py <path-to-awl-binary>")
    binary = sys.argv[1]

    fixture_dir = tempfile.mkdtemp(prefix="awl-atspi-fixture-")
    fixture_path = os.path.join(fixture_dir, "probe.md")
    with open(fixture_path, "w", encoding="utf-8") as f:
        f.write("".join(line + "\n" for line in FIXTURE_LINES))

    global _AWL_LOG_PATH
    awl_log_path = os.path.join(fixture_dir, "awl.log")
    _AWL_LOG_PATH = awl_log_path
    awl_log = open(awl_log_path, "wb")

    # Enable accessibility on the session bus BEFORE awl exists at all. This
    # is the actual AT-registration handshake (see set_bus_enabled's doc),
    # not a delay: zbus's property-changed stream self-primes from whatever
    # value IsEnabled already holds, so doing this first removes any race
    # against exactly when awl's own background thread subscribes.
    print("ATSPI-PROBE: enabling org.a11y.Status.IsEnabled before awl starts")
    set_bus_enabled(True)

    print(f"ATSPI-PROBE: launching {binary} {fixture_path}")
    proc = subprocess.Popen(
        [binary, fixture_path],
        stdout=awl_log,
        stderr=subprocess.STDOUT,
        env=dict(os.environ),
    )

    try:
        rc = Atspi.init()
        if rc not in (0, 1):
            fail(f"Atspi.init() returned {rc}")

        desktop = Atspi.get_desktop(0)

        deadline = time.time() + APP_TIMEOUT_S
        app = None
        last_bump = time.time()
        while time.time() < deadline:
            if proc.poll() is not None:
                fail(
                    f"awl exited early with code {proc.returncode} before the "
                    "AT-SPI app node ever appeared"
                )
            app = find_by_pid(desktop, proc.pid)
            if app is not None:
                break
            # Retry path: force a genuine IsEnabled edge every few seconds,
            # in case the pre-launch `set_bus_enabled(True)` above raced
            # awl's own subscription in some way this probe's reading of the
            # accesskit_unix source did not anticipate. A real screen reader
            # only has to do this once; a probe that cannot assume it read
            # the timing right does it defensively instead of assuming
            # "still not there" means "broken".
            if time.time() - last_bump > 5.0:
                bump_bus_enabled()
                last_bump = time.time()
            time.sleep(POLL_S)
        if app is None:
            fail(
                f"no AT-SPI application for pid {proc.pid} appeared under the "
                f"desktop within {APP_TIMEOUT_S}s, despite setting and "
                "re-bumping org.a11y.Status.IsEnabled — the bridge never "
                "registered with the accessibility bus"
            )

        # No ROLE_FRAME lookup here — see the module docstring: awl's tree has
        # no accesskit::Role::Window anywhere, so no AT-SPI Frame exists at
        # any depth, confirmed structurally, not by timing. The document is
        # searched directly under the application node instead.
        #
        # Retried, not a single shot: finding `app` on the desktop and
        # RegisterInterfaces for every one of its descendants reaching the
        # bus are two different steps (adapter.rs's `register_interfaces` per
        # node vs. the AT-SPI Socket embedding), and an earlier run's ~0.65s
        # gap between "app found" and "document searched" was fast enough to
        # be a legitimate open question about whether this was racing that
        # propagation — closed here rather than left ambiguous again.
        document_deadline = time.time() + DOCUMENT_TIMEOUT_S
        document = None
        while time.time() < document_deadline:
            document = find_role(app, Atspi.Role.ENTRY)
            if document is not None:
                break
            time.sleep(POLL_S)
        if document is None:
            fail(
                "no ROLE_ENTRY under the awl application node within "
                f"{DOCUMENT_TIMEOUT_S}s — SemanticSnapshot's editable "
                "multiline document node did not cross the bridge"
            )

        state = document.get_state_set()
        if not state.contains(Atspi.StateType.EDITABLE):
            fail("document node is missing STATE_EDITABLE")
        if not state.contains(Atspi.StateType.MULTI_LINE):
            fail("document node is missing STATE_MULTI_LINE")

        focus_deadline = time.time() + FOCUS_TIMEOUT_S
        while (
            not state.contains(Atspi.StateType.FOCUSED)
            and time.time() < focus_deadline
        ):
            time.sleep(POLL_S)
            state = document.get_state_set()
        if not state.contains(Atspi.StateType.FOCUSED):
            fail(
                "document node never reports STATE_FOCUSED — focus did not "
                "cross the bridge"
            )

        run_count = document.get_child_count()
        if run_count != len(EXPECTED_RUN_TEXT):
            fail(
                f"document has {run_count} children, expected "
                f"{len(EXPECTED_RUN_TEXT)} stable line runs (item 218's shape) "
                "for the 3-line fixture — a monolithic single-node document "
                "(the pre-218 shape) would fail this exact check"
            )

        for i, want in enumerate(EXPECTED_RUN_TEXT):
            run = document.get_child_at_index(i)
            if run is None:
                fail(f"line run {i} is missing")
            if run.get_role() != Atspi.Role.STATIC:
                fail(
                    f"line run {i} has role {run.get_role_name()!r}, expected "
                    "'static' (accesskit's Role::TextRun -> AtspiRole::Static "
                    "mapping)"
                )
            got = text_of(run)
            if got != want:
                fail(
                    f"line run {i} text is {got!r}, expected {want!r} — item "
                    "218's per-line run text did not cross the bridge intact"
                )

        caret = caret_offset_of(document)
        if caret != 0:
            fail(
                f"document caret_offset is {caret} at launch, expected 0 — "
                "the Text interface is present but its initial value is wrong"
            )

        # Live selection: drive a real Shift+Right through X11 (not
        # synthesized on the AT-SPI side) and confirm it crosses the bridge.
        # Every call is timeout-bounded — see SUBPROCESS_TIMEOUT_S's doc.
        found = run_xdotool(["search", "--name", "^awl - "], check=False)
        winids = [w for w in found.stdout.split() if w]
        if not winids:
            fail(
                "xdotool found no window titled 'awl - ...' to drive a live "
                f"selection into (stderr: {found.stderr.strip()!r})"
            )
        winid = winids[0]
        run_xdotool(["windowfocus", "--sync", winid], check=False)
        run_xdotool(
            ["key", "--window", winid, "--clearmodifiers", "shift+Right"],
            check=False,
        )

        sel_deadline = time.time() + SELECTION_TIMEOUT_S
        n_sel, sel_range = 0, None
        while time.time() < sel_deadline:
            n_sel, sel_range = selection_of(document)
            if sel_range is not None:
                break
            time.sleep(POLL_S)
        if sel_range is None:
            fail(
                "Shift+Right into the live window never produced an AT-SPI "
                "selection on the document node (n_selections stayed 0) — "
                "selection did not cross the bridge"
            )
        if tuple(sel_range) != (0, 1):
            fail(
                f"AT-SPI selection after Shift+Right is {tuple(sel_range)}, "
                "expected (0, 1)"
            )

        print(
            "ATSPI-PROBE PASS: awl registered with the AT-SPI2 bus; the "
            f"editable multiline document (focused), its {run_count} stable "
            "line runs with matching text, and a live keyboard-driven "
            "selection all crossed the bridge intact."
        )
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=5)
        awl_log.close()


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception:  # noqa: BLE001 — a probe that dies quietly proves nothing
        # An unanticipated exception (e.g. a wrong guess at the GI Atspi API
        # shape) must still surface awl's own output, not just a bare
        # traceback with none of the context that would explain it.
        import traceback

        _dump_awl_log()
        print("ATSPI-PROBE FAIL: unexpected exception in the probe itself:", file=sys.stderr)
        traceback.print_exc()
        sys.exit(1)
