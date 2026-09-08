/// The line band changed by one text synchronization and whether its
/// document-wide styling context remained local to that band.
#[derive(Clone, Copy, Debug)]
pub(super) struct TextChange {
    pub(super) prefix: usize,
    pub(super) old_end: usize,
    pub(super) new_end: usize,
    pub(super) geometry_safe: bool,
}

impl TextChange {
    /// Whether changed lines can be compared with and substituted into the prior
    /// vertical row partition. Empty and line-structural bands, changed wrap
    /// widths, and document-scope invalidations all require a complete rebuild.
    pub(super) fn can_retain_geometry(self, width_stable: bool) -> bool {
        self.old_end - self.prefix == self.new_end - self.prefix
            && self.new_end > self.prefix
            && self.geometry_safe
            && width_stable
    }
}

pub(super) fn is_markdown_geometry_boundary(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```")
        || line.starts_with("~~~")
        || line == "---"
        || line.contains('|')
        || line.contains("![")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_geometry_gate_enrolls_only_nonempty_local_equal_line_bands() {
        let ordinary = TextChange {
            prefix: 50,
            old_end: 51,
            new_end: 51,
            geometry_safe: true,
        };
        assert!(ordinary.can_retain_geometry(true));

        let rejected = [
            TextChange {
                prefix: 50,
                old_end: 50,
                new_end: 50,
                geometry_safe: true,
            },
            TextChange {
                prefix: 50,
                old_end: 51,
                new_end: 52,
                geometry_safe: true,
            },
            TextChange {
                geometry_safe: false,
                ..ordinary
            },
        ];
        for change in rejected {
            assert!(!change.can_retain_geometry(true));
        }
        assert!(!ordinary.can_retain_geometry(false));
    }
}
