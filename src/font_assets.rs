//! Font bytes shared by the live renderer and closed PDF font roster.
//!
//! Each face is included exactly once so both consumers point at one immutable
//! blob. This owns bytes only: renderer pitch/weight enrollment and PDF role,
//! family, embedding, and fallback policy stay with their existing rosters.

pub(crate) static BITTER_REGULAR: [u8; 131_696] =
    *include_bytes!("../assets/fonts/Bitter-Regular.ttf");
pub(crate) static BITTER_BOLD: [u8; 120_056] = *include_bytes!("../assets/fonts/Bitter-Bold.ttf");
pub(crate) static IBM_PLEX_MONO_LIGHT: [u8; 133_392] =
    *include_bytes!("../assets/fonts/IBMPlexMono-Light.ttf");
pub(crate) static IBM_PLEX_MONO_BOLD: [u8; 68_420] =
    *include_bytes!("../assets/fonts/IBMPlexMono-Bold.ttf");
pub(crate) static NOTO_SERIF_JP_REGULAR: [u8; 3_699_144] =
    *include_bytes!("../assets/fonts/NotoSerifJP-Regular.ttf");
pub(crate) static NOTO_SANS_JP_REGULAR: [u8; 2_617_488] =
    *include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf");

#[cfg(test)]
mod tests {
    #[test]
    fn production_consumers_do_not_reinclude_shared_faces() {
        let _serial = crate::testlock::serial();
        let render = include_str!("render.rs");
        let pdf = include_str!("export/pdf/fonts.rs");
        for file in [
            "Bitter-Regular.ttf",
            "Bitter-Bold.ttf",
            "IBMPlexMono-Light.ttf",
            "IBMPlexMono-Bold.ttf",
            "NotoSerifJP-Regular.ttf",
            "NotoSansJP-Regular.ttf",
        ] {
            assert!(
                !render.contains(&format!("include_bytes!(\"../assets/fonts/{file}\")")),
                "render.rs reintroduced its own {file} blob"
            );
            assert!(
                !pdf.contains(&format!("include_bytes!(\"../../../assets/fonts/{file}\")")),
                "export/pdf/fonts.rs reintroduced its own {file} blob"
            );
        }
    }
}
