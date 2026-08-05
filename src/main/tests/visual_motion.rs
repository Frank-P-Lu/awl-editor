use super::super::*;
use super::{keyspec, replay_keys};
use crate::testscratch::ScratchDir;

// ---- VISUAL-LINE MOVEMENT (Phase 2) ----------------------------------
//
// These drive the REAL keymap through `replay_keys` with a layout oracle
// shaped at a NARROW measure, exactly as the live window / `--keys --measure`
// CLI do, so a long line soft-wraps and the motions must follow the VISUAL
// rows. The page globals are process-wide, so each test holds `page::test_lock()`
// and restores the default measure. On a GPU-less host the oracle is `None`,
// motion falls back to logical, and the test SKIPS (prints + returns).

/// Build a narrow-measure oracle, replay `keys` through the real keymap, and
/// return the resulting (line, col) — or `None` when no wgpu adapter exists
/// (skip). Holds the page lock for the whole replay and restores the measure.
fn replay_visual(text: &str, measure: usize, keys: &str) -> Option<(usize, usize)> {
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(measure);
    let mut buffer = Buffer::from_str(text);
    let opts = CaptureOpts::default();
    let out = capture::build_oracle(&buffer, &opts).map(|mut op| {
        let keys = keyspec::parse_keys(keys).unwrap();
        let root = PathBuf::from("/tmp");
        replay_keys(
            &mut buffer,
            &keys,
            &[],
            &root,
            None,
            &Config::empty(),
            Some(&mut op),
        );
        buffer.cursor_line_col()
    });
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    out
}

const LONG: &str = "the quick brown fox jumps over the lazy dog today\nNEXT\n";

const LONG_LINE0_LEN: usize = 49; // chars before the first '\n'

#[test]
fn visual_c_n_lands_on_next_visual_row_not_next_paragraph() {
    let Some((line, col)) = replay_visual(LONG, 15, "C-n") else {
        eprintln!("skipping visual_c_n_lands_on_next_visual_row: no wgpu adapter");
        return;
    };
    assert_eq!(
        line, 0,
        "C-n stays on the wrapped logical line, not paragraph 2"
    );
    assert!(
        col > 0,
        "C-n moved off col 0 onto the next visual row, got {col}"
    );
    assert!(
        col < LONG_LINE0_LEN,
        "the landing is a wrap boundary mid-line, not the logical end ({col})"
    );
}

#[test]
fn visual_c_e_stops_at_visual_row_end_not_logical_line_end() {
    let Some((line, col)) = replay_visual(LONG, 15, "C-e") else {
        eprintln!("skipping visual_c_e_stops_at_visual_row_end: no wgpu adapter");
        return;
    };
    assert_eq!(line, 0);
    assert!(col > 0, "C-e moved to the visual row end");
    assert!(
        col < LONG_LINE0_LEN,
        "C-e stopped at the VISUAL row end ({col}), not the logical line end ({LONG_LINE0_LEN})"
    );
}

#[test]
fn visual_goal_x_is_preserved_across_c_n_then_c_p() {
    let down_up = replay_visual(LONG, 15, "C-f C-f C-f C-f C-f C-n C-p");
    let just_right = replay_visual(LONG, 15, "C-f C-f C-f C-f C-f");
    let (Some(down_up), Some(just_right)) = (down_up, just_right) else {
        eprintln!("skipping visual_goal_x_preserved: no wgpu adapter");
        return;
    };
    assert_eq!(just_right, (0, 5), "five C-f land at col 5");
    assert_eq!(
        down_up, just_right,
        "C-n then C-p returns to the starting column via the sticky goal-x"
    );
}

#[test]
fn visual_c_a_goes_to_visual_row_start() {
    let start = replay_visual(LONG, 15, "C-n");
    let from_mid = replay_visual(LONG, 15, "C-n C-f C-f C-a");
    let (Some(start), Some(from_mid)) = (start, from_mid) else {
        eprintln!("skipping visual_c_a_goes_to_visual_row_start: no wgpu adapter");
        return;
    };
    assert_eq!(start.0, 0);
    assert!(start.1 > 0, "C-n reached a wrapped row start > 0");
    assert_eq!(
        from_mid, start,
        "C-a snaps back to the VISUAL row start, not the logical line start (col 0)"
    );
}

#[test]
fn visual_c_n_at_last_visual_row_crosses_to_next_logical_line() {
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(15);
    let probe = Buffer::from_str(LONG);
    let opts = CaptureOpts::default();
    let result = capture::build_oracle(&probe, &opts).map(|mut op| {
        let mut steps = 0usize;
        {
            let oracle = op.as_oracle();
            let (mut l, mut c) = (0usize, 0usize);
            loop {
                let (nl, nc) =
                    oracle.visual_line_down(l, c, 0.0, crate::caret::Affinity::Downstream);
                steps += 1;
                if nl != 0 {
                    break;
                }
                assert!(steps < 100, "line 0 never ended");
                l = nl;
                c = nc;
            }
        }
        assert!(steps >= 2, "line 0 should wrap into multiple visual rows");
        let root = PathBuf::from("/tmp");
        let mut b0 = Buffer::from_str(LONG);
        let keys_stay = keyspec::parse_keys(&"C-n ".repeat(steps - 1)).unwrap();
        replay_keys(
            &mut b0,
            &keys_stay,
            &[],
            &root,
            None,
            &Config::empty(),
            Some(&mut op),
        );
        let stay = b0.cursor_line_col();
        let mut b1 = Buffer::from_str(LONG);
        let keys_cross = keyspec::parse_keys(&"C-n ".repeat(steps)).unwrap();
        replay_keys(
            &mut b1,
            &keys_cross,
            &[],
            &root,
            None,
            &Config::empty(),
            Some(&mut op),
        );
        (stay, b1.cursor_line_col())
    });
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let Some((stay, cross)) = result else {
        eprintln!("skipping visual_c_n_crosses_to_next_logical_line: no wgpu adapter");
        return;
    };
    assert_eq!(
        stay.0, 0,
        "one C-n short keeps us on line 0's last visual row"
    );
    assert_eq!(
        cross.0, 1,
        "the last-row C-n crosses into the next logical line"
    );
    assert_eq!(cross.1, 0, "we land on line 1's FIRST visual row");
}

#[test]
fn regression_non_wrapped_doc_visual_equals_logical_byte_identical() {
    // REGRESSION GUARD: on a NON-wrapped document (every logical line fits in
    // one visual row) visual motion == logical motion. Identical-content lines
    // make the vertical goal-x round-trip exact even on a proportional font.
    // Replay the SAME keys with the oracle (visual) and without it (logical);
    // the resulting cursors — and the rendered PNGs — must be IDENTICAL.
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let text = "hello world foo\nhello world foo\nhello world foo\n";
    let keys = keyspec::parse_keys("C-f C-f C-f C-f C-f C-n C-n C-e C-a C-p C-k").unwrap();
    let root = PathBuf::from("/tmp");
    let opts = CaptureOpts::default();

    let mut logical = Buffer::from_str(text);
    replay_keys(
        &mut logical,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
    );

    let mut visual = Buffer::from_str(text);
    let Some(mut op) = capture::build_oracle(&visual, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        eprintln!("skipping regression_non_wrapped byte-identical: no wgpu adapter");
        return;
    };
    replay_keys(
        &mut visual,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        Some(&mut op),
    );

    assert_eq!(
        visual.cursor_line_col(),
        logical.cursor_line_col(),
        "non-wrapped: visual motion must equal logical motion"
    );

    // Byte-identical captures: render both buffers and diff the PNG bytes.
    // PID-suffixed (not just `serial()`-guarded): `serial()` is a per-process
    // reentrant lock, so a SECOND concurrent `cargo test` process (e.g. a
    // parallel native + AWL_CONVENTION_FORCE=linux run) can't be excluded by
    // it — only a unique path can (mirrors every other temp-file test).
    let dir = std::env::temp_dir();
    let pid = std::process::id();
    let pv = dir.join(format!("awl_vl_visual_{pid}.png"));
    let pl = dir.join(format!("awl_vl_logical_{pid}.png"));
    capture::capture_with(&pv, &visual, &opts).expect("render visual");
    capture::capture_with(&pl, &logical, &opts).expect("render logical");
    let bv = std::fs::read(&pv).expect("read visual png");
    let bl = std::fs::read(&pl).expect("read logical png");
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    assert_eq!(
        bv, bl,
        "non-wrapped short-line doc: visual + logical captures are byte-identical"
    );
    let _ = std::fs::remove_file(&pv);
    let _ = std::fs::remove_file(pv.with_extension("json"));
    let _ = std::fs::remove_file(&pl);
    let _ = std::fs::remove_file(pl.with_extension("json"));
}

#[test]
fn regression_edit_then_wrapped_motion_sees_fresh_wrap_geometry() {
    // THE known stale case this round retires: a spec that EDITS (wrapping
    // line 0) and then moves DOWN. The pre-phase oracle still held the
    // pre-replay shape (line 0 short, unwrapped), so C-n stepped straight
    // into logical line 1 at (1, 0); fresh per-action geometry lands on
    // line 0's SECOND visual row instead.
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(15);
    let mut buffer = Buffer::from_str("ab\ntail\n");
    let opts = CaptureOpts::default();
    let Some(mut op) = capture::build_oracle(&buffer, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        eprintln!("skipping regression_edit_then_wrapped_motion: no wgpu adapter");
        return;
    };
    let mut spec: Vec<String> = "the quick brown fox jumps over"
        .chars()
        .map(|c| {
            if c == ' ' {
                "Space".to_string()
            } else {
                c.to_string()
            }
        })
        .collect();
    spec.push("s-Up".to_string()); // BufferStart (mac native)
    spec.push("C-n".to_string()); // NextLine
    let keys = keyspec::parse_keys(&spec.join(" ")).unwrap();
    let root = PathBuf::from("/tmp");
    replay_keys(
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        Some(&mut op),
    );
    let (line, col) = buffer.cursor_line_col();
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    assert_eq!(
        line, 0,
        "Down follows the freshly-wrapped line 0 (stale geometry crossed into line 1)"
    );
    assert!(
        col > 0,
        "landing on line 0's second visual row, got col {col}"
    );
}

#[test]
fn zoom_change_mid_replay_re_wraps_the_oracle_for_later_motion() {
    // With the column capped by the WINDOW (MAX_MEASURE), a bigger zoom
    // fits fewer chars per visual row — so Down after a replayed Cmd-+
    // must land at a strictly SMALLER column than the same Down at zoom
    // 1.0. The pre-phase oracle kept its build-time zoom, landing the two
    // replays identically.
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::MAX_MEASURE);
    let text = format!("{}\ntail\n", "word ".repeat(80));
    let root = PathBuf::from("/tmp");
    let opts = CaptureOpts::default();

    let mut plain = Buffer::from_str(&text);
    let Some(mut op1) = capture::build_oracle(&plain, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        eprintln!("skipping zoom_change_mid_replay_re_wraps_the_oracle: no wgpu adapter");
        return;
    };
    let keys_plain = keyspec::parse_keys("C-n").unwrap();
    replay_keys(
        &mut plain,
        &keys_plain,
        &[],
        &root,
        None,
        &Config::empty(),
        Some(&mut op1),
    );
    let (l1, c1) = plain.cursor_line_col();

    let mut zoomed = Buffer::from_str(&text);
    let Some(mut op2) = capture::build_oracle(&zoomed, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        eprintln!("skipping zoom_change_mid_replay_re_wraps_the_oracle: no wgpu adapter");
        return;
    };
    let keys_zoom = keyspec::parse_keys("s-= C-n").unwrap();
    replay_keys(
        &mut zoomed,
        &keys_zoom,
        &[],
        &root,
        None,
        &Config::empty(),
        Some(&mut op2),
    );
    let (l2, c2) = zoomed.cursor_line_col();

    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    assert_eq!((l1, l2), (0, 0), "both Downs stay on the wrapped line 0");
    assert!(
        c1 > 0 && c2 > 0,
        "both landed on a second visual row: {c1}, {c2}"
    );
    assert!(
        c2 < c1,
        "the zoomed row holds fewer chars, so its wrap boundary is earlier: {c2} < {c1}"
    );
}

#[test]
fn goto_switch_mid_replay_reshapes_the_oracle_to_the_arriving_buffer() {
    // The Goto arm swaps the ACTIVE buffer (and re-applies its sticky page
    // measure) mid-replay; a following Down must read the ARRIVING
    // buffer's wrap geometry. Launched on a CODE file (configured measure
    // 100 — b.md's long line would NOT wrap there), the switch to the
    // prose b.md re-applies measure 15 and swaps the text: both must reach
    // the oracle for Down to stay on b.md's wrapped line 0. The pre-phase
    // oracle stayed shaped on a.rs, so Down crossed into line 1 at (1, 0).
    let _fs = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-oracle-goto-{}", std::process::id())),
    );
    std::fs::write(dir.join("a.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.join("b.md"), "the quick brown fox jumps over\ntail\n").unwrap();
    let cfg = Config {
        page_width_prose: Some(15),
        page_width_code: Some(100),
        ..Config::empty()
    };
    crate::page::set_page_on(true);
    crate::page::set_measure(100); // the launch file's own (code) measure
    let mut buffer = Buffer::from_file(&dir.join("a.rs"));
    let corpus = vec!["a.rs".to_string(), "b.md".to_string()];
    let opts = CaptureOpts::default();
    let Some(mut op) = capture::build_oracle(&buffer, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        eprintln!("skipping goto_switch_mid_replay_reshapes_the_oracle: no wgpu adapter");
        return;
    };
    let keys = keyspec::parse_keys("s-o b . m d RET C-n").unwrap();
    replay_keys(&mut buffer, &keys, &corpus, &dir, None, &cfg, Some(&mut op));
    let (line, col) = buffer.cursor_line_col();
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    assert_eq!(
        buffer.path(),
        Some(dir.join("b.md").as_path()),
        "the Goto switch landed on b.md"
    );
    assert_eq!(
        line, 0,
        "Down follows b.md's line 0, wrapped at ITS re-applied measure"
    );
    assert!(
        col > 0,
        "landing on line 0's second visual row, got col {col}"
    );
}
