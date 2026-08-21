//! Precise, test-only filesystem failures for the durable-write matrix.

use super::*;

type RaceTarget = Arc<std::sync::Mutex<Option<(std::path::PathBuf, Vec<u8>)>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ScriptedOperation {
    CreateDirAll,
    Write,
    Rename,
    RenameNoReplace,
    RemoveFile,
}

impl ScriptedOperation {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            ScriptedOperation::CreateDirAll => "create-dir-all",
            ScriptedOperation::Write => "write",
            ScriptedOperation::Rename => "rename",
            ScriptedOperation::RenameNoReplace => "rename-no-replace",
            ScriptedOperation::RemoveFile => "remove-file",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ScriptedFailure {
    pub(crate) operation: ScriptedOperation,
    pub(crate) ordinal: usize,
    pub(crate) kind: io::ErrorKind,
    pub(crate) reason: &'static str,
}

/// Wraps `InMemoryFs`; only the selected mutating call differs.
#[derive(Debug, Clone)]
pub(crate) struct ScriptedFs {
    inner: InMemoryFs,
    failure: ScriptedFailure,
    counts: Arc<std::sync::Mutex<std::collections::BTreeMap<ScriptedOperation, usize>>>,
    trace: Arc<std::sync::Mutex<Vec<String>>>,
    race_target: RaceTarget,
}

impl ScriptedFs {
    pub(crate) fn new(inner: InMemoryFs, failure: ScriptedFailure) -> Self {
        assert!(
            failure.ordinal > 0,
            "scripted operation ordinals are one-based"
        );
        Self {
            inner,
            failure,
            counts: Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new())),
            trace: Arc::new(std::sync::Mutex::new(Vec::new())),
            race_target: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Arrange for a competing creator to publish `data` immediately before
    /// the no-replace publish operation. This makes the TOCTOU window a
    /// deterministic filesystem law rather than a timing test.
    pub(crate) fn race_create_before_no_replace(
        self,
        path: impl Into<std::path::PathBuf>,
        data: &[u8],
    ) -> Self {
        *self.race_target.lock().unwrap() = Some((path.into(), data.to_vec()));
        self
    }

    fn mutation(&self, operation: ScriptedOperation, detail: String) -> io::Result<()> {
        let ordinal = {
            let mut counts = self.counts.lock().unwrap();
            let count = counts.entry(operation).or_default();
            *count += 1;
            *count
        };
        self.trace
            .lock()
            .unwrap()
            .push(format!("{}#{ordinal} {detail}", operation.name()));
        if operation == self.failure.operation && ordinal == self.failure.ordinal {
            return Err(io::Error::new(
                self.failure.kind,
                format!(
                    "scripted {}#{}: {}",
                    operation.name(),
                    ordinal,
                    self.failure.reason
                ),
            ));
        }
        Ok(())
    }

    pub(crate) fn trace(&self) -> Vec<String> {
        self.trace.lock().unwrap().clone()
    }
}

impl FileSystem for ScriptedFs {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        self.inner.read_to_string(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner.read(path)
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.mutation(ScriptedOperation::Write, path.display().to_string())?;
        self.inner.write(path, data)
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        self.mutation(ScriptedOperation::CreateDirAll, path.display().to_string())?;
        self.inner.create_dir_all(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.mutation(
            ScriptedOperation::Rename,
            format!("{} -> {}", from.display(), to.display()),
        )?;
        self.inner.rename(from, to)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn rename_no_replace(&self, from: &Path, to: &Path) -> io::Result<()> {
        if let Some((path, data)) = self.race_target.lock().unwrap().take() {
            self.inner.write(&path, &data)?;
        }
        self.mutation(
            ScriptedOperation::RenameNoReplace,
            format!("{} -> {}", from.display(), to.display()),
        )?;
        self.inner.rename_no_replace(from, to)
    }

    fn exists(&self, path: &Path) -> bool {
        self.inner.exists(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.inner.is_dir(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirEntry>> {
        self.inner.read_dir(path)
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        self.inner.metadata(path)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.mutation(ScriptedOperation::RemoveFile, path.display().to_string())?;
        self.inner.remove_file(path)
    }
}
