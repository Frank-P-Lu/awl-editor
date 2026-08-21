use super::{DirEntry, FileSystem, Metadata};
use crate::clock::SystemTime;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

// --- In-memory backend (tests + the hermetic scenario sandbox) -------------
//
// Two consumers: fs-touching unit tests (no real disk, no temp-dir litter) and
// — since the hermetic-scenario round — the PRODUCTION strict-replay sandbox
// (`crate::scenario`), which seeds one of these from the CLI-named storyboard
// inputs and installs it so a scenario run never touches the user's real
// files. Gated `any(test, not(wasm32))`: native builds carry it for the
// scenario door; a wasm release build (whose sandbox is `WebFs`) doesn't.

#[cfg(any(test, not(target_arch = "wasm32")))]
#[derive(Debug, Clone, Default)]
pub struct InMemoryFs {
    inner: Arc<RwLock<MemState>>,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
#[derive(Debug, Default)]
struct MemState {
    files: std::collections::BTreeMap<PathBuf, MemFile>,
    dirs: std::collections::BTreeSet<PathBuf>,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
struct MemFile {
    bytes: Vec<u8>,
    modified: SystemTime,
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl InMemoryFs {
    pub fn new() -> Self {
        let fs = InMemoryFs::default();
        fs.inner.write().unwrap().dirs.insert(PathBuf::from("/"));
        fs
    }

    #[cfg(test)]
    pub fn with_file(self, path: impl AsRef<Path>, contents: &str) -> Self {
        self.write(path.as_ref(), contents.as_bytes()).unwrap();
        self
    }

    #[cfg(test)]
    pub fn with_dir(self, path: impl AsRef<Path>) -> Self {
        self.create_dir_all(path.as_ref()).unwrap();
        self
    }

    fn insert_dirs(state: &mut MemState, path: &Path) {
        let mut cur = Some(path);
        while let Some(p) = cur {
            state.dirs.insert(p.to_path_buf());
            cur = p.parent();
        }
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
impl FileSystem for InMemoryFs {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner
            .read()
            .unwrap()
            .files
            .get(path)
            .map(|f| f.bytes.clone())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        let now = crate::clock::system_now();
        let mut state = self.inner.write().unwrap();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            InMemoryFs::insert_dirs(&mut state, parent);
        }
        state.files.insert(
            path.to_path_buf(),
            MemFile {
                bytes: data.to_vec(),
                modified: now,
            },
        );
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        let mut state = self.inner.write().unwrap();
        InMemoryFs::insert_dirs(&mut state, path);
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut state = self.inner.write().unwrap();
        let file = state
            .files
            .remove(from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))?;
        if let Some(parent) = to.parent()
            && !parent.as_os_str().is_empty()
        {
            InMemoryFs::insert_dirs(&mut state, parent);
        }
        state.files.insert(to.to_path_buf(), file);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn rename_no_replace(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut state = self.inner.write().unwrap();
        if state.files.contains_key(to) || state.dirs.contains(to) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "destination exists",
            ));
        }
        let file = state
            .files
            .remove(from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))?;
        if let Some(parent) = to.parent()
            && !parent.as_os_str().is_empty()
        {
            InMemoryFs::insert_dirs(&mut state, parent);
        }
        state.files.insert(to.to_path_buf(), file);
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        let state = self.inner.read().unwrap();
        state.files.contains_key(path) || state.dirs.contains(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.inner.read().unwrap().dirs.contains(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
        let state = self.inner.read().unwrap();
        if !state.dirs.contains(path) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "no such directory"));
        }
        let mut out = Vec::new();
        let is_child = |p: &Path| p.parent() == Some(path);
        for f in state.files.keys().filter(|p| is_child(p)) {
            out.push(DirEntry {
                path: f.clone(),
                name: leaf_name(f),
                is_dir: false,
                is_file: true,
                // This backend has no symlink concept — a fixture that needs
                // one uses a real scratch directory instead.
                is_symlink: false,
            });
        }
        for d in state
            .dirs
            .iter()
            .filter(|p| p.as_path() != path && is_child(p))
        {
            out.push(DirEntry {
                path: d.clone(),
                name: leaf_name(d),
                is_dir: true,
                is_file: false,
                is_symlink: false,
            });
        }
        Ok(out)
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        let state = self.inner.read().unwrap();
        if let Some(f) = state.files.get(path) {
            Ok(Metadata {
                modified: Some(f.modified),
                len: Some(f.bytes.len() as u64),
            })
        } else if state.dirs.contains(path) {
            Ok(Metadata {
                modified: None,
                len: None,
            })
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "no such file"))
        }
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        let mut state = self.inner.write().unwrap();
        state
            .files
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such file"))
    }
}

#[cfg(any(test, not(target_arch = "wasm32")))]
fn leaf_name(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}
