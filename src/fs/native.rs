use super::{DirEntry, FileSystem, Metadata};
use std::io;
use std::path::Path;

#[derive(Debug, Default, Clone, Copy)]
pub struct NativeFs;

impl FileSystem for NativeFs {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        std::fs::write(path, data)
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    /// `DirEntry::file_type()` is free — the readdir already carried it — but
    /// it reports the LINK's own type, never the target's, so classifying by
    /// it alone makes a symlinked folder neither dir nor file and every level
    /// reader drops it silently. A symlink (and ONLY a symlink) therefore
    /// costs one following `metadata` on its own path:
    ///
    /// * target is a directory or file → the entry behaves as that, and the
    ///   picker shows it as what it points to;
    /// * the stat FAILS — a broken link, a link loop (`ELOOP`, which the
    ///   kernel reports rather than chasing), a permission wall, a dead
    ///   network mount — → neither dir nor file, so the entry is omitted.
    ///   There is nothing to open and nothing to descend, and a name that
    ///   errors on Enter is worse than an absent one.
    ///
    /// The extra stat is bounded to entries the user themself linked; an
    /// ordinary child costs nothing new, and a dead mount holding a plain
    /// directory already blocks in `read_dir` above this line.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let Ok(entry) = entry else { continue };
            let Ok(ft) = entry.file_type() else { continue };
            let path = entry.path();
            let is_symlink = ft.is_symlink();
            let (is_dir, is_file) = if is_symlink {
                match std::fs::metadata(&path) {
                    Ok(md) => (md.is_dir(), md.is_file()),
                    Err(_) => (false, false),
                }
            } else {
                (ft.is_dir(), ft.is_file())
            };
            out.push(DirEntry {
                path,
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir,
                is_file,
                is_symlink,
            });
        }
        Ok(out)
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let md = std::fs::metadata(path)?;
        Ok(Metadata {
            modified: md.modified().ok(),
            len: Some(md.len()),
        })
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }
}
