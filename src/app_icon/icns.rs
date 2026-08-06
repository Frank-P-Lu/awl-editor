//! THE `.icns` CONTAINER — pack and parse, in one place, with no shell-out.
//!
//! An Apple icon file is a trivially simple archive and we write it ourselves
//! rather than shelling out to `iconutil`, for three reasons that all matter to
//! this repo:
//!
//!   * **Determinism is provable.** The container is a pure function of its
//!     reps, so `cargo test` can re-pack a committed `.icns` from the PNGs
//!     inside it and assert the bytes come back identical — a real
//!     regeneration law that needs no browser, no macOS tool, and no network.
//!   * **One owner.** The same code packs the canonical `Awl.icns` and each
//!     world's Dock image, so those can never drift into two formats.
//!   * **No tool on the critical path.** `iconutil` is macOS-only; the pack
//!     step must run wherever the export runs, and the PARSE half has to work
//!     on Linux CI too (the law tests read the committed assets on every
//!     platform).
//!
//! FORMAT (Apple's icns, the modern PNG-rep form). Header: the magic `icns`
//! then a big-endian u32 holding the TOTAL file length, header included. Then
//! a flat sequence of chunks, each an OSType (4 ASCII bytes naming the rep)
//! plus a big-endian u32 length that also counts its own 8-byte header, then
//! the payload — for every type below, a complete PNG file. There is no
//! padding and no alignment. A `TOC ` chunk is optional (an index of the
//! chunks that follow) and we deliberately omit it: it is redundant data that
//! would be a second thing to keep consistent.
//!
//! We do NOT write the legacy `is32`/`il32`/`s8mk` masks — those are the
//! pre-10.7 RLE formats, and nothing that runs on `LSMinimumSystemVersion`
//! 11.0 (see `scripts/package-macos.sh`) reads them.

use std::path::Path;

use crate::theme::Theme;

/// One representation slot in the container: the OSType that names it and the
/// square pixel edge its PNG must have.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rep {
    /// The four-byte OSType, e.g. `ic07`.
    pub ostype: [u8; 4],
    /// The square pixel edge of the PNG this slot carries.
    pub px: u32,
}

impl Rep {
    /// The OSType as text, for messages and test names.
    pub fn name(&self) -> &str {
        // Every OSType below is ASCII by construction.
        std::str::from_utf8(&self.ostype).unwrap_or("????")
    }
}

const fn rep(ostype: &[u8; 4], px: u32) -> Rep {
    Rep {
        ostype: *ostype,
        px,
    }
}

/// THE REP ROSTER we write, in file order — the same set `iconutil` produces
/// from a full `.iconset`, minus the legacy masks. Each Retina slot repeats a
/// pixel size that also appears as a 1× slot (a 32px PNG is both "32×32 @1×"
/// and "16×16 @2×"); that duplication is Apple's design, not a mistake, and it
/// is why the packer is fed by SIZE rather than by slot.
///
/// A no-wildcard consequence: [`icns_sizes`] is derived from this array, so
/// adding a slot automatically demands its tile and the law tests follow.
pub const REPS: [Rep; 10] = [
    rep(b"icp4", 16),   // 16×16
    rep(b"icp5", 32),   // 32×32
    rep(b"ic11", 32),   // 16×16@2x
    rep(b"ic12", 64),   // 32×32@2x
    rep(b"ic07", 128),  // 128×128
    rep(b"ic13", 256),  // 128×128@2x
    rep(b"ic08", 256),  // 256×256
    rep(b"ic14", 512),  // 256×256@2x
    rep(b"ic09", 512),  // 512×512
    rep(b"ic10", 1024), // 512×512@2x
];

/// The DISTINCT pixel sizes the roster needs, ascending — the tiles the
/// exporter must have rendered before a world can be packed.
pub fn icns_sizes() -> Vec<u32> {
    let mut sizes: Vec<u32> = REPS.iter().map(|r| r.px).collect();
    sizes.sort_unstable();
    sizes.dedup();
    sizes
}

/// The exporter's tile file name for one world at one size — the ONE place
/// the `<World>-<preset>-<size>.png` convention is spelled out, shared by the
/// packer and by the law tests so a rename cannot half-land.
pub fn tile_file_name(theme: &Theme, px: u32) -> String {
    format!("{}-{}-{px}.png", theme.name, theme.icon_cursor.slug())
}

/// Pack `(size → PNG bytes)` into a complete `.icns`, following [`REPS`].
///
/// Fails when a rep's size is missing from `pngs`, or when the PNG at that
/// size is not actually square at that edge — a mislabelled rep is exactly the
/// defect that makes macOS fall back to the generic application icon, so it is
/// an error here rather than a silent write.
pub fn pack(pngs: &[(u32, Vec<u8>)]) -> anyhow::Result<Vec<u8>> {
    let mut body: Vec<u8> = Vec::new();
    for r in REPS.iter() {
        let png = pngs
            .iter()
            .find(|(px, _)| *px == r.px)
            .map(|(_, bytes)| bytes)
            .ok_or_else(|| {
                anyhow::anyhow!("rep {} wants a {}px PNG, none supplied", r.name(), r.px)
            })?;
        let (w, h) = png_size(png)
            .ok_or_else(|| anyhow::anyhow!("rep {}: not a PNG (no readable IHDR)", r.name()))?;
        if w != r.px || h != r.px {
            anyhow::bail!("rep {} wants {}×{}, got {w}×{h}", r.name(), r.px, r.px);
        }
        let len = u32::try_from(png.len() + 8)
            .map_err(|_| anyhow::anyhow!("rep {} is absurdly large", r.name()))?;
        body.extend_from_slice(&r.ostype);
        body.extend_from_slice(&len.to_be_bytes());
        body.extend_from_slice(png);
    }
    let total =
        u32::try_from(body.len() + 8).map_err(|_| anyhow::anyhow!("icns would exceed 4 GiB"))?;
    let mut out = Vec::with_capacity(body.len() + 8);
    out.extend_from_slice(b"icns");
    out.extend_from_slice(&total.to_be_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// Parse an `.icns` back into its `(OSType, PNG bytes)` chunks, in file order.
///
/// Strict on purpose: the magic, the declared total length and every chunk
/// length must agree with the actual bytes. This is the parser the law tests
/// use as their oracle, so a lenient reader would let a malformed committed
/// asset pass. Also the reader half of [`linux_icon_png`] — no longer test-only.
pub fn unpack(bytes: &[u8]) -> anyhow::Result<Vec<([u8; 4], &[u8])>> {
    if bytes.len() < 8 || &bytes[0..4] != b"icns" {
        anyhow::bail!("not an icns (bad magic)");
    }
    let total = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
    if total != bytes.len() {
        anyhow::bail!("icns declares {total} bytes, file holds {}", bytes.len());
    }
    let mut out = Vec::new();
    let mut pos = 8;
    while pos < bytes.len() {
        if pos + 8 > bytes.len() {
            anyhow::bail!("truncated chunk header at {pos}");
        }
        let mut ostype = [0u8; 4];
        ostype.copy_from_slice(&bytes[pos..pos + 4]);
        let len = u32::from_be_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;
        if len < 8 || pos + len > bytes.len() {
            anyhow::bail!("chunk {:?} declares {len} bytes at {pos}", ostype);
        }
        out.push((ostype, &bytes[pos + 8..pos + len]));
        pos += len;
    }
    Ok(out)
}

/// The `(width, height)` a PNG's IHDR declares, or `None` when the bytes are
/// not a PNG. Eight-byte signature, then a length+`IHDR` chunk whose first two
/// big-endian u32s are the dimensions — read directly rather than by decoding,
/// so the check costs nothing and works on any bit depth.
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[0..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Some((w, h))
}

/// Read one world's exported tiles out of `tiles_dir` and pack them.
///
/// The preset is not a parameter: it comes off the world itself
/// ([`Theme::icon_cursor`]), which is what keeps "the icon we ship" and "the
/// candidate the judge picked" the same bytes rather than two decisions.
pub fn pack_world(tiles_dir: &Path, theme: &Theme) -> anyhow::Result<Vec<u8>> {
    let mut pngs = Vec::new();
    for px in icns_sizes() {
        let path = tiles_dir.join(tile_file_name(theme, px));
        let bytes = std::fs::read(&path).map_err(|e| {
            anyhow::anyhow!(
                "{}: {e} — run scripts/export-icons.sh first (it renders every tile)",
                path.display()
            )
        })?;
        pngs.push((px, bytes));
    }
    pack(&pngs)
}

/// The LINUX DESKTOP ICON: one PNG cut straight out of an already-packed
/// `.icns` via [`unpack`], never a second hand-drawn source. `--export-linux-icon`
/// (`src/main/args.rs`) hands this the committed canonical `assets/macos/Awl.icns`
/// bytes so `scripts/package-appimage.sh` can populate the AppImage's
/// `.desktop`-referenced icon and the `hicolor` theme directory from the exact
/// artwork Finder and the Dock already show — the freedesktop-recommended
/// 256×256 size, which [`REPS`] already carries as the `ic08`/`ic13` slot (both
/// reps are the same pixel edge, cut from the one 256px tile the exporter
/// rendered, so which of the two chunks answers first is immaterial).
pub const LINUX_ICON_PX: u32 = 256;

pub fn linux_icon_png(icns_bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let rep = REPS
        .iter()
        .find(|r| r.px == LINUX_ICON_PX)
        .ok_or_else(|| anyhow::anyhow!("no {LINUX_ICON_PX}px rep declared in REPS"))?;
    let chunks = unpack(icns_bytes)?;
    chunks
        .into_iter()
        .find(|(ostype, _)| *ostype == rep.ostype)
        .map(|(_, bytes)| bytes.to_vec())
        .ok_or_else(|| anyhow::anyhow!("icns has no {} ({LINUX_ICON_PX}px) chunk", rep.name()))
}
