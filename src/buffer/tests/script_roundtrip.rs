use super::super::*;
use super::*;

// --- UNDO/REDO ROUNDTRIP INVARIANT: a deterministic mixed-op script --------
//
// The coalescing suite above pins the GROUPING rules one op-shape at a time;
// this pins the ENGINE's global invariant over a mixed script: undo-to-bottom
// recovers the original document, redo-to-top recovers the final one, the redo
// walk retraces the undo trajectory state-for-state, and interleaved undo/redo
// at arbitrary depths always lands on a trajectory state. Table-driven and
// fully deterministic — no clock, no randomness.

/// One scripted operation for the roundtrip invariant. Pure `Buffer` calls
/// only, chosen to cover every edit family the engine records: coalescing
/// self-inserts (incl. multi-byte unicode), backspace/forward-delete runs,
/// kill-line + yank, the atomic replace seams, and the motion-seal the app
/// performs between edit bursts.
enum ScriptOp {
    /// Type `str` char-by-char (self-insert, coalescing as live typing would).
    Type(&'static str),
    /// Backspace N times (a coalescing delete run).
    Backspace(usize),
    /// C-d N times (a coalescing forward-delete run).
    DeleteForward(usize),
    /// C-k at the current cursor.
    KillLine,
    /// C-y the current kill buffer (its own atomic group).
    Yank,
    /// Replace char range [a, b) with text (the spell-picker seam; atomic).
    Replace(usize, usize, &'static str),
    /// MOTION-SEAL: seal the open undo group + park the cursor at `idx`
    /// (clamped) — exactly what the app does between edit bursts.
    Seal(usize),
    /// Select [a, b) (clamped) then type `c` over it — an atomic
    /// selection-replace edit.
    SelectType(usize, usize, char),
}

fn run_script_op(buf: &mut Buffer, op: &ScriptOp) {
    match op {
        ScriptOp::Type(s) => {
            for c in s.chars() {
                buf.insert_char(c);
            }
        }
        ScriptOp::Backspace(n) => {
            for _ in 0..*n {
                buf.delete_backward();
            }
        }
        ScriptOp::DeleteForward(n) => {
            for _ in 0..*n {
                buf.delete_forward();
            }
        }
        ScriptOp::KillLine => buf.kill_line(),
        ScriptOp::Yank => buf.yank(),
        ScriptOp::Replace(a, b, s) => buf.replace_char_range(*a, *b, s),
        ScriptOp::Seal(idx) => {
            buf.seal_undo_group();
            buf.set_cursor(*idx);
        }
        ScriptOp::SelectType(a, b, c) => {
            buf.select_range(*a, *b);
            buf.insert_char(*c);
        }
    }
}

#[test]
fn mixed_op_script_undo_redo_roundtrip_invariant() {
    // The one deterministic script, replayed over TWO starting documents (the
    // empty scratch and a multi-line unicode doc) so the invariant holds from
    // both a cold start and mid-document surgery.
    let script: &[ScriptOp] = &[
        ScriptOp::Type("héllo wörld"),
        ScriptOp::Seal(5),
        ScriptOp::Type(" 日本語🦘"),
        ScriptOp::Seal(0),
        ScriptOp::KillLine,
        ScriptOp::Yank,
        ScriptOp::Seal(3),
        ScriptOp::DeleteForward(2),
        ScriptOp::Type("mixed ops\nsecond line"),
        ScriptOp::Backspace(4),
        ScriptOp::SelectType(1, 6, 'X'),
        ScriptOp::Replace(0, 3, "swapped—"),
        ScriptOp::Yank,
    ];
    for start in ["", "alpha béta\nガンマ delta\nepsilon\n"] {
        let mut buf = b(start);
        // Snapshot the text after EVERY op — the op-boundary states.
        let mut op_snaps: Vec<String> = vec![buf.text()];
        for op in script {
            run_script_op(&mut buf, op);
            op_snaps.push(buf.text());
        }
        let final_text = buf.text();
        assert_ne!(final_text, start, "the script must actually edit");

        // UNDO TO BOTTOM, recording the full trajectory (top state first).
        let mut down: Vec<String> = vec![final_text.clone()];
        while buf.can_undo() {
            buf.undo();
            down.push(buf.text());
        }
        assert_eq!(
            buf.text(),
            start,
            "undo-to-bottom restores the original document"
        );
        assert_eq!(
            buf.cursor_char(),
            0,
            "the cursor rides back to its pre-script seat"
        );
        assert!(!buf.can_undo());

        // REDO TO TOP retraces the SAME trajectory in reverse, state-for-state.
        let mut pos = down.len() - 1; // index into `down` of the current state
        while buf.can_redo() {
            buf.redo();
            pos -= 1;
            assert_eq!(
                buf.text(),
                down[pos],
                "each redo step retraces the undo trajectory"
            );
        }
        assert_eq!(pos, 0, "redo drains back to the top");
        assert_eq!(
            buf.text(),
            final_text,
            "redo-to-top restores the final document"
        );

        // Every OP-BOUNDARY snapshot appears ON the trajectory, in order (an
        // op may contribute several groups — whitespace seals, yank atomicity
        // — so the trajectory has extra INTRA-op states between them).
        let up: Vec<&String> = down.iter().rev().collect(); // original → final
        let mut j = 0usize;
        for snap in &op_snaps {
            while j < up.len() && up[j] != snap {
                j += 1;
            }
            assert!(
                j < up.len(),
                "op-boundary state {snap:?} missing from the undo/redo trajectory"
            );
        }

        // INTERLEAVED undo/redo at several depths: walk a deterministic dance
        // from the top, tracking the expected trajectory index — every stop
        // must land exactly on the recorded state.
        let bottom = down.len() - 1;
        let mut pos = 0usize; // 0 == top (final text)
        for &(u, r) in &[(3usize, 1usize), (5, 2), (2, 4), (bottom, bottom)] {
            for _ in 0..u {
                if buf.can_undo() {
                    buf.undo();
                    pos += 1;
                }
            }
            assert_eq!(buf.text(), down[pos], "after an undo run of {u}");
            for _ in 0..r {
                if buf.can_redo() {
                    buf.redo();
                    pos -= 1;
                }
            }
            assert_eq!(buf.text(), down[pos], "after a redo run of {r}");
        }
    }
}
