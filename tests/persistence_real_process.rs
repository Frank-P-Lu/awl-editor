//! POSIX-only persistence arms: process death, atomic export replacement, and
//! a measured large-manuscript save. Scripted filesystems cannot witness these.

#![cfg(unix)]

mod common;

use common::ScratchDir;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

fn sandbox(tag: &str) -> ScratchDir {
    ScratchDir::claim(
        &std::env::temp_dir(),
        &format!("awl-persistence-real-{tag}-{}", std::process::id()),
    )
}

fn isolated_awl(root: &Path) -> Command {
    let mut command = common::awl(root);
    command
        .env("HOME", root)
        .env("XDG_CONFIG_HOME", root.join("xdg-config"))
        .env("XDG_DATA_HOME", root.join("xdg-data"))
        .env_remove("AWL_FAULT_DELAY_MS")
        .env_remove("AWL_FAULT_OBSERVED_WRITE");
    command
}

fn probe(root: &Path, operation: &str, paths: &[&Path]) -> Command {
    let mut command = isolated_awl(root);
    command
        .arg("--persistence-fault-probe")
        .arg(operation)
        .args(paths);
    command
}

fn spawn_delayed(mut command: Command) -> Child {
    command
        .env("AWL_FAULT_OBSERVED_WRITE", "1")
        .env("AWL_FAULT_DELAY_MS", "30000")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("persistence probe child starts")
}

fn kill_after_observed_tmp_write(child: &mut Child) -> String {
    let stdout = child.stdout.take().expect("probe stdout is piped");
    for line in BufReader::new(stdout).lines() {
        let line = line.expect("probe observation line is readable");
        if line.starts_with("fault-observed tmp-write ") {
            child.kill().expect("SIGKILL the observed in-flight writer");
            let _ = child.wait();
            return line;
        }
    }
    panic!("probe exited before reporting its completed temporary write");
}

fn assert_old_or_new(actual: &[u8], old: &[u8], new: &[u8], subject: &str) {
    assert!(
        actual == old || actual == new,
        "{subject} is torn: {} bytes match neither the {}-byte old nor {}-byte new file",
        actual.len(),
        old.len(),
        new.len()
    );
}

#[test]
fn observed_write_synchronized_kill_during_autosave_relaunches_to_complete_bytes() {
    let root = sandbox("autosave");
    let target = root.join("manuscript.md");
    let payload_path = root.join("new.md");
    let old = b"old complete manuscript\n".repeat(1024);
    let new = b"new complete manuscript\n".repeat(2048);
    std::fs::write(&target, &old).unwrap();
    std::fs::write(&payload_path, &new).unwrap();

    let mut child = spawn_delayed(probe(&root, "autosave", &[&target, &payload_path]));
    let observation = kill_after_observed_tmp_write(&mut child);
    assert!(observation.contains(".manuscript.md.awl-tmp"));

    let disk = std::fs::read(&target).unwrap();
    assert_old_or_new(&disk, &old, &new, "autosaved manuscript");

    let output = probe(&root, "relaunch", &[&target])
        .output()
        .expect("real App relaunch probe runs");
    assert!(output.status.success(), "relaunch failed: {output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let (_, relaunched) = stdout
        .split_once('\n')
        .expect("relaunch prints a header then the App buffer");
    assert_eq!(
        relaunched.as_bytes(),
        disk,
        "relaunch loaded the complete disk state"
    );
}

#[test]
fn interrupted_export_replacement_is_old_complete_or_new_complete_never_torn() {
    let root = sandbox("export");
    let expected_root = root.join("expected");
    let trial_root = root.join("trial");
    std::fs::create_dir_all(&expected_root).unwrap();
    std::fs::create_dir_all(&trial_root).unwrap();
    let payload_path = root.join("payload.md");
    let payload = "# Replacement export\n\n".to_string() + &"measured prose\n".repeat(4096);
    std::fs::write(&payload_path, payload).unwrap();

    let expected_source = expected_root.join("document.md");
    std::fs::write(&expected_source, "source remains untouched\n").unwrap();
    let expected = probe(&root, "export", &[&expected_source, &payload_path])
        .output()
        .expect("complete export probe runs");
    assert!(
        expected.status.success(),
        "expected export failed: {expected:?}"
    );
    let new = std::fs::read(expected_source.with_extension("html")).unwrap();

    let source = trial_root.join("document.md");
    let target = source.with_extension("html");
    let source_before = b"source remains untouched\n";
    let old = b"old complete export\n".repeat(1024);
    std::fs::write(&source, source_before).unwrap();
    std::fs::write(&target, &old).unwrap();
    let mut child = spawn_delayed(probe(&root, "export", &[&source, &payload_path]));
    let observation = kill_after_observed_tmp_write(&mut child);
    assert!(observation.contains(".document.html.awl-tmp"));

    let disk = std::fs::read(&target).unwrap();
    assert_old_or_new(&disk, &old, &new, "replacement export");
    assert_eq!(std::fs::read(&source).unwrap(), source_before);
}

fn field(report: &str, key: &str) -> u64 {
    report
        .split_whitespace()
        .find_map(|part| part.strip_prefix(&format!("{key}=")))
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("large-manuscript report lacks {key}: {report}"))
}

#[test]
fn large_manuscript_save_reports_bounded_size_time_and_memory() {
    const MANUSCRIPT_BYTES: usize = 12 * 1024 * 1024;
    const MAX_ELAPSED_MS: u64 = 15_000;
    const MAX_RSS_BYTES: u64 = 512 * 1024 * 1024;

    let root = sandbox("large");
    let target = root.join("large.md");
    let payload_path = root.join("large-new.md");
    let line = "A measured manuscript line with enough prose to be realistic.\n";
    let payload = line.repeat(MANUSCRIPT_BYTES.div_ceil(line.len()));
    std::fs::write(&target, "old\n").unwrap();
    std::fs::write(&payload_path, &payload).unwrap();

    let output = probe(&root, "large-save", &[&target, &payload_path])
        .output()
        .expect("large-manuscript probe runs");
    assert!(output.status.success(), "large save failed: {output:?}");
    let report = String::from_utf8(output.stdout).unwrap();
    let bytes = field(&report, "bytes");
    let elapsed_ms = field(&report, "elapsed_ms");
    let rss_bytes = field(&report, "rss_bytes");
    assert_eq!(bytes as usize, payload.len());
    assert_eq!(std::fs::read(&target).unwrap(), payload.as_bytes());
    assert!(elapsed_ms <= MAX_ELAPSED_MS, "slow large save: {report}");
    assert!(
        rss_bytes >= bytes && rss_bytes <= MAX_RSS_BYTES,
        "large-save RSS is missing or out of bound: {report}"
    );
    let history_dir = root.join("xdg-data/awl/history");
    let logs: Vec<PathBuf> = std::fs::read_dir(&history_dir)
        .expect("default-on history wrote its large snapshot")
        .flatten()
        .map(|entry| entry.path())
        .collect();
    assert_eq!(logs.len(), 1, "one large source has one history log");
    assert!(std::fs::metadata(&logs[0]).unwrap().len() >= bytes);
    eprintln!("{report}");
}
