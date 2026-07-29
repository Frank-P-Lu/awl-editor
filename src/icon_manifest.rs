//! THE ICON EXPORT MANIFEST — the one machine-readable serialization of the
//! facts an app-icon needs: per world, the four palette tokens the lockup uses
//! and the display face it wears; per face, the bundled font FILES that face
//! actually resolves to.
//!
//! WHY IT IS GENERATED AND NOT WRITTEN: the icon exporter is a web compositor
//! (`scripts/icons/`) — a second runtime that has to know Tawny is
//! `#16181d`/`#ffc05e` in IBM Plex Mono. Hand-copying those into a JS/JSON
//! table would create a SECOND owner of the palette, free to drift the moment
//! a world is retuned. So nothing here is authored: every color comes from
//! [`crate::theme::THEMES`] through [`Srgb::hex`] (the same serializer the
//! capture sidecar uses), and every font FILE comes from parsing the bundled
//! `assets/fonts/*.ttf` name tables with fontdb — the same library the live
//! renderer registers those faces through. Retune a world and the next export
//! is simply different; there is no table to remember to update.
//!
//! THE WEIGHT TRIPWIRE, STRUCTURALLY: IBM Plex Mono ships as Weight 300, and a
//! CSS `@font-face` declared at the default 400 would silently drop it (the
//! same trap `mono_safe_weight()` exists for on the live side — see
//! `docs/fonts.md`). This manifest never assumes 400: it reports each face's
//! REAL weight as read from the file, so the exporter's `@font-face` block is
//! declared at the weight the file actually is.
//!
//! Entry point: `awl --icon-manifest [FONTS_DIR]` (default `assets/fonts`,
//! resolved against the current directory — run it from the repo root). Prints
//! JSON to stdout and exits; see `scripts/icons/README.md`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::capture::json_string;
use crate::theme::{IconCursor, IconGround, THEMES, Theme};

/// Bumped when the JSON SHAPE changes (fields added/renamed/removed), so a
/// stale exporter fails loudly instead of silently reading a missing key.
///
/// * 1 — worlds (palette + face + ambient flag) and faces (files + weights).
/// * 2 — each world also carries the `cursor` its SHIPPED icon wears
///   ([`crate::theme::IconCursor`]), so the exporter's "what actually ships"
///   sheet reads the assignment from `worlds.rs` instead of a second list.
/// * 3 — each world also carries `ground`, the tile's ACTUAL ground color
///   ([`crate::theme::Theme::icon_ground_color`], item 121): `base_100`
///   unless the world opts into a blend toward `base_300`. `base_100` still
///   ships too (the same token under both names for every world that has not
///   opted in), but every consumer painting the tile's ground reads `ground`.
pub const MANIFEST_SCHEMA: u32 = 3;

/// The default fonts directory, relative to the repo root.
pub const DEFAULT_FONTS_DIR: &str = "assets/fonts";

/// One bundled font FILE, as read from its own `name`/`OS/2` tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFile {
    /// File name only (never a path) — the exporter joins it with the fonts dir
    /// it was pointed at, so no machine-specific path is ever serialized.
    pub file: String,
    /// The weight the FILE declares (IBM Plex Mono's Regular slot is 300).
    pub weight: u16,
}

/// A display face and the bundled files it resolves to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Face {
    pub family: String,
    /// The face nearest Regular (400) among the family's upright files.
    pub regular: FontFile,
    /// The family's upright 700 companion, when one is bundled.
    pub bold: Option<FontFile>,
    /// Every world wearing this face, in cycle order. A face shared by two
    /// worlds is ONE entry with two names — which is what makes "shared faces
    /// share one optical tuning" structural in the exporter rather than a
    /// convention it has to remember.
    pub worlds: Vec<String>,
}

/// Every upright file bundled under `dir`, indexed by the family name its own
/// `name` table declares. Parsed one file at a time so each face keeps the
/// FILE it came from (fontdb's in-memory sources do not carry a path).
fn bundled_faces(dir: &Path) -> anyhow::Result<BTreeMap<String, Vec<FontFile>>> {
    let mut out: BTreeMap<String, Vec<FontFile>> = BTreeMap::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| anyhow::anyhow!("cannot read fonts dir {}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("ttf") || x.eq_ignore_ascii_case("otf"))
        })
        .collect();
    // Sorted so the manifest is byte-stable across filesystems.
    entries.sort();

    for path in entries {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let bytes = std::fs::read(&path)
            .map_err(|e| anyhow::anyhow!("cannot read font {}: {e}", path.display()))?;
        let mut db = glyphon::cosmic_text::fontdb::Database::new();
        db.load_font_data(bytes);
        for info in db.faces() {
            if info.style != glyphon::cosmic_text::fontdb::Style::Normal {
                continue;
            }
            let Some((family, _)) = info.families.first() else {
                continue;
            };
            out.entry(family.clone()).or_default().push(FontFile {
                file: name.to_string(),
                weight: info.weight.0,
            });
        }
    }
    Ok(out)
}

/// Resolve every world's display face against the bundled files.
///
/// Fails — naming the world and the family — if a shipped world's `Theme::font`
/// matches no bundled file. A missing face must stop the export, never quietly
/// fall through to a system font (which is exactly how a wrong-looking icon
/// would ship unnoticed).
pub fn faces_for(themes: &[Theme], fonts_dir: &Path) -> anyhow::Result<Vec<Face>> {
    let bundled = bundled_faces(fonts_dir)?;
    let mut faces: Vec<Face> = Vec::new();
    for theme in themes {
        if let Some(f) = faces.iter_mut().find(|f| f.family == theme.font) {
            f.worlds.push(theme.name.to_string());
            continue;
        }
        let files = bundled.get(theme.font).ok_or_else(|| {
            anyhow::anyhow!(
                "world {:?} wants display face {:?}, which no file in {} declares",
                theme.name,
                theme.font,
                fonts_dir.display()
            )
        })?;
        // The Regular SLOT is "nearest 400", not "== 400": IBM Plex Mono's is
        // the Weight-300 Light. Ties break toward the lighter file so the pick
        // is total and order-independent.
        let regular = files
            .iter()
            .min_by_key(|f| (f.weight.abs_diff(400), f.weight))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no upright file for face {:?}", theme.font))?;
        let bold = files
            .iter()
            .filter(|f| f.weight >= 600 && f.file != regular.file)
            .min_by_key(|f| (f.weight.abs_diff(700), f.weight))
            .cloned();
        faces.push(Face {
            family: theme.font.to_string(),
            regular,
            bold,
            worlds: vec![theme.name.to_string()],
        });
    }
    faces.sort_by(|a, b| a.family.cmp(&b.family));
    Ok(faces)
}

fn font_file_json(f: &FontFile) -> String {
    format!(
        "{{ \"file\": {}, \"weight\": {} }}",
        json_string(&f.file),
        f.weight
    )
}

fn world_json(t: &Theme) -> String {
    // The four tokens the lockup spends, named exactly as the theme names them
    // so a reader can trace each straight back to `worlds.rs`: base_100 is the
    // icon ground, base_content inks "aw", primary is the fake cursor,
    // primary_content inks the "l" sitting on it.
    format!(
        "    {{ \"name\": {}, \"dark\": {}, \"base_100\": {}, \"ground\": {}, \"base_content\": {}, \"primary\": {}, \"primary_content\": {}, \"font\": {}, \"cursor\": {}, \"ambient_motion\": {} }}",
        json_string(t.name),
        t.dark,
        json_string(&t.base_100.hex()),
        // The tile's ACTUAL ground (item 121: `Theme::icon_ground_color`) —
        // `base_100` for every world by default, or a bounded blend toward
        // `base_300` for a world that has opted in. Every consumer that
        // paints the tile's ground reads THIS field, never `base_100`.
        json_string(&t.icon_ground_color().hex()),
        json_string(&t.base_content.hex()),
        json_string(&t.primary.hex()),
        json_string(&t.primary_content.hex()),
        json_string(t.font),
        // The SHIPPED logo-cursor for this world, straight off the `Theme`
        // (`worlds.rs`) — so the exporter's shipped sheet and the packer that
        // cuts the .icns both read the assignment from the world literal that
        // a new world cannot compile without filling in.
        json_string(t.icon_cursor.slug()),
        // Mangrove's and Firetail's lava grounds are the only worlds whose real
        // canvas MOVES. An icon is one still frame, so the exporter flattens
        // them to base_100 (or this world's `icon_ground` blend) — recorded
        // here so that flattening is a declared fact rather than an
        // undocumented omission.
        t.has_ambient_motion(),
    )
}

fn face_json(f: &Face) -> String {
    let bold = match &f.bold {
        Some(b) => font_file_json(b),
        None => "null".to_string(),
    };
    let worlds = f
        .worlds
        .iter()
        .map(|w| json_string(w))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "    {{ \"family\": {}, \"regular\": {}, \"bold\": {}, \"worlds\": [{}] }}",
        json_string(&f.family),
        font_file_json(&f.regular),
        bold,
        worlds
    )
}

/// The whole manifest as JSON text (trailing newline included).
pub fn manifest_json(fonts_dir: &Path) -> anyhow::Result<String> {
    let faces = faces_for(&THEMES, fonts_dir)?;
    let worlds = THEMES
        .iter()
        .map(world_json)
        .collect::<Vec<_>>()
        .join(",\n");
    let faces_json = faces.iter().map(face_json).collect::<Vec<_>>().join(",\n");
    Ok(format!(
        "{{\n  \"schema\": {MANIFEST_SCHEMA},\n  \"generated_by\": \"awl --icon-manifest\",\n  \"source\": \"src/theme/worlds.rs (palette + face) and assets/fonts/*.ttf (files + weights) — GENERATED, do not hand-edit\",\n  \"worlds\": [\n{worlds}\n  ],\n  \"faces\": [\n{faces_json}\n  ]\n}}\n"
    ))
}

/// Item 121: a Block-cursor GROUND comparison for one named world, at the
/// exact A/B/C states the item names (`IconGround::ALL` — `base_100`, 25%
/// and 40% toward `base_300`). Every hex comes from
/// [`crate::theme::Theme::icon_ground_color`], the SAME blend a world's real
/// `icon_ground` field would compute if it opted in — never a hand-picked
/// value. A generic CLI capability (`--ground-audition WORLD`), not a
/// Firetail-name branch: any shipped world's name works, and this is a
/// one-off DECISION AID fed through the same `build.mjs`/`render.mjs` the
/// canonical export uses — it writes nothing under `assets/macos/`.
///
/// Forces the comparison onto the Block preset regardless of the named
/// world's own shipped `icon_cursor`: item 121's whole point is auditioning
/// candidate GROUNDS against the one silhouette already picked, not
/// re-litigating the cursor shape mid-comparison.
pub fn ground_audition_json(world_name: &str, fonts_dir: &Path) -> anyhow::Result<String> {
    let theme = THEMES
        .iter()
        .find(|t| t.name == world_name)
        .ok_or_else(|| {
            anyhow::anyhow!("no shipped world named {world_name:?} (see `awl --list-worlds`)")
        })?;
    let faces = faces_for(std::slice::from_ref(theme), fonts_dir)?;
    let worlds = IconGround::ALL
        .iter()
        .map(|ground| {
            let mut t = *theme;
            t.icon_ground = *ground;
            t.icon_cursor = IconCursor::Block;
            format!(
                "    {{ \"name\": {}, \"dark\": {}, \"base_100\": {}, \"ground\": {}, \"base_content\": {}, \"primary\": {}, \"primary_content\": {}, \"font\": {}, \"cursor\": {}, \"ambient_motion\": {} }}",
                json_string(&format!("{} \u{2014} {}", t.name, ground.slug())),
                t.dark,
                json_string(&t.base_100.hex()),
                json_string(&t.icon_ground_color().hex()),
                json_string(&t.base_content.hex()),
                json_string(&t.primary.hex()),
                json_string(&t.primary_content.hex()),
                json_string(t.font),
                json_string(t.icon_cursor.slug()),
                t.has_ambient_motion(),
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let faces_json = faces.iter().map(face_json).collect::<Vec<_>>().join(",\n");
    Ok(format!(
        "{{\n  \"schema\": {MANIFEST_SCHEMA},\n  \"generated_by\": \"awl --ground-audition {world_name}\",\n  \"source\": \"item 121 ground audition \u{2014} Theme::icon_ground_color() over {world_name}'s real base_100/base_300, never a hand-picked hex\",\n  \"worlds\": [\n{worlds}\n  ],\n  \"faces\": [\n{faces_json}\n  ]\n}}\n"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fonts_dir() -> std::path::PathBuf {
        // Tests run with CWD == the crate root.
        std::path::PathBuf::from(DEFAULT_FONTS_DIR)
    }

    /// THE LAW: every shipped world's display face resolves to a bundled file.
    /// A world added with a face nobody bundled fails HERE, not silently in a
    /// browser that quietly substituted Helvetica.
    #[test]
    fn every_shipped_world_resolves_to_a_bundled_file() {
        let _g = crate::testlock::serial();
        let faces = faces_for(&THEMES, &fonts_dir()).expect("all faces resolve");
        let named: Vec<&str> = faces
            .iter()
            .flat_map(|f| f.worlds.iter().map(String::as_str))
            .collect();
        for t in THEMES.iter() {
            assert!(
                named.contains(&t.name),
                "world {} missing from the face roster",
                t.name
            );
        }
        assert_eq!(named.len(), THEMES.len(), "each world listed exactly once");
        for f in &faces {
            assert!(
                f.regular.file.ends_with(".ttf"),
                "{} regular is a real file",
                f.family
            );
        }
    }

    /// Shared faces are ONE entry with several worlds — the structural reason a
    /// shared face cannot pick up two different optical tunings downstream.
    #[test]
    fn shared_faces_collapse_to_one_entry() {
        let _g = crate::testlock::serial();
        let faces = faces_for(&THEMES, &fonts_dir()).expect("all faces resolve");
        let mut families: Vec<&str> = faces.iter().map(|f| f.family.as_str()).collect();
        families.sort_unstable();
        let mut deduped = families.clone();
        deduped.dedup();
        assert_eq!(families, deduped, "no family appears twice");
        let bitter = faces
            .iter()
            .find(|f| f.family == "Bitter")
            .expect("Bitter is bundled and worn");
        assert!(
            bitter.worlds.len() >= 2,
            "Bitter is shared (Mopoke + Magpie), got {:?}",
            bitter.worlds
        );
    }

    /// The Weight-300 tripwire, made structural: the manifest reports IBM Plex
    /// Mono's REAL weight, so no consumer can declare it at 400 and lose it.
    #[test]
    fn plex_mono_regular_slot_reports_its_real_weight() {
        let _g = crate::testlock::serial();
        let faces = faces_for(&THEMES, &fonts_dir()).expect("all faces resolve");
        let plex = faces
            .iter()
            .find(|f| f.family == "IBM Plex Mono")
            .expect("Tawny's face");
        assert_eq!(
            plex.regular.weight, 300,
            "IBM Plex Mono ships Light; a 400 claim would be fabricated"
        );
        assert_eq!(plex.regular.file, "IBMPlexMono-Light.ttf");
    }

    /// The palette in the manifest IS the palette in `worlds.rs` — spot-checked
    /// through the same `Srgb::hex` the sidecar uses, so a drifted copy is
    /// impossible by construction and a broken serializer is caught here.
    #[test]
    fn palette_is_the_theme_palette() {
        let _g = crate::testlock::serial();
        let json = manifest_json(&fonts_dir()).expect("manifest builds");
        for t in THEMES.iter() {
            let expect = format!(
                "\"name\": \"{}\", \"dark\": {}, \"base_100\": \"{}\"",
                t.name,
                t.dark,
                t.base_100.hex()
            );
            assert!(json.contains(&expect), "manifest carries {expect}");
        }
        assert!(json.contains(&format!("\"schema\": {MANIFEST_SCHEMA}")));
    }

    /// Same inputs, same bytes — the export pipeline's determinism starts here.
    #[test]
    fn manifest_is_byte_stable() {
        let _g = crate::testlock::serial();
        let a = manifest_json(&fonts_dir()).expect("manifest builds");
        let b = manifest_json(&fonts_dir()).expect("manifest builds");
        assert_eq!(a, b);
    }

    /// A world whose face nobody bundles STOPS the export, naming the world.
    #[test]
    fn a_missing_face_is_an_error_not_a_fallback() {
        let _g = crate::testlock::serial();
        let mut fake = THEMES[0];
        fake.name = "Nonesuch";
        fake.font = "No Such Family";
        let err = faces_for(&[fake], &fonts_dir()).expect_err("must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("Nonesuch") && msg.contains("No Such Family"),
            "{msg}"
        );
    }

    /// Item 121's ground audition: names the world, forces every entry onto
    /// Block, and carries the three A/B/C ground hexes computed by the SAME
    /// `Theme::icon_ground_color` the manifest itself calls — never a second
    /// copy of the blend math.
    #[test]
    fn ground_audition_names_three_block_entries_at_the_real_blend() {
        let _g = crate::testlock::serial();
        let json = ground_audition_json("Firetail", &fonts_dir()).expect("audition builds");
        assert_eq!(json.matches("\"cursor\": \"block\"").count(), 3);
        assert_eq!(json.matches("\"font\": \"Monaspace Xenon\"").count(), 3);
        for hex in ["#17090c", "#260c13", "#2f0e18"] {
            assert!(
                json.contains(&format!("\"ground\": \"{hex}\"")),
                "audition is missing ground {hex}:\n{json}"
            );
        }
        // Firetail's actual palette tokens ride along unchanged — the
        // audition never touches primary/primary_content/base_content.
        let firetail = THEMES.iter().find(|t| t.name == "Firetail").unwrap();
        for hex in [
            firetail.base_content.hex(),
            firetail.primary.hex(),
            firetail.primary_content.hex(),
        ] {
            assert_eq!(
                json.matches(&format!("\"{hex}\"")).count(),
                3,
                "expected {hex} on all three audition rows"
            );
        }
    }

    /// A world nobody ships stops the audition, naming it — the same
    /// discipline `--icon-manifest` itself carries for a missing face.
    #[test]
    fn ground_audition_of_an_unshipped_world_is_an_error_not_a_fallback() {
        let _g = crate::testlock::serial();
        let err = ground_audition_json("Nonesuch", &fonts_dir()).expect_err("must fail");
        assert!(err.to_string().contains("Nonesuch"), "{err}");
    }
}
