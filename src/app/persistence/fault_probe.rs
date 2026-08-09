//! Killable, GPU-less `App` journeys for the persistence integration laws.

use super::super::*;
use anyhow::{Context, bail};
use std::io::Write;
use std::path::{Path, PathBuf};

fn probe_config(history: bool) -> Config {
    Config {
        history: Some(history),
        session_restore: Some(false),
        reduce_motion: Some(false),
        ..Config::empty()
    }
}

fn app_on(path: &Path, history: bool) -> App {
    let root = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    App::new(
        Some(path.to_path_buf()),
        root,
        None,
        None,
        probe_config(history),
    )
}

fn payload(path: &Path) -> anyhow::Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("failed to read probe payload {}", path.display()))
}

/// One hidden diagnostic with four bounded operations.  Tests kill the process
/// only after observing the atomic writer's temporary sibling on disk.
pub(super) fn run(operation: &str, args: &[PathBuf]) -> anyhow::Result<()> {
    match (operation, args) {
        ("autosave", [target, payload_path]) => {
            let mut app = app_on(target, false);
            app.document.set_text(&payload(payload_path)?);
            app.autosave_flush();
            println!("persistence-probe autosave complete");
        }
        ("relaunch", [target]) => {
            let app = app_on(target, false);
            println!(
                "persistence-probe relaunch bytes={}",
                app.document.buffer().text().len()
            );
            print!("{}", app.document.buffer().text());
        }
        ("export", [source, payload_path]) => {
            let mut app = app_on(source, false);
            app.document.set_text(&payload(payload_path)?);
            app.export_document(crate::export::Format::Html, None);
            println!("persistence-probe export complete");
        }
        ("export-bytes", [source, payload_path]) => {
            let mut app = app_on(source, false);
            app.document.set_text(&payload(payload_path)?);
            let bytes = app.export_bytes(crate::export::Format::Html);
            std::io::stdout().write_all(&bytes)?;
        }
        ("large-save", [target, payload_path]) => {
            let body = payload(payload_path)?;
            let bytes = body.len();
            let mut app = app_on(target, true);
            app.document.set_text(&body);
            let started = std::time::Instant::now();
            app.manual_save();
            let elapsed_ms = started.elapsed().as_millis();
            let rss_bytes = peak_rss_bytes().unwrap_or(0);
            println!(
                "persistence-probe large bytes={bytes} elapsed_ms={elapsed_ms} \
                 rss_bytes={rss_bytes}"
            );
        }
        _ => bail!(
            "--persistence-fault-probe expects autosave TARGET PAYLOAD | relaunch TARGET | \
             export SOURCE PAYLOAD | export-bytes SOURCE PAYLOAD | \
             large-save TARGET PAYLOAD"
        ),
    }
    Ok(())
}

#[cfg(unix)]
fn peak_rss_bytes() -> Option<u64> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    // SAFETY: `getrusage` initializes the supplied `rusage` on a zero return;
    // the pointer is valid for exactly one value.
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
        return None;
    }
    // SAFETY: the successful call above initialized the value.
    let usage = unsafe { usage.assume_init() };
    let raw = u64::try_from(usage.ru_maxrss).ok()?;
    #[cfg(target_os = "macos")]
    return Some(raw);
    #[cfg(not(target_os = "macos"))]
    return raw.checked_mul(1024);
}

#[cfg(not(unix))]
fn peak_rss_bytes() -> Option<u64> {
    None
}
