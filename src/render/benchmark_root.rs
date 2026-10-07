//! Locate optional native benchmark fixtures at runtime without embedding a checkout path.
use std::path::{Path, PathBuf};

pub(super) fn resolve() -> PathBuf {
    let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    select(std::env::current_exe().ok().as_deref(), &current)
}

fn select(executable: Option<&Path>, current: &Path) -> PathBuf {
    executable
        .and_then(|path| {
            path.ancestors().find(|root| {
                root.join("Cargo.toml").is_file()
                    && root.join("src/main.rs").is_file()
                    && root.join("samples/tour.md").is_file()
            })
        })
        .map(Path::to_path_buf)
        .unwrap_or_else(|| current.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> crate::testscratch::ScratchDir {
        let scratch = crate::testscratch::ScratchDir::new(
            std::env::temp_dir().join(format!("awl-benchmark-root-{}", std::process::id())),
        );
        for path in ["Cargo.toml", "src/main.rs", "samples/tour.md"] {
            let path = scratch.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "synthetic fixture").unwrap();
        }
        scratch
    }

    #[test]
    fn native_benchmarks_find_the_checkout_from_every_build_and_bundle_layout() {
        let _guard = crate::testlock::serial();
        let root = fixture();
        for binary in [
            "target/debug/awl",
            "target/release/awl",
            "target/aarch64-apple-darwin/release/awl",
            "target/x86_64-apple-darwin/release/awl",
            "target/review/Awl.app/Contents/MacOS/awl",
        ] {
            for current in [Path::new("."), Path::new("/unrelated/runtime/location")] {
                assert_eq!(select(Some(&root.join(binary)), current), *root);
            }
        }
    }

    #[test]
    fn installed_benchmarks_and_incomplete_ancestors_use_the_runtime_directory() {
        let _guard = crate::testlock::serial();
        let current = Path::new("/synthetic/runtime/checkout");
        assert_eq!(select(None, current), current);
        assert_eq!(
            select(
                Some(Path::new(
                    "/synthetic/Applications/Awl.app/Contents/MacOS/awl"
                )),
                current
            ),
            current
        );
        for missing in ["Cargo.toml", "src/main.rs", "samples/tour.md"] {
            let root = fixture();
            std::fs::remove_file(root.join(missing)).unwrap();
            assert_eq!(
                select(Some(&root.join("target/release/awl")), current),
                current
            );
        }
    }
}
