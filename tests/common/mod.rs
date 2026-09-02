use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// Sequence counter used to make each temp directory unique within a process.
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// The absolute path to the `git-overlay` binary under test, as compiled for
/// this test target.
pub fn binary() -> &'static str {
    // Cargo sets `CARGO_BIN_EXE_<name>` (with the name verbatim, hyphens
    // included) to the built binary's path when compiling integration tests.
    env!("CARGO_BIN_EXE_git-overlay")
}

/// A self-cleaning temporary directory for one end-to-end test.
///
/// A unique directory is created under the system temp directory on
/// construction and removed recursively when dropped, so tests leave nothing
/// behind even if an assertion fails.
pub struct TestDir {
    path: PathBuf,
}

impl TestDir {
    /// Creates a fresh, uniquely named temp directory.
    pub fn new() -> Self {
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut path = std::env::temp_dir();
        path.push(format!(
            "git-overlay-test-{}-{}",
            std::process::id(),
            seq
        ));

        fs::create_dir_all(&path).expect("failed to create temp dir for test");

        Self { path }
    }

    /// The absolute path to the root of this temporary directory.
    #[allow(dead_code)] // part of the reusable test helper API
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Creates a fresh, empty subdirectory named `name` and returns its
    /// absolute path.
    pub fn create_dir(&self, name: &str) -> PathBuf {
        let dir = self.path.join(name);
        fs::create_dir_all(&dir).expect("failed to create subdirectory");
        dir
    }

    /// Writes `content` (UTF-8) to `relative` inside `dir`, creating any
    /// missing parent directories. Returns the absolute path of the file.
    /// Useful for dropping fixed-content files into either the repository or
    /// the overlay directory.
    pub fn write_file(&self, dir: &Path, relative: &str, content: &str) -> PathBuf {
        let file = dir.join(relative);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent).expect("failed to create parent directories");
        }
        fs::write(&file, content).expect("failed to write test file");
        file
    }

    /// Writes `relative` inside the temp root with `content`, creating any
    /// missing parent directories. Returns the absolute path of the file.
    #[allow(dead_code)] // used only by the E2E target
    pub fn write(&self, relative: &str, content: &str) -> PathBuf {
        self.write_file(&self.path, relative, content)
    }

    /// Writes `n` files named `name_prefix`...`n` into `dir`. Used to build
    /// large fixtures for scaling tests. Returns the directory's path.
    pub fn write_many(&self, dir: &Path, name_prefix: &str, n: usize) {
        for i in 0..n {
            self.write_file(dir, &format!("{name_prefix}{i}"), "content");
        }
    }

    /// Creates an empty Git repository named `name` using the real `git` CLI
    /// and returns its absolute path.
    pub fn create_git_repo(&self, name: &str) -> PathBuf {
        let dir = self.create_dir(name);

        let status = Command::new("git")
            .arg("init")
            .arg("--quiet")
            .current_dir(&dir)
            .status()
            .expect("failed to run `git init`");

        assert!(
            status.success(),
            "`git init` failed in {}",
            dir.display()
        );

        dir
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        // Best-effort cleanup; a failed removal is not worth failing the test.
        let _ = fs::remove_dir_all(&self.path);
    }
}