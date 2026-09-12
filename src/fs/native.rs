use super::{BoundedRead, DirEntry, FileSystem, Metadata};
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, Default, Clone, Copy)]
pub struct NativeFs;

pub(super) fn stable_complete(initial_len: u64, final_len: u64, bytes_read: usize) -> bool {
    initial_len == bytes_read as u64 && final_len == bytes_read as u64
}

impl FileSystem for NativeFs {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn read_bounded(&self, path: &Path, max_bytes: usize) -> BoundedRead {
        let mut file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) => {
                return BoundedRead::Failed {
                    bytes: Vec::new(),
                    _error: error,
                };
            }
        };
        let initial_len = match file.metadata() {
            Ok(metadata) => metadata.len(),
            Err(error) => {
                return BoundedRead::Failed {
                    bytes: Vec::new(),
                    _error: error,
                };
            }
        };
        let mut bytes = Vec::with_capacity(max_bytes.min(64 * 1024));
        while bytes.len() < max_bytes {
            let remaining = max_bytes - bytes.len();
            let mut chunk = [0u8; 16 * 1024];
            let chunk_len = remaining.min(chunk.len());
            match file.read(&mut chunk[..chunk_len]) {
                Ok(0) => {
                    return match file.metadata() {
                        Ok(metadata)
                            if stable_complete(initial_len, metadata.len(), bytes.len()) =>
                        {
                            BoundedRead::Complete(bytes)
                        }
                        Ok(_) => BoundedRead::LimitReached(bytes),
                        Err(error) => BoundedRead::Failed {
                            bytes,
                            _error: error,
                        },
                    };
                }
                Ok(n) => bytes.extend_from_slice(&chunk[..n]),
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    return BoundedRead::Failed {
                        bytes,
                        _error: error,
                    };
                }
            }
        }

        // At the exact cap, ask the already-open handle for its current size.
        // A one-byte EOF probe would itself exceed the read budget. Regular
        // files are the indexer's subject; if the size cannot prove completion,
        // conservatively reject the prefix as limited.
        match file.metadata() {
            Ok(metadata) if stable_complete(initial_len, metadata.len(), bytes.len()) => {
                BoundedRead::Complete(bytes)
            }
            Ok(_) => BoundedRead::LimitReached(bytes),
            Err(error) => BoundedRead::Failed {
                bytes,
                _error: error,
            },
        }
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

    #[cfg(not(target_arch = "wasm32"))]
    fn rename_no_replace(&self, from: &Path, to: &Path) -> io::Result<()> {
        // `hard_link` is the native macOS/Linux create-if-absent primitive:
        // unlike rename it fails when `to` already exists, without a TOCTOU
        // preflight. The temp and destination are same-directory siblings.
        std::fs::hard_link(from, to)?;
        std::fs::remove_file(from)
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
