use std::borrow::Cow;

use super::normalize_eol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Eol {
    #[default]
    Lf,
    Crlf,
}

impl Eol {
    pub fn detect(s: &str) -> Eol {
        let bytes = s.as_bytes();
        let total_lf = memchr::memchr_iter(b'\n', bytes).count();
        let crlf = memchr::memmem::find_iter(bytes, b"\r\n").count();
        let lone_lf = total_lf - crlf;
        if crlf > lone_lf { Eol::Crlf } else { Eol::Lf }
    }

    /// Encode text for disk without doubling CRLF introduced by other input paths.
    pub fn encode<'a>(&self, lf_text: &'a str) -> Cow<'a, str> {
        match self {
            Eol::Lf => Cow::Borrowed(lf_text),
            Eol::Crlf if lf_text.contains('\n') => {
                Cow::Owned(normalize_eol(lf_text).replace('\n', "\r\n"))
            }
            Eol::Crlf => Cow::Borrowed(lf_text),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Eol::Lf => "LF",
            Eol::Crlf => "CRLF",
        }
    }

    pub fn toggled(&self) -> Eol {
        match self {
            Eol::Lf => Eol::Crlf,
            Eol::Crlf => Eol::Lf,
        }
    }
}
