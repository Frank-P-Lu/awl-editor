//! CLI capture and scalar argument parsers.

use super::*;

pub(super) fn parse_sel(s: &str) -> Result<((usize, usize), (usize, usize))> {
    let (a, b) = s
        .split_once('-')
        .ok_or_else(|| anyhow::anyhow!("--sel expects L0:C0-L1:C1, got {s:?}"))?;
    let parse_pt = |p: &str| -> Result<(usize, usize)> {
        let (l, c) = p
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("--sel endpoint expects L:C, got {p:?}"))?;
        Ok((l.trim().parse()?, c.trim().parse()?))
    };
    let p0 = parse_pt(a)?;
    let p1 = parse_pt(b)?;
    // Order so the first endpoint is earlier in the buffer.
    Ok(if p0 <= p1 { (p0, p1) } else { (p1, p0) })
}

/// Parse a `--capture-timeline "0,16,50,150"` argument into a cumulative-ms step
/// sequence. Each entry is the virtual-clock time (ms since the move started) at
/// which a frame is rendered; the dt fed to step `i` is `t[i]-t[i-1]`.
pub(super) fn parse_steps(s: &str) -> Result<Vec<u32>> {
    let steps: Vec<u32> = s
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| {
            p.parse::<u32>().map_err(|_| {
                anyhow::anyhow!("bad --capture-timeline step {p:?} (want ms integers)")
            })
        })
        .collect::<Result<_>>()?;
    if steps.is_empty() {
        bail!("--capture-timeline needs at least one ms step (e.g. \"0,16,50,150\")");
    }
    Ok(steps)
}

/// Parse a `--capture-size "WxH"` argument into PHYSICAL canvas dimensions. Accepts
/// `x` or `X` as the separator (e.g. "2400x1600").
pub(super) fn parse_size(s: &str) -> Result<(u32, u32)> {
    let (w, h) = s
        .split_once(['x', 'X'])
        .ok_or_else(|| anyhow::anyhow!("--capture-size expects WxH, got {s:?}"))?;
    let w: u32 = w
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("bad --capture-size width in {s:?}"))?;
    let h: u32 = h
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("bad --capture-size height in {s:?}"))?;
    if w == 0 || h == 0 {
        bail!("--capture-size dimensions must be non-zero, got {s:?}");
    }
    Ok((w, h))
}

/// Parse a `--capture-dpi` factor: a FINITE, strictly-positive scale (mirrors
/// parse_size's non-zero guard). A non-finite (`inf`/`nan`) or `<= 0` factor
/// would scale the canvas to a degenerate / zero-area render target, so reject it
/// up front rather than render garbage.
pub(super) fn parse_dpi(s: &str) -> Result<f32> {
    let v: f32 = s
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("bad --capture-dpi {s:?}"))?;
    if !v.is_finite() || v <= 0.0 {
        bail!("--capture-dpi must be finite and > 0, got {s:?}");
    }
    Ok(v)
}

/// Parse a `--zoom` factor: a FINITE, strictly-positive scale (mirrors
/// parse_dpi's guard). A non-finite (`inf`/`nan`) factor would poison every
/// zoom-derived metric downstream (NaN propagates through the step/clamp
/// arithmetic), so reject it up front with a readable error rather than render
/// garbage; the in-range [0.5, 3.0] clamp stays `render::clamp_zoom`'s job.
pub(super) fn parse_zoom(s: &str) -> Result<f32> {
    let v: f32 = s
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("bad --zoom {s:?}"))?;
    if !v.is_finite() || v <= 0.0 {
        bail!("--zoom must be finite and > 0, got {s:?}");
    }
    Ok(v)
}

/// Parse a `--measure` column width: a strictly-positive char count (mirrors
/// parse_size's non-zero guard — a zero-width writing column is degenerate).
pub(super) fn parse_measure(s: &str) -> Result<usize> {
    let n: usize = s
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("bad --measure {s:?}"))?;
    if n == 0 {
        bail!("--measure must be > 0, got {s:?}");
    }
    Ok(n)
}

/// Resolve the [`CaptureKind`] a parsed command line selects, from the exact
/// booleans `parse_args` already derived from its flags. Pulled out of
/// `parse_args` itself (rather than left as an inline `if`/`else if` chain) so
/// growing this precedence — as `--screenshot-app` just did — costs this
/// function a branch, not `parse_args` a clippy exception. The precedence
/// mirrors the `Mode` construction below it: held > timeline > motion >
/// screenshot-app > screenshot-frames > plain screenshot; no output path at
/// all means the windowed editor. (Every one of these flags also sets `out`
/// and is checked for conflicts by `ensure_single_capture_mode`, so in
/// practice at most one of `held`/`timeline`/`motion`/`screenshot_app`/
/// `screenshot_frames` is ever true — this order only documents which arm a
/// future combination would fall into.)
pub(super) fn resolve_capture_kind(
    has_out: bool,
    held: bool,
    timeline: bool,
    motion: bool,
    screenshot_app: bool,
    screenshot_frames: bool,
) -> CaptureKind {
    if !has_out {
        CaptureKind::Windowed
    } else if held {
        CaptureKind::Held
    } else if timeline {
        CaptureKind::Timeline
    } else if motion {
        CaptureKind::Motion
    } else if screenshot_app {
        CaptureKind::ScreenshotApp
    } else if screenshot_frames {
        CaptureKind::ScreenshotFrames
    } else {
        CaptureKind::Screenshot
    }
}

/// The capture mode resolved from the CLI flags, used ONLY to decide which
/// verification hooks the run honors (the real `Mode` is built separately). The
/// precedence mirrors the `Mode` construction below: held > timeline > motion >
/// plain screenshot; no output path at all means the windowed editor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum CaptureKind {
    Windowed,
    Screenshot,
    Motion,
    Timeline,
    Held,
    /// `--screenshot-app`: a real headless `App`, not a `ReplaySession`. It
    /// threads canvas/dpi/root/workspace onto its own `LiveAppSpec`, but NOT
    /// the per-frame render hooks (`--sel`/`--zoom`/`--scroll`/
    /// `--preedit`/`--search`/`--search-case`/`--search-replace`) or
    /// `--default-folder`: the live App owns that state via real driving, and
    /// `LiveAppSpec` carries no slot the latter could land in — see
    /// `unused_hooks` below.
    ScreenshotApp,
    /// `--screenshot-frames`: the virtual-clock frame-loop capture. It threads
    /// canvas/dpi onto its own `Mode::ScreenshotFrames` fields (`capture::
    /// capture_frames` reads them off the `CaptureOpts` it is handed), but the
    /// document is a STATIONARY backdrop loaded straight off disk — there is
    /// no `--keys` replay, no project-root resolution, and no per-frame render
    /// override — so everything else this struct tracks is refused rather than
    /// silently dropped. See `unused_hooks` below.
    ScreenshotFrames,
}

/// Which verification-hook flags were SUPPLIED on the command line (each bool =
/// "this flag was given"). Used to reject a hook the chosen mode would silently
/// drop — see `unused_hooks`.
#[derive(Clone, Copy, Default)]
pub(super) struct SuppliedHooks {
    pub(super) sel: bool,
    pub(super) zoom: bool,
    pub(super) scroll: bool,
    pub(super) preedit: bool,
    pub(super) search: bool,
    pub(super) search_case: bool,
    pub(super) search_replace: bool,
    pub(super) capture_size: bool,
    pub(super) capture_dpi: bool,
    pub(super) root: bool,
    pub(super) workspace: bool,
    pub(super) default_folder: bool,
}

/// Return the supplied hooks that the chosen `kind` does NOT thread into its
/// `Mode` (so it would silently ignore them), in a stable order. Each `Mode`
/// variant carries only a subset of the hooks: the per-frame render hooks
/// (`--sel`/`--zoom`/`--scroll`/`--preedit`/`--search`/`--search-case`) ride
/// `CaptureOpts` and reach ONLY the plain `--screenshot` mode — `ScreenshotApp`
/// deliberately excludes them too, because the live `App` owns that state via
/// real driving and an override would misrepresent the editor being
/// photographed (see `CaptureKind::ScreenshotApp`'s own doc), and
/// `ScreenshotFrames` excludes them because its document is a stationary
/// backdrop, not something a replay or an override poses; `--capture-size`/
/// `--capture-dpi` reach screenshot/timeline/held/screenshot-app/
/// screenshot-frames (every mode that renders a real frame — not
/// motion/windowed); `--root` reaches every mode but motion and
/// screenshot-frames (the latter loads its backdrop straight off disk, with no
/// project resolution at all); `--workspace` reaches
/// screenshot/screenshot-app/windowed (`LiveAppSpec` carries it);
/// `--default-folder` reaches only screenshot + the windowed editor
/// (`LiveAppSpec` has no slot for it, and `ScreenshotFrames` has neither slot
/// nor a resolved root to hang one off of). An empty result means every
/// supplied hook is honored. (Process-global flags —
/// `--theme`/`--caret-mode`/`--measure`/`--page`/`--debug` — compose with
/// every mode and so are never "unused". `--keys` is refused for
/// `ScreenshotFrames` by its own dedicated check in `parse_args`, not through
/// this table — see the call site.)
pub(super) fn unused_hooks(kind: CaptureKind, h: &SuppliedHooks) -> Vec<&'static str> {
    let mut u = Vec::new();
    // Per-frame render hooks: only the plain `--screenshot` mode threads `CaptureOpts`.
    if kind != CaptureKind::Screenshot {
        for (name, set) in [
            ("--sel", h.sel),
            ("--zoom", h.zoom),
            ("--scroll", h.scroll),
            ("--preedit", h.preedit),
            ("--search", h.search),
            ("--search-case", h.search_case),
            ("--search-replace", h.search_replace),
        ] {
            if set {
                u.push(name);
            }
        }
    }
    // Canvas size / dpi: every mode that renders a real frame from its own
    // `CaptureOpts`-shaped canvas carries them (screenshot, timeline, held,
    // the live-`App` door, and the frame-loop door); motion + windowed don't.
    let canvas_ok = matches!(
        kind,
        CaptureKind::Screenshot
            | CaptureKind::Timeline
            | CaptureKind::Held
            | CaptureKind::ScreenshotApp
            | CaptureKind::ScreenshotFrames
    );
    if !canvas_ok {
        if h.capture_size {
            u.push("--capture-size");
        }
        if h.capture_dpi {
            u.push("--capture-dpi");
        }
    }
    // Project root: every mode but motion and screenshot-frames threads it
    // (windowed scopes its project; screenshot-frames resolves none at all).
    if matches!(kind, CaptureKind::Motion | CaptureKind::ScreenshotFrames) && h.root {
        u.push("--root");
    }
    // Workspace: screenshot, the windowed editor, and the live-`App` door
    // (`LiveAppSpec` carries it too). Default-folder: screenshot + windowed
    // only — neither `LiveAppSpec` field list has a slot for it.
    let workspace_ok = matches!(
        kind,
        CaptureKind::Screenshot | CaptureKind::Windowed | CaptureKind::ScreenshotApp
    );
    if !workspace_ok && h.workspace {
        u.push("--workspace");
    }
    let default_folder_ok = matches!(kind, CaptureKind::Screenshot | CaptureKind::Windowed);
    if !default_folder_ok && h.default_folder {
        u.push("--default-folder");
    }
    u
}

/// Reject MORE THAN ONE capture-mode flag. Each capture-mode flag sets the output
/// path AND selects a `Mode` by a fixed precedence, so passing two would silently
/// honor one and drop the other; name them all and refuse instead.
pub(super) fn ensure_single_capture_mode(modes: &[&str]) -> Result<()> {
    if modes.len() > 1 {
        bail!(
            "conflicting capture-mode flags: {} (choose exactly one)",
            modes.join(", ")
        );
    }
    Ok(())
}

/// Parse a `--capture-held` direction (`left|right|up|down`).
pub(super) fn parse_held_dir(s: &str) -> Result<capture::HeldDir> {
    match s.to_ascii_lowercase().as_str() {
        "left" | "l" => Ok(capture::HeldDir::Left),
        "right" | "r" => Ok(capture::HeldDir::Right),
        "up" | "u" => Ok(capture::HeldDir::Up),
        "down" | "d" => Ok(capture::HeldDir::Down),
        _ => bail!("bad --capture-held direction {s:?} (want left|right|up|down)"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn parse_soak_seconds(s: &str) -> Result<std::time::Duration> {
    let seconds: f64 = s
        .parse()
        .map_err(|_| anyhow::anyhow!("bad --soak-gpu-seconds {s:?} (want a positive number)"))?;
    if !seconds.is_finite() || seconds <= 0.0 {
        bail!("--soak-gpu-seconds must be finite and > 0, got {s:?}");
    }
    Ok(std::time::Duration::from_secs_f64(seconds))
}
