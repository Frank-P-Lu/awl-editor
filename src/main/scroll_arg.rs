//! `--scroll ROW[:SUBPX]` parser.

pub(super) fn parse(raw: &str) -> anyhow::Result<crate::render::ScrollPos> {
    let bad = || anyhow::anyhow!("bad --scroll {raw:?}");
    let (row, px_q) = match raw.split_once(':') {
        Some((row, px_q)) => (
            row.parse().map_err(|_| bad())?,
            px_q.parse().map_err(|_| bad())?,
        ),
        None => (raw.parse().map_err(|_| bad())?, 0),
    };
    Ok(crate::render::ScrollPos { row, px_q })
}
