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
/// `CaptureOpts` and reach ONLY the plain `--screenshot` mode; `--capture-size`/
/// `--capture-dpi` reach screenshot/timeline/held (not motion/windowed); `--root`
/// reaches every mode but motion; `--workspace`/`--default-folder` reach only
/// screenshot + the windowed editor. An empty result means every supplied hook is
/// honored. (Process-global flags — `--theme`/`--caret-mode`/`--measure`/`--page`/
/// `--debug` — compose with every mode and so are never "unused".)
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
    // Canvas size / dpi: screenshot, timeline, held carry them; motion + windowed don't.
    let canvas_ok = matches!(
        kind,
        CaptureKind::Screenshot | CaptureKind::Timeline | CaptureKind::Held
    );
    if !canvas_ok {
        if h.capture_size {
            u.push("--capture-size");
        }
        if h.capture_dpi {
            u.push("--capture-dpi");
        }
    }
    // Project root: every mode but motion threads it (windowed scopes its project).
    if kind == CaptureKind::Motion && h.root {
        u.push("--root");
    }
    // Workspace / default-folder: only the plain screenshot mode + the windowed editor.
    let ws_ok = matches!(kind, CaptureKind::Screenshot | CaptureKind::Windowed);
    if !ws_ok {
        if h.workspace {
            u.push("--workspace");
        }
        if h.default_folder {
            u.push("--default-folder");
        }
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
