#!/usr/bin/env python3
"""Validate public codesign metadata against the repository's Developer ID team."""
import re
import subprocess
import sys


def verify_metadata(signature: str, team: str):
    if not re.fullmatch(r'[A-Z0-9]{10}', team):
        raise ValueError('invalid pinned Developer ID team')
    expected = rf'^Authority=Developer ID Application: .+ \({re.escape(team)}\)$'
    flags = re.search(r'^CodeDirectory .+ flags=0x([0-9a-fA-F]+)\(', signature, re.M)
    timestamp = re.search(r'^Timestamp=(.+)$', signature, re.M)
    if (not re.search(rf'^TeamIdentifier={re.escape(team)}$', signature, re.M)
            or not re.search(expected, signature, re.M)
            or flags is None or not int(flags.group(1), 16) & 0x10000
            or timestamp is None or timestamp.group(1).strip().lower() in {'none', 'not set'}):
        raise ValueError('signature must use the pinned Developer ID team, hardened runtime and secure timestamp')


def main():
    if len(sys.argv) != 3:
        raise SystemExit('usage: verify-macos-signature.py APP TEAM_ID')
    result = subprocess.run(['codesign', '-dv', '--verbose=4', sys.argv[1]],
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
    verify_metadata(result.stderr, sys.argv[2])
    print('Developer ID identity, hardened runtime and secure timestamp verified')


if __name__ == '__main__':
    main()
