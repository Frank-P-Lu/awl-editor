//! THE APP-ICON LAWS.
//!
//! Two oracles, deliberately: the `.icns` CONTAINER is asserted structurally
//! (magic, lengths, rep roster, and a re-pack that must reproduce the committed
//! bytes), and the ARTWORK is asserted by counting pixels — because a container
//! that parses proves nothing about what the Dock shows. This repo has already
//! been burned once by a state oracle reporting a selected row that rendered
//! fully invisible, so "the `l` is legible" here means "there are
//! `primary_content` pixels forming a tall stem inside the `primary` slab", not
//! "the exporter said so".
//!
//! Every sweep reads `theme::THEMES` and matches on closed enums; nothing here
//! carries a second list of worlds a new one could quietly dodge.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::icns::{self, REPS};
use super::*;
use crate::theme::{CaretBlockStyle, IconCursor, Srgb};

// --------------------------------------------------------------- fixtures ---

fn root() -> PathBuf {
    // Tests run with CWD == the crate root (same convention as
    // `icon_manifest`'s font-directory tests).
    PathBuf::from(".")
}

fn icon_path(world: &str) -> PathBuf {
    root().join(WORLD_ICON_DIR).join(format!("{world}.icns"))
}

fn icon_bytes(world: &str) -> Vec<u8> {
    let p = icon_path(world);
    std::fs::read(&p)
        .unwrap_or_else(|e| panic!("{}: {e} — run scripts/export-icons.sh", p.display()))
}

/// One rep decoded to RGBA8. `px` must be one of [`REPS`]'s pixel sizes.
fn rep_rgba(icns_bytes: &[u8], px: u32) -> image::RgbaImage {
    let chunks = icns::unpack(icns_bytes).expect("committed icns parses");
    let want = REPS.iter().find(|r| r.px == px).expect("a rep at this size");
    let (_, png) = chunks
        .iter()
        .find(|(t, _)| *t == want.ostype)
        .unwrap_or_else(|| panic!("no {} chunk", want.name()));
    image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .expect("rep decodes")
        .to_rgba8()
}

fn near(px: &[u8], rgb: [u8; 3], tol: i32) -> bool {
    (0..3).all(|i| (px[i] as i32 - rgb[i] as i32).abs() <= tol)
}

fn opaque(px: &[u8]) -> bool {
    px[3] >= 128
}

/// A bounding box in pixels, inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Bbox {
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
}

impl Bbox {
    fn w(&self) -> u32 {
        self.x1 - self.x0 + 1
    }
    fn h(&self) -> u32 {
        self.y1 - self.y0 + 1
    }
    fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.y0 && y <= self.y1
    }
    fn grow(bb: &mut Option<Bbox>, x: u32, y: u32) {
        *bb = Some(match *bb {
            None => Bbox { x0: x, y0: y, x1: x, y1: y },
            Some(b) => Bbox {
                x0: b.x0.min(x),
                y0: b.y0.min(y),
                x1: b.x1.max(x),
                y1: b.y1.max(y),
            },
        });
    }
}

/// Every opaque pixel matching `rgb` (within `tol`), as a mask.
fn mask_of(img: &image::RgbaImage, rgb: [u8; 3], tol: i32) -> Vec<bool> {
    let (w, h) = img.dimensions();
    let mut mask = vec![false; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y).0;
            if opaque(&p) && near(&p, rgb, tol) {
                mask[(y * w + x) as usize] = true;
            }
        }
    }
    mask
}

/// THE CURSOR SLAB, found as SHAPE rather than colour — the same rule
/// `scripts/icons/verify.py` uses, and for the same two reasons:
///
///   * a knocked-out `l` can SPLIT the slab into two slivers (the narrow pill
///     does exactly that whenever the glyph is wider than the pill), so pieces
///     spanning the same vertical extent right beside each other are re-merged;
///   * Wagtail and Cassowary paint `primary` and `base_content` the SAME value,
///     so the "aw" letters are literally the cursor's colour. They fall out of
///     the same vertical test: x-height letters cover well under 60% of the
///     slab's height.
///
/// Returns `(pixel count, bbox, the merged mask)`.
fn cursor_slab(mask: &[bool], w: u32, h: u32) -> (u32, Option<Bbox>, Vec<bool>) {
    let mut comp: Vec<i32> = vec![-1; mask.len()];
    let mut parts: Vec<(u32, Bbox)> = Vec::new();
    for start in 0..mask.len() {
        if !mask[start] || comp[start] >= 0 {
            continue;
        }
        let id = parts.len() as i32;
        let mut stack = vec![start];
        comp[start] = id;
        let mut size = 0u32;
        let mut bb: Option<Bbox> = None;
        while let Some(i) = stack.pop() {
            size += 1;
            let (x, y) = (i as u32 % w, i as u32 / w);
            Bbox::grow(&mut bb, x, y);
            let mut neighbours: Vec<usize> = Vec::with_capacity(4);
            if x > 0 {
                neighbours.push(i - 1);
            }
            if x < w - 1 {
                neighbours.push(i + 1);
            }
            if y > 0 {
                neighbours.push(i - w as usize);
            }
            if y < h - 1 {
                neighbours.push(i + w as usize);
            }
            for j in neighbours {
                if mask[j] && comp[j] < 0 {
                    comp[j] = id;
                    stack.push(j);
                }
            }
        }
        parts.push((size, bb.expect("a visited pixel")));
    }
    let Some(biggest) = (0..parts.len()).max_by_key(|i| parts[*i].0) else {
        return (0, None, vec![false; mask.len()]);
    };
    let (mut size, mut bb) = parts[biggest];
    let mut merged: Vec<i32> = vec![biggest as i32];
    let height = bb.h();
    for (id, (s, b)) in parts.iter().enumerate() {
        if id == biggest {
            continue;
        }
        let overlap = bb.y1.min(b.y1) as i64 - bb.y0.max(b.y0) as i64 + 1;
        let gap = (b.x0 as i64 - bb.x1 as i64)
            .max(bb.x0 as i64 - b.x1 as i64)
            .max(0);
        if overlap >= (0.6 * height as f64) as i64 && gap <= (0.15 * w as f64) as i64 {
            size += s;
            merged.push(id as i32);
            bb = Bbox {
                x0: bb.x0.min(b.x0),
                y0: bb.y0.min(b.y0),
                x1: bb.x1.max(b.x1),
                y1: bb.y1.max(b.y1),
            };
        }
    }
    let slab: Vec<bool> = comp.iter().map(|c| merged.contains(c)).collect();
    (size, Some(bb), slab)
}

/// THE KNOCKED-OUT LETTER, by SCANLINE INTERIOR — the discriminator that a
/// bounding box alone cannot give.
///
/// A slab's bbox is a rectangle, but a pill (or a squircle-cornered block) is
/// not, so the bbox's own corners hold plain GROUND. On Mopoke that ground
/// (`#1b1814`) sits within antialiasing distance of `primary_content`
/// (`#261a08`), and on Wagtail the two tokens are the SAME value — so "a
/// `primary_content` pixel inside the slab's bbox" happily counts the rounded
/// corner's background as the letter, and the measured `l` grows to the whole
/// slab. That is precisely the false green this repo's tripwire warns about.
///
/// So a letter pixel must be INTERIOR to the slab on its own scanline: for its
/// row, there is slab to its left AND slab to its right. Ground in a rounded
/// corner fails that (it is outside the row's slab span); a letter knocked out
/// of the middle passes it, including the narrow preset's case where the glyph
/// splits the pill into two slivers and sits between them.
fn letter_mask(
    img: &image::RgbaImage,
    slab: &[bool],
    ink: [u8; 3],
    tol: i32,
) -> Vec<bool> {
    let (w, h) = img.dimensions();
    let mut out = vec![false; slab.len()];
    for y in 0..h {
        let row = (y * w) as usize;
        let xs: Vec<u32> = (0..w).filter(|x| slab[row + *x as usize]).collect();
        let (Some(&lo), Some(&hi)) = (xs.first(), xs.last()) else {
            continue;
        };
        for x in (lo + 1)..hi {
            let i = row + x as usize;
            if slab[i] {
                continue;
            }
            let p = img.get_pixel(x, y).0;
            if opaque(&p) && near(&p, ink, tol) {
                out[i] = true;
            }
        }
    }
    out
}

/// The bbox of a mask, or `None` when it is empty.
fn mask_bbox(mask: &[bool], w: u32) -> Option<Bbox> {
    let mut bb = None;
    for (i, on) in mask.iter().enumerate() {
        if *on {
            Bbox::grow(&mut bb, i as u32 % w, i as u32 / w);
        }
    }
    bb
}

fn rgb(c: Srgb) -> [u8; 3] {
    c.rgb_bytes()
}

fn world(name: &str) -> &'static crate::theme::Theme {
    THEMES
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("{name} is a shipped world"))
}

// ------------------------------------------------------- roster + wiring ---

/// THE BIJECTION. Every world in `THEMES` resolves to exactly ONE committed
/// icon, and every committed icon names a live world — swept off `THEMES`
/// itself and off the directory listing, so neither side can grow a member the
/// other does not have. A new world lands here as a missing file; a retired
/// world lands here as an orphan asset.
#[test]
fn every_shipped_world_resolves_to_exactly_one_committed_icon() {
    let _g = crate::testlock::serial();
    let dir = root().join(WORLD_ICON_DIR);
    let on_disk: BTreeSet<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".icns"))
        .collect();
    let wanted: BTreeSet<String> = THEMES.iter().map(|t| format!("{}.icns", t.name)).collect();

    let missing: Vec<&String> = wanted.difference(&on_disk).collect();
    assert!(missing.is_empty(), "shipped worlds with no icon: {missing:?}");
    let orphans: Vec<&String> = on_disk.difference(&wanted).collect();
    assert!(
        orphans.is_empty(),
        "icons that map back to no live world: {orphans:?}"
    );
    assert_eq!(
        on_disk.len(),
        THEMES.len(),
        "one icon per world, no more and no fewer"
    );
}

/// The EMBEDDED table (generated by `--pack-icns`) is exactly the committed
/// files, in `THEMES` order — the check that catches a table nobody regenerated
/// after adding a world. macOS-only because only that build embeds the bytes.
#[test]
#[cfg(target_os = "macos")]
fn the_embedded_table_is_the_committed_files_in_world_order() {
    let _g = crate::testlock::serial();
    let names: Vec<&str> = THEMES.iter().map(|t| t.name).collect();
    assert_eq!(
        embedded_worlds(),
        names,
        "the generated table is stale — re-run scripts/export-icons.sh"
    );
    for t in THEMES.iter() {
        let embedded = icns_for(t.name).unwrap_or_else(|| panic!("{} embeds an icon", t.name));
        assert_eq!(
            embedded,
            icon_bytes(t.name).as_slice(),
            "{}'s embedded bytes differ from the committed file",
            t.name
        );
    }
}

/// The canonical bundle icon (`CFBundleIconFile`) IS the DEFAULT world's icon,
/// byte for byte. Retargeting the default retargets Finder's icon with it —
/// there is no third, separately-authored artwork to drift.
#[test]
fn the_canonical_bundle_icon_is_the_default_worlds_icon() {
    let _g = crate::testlock::serial();
    let canonical = std::fs::read(root().join(CANONICAL_ICNS)).expect("Awl.icns is committed");
    let w = canonical_world();
    assert_eq!(
        canonical,
        icon_bytes(w.name),
        "Awl.icns must be {}'s icon (the DEFAULT world)",
        w.name
    );
}

/// The taste verdict, PINNED. These eighteen assignments were judged by eye
/// against each face's own `l` at Dock and app-switcher sizes; a silent change
/// to one is a change of the product's face, so it fails here and has to be
/// argued for. The tally is asserted too, because "everything drifted to the
/// block" is what this roster is most exposed to.
#[test]
fn the_shipped_preset_roster_is_the_judged_assignment() {
    let _g = crate::testlock::serial();
    let expected: [(&str, IconCursor); 18] = [
        ("Tawny", IconCursor::Block),
        ("Mopoke", IconCursor::Pill),
        ("Currawong", IconCursor::Pill),
        ("Potoroo", IconCursor::Block),
        ("Gumtree", IconCursor::Block),
        ("Bilby", IconCursor::Block),
        ("Saltpan", IconCursor::Pill),
        ("Quokka", IconCursor::Pill),
        ("Bombora", IconCursor::Block),
        ("Bowerbird", IconCursor::Pill),
        ("Mulga", IconCursor::Pill),
        ("Mangrove", IconCursor::Block),
        ("Galah", IconCursor::Narrow),
        ("Magpie", IconCursor::Block),
        ("Brolga", IconCursor::Pill),
        ("Wagtail", IconCursor::Block),
        ("Firetail", IconCursor::Pill),
        ("Cassowary", IconCursor::Block),
    ];
    for (name, want) in expected {
        assert_eq!(world(name).icon_cursor, want, "{name}'s assigned logo-cursor");
    }
    assert_eq!(expected.len(), THEMES.len(), "every world is named above");

    let count = |c: IconCursor| THEMES.iter().filter(|t| t.icon_cursor == c).count();
    assert_eq!(
        (
            count(IconCursor::Block),
            count(IconCursor::Pill),
            count(IconCursor::Narrow)
        ),
        (9, 8, 1),
        "the judged tally: 9 block / 8 pill / 1 narrow"
    );
}

/// DEFECT 3, resolved: the super-narrow pill sits INSIDE the glyph's advance,
/// so on a footed or serifed face the overhang falls outside it and gets
/// painted `primary_content` out on the ground — the mark reads as `‖` or
/// `aw!`. Figtree's bare geometric stem is the one `l` with nothing to
/// overhang, so it is the one world allowed to wear it. The fix is the
/// assignment, never a bent colour law.
#[test]
fn the_narrow_pill_is_galahs_alone() {
    let _g = crate::testlock::serial();
    let narrow: Vec<&str> = THEMES
        .iter()
        .filter(|t| t.icon_cursor == IconCursor::Narrow)
        .map(|t| t.name)
        .collect();
    assert_eq!(
        narrow,
        vec!["Galah"],
        "only Galah's Figtree stem earns the narrow pill"
    );
    assert_eq!(
        world("Galah").font,
        "Figtree",
        "the reason is the FACE, not the world"
    );
}

/// The two CONFUSABLE PAIRS the judge named: same display face, near-identical
/// ground. Their palettes are world law and cannot move, so the icons are told
/// apart by SILHOUETTE — which means the pair must never collapse onto one
/// preset. Pixel proof that the silhouettes actually differ lives in
/// `small_sizes_keep_every_pair_of_worlds_apart`.
#[test]
fn confusable_pairs_never_share_a_logo_cursor() {
    let _g = crate::testlock::serial();
    for (a, b, same_face, why) in [
        (
            "Potoroo",
            "Firetail",
            true,
            "Monaspace Xenon on near-identical warm-black grounds",
        ),
        (
            "Saltpan",
            "Bilby",
            false,
            "two different serifs, but both cream grounds with a brown/gold mark",
        ),
    ] {
        assert_eq!(
            world(a).font == world(b).font,
            same_face,
            "{a}/{b}: the near-pair's face relationship changed — re-judge the split"
        );
        assert_ne!(
            world(a).icon_cursor,
            world(b).icon_cursor,
            "{a} and {b} are a near-pair ({why}) — the preset split IS the separation"
        );
    }
}

/// Two worlds are LAW-BOUND to the block, not re-judgeable taste: Wagtail
/// because a world with exactly two legal values cannot carry a rounded
/// softness, and Cassowary because its own caret law already draws
/// [`CaretBlockStyle::Filled`] — a lit cell with the glyph knocked out in the
/// ground IS this icon. Each half asserts the TIE, not just the value.
#[test]
fn the_two_law_bound_worlds_keep_their_block() {
    let _g = crate::testlock::serial();
    let wagtail = world("Wagtail");
    assert_eq!(
        wagtail.icon_cursor,
        IconCursor::Block,
        "the 1-bit world's icon is inverse video"
    );
    assert_eq!(
        wagtail.render_caps.selection_style,
        crate::theme::SelectionStyle::InverseVideo,
        "Wagtail is the true 1-bit world this law is about"
    );
    let cassowary = world("Cassowary");
    assert_eq!(cassowary.icon_cursor, IconCursor::Block);
    assert_eq!(
        cassowary.render_caps.caret_block_style,
        CaretBlockStyle::Filled,
        "Cassowary's block icon follows its OWN ink-caret law"
    );
}

// ------------------------------------------------------------- container ---

/// Every committed icon is a well-formed `.icns` carrying the FULL rep roster,
/// each chunk's PNG actually square at the size its OSType claims. A
/// mislabelled or missing rep is exactly what makes macOS shrug and draw the
/// generic application icon, so it fails here rather than in the Dock.
#[test]
fn every_icon_carries_the_full_rep_roster_at_the_declared_sizes() {
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        let bytes = icon_bytes(t.name);
        let chunks = icns::unpack(&bytes).unwrap_or_else(|e| panic!("{}: {e}", t.name));
        assert_eq!(
            chunks.len(),
            REPS.len(),
            "{} carries {} chunks, the roster is {}",
            t.name,
            chunks.len(),
            REPS.len()
        );
        for (rep, (ostype, png)) in REPS.iter().zip(chunks.iter()) {
            assert_eq!(&rep.ostype, ostype, "{}: rep order", t.name);
            let (w, h) = icns::png_size(png)
                .unwrap_or_else(|| panic!("{} rep {} is not a PNG", t.name, rep.name()));
            assert_eq!(
                (w, h),
                (rep.px, rep.px),
                "{} rep {} claims {}px",
                t.name,
                rep.name(),
                rep.px
            );
        }
    }
}

/// REGENERATION IS BYTE-DETERMINISTIC. Re-pack every committed icon from the
/// PNGs inside it: the container is a pure function of its reps, so the bytes
/// must come back identical. Paired with `scripts/export-icons.sh --check`
/// (which re-renders every tile in a second browser and diffs sha256s) this
/// covers the whole pipeline — the render half by the export gate, the pack
/// half here, inside `cargo test`, with no browser involved.
#[test]
fn repacking_a_committed_icon_reproduces_it_byte_for_byte() {
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        let bytes = icon_bytes(t.name);
        let chunks = icns::unpack(&bytes).expect("parses");
        // Feed the packer by SIZE, exactly as `pack_world` feeds it from tiles.
        let mut pngs: Vec<(u32, Vec<u8>)> = Vec::new();
        for (ostype, png) in &chunks {
            let rep = REPS.iter().find(|r| r.ostype == *ostype).expect("known rep");
            if !pngs.iter().any(|(px, _)| *px == rep.px) {
                pngs.push((rep.px, png.to_vec()));
            }
        }
        let repacked = icns::pack(&pngs).expect("re-packs");
        assert_eq!(
            repacked, bytes,
            "{}: re-packing is not byte-identical",
            t.name
        );
    }
}

/// A rep whose PNG is the wrong size is an ERROR, never a silently written
/// container — the guard that keeps the determinism law above meaningful.
#[test]
fn packing_a_mismatched_or_missing_rep_is_an_error() {
    let _g = crate::testlock::serial();
    let bytes = icon_bytes(THEMES[0].name);
    let chunks = icns::unpack(&bytes).expect("parses");
    let small = chunks
        .iter()
        .find(|(t, _)| *t == *b"icp4")
        .map(|(_, p)| p.to_vec())
        .expect("the 16px rep");
    // Offer the 16px PNG for every slot: the first slot that wants something
    // else must refuse.
    let pngs: Vec<(u32, Vec<u8>)> = icns::icns_sizes()
        .into_iter()
        .map(|px| (px, small.clone()))
        .collect();
    let err = icns::pack(&pngs).expect_err("a mismatched rep must not pack");
    assert!(err.to_string().contains("wants"), "{err}");
    // And a MISSING size is an error too, not a skipped chunk.
    let err = icns::pack(&pngs[..1]).expect_err("a missing rep must not pack");
    assert!(err.to_string().contains("none supplied"), "{err}");
}

/// The parser is strict: bad magic, a lying total length and a lying chunk
/// length are all rejected. A lenient reader would let a corrupt committed
/// asset pass every law above.
#[test]
fn the_parser_rejects_a_malformed_container() {
    let _g = crate::testlock::serial();
    let good = icon_bytes(THEMES[0].name);
    assert!(icns::unpack(b"not an icns at all").is_err());
    let mut bad_magic = good.clone();
    bad_magic[0] = b'x';
    assert!(icns::unpack(&bad_magic).is_err(), "bad magic");
    let mut bad_total = good.clone();
    bad_total[7] = bad_total[7].wrapping_add(1);
    assert!(icns::unpack(&bad_total).is_err(), "lying total length");
    // The first chunk's header is bytes 8..16: OSType, then its length.
    let mut bad_chunk = good.clone();
    bad_chunk[13] = 0xff; // now claims more bytes than the file has
    assert!(icns::unpack(&bad_chunk).is_err(), "over-long chunk length");
    let mut zero_chunk = good.clone();
    zero_chunk[12..16].copy_from_slice(&0u32.to_be_bytes()); // shorter than its own header
    assert!(icns::unpack(&zero_chunk).is_err(), "impossible chunk length");
    assert!(
        icns::unpack(&good[..good.len() - 1]).is_err(),
        "a truncated file no longer matches its declared total"
    );
}

// ------------------------------------------------------------ the pixels ---

/// THE FOUR TOKENS, asserted by arithmetic at the Dock's own 128px rep: the
/// ground IS `base_100`, the slab IS `primary`, the `l` knocked out of it IS
/// `primary_content`, and `aw` outside it IS `base_content`. Colour identity is
/// checked against the world's real theme tokens at a tolerance that only
/// admits antialiasing — so a palette retune that never reached the export
/// fails here rather than shipping a wrong-coloured Dock icon.
#[test]
fn every_icon_paints_its_own_four_theme_tokens() {
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        let img = rep_rgba(&icon_bytes(t.name), 128);
        let (w, h) = img.dimensions();
        let area = (w * h) as f64;
        let cursor_mask = mask_of(&img, rgb(t.primary), 6);
        let (slab_px, slab_bbox, slab) = cursor_slab(&cursor_mask, w, h);
        let slab_bbox = slab_bbox.unwrap_or_else(|| panic!("{}: no cursor slab at all", t.name));
        assert!(
            slab_px as f64 >= area * 0.004,
            "{}: the fake cursor is barely painted ({slab_px}px of {area})",
            t.name
        );

        let cursor_ink = letter_mask(&img, &slab, rgb(t.primary_content), 24)
            .iter()
            .filter(|on| **on)
            .count() as u32;
        let mut ground = 0u32;
        let mut wordmark_ink = 0u32;
        for y in 0..h {
            for x in 0..w {
                let p = img.get_pixel(x, y).0;
                if !opaque(&p) {
                    continue;
                }
                if near(&p, rgb(t.base_100), 6) {
                    ground += 1;
                }
                if !slab_bbox.contains(x, y) && near(&p, rgb(t.base_content), 24) {
                    wordmark_ink += 1;
                }
            }
        }
        assert!(
            ground as f64 >= area * 0.30,
            "{}: base_100 is not the dominant ground ({ground}px of {area})",
            t.name
        );
        assert!(
            cursor_ink as f64 >= (area * 0.0006).max(3.0),
            "{}: no primary_content `l` knocked out of the cursor ({cursor_ink}px)",
            t.name
        );
        assert!(
            wordmark_ink as f64 >= area * 0.004,
            "{}: `aw` is not inked in base_content ({wordmark_ink}px)",
            t.name
        );
    }
}

/// THE `l` IS A LEGIBLE, BASELINE-ALIGNED LETTER — not a blank slab and not a
/// stray speck. At the MASTER (512px) and at the Dock's own 128px:
///
///   * the knocked-out ink inside the slab is TALLER THAN WIDE (a stem, not a
///     smudge) and reaches at least half the slab's height;
///   * its BASELINE agrees with the wordmark's: `aw` and `l` are one inline run
///     of text at one size, so their ink must bottom out together. The
///     tolerance scales with the tile (5% of its edge, floored at 2px), which
///     is antialiasing plus the ordinary overshoot of a rounded `a`/`w` bowl.
///
/// The ladder BELOW the Dock size is reported, never gated — `assets/macos/
/// candidates/legibility.txt` records how far down each world keeps its
/// interior `l` (24px for the sans/mono worlds, 64px for the small-x-height
/// serifs), and four worlds honestly stop resolving the letter's own colour
/// before the app switcher does. What survives down there is asserted instead
/// by `the_mark_survives_at_app_switcher_size` and
/// `small_sizes_keep_every_pair_of_worlds_apart`: shape and hue, not the
/// letter. Claiming letter legibility at 32px would be the kind of green that
/// means nothing.
#[test]
fn the_l_reads_as_a_stem_on_the_wordmarks_own_baseline() {
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        for px in [512u32, 128] {
            let img = rep_rgba(&icon_bytes(t.name), px);
            let (w, h) = img.dimensions();
            let cursor_mask = mask_of(&img, rgb(t.primary), 6);
            let (_, slab_bbox, slab) = cursor_slab(&cursor_mask, w, h);
            let slab_bbox =
                slab_bbox.unwrap_or_else(|| panic!("{} @{px}: no cursor slab", t.name));

            // The knocked-out letter: primary_content INTERIOR to the slab (see
            // `letter_mask`). The wordmark: base_content outside the slab.
            let stem = letter_mask(&img, &slab, rgb(t.primary_content), 16);
            let mut word: Option<Bbox> = None;
            for y in 0..h {
                for x in 0..w {
                    let p = img.get_pixel(x, y).0;
                    if opaque(&p) && !slab_bbox.contains(x, y) && near(&p, rgb(t.base_content), 16)
                    {
                        Bbox::grow(&mut word, x, y);
                    }
                }
            }
            let slab = slab_bbox;
            let stem = mask_bbox(&stem, w)
                .unwrap_or_else(|| panic!("{} @{px}: the `l` is not knocked out", t.name));
            let word = word.unwrap_or_else(|| panic!("{} @{px}: `aw` has no ink", t.name));
            assert!(
                stem.h() > stem.w(),
                "{} @{px}: the knocked-out `l` is {}x{} — wider than tall is not a stem",
                t.name,
                stem.w(),
                stem.h()
            );
            assert!(
                stem.h() as f64 >= slab.h() as f64 * 0.5,
                "{} @{px}: the `l` reaches only {} of the slab's {}px",
                t.name,
                stem.h(),
                slab.h()
            );
            let tol = ((px as f64 * 0.05).ceil() as i64).max(2);
            let delta = (stem.y1 as i64 - word.y1 as i64).abs();
            assert!(
                delta <= tol,
                "{} @{px}: the `l` bottoms at y={} but `aw` at y={} (tolerance {tol}px) — \
                 they are one inline run and must share a baseline",
                t.name,
                stem.y1,
                word.y1
            );
        }
    }
}

/// THE SAFE AREA, measured against the icon's own SHAPE rather than a square.
/// Two claims, by arithmetic at the 512px master:
///
///   * the tile is a real SQUIRCLE — its four corners are fully transparent, so
///     the Dock draws awl's shape rather than a square nobody designed;
///   * no INK comes near the edge. For every pixel that is neither the ground
///     nor transparent, the distance to the icon's own opaque boundary — along
///     that pixel's row AND its column, so the measurement follows the rounded
///     corner instead of assuming a rectangle — is at least 4% of the tile.
///
/// Why 4%: the measured worst case across the whole shipped roster is 5.86%
/// (Potoroo and Firetail, whose Monaspace Xenon carries the widest advances);
/// the roomiest is Bombora at 17.4%. The floor sits under the worst case with
/// margin for antialiasing wobble, and well above zero — so a lockup that grew
/// into the edge, or a face swap that widened the wordmark past the tile, fails
/// here rather than shipping an icon the corner rounding clips.
#[test]
fn no_ink_escapes_the_safe_area_and_the_corners_stay_clear() {
    let _g = crate::testlock::serial();
    const CLEARANCE: f64 = 0.04;
    for t in THEMES.iter() {
        let img = rep_rgba(&icon_bytes(t.name), 512);
        let (w, h) = img.dimensions();
        for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)] {
            assert_eq!(
                img.get_pixel(x, y).0[3],
                0,
                "{}: corner ({x},{y}) is not transparent — the squircle is missing",
                t.name
            );
        }
        // The icon's opaque extent per row and per column: the boundary the
        // clearance is measured against.
        let mut row: Vec<Option<(u32, u32)>> = vec![None; h as usize];
        let mut col: Vec<Option<(u32, u32)>> = vec![None; w as usize];
        for y in 0..h {
            for x in 0..w {
                if !opaque(&img.get_pixel(x, y).0) {
                    continue;
                }
                row[y as usize] = Some(match row[y as usize] {
                    None => (x, x),
                    Some((a, b)) => (a.min(x), b.max(x)),
                });
                col[x as usize] = Some(match col[x as usize] {
                    None => (y, y),
                    Some((a, b)) => (a.min(y), b.max(y)),
                });
            }
        }
        let floor = (w as f64 * CLEARANCE) as u32;
        let mut worst = u32::MAX;
        for y in 0..h {
            for x in 0..w {
                let p = img.get_pixel(x, y).0;
                if !opaque(&p) || near(&p, rgb(t.base_100), 10) {
                    continue;
                }
                let (x0, x1) = row[y as usize].expect("an opaque pixel has a row span");
                let (y0, y1) = col[x as usize].expect("an opaque pixel has a column span");
                worst = worst.min(x - x0).min(x1 - x).min(y - y0).min(y1 - y);
            }
        }
        assert!(
            worst != u32::MAX,
            "{}: the icon has no ink at all, only ground",
            t.name
        );
        assert!(
            worst >= floor,
            "{}: ink comes within {worst}px of the icon's edge ({:.2}% of the tile); \
             the floor is {floor}px ({:.0}%)",
            t.name,
            worst as f64 / w as f64 * 100.0,
            CLEARANCE * 100.0
        );
    }
}

/// AT APP-SWITCHER SIZE the letter stops being the claim and the MARK is: at
/// 32px every world still paints a real cursor slab in its own `primary`, and
/// still carries non-ground ink. That is what the verdict says survives down
/// there ("mark-shape and hue"), and it is all that is asserted — the 16px slot
/// carries ground plus a speck on every candidate, which is why no test claims
/// anything about it.
#[test]
fn the_mark_survives_at_app_switcher_size() {
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        let img = rep_rgba(&icon_bytes(t.name), 32);
        let (w, h) = img.dimensions();
        let area = (w * h) as f64;
        let (slab_px, slab, _) = cursor_slab(&mask_of(&img, rgb(t.primary), 8), w, h);
        assert!(
            slab.is_some() && slab_px as f64 >= area * 0.004,
            "{} @32: the cursor slab does not survive ({slab_px}px of {area})",
            t.name
        );
        let ink = img
            .pixels()
            .filter(|p| opaque(&p.0) && !near(&p.0, rgb(t.base_100), 10))
            .count();
        assert!(
            ink as f64 >= area * 0.05,
            "{} @32: only {ink}px of {area} are anything but ground",
            t.name
        );
    }
}

/// EVERY PAIR OF WORLDS STAYS APART at app-switcher size — and WHICH pair stays
/// apart least is COMPUTED from the rendered set on every run, never guessed.
///
/// The predecessor of this law hand-picked two "near pairs" by shared-FACE
/// reasoning (Potoroo/Firetail, Saltpan/Bilby) and asserted those two harder. It
/// missed the actual global minimum by a wide margin: **Currawong/Cassowary
/// differ on only 12.3% of their 32px pixels**, because those two share a
/// near-black GROUND (`#050506` vs `#060607`) and at 32px the ground IS most of
/// the tile. Three things were wrong with the list, and all three are the same
/// mistake — a human predicting which pair to watch:
///   * it missed the minimum, which was pinned by NOTHING but the generic floor;
///   * shared face is *anti*-predictive — the five same-face pairs span 12.3%
///     (Currawong/Cassowary) to 97.3% (Mopoke/Magpie), so the criterion selects
///     nothing;
///   * one of the two hand-picked pairs did not even share a face (Saltpan is
///     Fraunces, Bilby is Newsreader) — the list had drifted off its own stated
///     rule, silently, because a name list is not checked against anything.
///
/// So nothing here names a pair. All 153 combinations are measured, the minimum
/// on each axis is found, and the MINIMUM is what the assertions bind. A world
/// rename cannot make this sweep quietly assert less, the way
/// `near_pairs.contains(..)` could.
///
/// THREE AXES, because the obvious one is a liar on its own:
///   * `differing` — fraction of pixels that visibly differ. Ground-dominated;
///     it is exactly what hid Currawong/Cassowary.
///   * `mean` — mean channel distance across the whole tile, UNthresholded, so a
///     pair differing a lot on a few pixels cannot read as identical.
///   * `ink` — fraction of NON-GROUND pixels that differ. The ground-independent
///     axis. It is the one that shows the near-black twins are in fact fine (86%
///     of their ink differs — mint-green vs golden-yellow), and it finds a
///     DIFFERENT global minimum: Potoroo/Firetail at 51%. Two axes, two
///     different closest pairs — which is the whole reason one hand-picked list
///     could never have covered this.
///
/// EACH AXIS CARRIES TWO BARS, doing different jobs:
///   * an ABSOLUTE FLOOR — the perceptual claim, "these two are not the same
///     app". Deliberately loose, and deliberately NOT retuned to sit just under
///     today's measurement: a genuine near-duplicate world lands near ZERO (see
///     the non-vacuity note below), so the floor does not need to be tight to
///     catch one, and a floor at 12% would encode the false claim that 12.3% is
///     near the edge of legibility when the ink axis says that pair differs on
///     86% of its ink. `differing`'s floor is the 10% this law already carried —
///     chosen before anyone had measured the minimum, and therefore not tuned to
///     it.
///   * a RATCHET against today's measured minimum, with [`RATCHET_SLACK`] of
///     relative give for an exporter re-render's antialiasing. This is the bar
///     that actually notices EROSION, and it needs no theory of which pair is at
///     risk: a new world that crowds ANY existing one below the current worst
///     fails here and is NAMED. It also survives roster growth, which a
///     per-pair baseline table would not — adding a nineteenth world adds
///     eighteen pairs and invalidates nothing.
///
/// The stricter 20% tier the two hand-picked pairs used to carry is GONE, not
/// weakened: its premise (same face ⇒ at risk) is false by measurement above,
/// applying it to the computed same-face set would fail today, and the one real
/// thing it pinned — Potoroo/Firetail's silhouette doing the work its palette
/// does not — is now pinned harder and roster-wide by the `ink` axis, on which
/// that pair IS the global minimum and therefore holds the ratchet.
///
/// Non-vacuity: adding a world that clones an existing palette and preset drives
/// `differing` to ~1% and trips the floor and all three ratchets, naming the
/// cloned pair.
#[test]
fn small_sizes_keep_every_pair_of_worlds_apart() {
    let _g = crate::testlock::serial();

    /// Relative give on each ratchet: enough to absorb a handful of
    /// antialiased pixels shifting when the offline exporter is re-run, far
    /// too little to absorb a pair actually moving closer together.
    const RATCHET_SLACK: f64 = 0.02;

    /// One pair's separation at 32px, on every axis this law knows.
    struct Pair {
        a: &'static str,
        b: &'static str,
        differing: f64,
        mean: f64,
        ink: f64,
    }

    let imgs: Vec<(&'static Theme, image::RgbaImage)> = THEMES
        .iter()
        .map(|t| (t, rep_rgba(&icon_bytes(t.name), 32)))
        .collect();

    let mut pairs: Vec<Pair> = Vec::new();
    for i in 0..imgs.len() {
        for j in (i + 1)..imgs.len() {
            let (ta, a) = &imgs[i];
            let (tb, b) = &imgs[j];
            let total = (a.width() * a.height()) as f64;
            let (mut differing, mut sum, mut ink, mut ink_differing) = (0u32, 0u64, 0u32, 0u32);
            for (pa, pb) in a.pixels().zip(b.pixels()) {
                let d: u32 = (0..4)
                    .map(|k| (pa.0[k] as i32 - pb.0[k] as i32).unsigned_abs())
                    .sum();
                let visible = d > 24;
                if visible {
                    differing += 1;
                }
                sum += d as u64;
                // A pixel counts as INK when it is non-ground in EITHER world —
                // each measured against its OWN base_100, since the whole point
                // of this axis is that the two grounds may be different colours
                // and still both be ground.
                let ground_a = !opaque(&pa.0) || near(&pa.0, rgb(ta.base_100), 10);
                let ground_b = !opaque(&pb.0) || near(&pb.0, rgb(tb.base_100), 10);
                if !ground_a || !ground_b {
                    ink += 1;
                    if visible {
                        ink_differing += 1;
                    }
                }
            }
            assert!(ink > 0, "{} vs {}: neither icon has any ink", ta.name, tb.name);
            pairs.push(Pair {
                a: ta.name,
                b: tb.name,
                differing: differing as f64 / total,
                mean: sum as f64 / total,
                ink: ink_differing as f64 / ink as f64,
            });
        }
    }
    assert_eq!(
        pairs.len(),
        THEMES.len() * (THEMES.len() - 1) / 2,
        "every combination of worlds is measured"
    );

    // (axis name, how to read it off a pair, absolute floor, measured baseline,
    // how to print a value). The baselines are what THIS roster measures today;
    // re-blessing one is a deliberate one-line edit that says "the icon set got
    // closer together on purpose".
    type Read = fn(&Pair) -> f64;
    let axes: [(&str, Read, f64, f64, bool); 3] = [
        ("differing pixels", |p| p.differing, 0.10, 0.12305, true),
        ("mean channel distance", |p| p.mean, 10.0, 19.52, false),
        ("differing INK pixels", |p| p.ink, 0.40, 0.5101, true),
    ];

    for (name, read, floor, baseline, pct) in axes {
        let show = |v: f64| if pct { format!("{:.2}%", v * 100.0) } else { format!("{v:.2}") };
        let mut sorted: Vec<&Pair> = pairs.iter().collect();
        sorted.sort_by(|x, y| read(x).total_cmp(&read(y)));
        let closest = sorted[0];
        let worst = read(closest);
        // The five nearest on this axis, so a failure hands over the
        // neighbourhood and not just a verdict.
        let roll = sorted
            .iter()
            .take(5)
            .map(|p| format!("{} vs {} = {}", p.a, p.b, show(read(p))))
            .collect::<Vec<_>>()
            .join("; ");

        assert!(
            worst >= floor,
            "{} vs {} are the closest pair on {name} at 32px ({}) and fall under the \
             absolute floor of {} — at app-switcher size they read as one app. \
             Five nearest: {roll}",
            closest.a,
            closest.b,
            show(worst),
            show(floor),
        );
        let ratchet = baseline * (1.0 - RATCHET_SLACK);
        assert!(
            worst >= ratchet,
            "{} vs {} are now the closest pair on {name} at 32px ({}), below the \
             measured baseline of {} (ratchet {}). Some change moved two worlds' icons \
             closer together than any pair has ever been. Either back it out, or — if \
             the crowding is intended — re-bless the baseline in this law and say why. \
             Five nearest: {roll}",
            closest.a,
            closest.b,
            show(worst),
            show(baseline),
            show(ratchet),
        );
    }
}

// ------------------------------------------------------------ the packer ---

/// The tile the packer asks for is the one the JUDGE picked: name, preset and
/// size, in the exporter's own convention. A rename that half-lands (packer vs
/// exporter) fails here rather than by quietly packing a stale candidate.
#[test]
fn the_packer_asks_for_the_worlds_assigned_preset() {
    let _g = crate::testlock::serial();
    assert_eq!(
        icns::tile_file_name(world("Galah"), 128),
        "Galah-narrow-128.png"
    );
    assert_eq!(
        icns::tile_file_name(world("Wagtail"), 1024),
        "Wagtail-block-1024.png"
    );
    for t in THEMES.iter() {
        assert!(
            icns::tile_file_name(t, 32).contains(t.icon_cursor.slug()),
            "{} packs its own preset",
            t.name
        );
    }
}

/// A missing tile STOPS the pack, naming the file — never a silently short
/// container (which is how a world would end up with a half-populated icon and
/// a Dock fallback nobody noticed).
#[test]
fn a_missing_tile_stops_the_pack() {
    let _g = crate::testlock::serial();
    let err =
        icns::pack_world(Path::new("/nonexistent/tiles"), &THEMES[0]).expect_err("must fail");
    let msg = err.to_string();
    assert!(
        msg.contains(THEMES[0].name) && msg.contains("export-icons.sh"),
        "{msg}"
    );
}

/// The rep roster's sizes are distinct and ascending, and every slot's size is
/// one of them — the derivation the pack step walks.
#[test]
fn the_rep_roster_derives_its_tile_sizes() {
    let _g = crate::testlock::serial();
    let sizes = icns::icns_sizes();
    assert_eq!(sizes, vec![16, 32, 64, 128, 256, 512, 1024]);
    for r in REPS.iter() {
        assert!(sizes.contains(&r.px), "{} has no tile size", r.name());
    }
}

/// The three logo-cursor slugs are the exporter's three preset keys and
/// nothing else — a no-wildcard match, so a fourth shape fails to compile
/// rather than exporting as one of the three by accident.
#[test]
fn the_cursor_slugs_are_the_exporters_preset_keys() {
    let _g = crate::testlock::serial();
    let slugs: Vec<&str> = IconCursor::ALL.iter().map(|c| c.slug()).collect();
    assert_eq!(slugs, vec!["block", "pill", "narrow"]);
    let tuning = std::fs::read_to_string(root().join("scripts/icons/tuning.json"))
        .expect("the exporter's tuning is committed");
    for slug in slugs {
        assert!(
            tuning.contains(&format!("\"{slug}\": {{")),
            "tuning.json has no `{slug}` preset"
        );
    }
    for t in THEMES.iter() {
        assert!(
            IconCursor::ALL.contains(&t.icon_cursor),
            "{} wears a shape outside the roster",
            t.name
        );
    }
}
