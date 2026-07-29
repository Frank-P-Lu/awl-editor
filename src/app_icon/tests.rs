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
    let want = REPS
        .iter()
        .find(|r| r.px == px)
        .expect("a rep at this size");
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
            None => Bbox {
                x0: x,
                y0: y,
                x1: x,
                y1: y,
            },
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
fn letter_mask(img: &image::RgbaImage, slab: &[bool], ink: [u8; 3], tol: i32) -> Vec<bool> {
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
    assert!(
        missing.is_empty(),
        "shipped worlds with no icon: {missing:?}"
    );
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

/// The taste verdict, PINNED. These nineteen assignments were judged by eye
/// against each face's own `l` at Dock and app-switcher sizes; a silent change
/// to one is a change of the product's face, so it fails here and has to be
/// argued for. The tally is asserted too, because "everything drifted to the
/// block" is what this roster is most exposed to.
#[test]
fn the_shipped_preset_roster_is_the_judged_assignment() {
    let _g = crate::testlock::serial();
    let expected: [(&str, IconCursor); 19] = [
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
        // Item 121: the poster silhouette moved off the pill onto the shared
        // Block preset; the ground is the user's C pick (`IconGround::
        // Blend40`), asserted by name in
        // `every_shipped_world_defaults_to_the_inert_base_100_ground_except_firetail`.
        ("Firetail", IconCursor::Block),
        ("Cassowary", IconCursor::Block),
        // ITEM 158: EB Garamond's `l` is footed, so Narrow (which sits INSIDE
        // the glyph's advance) is out by the law two tests down; Block is the
        // preset the same face already carries on Bombora, and it splits
        // Paperbark from Saltpan, the other warm-cream world, in a dock row.
        ("Paperbark", IconCursor::Block),
    ];
    for (name, want) in expected {
        assert_eq!(
            world(name).icon_cursor,
            want,
            "{name}'s assigned logo-cursor"
        );
    }
    assert_eq!(expected.len(), THEMES.len(), "every world is named above");

    let count = |c: IconCursor| THEMES.iter().filter(|t| t.icon_cursor == c).count();
    assert_eq!(
        (
            count(IconCursor::Block),
            count(IconCursor::Pill),
            count(IconCursor::Narrow)
        ),
        (11, 7, 1),
        "the judged tally: 11 block / 7 pill / 1 narrow"
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

/// A CONFUSABLE PAIR the judge named: same display face, near-identical
/// ground, palette held as world law so the icons are told apart by
/// SILHOUETTE — which means the pair must never collapse onto one preset.
/// Pixel proof that the silhouettes actually differ lives in
/// `small_sizes_keep_every_pair_of_worlds_apart`.
///
/// Potoroo/Firetail USED to be this list's other entry: same face, same
/// preset-split strategy. Item 121 moved Firetail off Pill onto the shared
/// Block preset — the poster silhouette the judge actually wants — so the
/// two now share a preset ON PURPOSE, and the silhouette-split strategy is
/// retired for this pair. Their separation is carried instead by the
/// numeric palette-distinctness laws
/// (`firetail_is_oxblood_wine_and_ember_not_potoroo_rust_or_bombora_violet`,
/// `theme::tests`) and by the roster-wide crowding sweep below, which is
/// free to name Potoroo/Firetail in its `Blessed` lists if they now crowd —
/// that is a conscious, reviewed acceptance, not a silent regression.
#[test]
fn confusable_pairs_never_share_a_logo_cursor() {
    let _g = crate::testlock::serial();
    assert_eq!(
        world("Potoroo").icon_cursor,
        world("Firetail").icon_cursor,
        "item 121: Potoroo and Firetail now deliberately share the Block preset"
    );
    let (a, b, same_face, why) = (
        "Saltpan",
        "Bilby",
        false,
        "two different serifs, but both cream grounds with a brown/gold mark",
    );
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
            let rep = REPS
                .iter()
                .find(|r| r.ostype == *ostype)
                .expect("known rep");
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
    icns::unpack(b"not an icns at all").unwrap_err();
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
    assert!(
        icns::unpack(&zero_chunk).is_err(),
        "impossible chunk length"
    );
    assert!(
        icns::unpack(&good[..good.len() - 1]).is_err(),
        "a truncated file no longer matches its declared total"
    );
}

// ------------------------------------------------------------ the pixels ---

/// THE FOUR TOKENS, asserted by arithmetic at the Dock's own 128px rep: the
/// ground IS `Theme::icon_ground_color()` (`base_100` unless the world opted
/// into an item-121 blend), the slab IS `primary`, the `l` knocked out of it
/// IS `primary_content`, and `aw` outside it IS `base_content`. Colour identity is
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
                if near(&p, rgb(t.icon_ground_color()), 6) {
                    ground += 1;
                }
                if !slab_bbox.contains(x, y) && near(&p, rgb(t.base_content), 24) {
                    wordmark_ink += 1;
                }
            }
        }
        assert!(
            ground as f64 >= area * 0.30,
            "{}: icon_ground_color() is not the dominant ground ({ground}px of {area})",
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
            let slab_bbox = slab_bbox.unwrap_or_else(|| panic!("{} @{px}: no cursor slab", t.name));

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
                if !opaque(&p) || near(&p, rgb(t.icon_ground_color()), 10) {
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
            .filter(|p| opaque(&p.0) && !near(&p.0, rgb(t.icon_ground_color()), 10))
            .count();
        assert!(
            ink as f64 >= area * 0.05,
            "{} @32: only {ink}px of {area} are anything but ground",
            t.name
        );
    }
}

/// One pair's separation at 32px, on every axis the crowding law (below)
/// knows. Module-level (not local to the law) so the non-vacuity probes
/// (`ibis_near_duplicate_is_caught_without_becoming_champion`,
/// `ordinary_new_world_passes_the_danger_zone_guard`,
/// `ground_clone_crowding_is_caught_regardless_of_rank`,
/// `roster_growth_does_not_dilute_a_known_erosion`) can build a `Pair` for a
/// world that was never exported and run it through the SAME check the law
/// runs, rather than a hand-reimplemented copy that could silently drift
/// from what ships.
#[derive(Clone)]
struct Pair {
    a: &'static str,
    b: &'static str,
    differing: f64,
    mean: f64,
    ink: f64,
}

/// Measures one pair of already-rendered 32px images against their OWN
/// ground colours. `a_ground`/`b_ground` are each caller-supplied (rather
/// than looked up on a `Theme`) so a synthetic image with no `Theme` of its
/// own — `Ibis`, `Probeworld` — can still be measured by this exact
/// arithmetic.
fn measure_pair(
    a_name: &'static str,
    a_img: &image::RgbaImage,
    a_ground: [u8; 3],
    b_name: &'static str,
    b_img: &image::RgbaImage,
    b_ground: [u8; 3],
) -> Pair {
    let total = (a_img.width() * a_img.height()) as f64;
    let (mut differing, mut sum, mut ink, mut ink_differing) = (0u32, 0u64, 0u32, 0u32);
    for (pa, pb) in a_img.pixels().zip(b_img.pixels()) {
        let d: u32 = (0..4)
            .map(|k| (pa.0[k] as i32 - pb.0[k] as i32).unsigned_abs())
            .sum();
        let visible = d > 24;
        if visible {
            differing += 1;
        }
        sum += d as u64;
        // A pixel counts as INK when it is non-ground in EITHER world — each
        // measured against its OWN ground, since the whole point of this axis
        // is that the two grounds may be different colours and still both be
        // ground.
        let ground_a = !opaque(&pa.0) || near(&pa.0, a_ground, 10);
        let ground_b = !opaque(&pb.0) || near(&pb.0, b_ground, 10);
        if !ground_a || !ground_b {
            ink += 1;
            if visible {
                ink_differing += 1;
            }
        }
    }
    assert!(ink > 0, "{a_name} vs {b_name}: neither icon has any ink");
    Pair {
        a: a_name,
        b: b_name,
        differing: differing as f64 / total,
        mean: sum as f64 / total,
        ink: ink_differing as f64 / ink as f64,
    }
}

/// Every real pair the shipped roster produces at 32px — the 153
/// combinations `small_sizes_keep_every_pair_of_worlds_apart` sweeps. Reads
/// `THEMES` (the one roster array every world sweep in this file reads, per
/// this file's own module doc) and nothing else, so a world added to
/// `THEMES` is automatically in this set with no second list to update.
fn all_real_pairs() -> Vec<Pair> {
    let imgs: Vec<(&'static Theme, image::RgbaImage)> = THEMES
        .iter()
        .map(|t| (t, rep_rgba(&icon_bytes(t.name), 32)))
        .collect();
    let mut pairs = Vec::new();
    for i in 0..imgs.len() {
        for j in (i + 1)..imgs.len() {
            let (ta, a) = &imgs[i];
            let (tb, b) = &imgs[j];
            pairs.push(measure_pair(
                ta.name,
                a,
                rgb(ta.icon_ground_color()),
                tb.name,
                b,
                rgb(tb.icon_ground_color()),
            ));
        }
    }
    pairs
}

/// Relative give on a blessed pair's erosion ratchet. Two independent runs of
/// the pinned offline exporter produced byte-identical output for all 540
/// generated tiles: the measured noise floor is zero pixels. The 0.05%
/// allowance is honest headroom for the committed decimal precision while
/// remaining tighter than a one-pixel erosion of a 32px tile.
const RATCHET_SLACK: f64 = 0.0005;

type Read = fn(&Pair) -> f64;

/// One pair already accepted into an axis's DANGER ZONE (below that axis's
/// `danger` threshold in [`axes`]): today it crowds meaningfully and a human
/// decided that crowding is fine. Order-insensitive against a measured
/// [`Pair`] — `a`/`b` here may come back either way from `all_real_pairs`,
/// which iterates `THEMES` by index, not by name.
///
/// This is deliberately NOT a per-pair table for the whole 153-pair roster —
/// item 99 already rejected that shape for breaking on every world addition.
/// It is a per-pair table ONLY for the pairs the roster's OWN measured
/// distribution already put in the danger zone: small today (12/10/5 entries
/// across the three axes), and it grows only when a pair genuinely crosses
/// into the zone, never merely because the roster gained a member. See
/// `small_sizes_keep_every_pair_of_worlds_apart`'s doc for the full case.
struct Blessed {
    a: &'static str,
    b: &'static str,
    /// The value measured for this pair on this axis the day it was
    /// blessed. [`RATCHET_SLACK`] of relative give below this before further
    /// movement counts as new erosion.
    baseline: f64,
}

impl Blessed {
    fn matches(&self, p: &Pair) -> bool {
        (self.a == p.a && self.b == p.b) || (self.a == p.b && self.b == p.a)
    }
}

/// Measured 2026-07-26 (item 102): every pair whose `differing` sits under
/// 28.33% today, i.e. the whole low cluster below the roster's own cliff to
/// 80.47% (Potoroo/Firetail) — see `axes()`'s `danger` value.
const DIFFERING_BLESSED: &[Blessed] = &[
    Blessed {
        a: "Currawong",
        b: "Cassowary",
        baseline: 0.130859375,
    },
    // Item 161's restrained optical lift moved Bilby's rendered cursor body
    // without changing palette, preset, or containment.
    Blessed {
        a: "Bilby",
        b: "Galah",
        baseline: 0.197265625,
    },
    Blessed {
        a: "Mopoke",
        b: "Mulga",
        baseline: 0.2119140625,
    },
    // Item 161.
    Blessed {
        a: "Bilby",
        b: "Magpie",
        baseline: 0.240234375,
    },
    // Item 161: Saltpan's own restrained lift.
    Blessed {
        a: "Saltpan",
        b: "Quokka",
        baseline: 0.236328125,
    },
    Blessed {
        a: "Galah",
        b: "Magpie",
        baseline: 0.240234375,
    },
    // Item 110 aligned every face's rendered baseline; the intentional seat
    // change moved this cream-ground near-pair without changing palette or
    // its deliberately split block/pill silhouettes. Item 161's optical lift
    // — applied to both Bilby and Saltpan, restrained on each — moved it
    // again.
    Blessed {
        a: "Bilby",
        b: "Saltpan",
        baseline: 0.2236328125,
    },
    Blessed {
        a: "Tawny",
        b: "Mulga",
        baseline: 0.2685546875,
    },
    Blessed {
        a: "Tawny",
        b: "Mopoke",
        baseline: 0.2822265625,
    },
    Blessed {
        a: "Tawny",
        b: "Bowerbird",
        baseline: 0.287109375,
    },
    Blessed {
        a: "Currawong",
        b: "Wagtail",
        baseline: 0.2958984375,
    },
    // ITEM 158 (Paperbark, the nineteenth world). At 32px the tile is mostly
    // GROUND — this axis's own doc says so — and Paperbark joins awl's existing
    // pale-warm light cluster, whose members already bless each other here
    // (Bilby/Galah 19.9%, Bilby/Saltpan 22.9%, Bilby/Magpie 24.2%). Its page
    // colour is world law, authored with the room, not an icon choice. What
    // separates these tiles at Dock size is the mark and the silhouette, and
    // that is the `ink` axis — on which Paperbark crowds NOBODY (it appears in
    // no INK_BLESSED entry). It does not become the champion on any axis
    // either: the roster minimum stays Currawong/Cassowary at 13.1%.
    Blessed {
        a: "Galah",
        b: "Paperbark",
        baseline: 0.2021484375,
    },
    Blessed {
        a: "Bilby",
        b: "Paperbark",
        // Re-blessed at integration: item 158 measured Paperbark against
        // Bilby's PRE-LIFT cursor, item 161 then lifted it. 20.31% -> 20.12%.
        baseline: 0.2012,
    },
    Blessed {
        a: "Saltpan",
        b: "Paperbark",
        // Re-blessed at integration, same cause as Bilby/Paperbark above:
        // Saltpan's cursor moved in item 161. 23.44% -> 23.14%.
        baseline: 0.2314,
    },
    Blessed {
        a: "Magpie",
        b: "Paperbark",
        baseline: 0.26953125,
    },
    Blessed {
        a: "Wagtail",
        b: "Cassowary",
        baseline: 0.2978515625,
    },
];

/// Measured 2026-07-26 (item 102): every pair whose `mean` sits under 70.0
/// today — the roster's smooth low continuum stops being worth watching
/// individually well before its own distant cliff (173.79 -> 473.61,
/// Bombora/Wagtail -> Gumtree/Mangrove), so this axis's `danger` is set with
/// margin above the tightest 10, not at that far cliff (see `axes()`'s doc).
const MEAN_BLESSED: &[Blessed] = &[
    Blessed {
        a: "Currawong",
        b: "Cassowary",
        baseline: 20.7490234375,
    },
    // Item 121: Firetail's icon cursor moved Pill -> Block (32.87 -> 25.94),
    // then its ground moved to the user's C pick, `IconGround::Blend40`
    // (25.94 -> 44.89) — the wine ground pulls it noticeably further from
    // Potoroo's rust, but not out of this axis's danger zone, so it stays
    // blessed at the new measured value.
    Blessed {
        a: "Potoroo",
        b: "Firetail",
        baseline: 44.88671875,
    },
    Blessed {
        a: "Galah",
        b: "Brolga",
        baseline: 55.6943359375,
    },
    Blessed {
        a: "Mopoke",
        b: "Mulga",
        baseline: 57.2373046875,
    },
    Blessed {
        a: "Tawny",
        b: "Mangrove",
        baseline: 57.84375,
    },
    // Item 161's restrained lift moved Saltpan's rendered ink.
    Blessed {
        a: "Saltpan",
        b: "Galah",
        baseline: 68.1943359375,
    },
    // Item 110's vertical-seat correction moves Bilby's rendered ink while
    // preserving every palette and cursor assignment. Item 161's further
    // optical lift moved it again.
    Blessed {
        a: "Bilby",
        b: "Galah",
        baseline: 60.4755859375,
    },
    Blessed {
        a: "Bilby",
        b: "Saltpan",
        baseline: 62.83203125,
    },
    Blessed {
        a: "Bowerbird",
        b: "Mulga",
        baseline: 67.619140625,
    },
    // Item 121: Tawny/Firetail was blessed here through two Firetail changes
    // (69.75 -> 66.60 for Block; see git history) — the user's C ground pick
    // (`IconGround::Blend40`) then widened it PAST the danger zone entirely
    // (66.60 -> 71.49, >= the 70.0 threshold), so the entry is REMOVED, not
    // re-blessed: `stale_blessed_entry_for_a_widened_pair_is_flagged` is the
    // law that would catch a stale entry left behind here.
    // ITEM 158 — the same pale-warm ground cluster as DIFFERING_BLESSED's own
    // Paperbark note; Bilby/Galah and Bilby/Saltpan are already blessed here
    // for the identical reason.
    Blessed {
        a: "Bilby",
        b: "Paperbark",
        // Re-blessed at integration (item 161 + item 158): 158 measured this
        // against Bilby's PRE-LIFT pixels, then 161 lifted Bilby's cursor, so
        // the pair moved 52.79 -> 52.47. Neither branch could see this alone.
        baseline: 52.47,
    },
    Blessed {
        a: "Galah",
        b: "Paperbark",
        baseline: 64.453125,
    },
];

/// Measured 2026-07-26 (item 102): every pair whose `ink` sits under 92%
/// today — the roster's own cluster tops out at 88.39% (Bowerbird/Firetail)
/// before a cliff to 94.22%; 92% sits between the two so the axis still
/// catches a scenario shaped like `Ibis` (85.43% after item 161, see
/// `ibis_near_duplicate_is_caught_without_becoming_champion`) without
/// pulling the entire 94%+ plateau into the blessed list.
const INK_BLESSED: &[Blessed] = &[
    // Item 121: Firetail's Block cursor first closed Potoroo/Firetail's ink
    // to 49.81% (from 54.31%, briefly the roster's global minimum) — then
    // the user's C ground pick (`IconGround::Blend40`) reopened it to
    // 66.42%, re-blessed at the new measured value. See
    // `confusable_pairs_never_share_a_logo_cursor`'s doc for why the
    // preset-split strategy was retired for this pair on purpose; the ground
    // change is what actually repairs the crowding it introduced.
    Blessed {
        a: "Potoroo",
        b: "Firetail",
        baseline: 0.6641509433962264,
    },
    // Item 161's restrained lift moved Bilby's rendered ink.
    Blessed {
        a: "Bilby",
        b: "Wagtail",
        baseline: 0.7058823529411765,
    },
    Blessed {
        a: "Magpie",
        b: "Wagtail",
        baseline: 0.8167202572347267,
    },
    Blessed {
        a: "Currawong",
        b: "Cassowary",
        baseline: 0.864_516_129_032_258,
    },
    Blessed {
        a: "Bowerbird",
        b: "Firetail",
        baseline: 0.8763250883392226,
    },
];

/// (axis name, how to read it off a pair, absolute floor, danger-zone
/// threshold, today's blessed pairs inside that zone, pct-format flag).
///
/// Re-blessing a NEW pair that crowds into the zone is a deliberate edit:
/// add one `Blessed` entry to the axis's list above, with a comment naming
/// the pair and why the crowding is accepted. A pair already blessed eroding
/// FURTHER is the same edit — bump its `baseline`. Either way the diff is
/// small and reads as what it is: item 99 already rejected a per-pair table
/// sized to the whole roster; this one is sized to the roster's own measured
/// danger zone instead, which is not the same shape.
// Each axis couples its metric, thresholds, exceptions, and direction as one fixed test table.
#[allow(clippy::type_complexity)]
fn axes() -> [(&'static str, Read, f64, f64, &'static [Blessed], bool); 3] {
    [
        (
            "differing pixels",
            |p| p.differing,
            0.10,
            0.35,
            DIFFERING_BLESSED,
            true,
        ),
        (
            "mean channel distance",
            |p| p.mean,
            10.0,
            70.0,
            MEAN_BLESSED,
            false,
        ),
        (
            "differing INK pixels",
            |p| p.ink,
            0.40,
            0.92,
            INK_BLESSED,
            true,
        ),
    ]
}

/// EVERY PAIR OF WORLDS STAYS APART at app-switcher size — and WHICH pairs
/// stay apart least are COMPUTED from the rendered set on every run, never
/// guessed.
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
///     it. (The floor is checked only against `sorted[0]`: it is monotone in
///     rank — if the single closest pair clears it, every less-close pair
///     necessarily does too.)
///   * a DANGER-ZONE MEMBERSHIP GUARD: every pair whose value falls under a
///     fixed threshold — not a fixed COUNT of ranks — is checked by name
///     against that axis's `Blessed` list, with [`RATCHET_SLACK`] of
///     relative give on a blessed pair's own baseline for an exporter
///     re-render's antialiasing. This is the bar that actually notices
///     EROSION, at any position in the sorted order.
///
///     Item 99 shipped a ratchet watching ONLY `sorted[0]`, the single
///     global champion. Item 102 (filed from item 99's own verification)
///     found the gap: a pair can crowd substantially and pass in total
///     silence as long as it never unseats the incumbent champion. The
///     verifier's proof was `Ibis` — Galah's icon blended 30% toward
///     Bilby's — landing at differing 18.16% / mean 20.46% / ink 90.73%,
///     closer than all 17 of Galah's real comparisons on every axis, and
///     the OLD law passed silently because Currawong/Cassowary (differing,
///     mean) and Potoroo/Firetail (ink) never lost the title. See
///     `ibis_near_duplicate_is_caught_without_becoming_champion` below.
///
///     THE FIRST FIX (item 102 round 1) watched a fixed TOP-6 order
///     statistic instead of only `sorted[0]`. Item 102's OWN independent
///     verification then broke that fix two ways, both the same underlying
///     bug — a fixed RANK COUNT has a cliff, and anything past the cliff is
///     invisible no matter how close it sits:
///       * a pair can be constructed to land at rank 6, 7 or 8 — just past a
///         `K = 6` window — while still sitting inside the roster's own
///         measured low cluster (12.3%-28.3% on `differing`); five such
///         probes (ground-preserving clones of Potoroo, Gumtree, Mangrove,
///         Wagtail, Firetail with every non-ground pixel recoloured) landed
///         at ranks 5-8 and passed with zero failures.
///       * ORDINARY roster growth dilutes a real, unchanged erosion out of a
///         fixed window with no attacker at all: adding 5-6 new worlds ahead
///         of `Ibis` in sorted order (their pairs merely closer than Ibis's,
///         not adversarial) pushed `Ibis vs Galah` from rank 5 to rank 6 on
///         `ink`, past `K = 6`, and it vanished from the failure list even
///         though its own measured value never moved.
///         Both are the same defect from two directions — a fixed rank count is
///         a window with edges, and either a crafted pair or ordinary growth can
///         land or push something past the edge.
///
///     THE FIX: stop counting ranks. Every pair below a fixed VALUE
///     threshold (`danger` in `axes()`) is checked, however many pairs that
///     turns out to be today or after the roster grows. There is no window
///     to dodge by rank and no count of intervening pairs that can push a
///     known pair out of view, because membership depends only on the
///     pair's OWN value, never its position relative to others. Rerunning
///     both broken scenarios against this design: all five ground-clone
///     probes are caught (each lands under `differing`'s 35% threshold, at
///     23-27%), and `Ibis` stays caught on every axis regardless of how many
///     ordinary worlds are added around it, because nothing here is counted
///     by rank. See `ground_clone_crowding_is_caught_regardless_of_rank` and
///     `roster_growth_does_not_dilute_a_known_erosion`.
///
///     WHY NOT A PERCENTILE OF THE POPULATION EITHER: a percentile guard
///     (e.g. "the 5th percentile of all pair distances may not fall") has
///     the same rank-indexing problem one layer up — `differing` and `ink`
///     are both empirically bimodal (a tight low cluster, a cliff, then a
///     tight high cluster), so a percentile's rank INDEX crosses that cliff
///     as the roster grows and the threshold swings non-monotonically
///     (measured: growing the roster by 7 ordinary worlds swung
///     `differing`'s p05 between 27% and 88%). A fixed VALUE threshold has
///     no index to cross a cliff with.
///
///     THE DANGER THRESHOLDS ARE DATA, chosen with margin from the roster's
///     own measured shape, not tuned to look tight: `differing`'s 35% sits
///     between the low cluster's own max (28.32%) and the next real value
///     (80.47%, an 80-point cliff); `ink`'s 92% sits between the cluster's
///     max (88.39%) and the next real value (94.22%), positioned low enough
///     to still catch an `Ibis`-shaped 90.73% erosion. `mean` has no nearby
///     cliff — it is a smooth continuum from ~20 up to a distant break at
///     174 -> 474 spanning HALF the roster (77 of 153 pairs) — so its 70.0
///     threshold is set with margin above the tightest 10 pairs rather than
///     at that far cliff; watching the smooth half's full width would turn
///     `mean` into exactly the size of table item 99 rejected.
///
///     THE BLESSED LIST IS BOUNDED, NOT A PER-PAIR TABLE FOR THE ROSTER:
///     item 99 rejected a 153-row table because every world addition adds
///     rows with no authored baseline, forcing either a block or a rubber
///     stamp. `Blessed` never lists a pair outside the danger zone, so an
///     ordinary new world that crowds nobody (see
///     `ordinary_new_world_passes_the_danger_zone_guard`) never touches it —
///     the list is sized to the roster's measured crowding (12/10/5 entries
///     today), not to `THEMES.len()`. This is what item 102's own text
///     offered as the alternative to a per-pair table: "a per-pair baseline
///     with roster-growth tolerance."
///
///     THE HONEST COST, MEASURED, NOT ASSERTED: a maintainer picking a new
///     world's palette WITHOUT cross-checking it against the shipped roster
///     will sometimes land inside a danger zone by coincidence, because this
///     roster's near-black and near-cream grounds already crowd a small
///     corner of colour space. A deterministic sweep of 30 non-adversarial
///     synthetic worlds (random grounds and inks, no screening against
///     `THEMES`) against the real 18-world roster tripped the guard on 2/30
///     (~7%) — real, named, single-pair collisions a maintainer would need
///     to look at once and either fix the palette or bless.
///
///     THIS COST COMPOUNDS WITH BATCH SIZE, MEASURED SEPARATELY: worlds ship
///     in waves, not one at a time (item 92 landed all 18 in one commit), and
///     the ~7% figure above is a per-world rate, not a per-wave one. Sweeping
///     batches of 1/3/7 uncurated synthetic worlds together (same
///     construction, three independent seeds, 30 trials each) trips the
///     guard on roughly 3-10% of batch=1 waves (consistent with the ~7%
///     single-world figure), 10-13% of batch=3 waves, and 40-57% of batch=7
///     waves — because a batch adds pairs against the shipped roster AND
///     against every other new world in the same wave, so the trip
///     probability compounds with wave size roughly like any one of N
///     independent coin flips landing heads. A maintainer landing several
///     worlds in one PR should expect to see one or more "add this Blessed
///     entry" prompts close to half the time, not the rare event the
///     per-world figure alone suggests. That is still the bounded, honest
///     tradeoff of any real crowding guard on a roster whose ground palette
///     already clusters this tightly: the fix is not to loosen the guard
///     until it stops noticing (a rubber stamp is not a law), it is that a
///     maintainer shipping a wave of new worlds should cross-check the wave
///     against `THEMES` BEFORE committing, the same discipline
///     `ordinary_new_world_passes_the_danger_zone_guard` already demonstrates
///     for one world at a time — this guard cannot make that discipline
///     optional, only tell a maintainer who skipped it.
///
///     A DIFFERENT SHAPE OF SILENT FAILURE, ALSO CLOSED: a mean over the
///     entire blessed list diluted coordinated erosion of only a subset.
///     The pinned exporter is deterministic — two independent canonical
///     exports were byte-identical across all 540 tiles — so the ratchet is
///     now pair-owned with 0.05% relative headroom. Every non-empty identity
///     subset is swept by
///     `every_nonempty_blessed_subset_at_1_9_percent_is_caught_by_pair_ratchets`;
///     there is no aggregate denominator left to hide behind. A separate
///     check catches the list's other failure direction — a `Blessed` row
///     whose pair no longer exists (a world renamed) or has widened back out
///     of the zone, which would otherwise sit inert forever with nothing
///     prompting its removal; see `stale_blessed_entry_for_a_renamed_world_is_flagged`
///     and `stale_blessed_entry_for_a_widened_pair_is_flagged`.
///
///     A LIMIT LEFT DELIBERATELY OPEN: neither check verifies that a
///     `Blessed.baseline` was HONEST at the moment it was entered — a
///     baseline typed in lower than the pair's true measured value at
///     blessing time silently widens that one pair's erosion corridor
///     forever, and nothing here (or practically anywhere in a
///     source-committed threshold) can distinguish an honest baseline from a
///     generous one after the fact, because the ratchet's whole point is to
///     tolerate the measured value moving below the committed number by a
///     bounded amount. This is a review/trust surface, not a code defect:
///     item 99 already accepted that a re-blessing edit is read by a human
///     before merge, the same trust any threshold committed to source code
///     carries. Bounded, not fixed: a `git blame` on `axes()` names who
///     entered which baseline and when, which is the actual audit trail this
///     design relies on.
///
/// The stricter 20% tier the two hand-picked pairs used to carry is GONE, not
/// weakened: its premise (same face ⇒ at risk) is false by measurement above,
/// applying it to the computed same-face set would fail today, and the one real
/// thing it pinned — Potoroo/Firetail told apart, back when their silhouettes
/// (Block vs Pill) did work their near-identical palette did not — is now
/// pinned harder and roster-wide by the `ink` axis, on which that pair IS the
/// global minimum and sits first in `INK_BLESSED`. Item 121 moved Firetail
/// onto Potoroo's own Block preset, retiring the silhouette split on purpose
/// (`confusable_pairs_never_share_a_logo_cursor`'s doc); the pair crowded
/// further on this axis (54.31% -> 49.81%) and on `mean`, which this law
/// caught and named — the very crowding the item exists to fix. The user's
/// follow-up ground pick (`IconGround::Blend40`, item 121's C) then REPAIRS
/// it: ink reopens to 66.42% and mean to 44.89 (still each axis's SECOND
/// tightest pair after Currawong/Cassowary, and still the `ink` global
/// minimum, but no longer the crowding the item was filed over).
///
/// Non-vacuity: adding a world that clones an existing palette and preset
/// drives `differing` to ~1% and trips the floor and the danger zone, naming
/// the cloned pair. A world that crowds an EXISTING pair without becoming the
/// global champion — the `Ibis` scenario — trips the danger-zone guard on its
/// own merits instead, regardless of rank; see
/// `ibis_near_duplicate_is_caught_without_becoming_champion`,
/// `ground_clone_crowding_is_caught_regardless_of_rank` and
/// `roster_growth_does_not_dilute_a_known_erosion`. An ordinary new world that
/// crowds nobody trips nothing; see
/// `ordinary_new_world_passes_the_danger_zone_guard`.
#[test]
fn small_sizes_keep_every_pair_of_worlds_apart() {
    let _g = crate::testlock::serial();
    let pairs = all_real_pairs();
    assert_eq!(
        pairs.len(),
        THEMES.len() * (THEMES.len() - 1) / 2,
        "every combination of worlds is measured"
    );
    let failures = check_pair_axes(&pairs);
    assert!(failures.is_empty(), "\n\n{}", failures.join("\n\n"));
}

/// Runs the floor + danger-zone membership guard (see `axes()`'s doc and
/// `small_sizes_keep_every_pair_of_worlds_apart`'s) over an ARBITRARY pair
/// set, returning one message per violation (empty == every axis passes).
/// Factored out of the law itself so the non-vacuity probes below exercise
/// the exact check the live law runs against a pair set that includes a
/// synthetic world, rather than a hand-reimplemented approximation of it
/// that could silently drift from what ships.
fn check_pair_axes(pairs: &[Pair]) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, read, floor, danger, blessed, pct) in axes() {
        let show = |v: f64| {
            if pct {
                format!("{:.2}%", v * 100.0)
            } else {
                format!("{v:.2}")
            }
        };

        let closest = pairs
            .iter()
            .min_by(|x, y| read(x).total_cmp(&read(y)))
            .expect("all_real_pairs always returns at least one pair");
        let worst = read(closest);
        if worst < floor {
            failures.push(format!(
                "{} vs {} are the closest pair on {name} at 32px ({}) and fall under the \
                 absolute floor of {} — at app-switcher size they read as one app.",
                closest.a,
                closest.b,
                show(worst),
                show(floor),
            ));
        }

        // DANGER-ZONE MEMBERSHIP, not a fixed count of ranks: every pair
        // under `danger`, however many that turns out to be, is checked by
        // NAME against `blessed`. Nothing here is counted by rank, so no
        // pair can dodge review by landing one place past a watched window,
        // and no amount of unrelated roster growth can push an
        // already-known pair out of view — see item 102's doc paragraph on
        // this function's caller for the two ways a fixed-rank design broke.
        let mut in_zone: Vec<&Pair> = pairs.iter().filter(|p| read(p) < danger).collect();
        in_zone.sort_by(|x, y| read(x).total_cmp(&read(y)));
        for p in in_zone {
            let v = read(p);
            match blessed.iter().find(|b| b.matches(p)) {
                None => failures.push(format!(
                    "{} vs {} crowd on {name} at 32px ({}) — inside the danger zone \
                     (< {}) but not on the blessed list. Some change moved this pair into \
                     crowding territory that item 102's guard exists to catch. Either back \
                     it out, or — if the crowding is intended — add `Blessed {{ a: {:?}, \
                     b: {:?}, baseline: {v} }}` to {name}'s list in `axes()` and say why.",
                    p.a,
                    p.b,
                    show(v),
                    show(danger),
                    p.a,
                    p.b,
                )),
                Some(b) => {
                    let ratchet = b.baseline * (1.0 - RATCHET_SLACK);
                    if v < ratchet {
                        failures.push(format!(
                            "{} vs {} were blessed on {name} at {} but now measure {} — \
                             eroded past {:.2}% slack (ratchet {}). Either back it out, or \
                             re-bless with the new value in `axes()` and say why.",
                            p.a,
                            p.b,
                            show(b.baseline),
                            show(v),
                            RATCHET_SLACK * 100.0,
                            show(ratchet),
                        ));
                    }
                }
            }
        }

        // STALE ENTRIES (item 102 round 3): a `Blessed` row the per-pair loop
        // above never visits — because its pair no longer exists (a world
        // renamed) or has widened back out of the danger zone — sits inert
        // forever with nothing prompting its removal. Checked against the
        // FULL pair set, not `in_zone`, since a widened-out pair is by
        // definition no longer in `in_zone`.
        for b in blessed {
            match pairs.iter().find(|p| b.matches(p)) {
                None => failures.push(format!(
                    "{name}'s blessed list names {} vs {} but no such pair exists in the \
                     roster any more (a world renamed?) — remove the stale `Blessed` entry \
                     from {name}'s list in `axes()`.",
                    b.a, b.b
                )),
                Some(p) => {
                    let v = read(p);
                    if v >= danger {
                        failures.push(format!(
                            "{name}'s blessed list still carries {} vs {} (baseline {}) but \
                             it now measures {} — outside the danger zone (>= {}) — remove \
                             the stale `Blessed` entry from {name}'s list in `axes()`.",
                            b.a,
                            b.b,
                            show(b.baseline),
                            show(v),
                            show(danger),
                        ));
                    }
                }
            }
        }
    }
    failures
}

/// NON-VACUITY (item 102): a pair that crowds badly WITHOUT ever becoming the
/// single global champion on any axis must still fail. This reproduces
/// item 102's own verification scenario exactly — `Ibis`, Galah's 32px icon
/// blended 30% toward Bilby's, channel-and-alpha lerp, `(g, b) -> round(0.7g
/// + 0.3b)` — inserted as if it were a genuine 19th world: `Ibis` paired
///   against all 18 real worlds, merged into the full 153-pair set (171
///   pairs total).
///
/// Under item 99's OLD single-champion ratchet this passed in total
/// silence: `Ibis` never displaces Currawong/Cassowary (differing, mean) or
/// Potoroo/Firetail (ink) as the incumbent minimum. Under
/// `check_pair_axes`'s danger-zone guard it must fail on all three axes —
/// `Ibis`'s own values (currently 16.60%, 18.72, 85.43%) fall under
/// every axis's `danger` threshold without ever leading any of them, and the
/// guard checks it by value, not by whether it happens to be the champion.
#[test]
fn ibis_near_duplicate_is_caught_without_becoming_champion() {
    let _g = crate::testlock::serial();
    let galah = world("Galah");
    let bilby = world("Bilby");
    let galah_img = rep_rgba(&icon_bytes(galah.name), 32);
    let bilby_img = rep_rgba(&icon_bytes(bilby.name), 32);
    assert_eq!(
        galah_img.dimensions(),
        bilby_img.dimensions(),
        "every 32px rep shares one tile size"
    );
    let (w, h) = galah_img.dimensions();
    let mut ibis_img = image::RgbaImage::new(w, h);
    for (x, y, out) in ibis_img.enumerate_pixels_mut() {
        let g = galah_img.get_pixel(x, y).0;
        let b = bilby_img.get_pixel(x, y).0;
        let mut px = [0u8; 4];
        for k in 0..4 {
            px[k] = (g[k] as f64 * 0.7 + b[k] as f64 * 0.3).round() as u8;
        }
        *out = image::Rgba(px);
    }

    let mut pairs = all_real_pairs();
    // Ibis vs every real world, classifying Ibis's own ground against
    // Galah's `base_100` — the blend is only 30% toward Bilby, so Galah's
    // ground still dominates, and this mirrors exactly how item 102's
    // verifier measured it (Ibis was never exported, so it has no `Theme`
    // of its own to read a ground colour off).
    for t in THEMES.iter() {
        let img = rep_rgba(&icon_bytes(t.name), 32);
        pairs.push(measure_pair(
            "Ibis",
            &ibis_img,
            rgb(galah.icon_ground_color()),
            t.name,
            &img,
            rgb(t.icon_ground_color()),
        ));
    }

    // Sanity: replay the same Ibis-vs-Galah construction before asking
    // whether the law catches it. Item 110 moved Bilby's rendered seat to
    // 16.80% / 18.77 / 85.57% from item 102's historical 18.16% / 20.46 /
    // 90.73%; item 161's further restrained lift on Bilby moves it again, to
    // today's 16.60% / 18.72 / 85.43%, without weakening the probe.
    let ibis_vs_galah = pairs
        .iter()
        .find(|p| (p.a == "Ibis" && p.b == "Galah") || (p.a == "Galah" && p.b == "Ibis"))
        .expect("Ibis vs Galah is in the extended set");
    assert!(
        (ibis_vs_galah.differing - 0.166016).abs() < 1e-3,
        "differing = {} (item 161 geometry expects 16.60%)",
        ibis_vs_galah.differing
    );
    assert!(
        (ibis_vs_galah.mean - 18.7168).abs() < 0.5,
        "mean = {} (item 161 geometry expects 18.72)",
        ibis_vs_galah.mean
    );
    assert!(
        (ibis_vs_galah.ink - 0.854271).abs() < 1e-3,
        "ink = {} (item 161 geometry expects 85.43%)",
        ibis_vs_galah.ink
    );

    let failures = check_pair_axes(&pairs);
    assert!(
        !failures.is_empty(),
        "Ibis crowds badly on every axis (see the sanity numbers above) but \
         check_pair_axes reported no failures — the danger-zone guard is not catching the \
         case item 102 was filed for"
    );
    for axis in [
        "differing pixels",
        "mean channel distance",
        "differing INK pixels",
    ] {
        assert!(
            failures.iter().any(|f| f.contains(axis)),
            "expected a failure mentioning {axis:?} (Ibis should crowd every axis); got:\n\n{}",
            failures.join("\n\n")
        );
    }
}

/// NON-VACUITY (item 102): an ORDINARY new world that shares no palette with
/// anything already shipped must still pass the new danger-zone guard — not
/// only the floor and the old single-champion check — otherwise growing the
/// roster normally would go red on its own, which is precisely the failure
/// mode item 99 already rejected a per-pair baseline table for.
///
/// `Probeworld` reuses Galah's silhouette (identical shape at every pixel)
/// but recolours every ground pixel and every non-ground ("ink") pixel to a
/// palette that shares nothing with any of the 18 shipped worlds — by
/// construction it can crowd nobody on colour, and its shape is Galah's own,
/// so it cannot crowd on silhouette either.
#[test]
fn ordinary_new_world_passes_the_danger_zone_guard() {
    let _g = crate::testlock::serial();
    let galah = world("Galah");
    let galah_img = rep_rgba(&icon_bytes(galah.name), 32);
    let ground_from = rgb(galah.icon_ground_color());
    // A dark-teal ground (nowhere near any shipped world's `base_100`) and a
    // saturated magenta ink. Magenta, not the first-tried near-white: this
    // roster's `primary`/`primary_content`/`base_content` tokens are almost
    // all warm creams, dark neutrals, or one mint-green — checked by hand
    // against every shipped value, magenta's per-channel-sum distance from
    // all of them clears several hundred, ten times the >24 "differs"
    // threshold, where near-white collided (Wagtail's `#FFFFFF` cursor is
    // only 19 away from a first-tried `#FAFAF6`, UNDER the threshold, so it
    // silently registered as "not differing" and crowded the ink axis).
    let new_ground = [0x0A, 0x2E, 0x33];
    let new_ink = [0xFF, 0x00, 0xC8];

    let mut probe_img = image::RgbaImage::new(galah_img.width(), galah_img.height());
    for (x, y, out) in probe_img.enumerate_pixels_mut() {
        let p = galah_img.get_pixel(x, y).0;
        if !opaque(&p) {
            *out = image::Rgba(p);
            continue;
        }
        let new_rgb = if near(&p, ground_from, 10) {
            new_ground
        } else {
            new_ink
        };
        *out = image::Rgba([new_rgb[0], new_rgb[1], new_rgb[2], p[3]]);
    }

    let mut pairs = all_real_pairs();
    for t in THEMES.iter() {
        let img = rep_rgba(&icon_bytes(t.name), 32);
        pairs.push(measure_pair(
            "Probeworld",
            &probe_img,
            new_ground,
            t.name,
            &img,
            rgb(t.icon_ground_color()),
        ));
    }

    let failures = check_pair_axes(&pairs);
    assert!(
        failures.is_empty(),
        "an ordinary new world that crowds nobody should pass the danger-zone guard; \
         got:\n\n{}",
        failures.join("\n\n")
    );
}

/// NON-VACUITY (item 102, ROUND 2 — closing the top-K rank cliff): a pair
/// constructed to land just PAST a fixed rank window while still sitting
/// inside the roster's own measured low cluster must still fail. This
/// reproduces item 102's own independent verification exactly: a
/// GROUND-PRESERVING clone of a real world — every ground pixel copied
/// byte-for-byte, every non-ground ("ink") pixel recoloured to a colour that
/// shares nothing with the original (pure green) — lands in `differing`'s
/// low cluster (12.3%-28.3%) purely because the ground, which dominates a
/// 32px tile, matches exactly. Under the item-102-round-1 top-6 order
/// statistic, five of these landed at ranks 5-8 — past the watched window —
/// and passed with zero failures. Under the danger-zone guard, rank is not
/// consulted at all: every one of the five is caught.
#[test]
fn ground_clone_crowding_is_caught_regardless_of_rank() {
    let _g = crate::testlock::serial();
    // The exact five worlds the round-1 verification named as landing past
    // the old top-6 window (ranks 5-8 on `differing`).
    for name in ["Potoroo", "Gumtree", "Mangrove", "Wagtail", "Firetail"] {
        let base = world(name);
        let base_img = rep_rgba(&icon_bytes(base.name), 32);
        let ground = rgb(base.icon_ground_color());
        let mut clone_img = image::RgbaImage::new(base_img.width(), base_img.height());
        for (x, y, out) in clone_img.enumerate_pixels_mut() {
            let px = base_img.get_pixel(x, y).0;
            *out = image::Rgba(if !opaque(&px) || near(&px, ground, 10) {
                px
            } else {
                [0, 255, 0, px[3]]
            });
        }

        let mut pairs = all_real_pairs();
        for t in THEMES.iter() {
            let img = rep_rgba(&icon_bytes(t.name), 32);
            pairs.push(measure_pair(
                "GroundClone",
                &clone_img,
                ground,
                t.name,
                &img,
                rgb(t.icon_ground_color()),
            ));
        }

        let failures = check_pair_axes(&pairs);
        assert!(
            failures
                .iter()
                .any(|f| f.contains("differing pixels") && f.contains(name)),
            "a ground-preserving clone of {name} (identical ground, fully recoloured ink) \
             should crowd {name} on `differing pixels` regardless of what rank it lands at; \
             got:\n\n{}",
            failures.join("\n\n")
        );
    }
}

/// NON-VACUITY (item 102, ROUND 2 — closing the dilution defect): a KNOWN
/// erosion (`Ibis`, unchanged) must stay caught even after several ORDINARY
/// new worlds are added around it — reproducing item 102's own independent
/// verification, where adding 5-6 ordinary worlds pushed `Ibis vs Galah`
/// from rank 5 to rank 6 on `ink`, past the old top-6 window, even though
/// `Ibis`'s own measured value never moved. The danger-zone guard has no
/// window for growth to push anything past: membership depends only on a
/// pair's own value.
#[test]
fn roster_growth_does_not_dilute_a_known_erosion() {
    let _g = crate::testlock::serial();
    let galah = world("Galah");
    let bilby = world("Bilby");
    let galah_img = rep_rgba(&icon_bytes(galah.name), 32);
    let bilby_img = rep_rgba(&icon_bytes(bilby.name), 32);
    let (w, h) = galah_img.dimensions();
    let mut ibis_img = image::RgbaImage::new(w, h);
    for (x, y, out) in ibis_img.enumerate_pixels_mut() {
        let g = galah_img.get_pixel(x, y).0;
        let b = bilby_img.get_pixel(x, y).0;
        let mut px = [0u8; 4];
        for k in 0..4 {
            px[k] = (g[k] as f64 * 0.7 + b[k] as f64 * 0.3).round() as u8;
        }
        *out = image::Rgba(px);
    }

    let mut pairs = all_real_pairs();
    for t in THEMES.iter() {
        let img = rep_rgba(&icon_bytes(t.name), 32);
        pairs.push(measure_pair(
            "Ibis",
            &ibis_img,
            rgb(galah.icon_ground_color()),
            t.name,
            &img,
            rgb(t.icon_ground_color()),
        ));
    }

    // Six ORDINARY growth worlds, each Galah's silhouette recoloured to a
    // palette checked (by construction, large per-channel deltas) to crowd
    // nobody on its own — the point is to grow the pair population around
    // `Ibis`, not to also be a second attack.
    let ground_from = rgb(galah.icon_ground_color());
    let growth_palettes: [([u8; 3], [u8; 3]); 6] = [
        ([0x0A, 0x2E, 0x33], [0xFF, 0x00, 0xC8]),
        ([0x33, 0x0A, 0x2E], [0x00, 0xC8, 0xFF]),
        ([0x2E, 0x33, 0x0A], [0xC8, 0x00, 0xFF]),
        ([0x08, 0x1A, 0x08], [0xFF, 0xC8, 0x00]),
        ([0x1A, 0x08, 0x1A], [0x00, 0xFF, 0x88]),
        ([0x08, 0x08, 0x1A], [0xFF, 0x88, 0x00]),
    ];
    for (i, (new_ground, new_ink)) in growth_palettes.iter().enumerate() {
        let name: &'static str = Box::leak(format!("Growth{i}").into_boxed_str());
        let mut growth_img = image::RgbaImage::new(galah_img.width(), galah_img.height());
        for (x, y, out) in growth_img.enumerate_pixels_mut() {
            let p = galah_img.get_pixel(x, y).0;
            if !opaque(&p) {
                *out = image::Rgba(p);
                continue;
            }
            let new_rgb = if near(&p, ground_from, 10) {
                *new_ground
            } else {
                *new_ink
            };
            *out = image::Rgba([new_rgb[0], new_rgb[1], new_rgb[2], p[3]]);
        }
        for t in THEMES.iter() {
            let img = rep_rgba(&icon_bytes(t.name), 32);
            pairs.push(measure_pair(
                name,
                &growth_img,
                *new_ground,
                t.name,
                &img,
                rgb(t.icon_ground_color()),
            ));
        }
        // Growth worlds also pair with each other and with Ibis, exactly as
        // a real roster growing by six worlds would.
        for (j, (other_ground, other_ink)) in growth_palettes.iter().enumerate() {
            if j <= i {
                continue;
            }
            let other_name: &'static str = Box::leak(format!("Growth{j}").into_boxed_str());
            let mut other_img = image::RgbaImage::new(galah_img.width(), galah_img.height());
            for (x, y, out) in other_img.enumerate_pixels_mut() {
                let p = galah_img.get_pixel(x, y).0;
                if !opaque(&p) {
                    *out = image::Rgba(p);
                    continue;
                }
                let new_rgb = if near(&p, ground_from, 10) {
                    *other_ground
                } else {
                    *other_ink
                };
                *out = image::Rgba([new_rgb[0], new_rgb[1], new_rgb[2], p[3]]);
            }
            pairs.push(measure_pair(
                name,
                &growth_img,
                *new_ground,
                other_name,
                &other_img,
                *other_ground,
            ));
        }
        pairs.push(measure_pair(
            "Ibis",
            &ibis_img,
            rgb(galah.icon_ground_color()),
            name,
            &growth_img,
            *new_ground,
        ));
    }

    let failures = check_pair_axes(&pairs);
    for axis in [
        "differing pixels",
        "mean channel distance",
        "differing INK pixels",
    ] {
        assert!(
            failures
                .iter()
                .any(|f| f.contains(axis) && f.contains("Ibis") && f.contains("Galah")),
            "Ibis vs Galah's own crowding never changed, but it went uncaught on {axis:?} \
             after growing the roster around it — the guard must not dilute with rank; \
             got:\n\n{}",
            failures.join("\n\n")
        );
    }
}

type Write = fn(&mut Pair, f64);

fn erosion_axes() -> [(&'static str, &'static [Blessed], Write); 3] {
    [
        ("differing pixels", DIFFERING_BLESSED, |p, v| {
            p.differing = v
        }),
        ("mean channel distance", MEAN_BLESSED, |p, v| p.mean = v),
        ("differing INK pixels", INK_BLESSED, |p, v| p.ink = v),
    ]
}

fn erode(pairs: &mut [Pair], blessed: &Blessed, write: Write, fraction: f64) {
    let pair = pairs
        .iter_mut()
        .find(|p| blessed.matches(p))
        .unwrap_or_else(|| {
            panic!(
                "{} vs {} is blessed but missing from the roster",
                blessed.a, blessed.b
            )
        });
    write(pair, blessed.baseline * (1.0 - fraction));
}

fn assert_pair_failed(failures: &[String], axis: &str, blessed: &Blessed) {
    assert!(
        failures.iter().any(|failure| {
            failure.contains(axis)
                && failure.contains(blessed.a)
                && failure.contains(blessed.b)
                && failure.contains("eroded past")
        }),
        "expected a pair-owned erosion failure for {} vs {} on {axis}; got:\n\n{}",
        blessed.a,
        blessed.b,
        failures.join("\n\n")
    );
}

/// Every non-empty identity subset of every blessed axis is guarded directly
/// by the affected pairs. A mean over the whole list cannot prove this: the
/// omitted members dilute a coordinated subset even though every changed pair
/// moved by the same shared cause.
#[test]
fn every_nonempty_blessed_subset_at_1_9_percent_is_caught_by_pair_ratchets() {
    let _g = crate::testlock::serial();
    let base = all_real_pairs();
    for (axis, blessed, write) in erosion_axes() {
        for mask in 1usize..(1usize << blessed.len()) {
            let mut pairs = base.clone();
            for (index, row) in blessed.iter().enumerate() {
                if mask & (1 << index) != 0 {
                    erode(&mut pairs, row, write, 0.019);
                }
            }
            let failures = check_pair_axes(&pairs);
            for (index, row) in blessed.iter().enumerate() {
                if mask & (1 << index) != 0 {
                    assert_pair_failed(&failures, axis, row);
                }
            }
        }
    }
}

#[test]
fn first_half_of_each_blessed_list_at_1_9_percent_fails() {
    let _g = crate::testlock::serial();
    let mut pairs = all_real_pairs();
    for (_, blessed, write) in erosion_axes() {
        for row in blessed.iter().take(blessed.len() / 2) {
            erode(&mut pairs, row, write, 0.019);
        }
    }
    let failures = check_pair_axes(&pairs);
    for (axis, blessed, _) in erosion_axes() {
        for row in blessed.iter().take(blessed.len() / 2) {
            assert_pair_failed(&failures, axis, row);
        }
    }
}

#[test]
fn eleven_of_twelve_differing_pairs_at_1_99_percent_fails() {
    let _g = crate::testlock::serial();
    let mut pairs = all_real_pairs();
    for row in DIFFERING_BLESSED.iter().take(11) {
        erode(&mut pairs, row, |p, v| p.differing = v, 0.0199);
    }
    let failures = check_pair_axes(&pairs);
    for row in DIFFERING_BLESSED.iter().take(11) {
        assert_pair_failed(&failures, "differing pixels", row);
    }
}

/// NON-VACUITY (item 102 round 3 — closing the stale-entry gap): a
/// `Blessed` entry whose pair no longer exists in the roster (the shape a
/// world rename takes) must be flagged, not sit inert forever. Simulated by
/// dropping Currawong/Cassowary — blessed on all three axes — from the real
/// pair set, exactly as a rename would.
#[test]
fn stale_blessed_entry_for_a_renamed_world_is_flagged() {
    let _g = crate::testlock::serial();
    let probe = Blessed {
        a: "Currawong",
        b: "Cassowary",
        baseline: 0.0,
    };
    let pairs: Vec<Pair> = all_real_pairs()
        .into_iter()
        .filter(|p| !probe.matches(p))
        .collect();

    let failures = check_pair_axes(&pairs);
    for axis in [
        "differing pixels",
        "mean channel distance",
        "differing INK pixels",
    ] {
        assert!(
            failures.iter().any(|f| {
                f.contains(axis)
                    && f.contains("Currawong")
                    && f.contains("Cassowary")
                    && f.contains("no such pair exists")
            }),
            "removing Currawong vs Cassowary (blessed on every axis) should flag a stale \
             `Blessed` entry on {axis:?}; got:\n\n{}",
            failures.join("\n\n")
        );
    }
}

/// NON-VACUITY (item 102 round 3 — closing the stale-entry gap, the other
/// direction): a `Blessed` entry whose pair widened back OUT of the danger
/// zone — crowding resolved by unrelated palette work, not by review — must
/// also be flagged, not sit inert. Mutates Currawong/Cassowary's `differing`
/// value alone, well past the 35% danger threshold, leaving its `mean`/`ink`
/// entries (still genuinely in zone) undisturbed.
#[test]
fn stale_blessed_entry_for_a_widened_pair_is_flagged() {
    let _g = crate::testlock::serial();
    let mut pairs = all_real_pairs();
    let p = pairs
        .iter_mut()
        .find(|p| {
            (p.a == "Currawong" && p.b == "Cassowary") || (p.a == "Cassowary" && p.b == "Currawong")
        })
        .expect("Currawong vs Cassowary is a real pair");
    p.differing = 0.50; // well clear of `differing`'s 35% danger threshold

    let failures = check_pair_axes(&pairs);
    assert!(
        failures.iter().any(|f| {
            f.contains("differing pixels")
                && f.contains("Currawong")
                && f.contains("Cassowary")
                && f.contains("outside the danger zone")
        }),
        "Currawong vs Cassowary widening past `differing`'s danger threshold should flag its \
         now-stale `Blessed` entry; got:\n\n{}",
        failures.join("\n\n")
    );
    // Its `mean`/`ink` entries are untouched and still genuinely in zone —
    // this must not, as a side effect, also complain about them.
    assert!(
        !failures.iter().any(|f| {
            (f.contains("mean channel distance") || f.contains("differing INK pixels"))
                && f.contains("Currawong")
                && f.contains("Cassowary")
        }),
        "only the widened axis should flag; got:\n\n{}",
        failures.join("\n\n")
    );
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
    let err = icns::pack_world(Path::new("/nonexistent/tiles"), &THEMES[0]).expect_err("must fail");
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

/// ITEM 121's icon-ground capability: every shipped world defaults to the
/// INERT `Base100` state — EXCEPT Firetail, the one world the user actually
/// chose a blend for (`Blend40`, from the A/B/C comparison sheet), pinned
/// here by NAME so a change of which world opted in is a conscious edit to
/// this test, not a silent roster drift. Every OTHER world opting in — now
/// or later — is exactly the silent regression this law exists to catch: a
/// no-wildcard sweep over `THEMES` with one named exception, not a loop that
/// happens to pass because nothing else has changed yet.
#[test]
fn every_shipped_world_defaults_to_the_inert_base_100_ground_except_firetail() {
    let _g = crate::testlock::serial();
    for t in THEMES.iter() {
        let expect = if t.name == "Firetail" {
            crate::theme::IconGround::Blend40
        } else {
            crate::theme::IconGround::Base100
        };
        assert_eq!(
            t.icon_ground, expect,
            "{}: icon_ground doesn't match its expected state — a second world opted \
             into a non-default ground, or Firetail's own pick moved, without this law \
             being updated on purpose",
            t.name
        );
        if t.name == "Firetail" {
            assert_ne!(
                t.icon_ground_color(),
                t.base_100,
                "Firetail's ground should be a real blend toward base_300, not base_100 itself"
            );
        } else {
            assert_eq!(
                t.icon_ground_color(),
                t.base_100,
                "{}: the inert default must render the tile's actual base_100, byte for byte",
                t.name
            );
        }
    }
}

/// `Theme::icon_ground_color` is the ONE owner of the blend arithmetic — this
/// pins it against INDEPENDENTLY computed hexes for Firetail (the world item
/// 121 is actually about), so a broken `Srgb::lerp` call or a swapped
/// endpoint fails here rather than only showing up as an odd-looking tile.
/// `base_100 = #17090c`, `base_300 = #521629` — hand-computed: 25%/40% of the
/// way from `(0x17,0x09,0x0c)` to `(0x52,0x16,0x29)` is `(0x26,0x0c,0x13)` /
/// `(0x2f,0x0e,0x18)`.
#[test]
fn icon_ground_color_blends_toward_base_300_by_the_named_fraction() {
    let _g = crate::testlock::serial();
    let firetail = world("Firetail");
    assert_eq!(firetail.base_100.hex(), "#17090c");
    assert_eq!(firetail.base_300.hex(), "#521629");

    let mut a = *firetail;
    a.icon_ground = crate::theme::IconGround::Base100;
    assert_eq!(a.icon_ground_color().hex(), "#17090c", "A: base_100 itself");

    let mut b = *firetail;
    b.icon_ground = crate::theme::IconGround::Blend25;
    assert_eq!(
        b.icon_ground_color().hex(),
        "#260c13",
        "B: 25% toward base_300, hand-computed"
    );

    let mut c = *firetail;
    c.icon_ground = crate::theme::IconGround::Blend40;
    assert_eq!(
        c.icon_ground_color().hex(),
        "#2f0e18",
        "C: 40% toward base_300, hand-computed"
    );
}
