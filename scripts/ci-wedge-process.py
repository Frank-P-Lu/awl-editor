#!/usr/bin/env python3
"""Own a hosted Cargo session and clean its descendants on every exit path."""
import os
import signal
import subprocess
import sys
import time


def executing_group(pgid):
    # A zombie is terminal. Darwin can reject signal-zero probes after the
    # last child is reaped; process state identifies actual remaining work.
    rows = subprocess.check_output(['ps', '-axo', 'pgid=,stat='],
                                   text=True, timeout=2).splitlines()
    return any(int(row.split()[0]) == pgid
               and not row.split()[1].startswith('Z')
               for row in rows if len(row.split()) == 2)


def stop_group(proc):
    # Cargo may have exited while its test executable is still running. The
    # session belongs to this launch, so cleanup does not depend on parent life.
    for sig, grace in ((signal.SIGTERM, 1.0), (signal.SIGKILL, 1.0)):
        if not executing_group(proc.pid):
            break
        try:
            os.killpg(proc.pid, sig)
        except ProcessLookupError:
            break
        except PermissionError:
            # The group can end between inspection and delivery. A denied
            # signal to remaining live work is an error, never a clean release.
            if executing_group(proc.pid):
                raise
            break
        deadline = time.monotonic() + grace
        while time.monotonic() < deadline:
            proc.poll()
            if not executing_group(proc.pid):
                proc.wait(timeout=3)
                return
            time.sleep(0.02)
    proc.wait(timeout=3)
    if executing_group(proc.pid):
        raise RuntimeError(
            f"cleanup incomplete: executing descendants remain in process group {proc.pid}"
        )


def main():
    convention, selector, budget = sys.argv[1:]
    seconds = float(budget)
    if seconds <= 0:
        raise ValueError('budget must be positive')
    proc = None
    launching = False
    pending_signal = None

    def cancelled(signum, _frame):
        nonlocal pending_signal
        # Do not throw away a newly created child before Popen hands its PID
        # back. Cancellation is delivered immediately once ownership is stored.
        if launching:
            pending_signal = signum
        else:
            raise InterruptedError(signum)

    for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(sig, cancelled)
    try:
        launching = True
        try:
            proc = subprocess.Popen(['cargo', 'test', selector],
                                    env=dict(os.environ, AWL_CONVENTION_FORCE=convention),
                                    start_new_session=True)
        finally:
            launching = False
        if pending_signal is not None:
            raise InterruptedError(pending_signal)
        try:
            result = proc.wait(timeout=seconds)
            return (result if result >= 0 else 128 - result), False
        except subprocess.TimeoutExpired:
            return 137, True
    except InterruptedError as exc:
        return 128 + exc.args[0], False
    finally:
        # A repeated cancellation must not interrupt the bounded cleanup itself.
        for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
            signal.signal(sig, signal.SIG_IGN)
        if proc is not None:
            stop_group(proc)


if __name__ == '__main__':
    status, timed_out = main()
    convention, _selector, budget = sys.argv[1:]
    if status == 0:
        print(f'ci-wedge-budget convention={convention} status=0 — the wedge did NOT hang.')
        print('Do not read this as the wedge fixed until it stays green.')
    elif timed_out:
        print(f'ci-wedge-budget convention={convention} status=137 — HUNG past {budget}s.')
        print('This is the wedge signature. The job is allowed to fail; the workflow is not cancelled.')
    else:
        print(f'ci-wedge-budget convention={convention} status={status} — failed before watchdog deadline.')
        print('Inspect the test failure or signal; the watchdog deadline did not fire.')
    sys.exit(status)
