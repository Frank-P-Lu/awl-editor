//! Byte-only image dimension and encoding detection shared by export callers.
use super::ImageMime;

/// Sniff the intrinsic dimensions + encoding of an image from its leading bytes,
/// WITHOUT decoding pixels. Handles PNG (the IHDR width/height at a fixed offset)
/// and baseline/progressive JPEG (scan for an `SOFn` marker). Returns `None` for
/// anything else — the caller then drops the image to alt text. Pure + total, so
/// the App's resolver and the tests share one honest dimension source.
pub fn sniff_image(bytes: &[u8]) -> Option<(u32, u32, ImageMime)> {
    // PNG: \x89PNG\r\n\x1a\n then an IHDR chunk whose data starts at byte 16
    // (width u32 BE at 16, height u32 BE at 20).
    const PNG_SIG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() >= 24 && bytes.starts_with(PNG_SIG) && &bytes[12..16] == b"IHDR" {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        if w > 0 && h > 0 {
            return Some((w, h, ImageMime::Png));
        }
    }
    // JPEG: starts with SOI (FFD8); walk segment markers to the first SOFn
    // (0xC0..=0xCF except C4/C8/CC), whose payload holds height then width (BE).
    if bytes.len() >= 4 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        let mut i = 2usize;
        while i + 9 < bytes.len() {
            if bytes[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = bytes[i + 1];
            // Standalone markers (no length): padding 0xFF, RSTn, SOI, EOI.
            if marker == 0xFF || (0xD0..=0xD9).contains(&marker) {
                i += 2;
                continue;
            }
            let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
            let is_sof = (0xC0..=0xCF).contains(&marker)
                && marker != 0xC4
                && marker != 0xC8
                && marker != 0xCC;
            if is_sof && i + 9 < bytes.len() {
                let h = u16::from_be_bytes([bytes[i + 5], bytes[i + 6]]) as u32;
                let w = u16::from_be_bytes([bytes[i + 7], bytes[i + 8]]) as u32;
                if w > 0 && h > 0 {
                    return Some((w, h, ImageMime::Jpeg));
                }
            }
            if len < 2 {
                break;
            }
            i += 2 + len;
        }
    }
    None
}
