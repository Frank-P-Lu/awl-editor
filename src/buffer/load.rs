//! File loading keeps read failures distinct from an intentionally new file.

use super::*;
use std::io;

impl Buffer {
    /// Read supported text and retain the observation of those exact bytes.
    /// Failure never creates an empty document bound to an existing file.
    pub(crate) fn load_file(path: &Path) -> io::Result<(Self, crate::external::Seen)> {
        let text = crate::openable::read_text(path)?;
        let seen = crate::external::Seen::after_write(path, text.as_bytes());
        let mut buffer = Self::from_rope(Rope::from_str(&normalize_eol(&text)), Some(path.into()));
        buffer.eol = Eol::detect(&text);
        Ok((buffer, seen))
    }

    /// Explicit opens may name a new file; reloads and session restore may not.
    pub(crate) fn open_file(path: &Path) -> io::Result<(Self, crate::external::Seen)> {
        match Self::load_file(path) {
            Err(error)
                if error.kind() == io::ErrorKind::NotFound && crate::openable::is_missing(path) =>
            {
                Ok((
                    Self::from_rope(Rope::new(), Some(path.into())),
                    crate::external::Seen::Absent,
                ))
            }
            result => result,
        }
    }

    /// Convenience for captures and fixtures. An unreadable existing file yields
    /// unbound scratch, so saving it cannot overwrite that file. Live transitions
    /// use the fallible owners and preserve their departing buffer on failure.
    pub fn from_file(path: &Path) -> Self {
        Self::open_file(path)
            .map(|(buffer, _)| buffer)
            .unwrap_or_else(|_| Self::scratch())
    }
}

#[cfg(test)]
mod tests;
