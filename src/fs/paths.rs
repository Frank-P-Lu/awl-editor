use super::active;
use std::io;
use std::path::{Path, PathBuf};

// --- Shared write / path helpers (both backends) ---------------------------

/// ATOMIC WRITE through the active backend: write `data` to a hidden temp
/// sibling (`.<name>.awl-tmp`, same directory so the rename never crosses a
/// filesystem), then `rename` it over `path`. On the native backend a same-dir
/// rename is POSIX-atomic, so a crash mid-save leaves either the OLD file or the
/// NEW one — never a truncated half-write. Uses ONLY the trait's `write` +
/// `rename`, so `InMemoryFs` and `WebFs` model it too (wasm keeps compiling).
/// Used by every buffer save (manual and autosave), the scratch stash, and —
/// after this round's audit — every other durable app-owned store.
///
/// **`AWL_FAULT_DELAY_MS` (DEV-ONLY, native-only, no CLI flag — mirrors
/// `AWL_CJK_FORCE`'s "total no-op unless set" contract):** when set to a
/// valid integer, sleeps that many milliseconds AFTER the tmp write and
/// BEFORE the rename — artificially widening the pre-rename window so the
/// kill-9 fault harness (`tests/fault_kill9.rs`) can reliably land a SIGKILL
/// INSIDE it and assert the target file still holds its OLD content (the
/// rename never happened, so nothing was torn). Unset in every normal run —
/// including every other test in this suite — so this is a genuine zero-cost
/// no-op the rest of the time; reading the env var on every call is cheap
/// enough that a `#[cfg(test)]` gate isn't worth the code-path divergence
/// between test and release builds this primitive most needs to stay honest.
pub fn write_atomic(path: &Path, data: &[u8]) -> io::Result<()> {
    let fs = active();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unnamed".to_string());
    let tmp_name = format!(".{name}.awl-tmp");
    let tmp = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.join(tmp_name),
        _ => PathBuf::from(tmp_name),
    };
    fs.write(&tmp, data)?;
    super::fault::after_tmp_write(&tmp);
    fs.rename(&tmp, path)
}

/// The no-clobber sibling of [`write_atomic`]. It retains the temporary-file
/// durability shape, then atomically publishes only if `path` is still absent.
/// Each attempt owns a process-and-sequence-qualified temp sibling: two
/// simultaneous creators can race on the destination, never on the bytes they
/// are about to publish. A failed publication removes only its own sibling.
#[cfg(not(target_arch = "wasm32"))]
pub fn write_atomic_new(path: &Path, data: &[u8]) -> io::Result<()> {
    let fs = active();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unnamed".to_string());
    static NEXT_TEMP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT_TEMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp_name = format!(".{name}.awl-tmp-{}-{sequence}", std::process::id());
    let tmp = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.join(tmp_name),
        _ => PathBuf::from(tmp_name),
    };
    fs.write(&tmp, data)?;
    super::fault::after_tmp_write(&tmp);
    if let Err(e) = fs.rename_no_replace(&tmp, path) {
        let _ = fs.remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

/// THE ONE HOME-DIRECTORY LOOKUP — `$HOME`, read live rather than cached,
/// because it is not a constant of the process: the MAS sandbox rewrites it to
/// the container's own home at startup (`mas.rs`) and every path awl derives
/// must follow that redirect. `None` when unset or empty, and on wasm.
/// [`data_root`] and `args::resolve_default_folder` BUILD paths under it;
/// `capture::redact` STRIPS it back out of a capture artifact. One lookup, so
/// the stripper can never disagree with the builders about where home is.
pub(crate) fn home_dir() -> Option<PathBuf> {
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var_os("HOME")
            .filter(|h| !h.is_empty())
            .map(PathBuf::from)
    }
}

pub fn data_root() -> PathBuf {
    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from("/awl")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(x) = std::env::var_os("XDG_DATA_HOME") {
            return PathBuf::from(x).join("awl");
        }
        if let Some(home) = home_dir() {
            return home.join(".local").join("share").join("awl");
        }
        PathBuf::from("awl-data")
    }
}

/// Where the PERSISTENT SCRATCH BUFFER stashes across quits: the no-path launch
/// buffer is written here (atomic, on the same autosave triggers + quit) and
/// restored on the next no-argument launch. Web-safe via the trait (WebFs).
pub fn scratch_stash_path() -> PathBuf {
    data_root().join("scratch.md")
}

/// THE WEB CONFIG PATH — where `config.toml` lives inside the virtual `WebFs`
/// root (a `localStorage` key, `awlfs:F:/awl/config.toml`), closing WEB.md's
/// former "no config file on the web" gap. Beside the scratch stash under
/// [`data_root`] (the SAME `/awl` virtual-root convention `scratch.md` already
/// uses for machine-owned state), deliberately NOT under the seeded content
/// root `/` (which holds the user's own documents). `main::wasm_start` is the
/// ONE caller (native's `config::config_path` resolves an OS path instead and
/// is never reached on wasm); every `Config` write door (`write_pref`/
/// `write_binding`/`write_default`) already routes through
/// `crate::fs::active()` + [`write_atomic`], so a `Config` loaded from THIS
/// path just works over `WebFs` with zero further plumbing.
#[cfg(target_arch = "wasm32")]
pub fn web_config_path() -> PathBuf {
    data_root().join("config.toml")
}
