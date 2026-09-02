//! Performance / scaling tests, kept separate from the functional E2E suite.
//!
//! These are opt-in because they can be slow and build large fixtures (up to
//! 10k files by default). Run them with a release build for meaningful
//! timings, single-threaded so parallel execution does not distort or
//! interleave the results:
//!
//! ```sh
//! cargo test --release --test scale -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Each benchmark sweeps a set of `(files, patterns)` points automatically,
//! starting from `1:1` and climbing to the configured maximums:
//!
//! | files | patterns |
//! |------:|---------:|
//! |     1 |        1 |
//! |    10 |       10 |
//! |   100 |      100 |
//! |  1000 |      100 |
//! | 10000 |      100 |
//!
//! The maximums (and hence the sweep) can be overridden with env vars:
//! - `SCALE_MAX_FILES`   — final file count (default 10_000)
//! - `SCALE_MAX_PATTERNS`— final pattern count (default 100)

mod common;

use std::env;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use common::{binary, TestDir};

/// Reads an env var as `usize`, falling back to `default` when unset or
/// unparsable.
fn param(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// The `(files, patterns)` sweep points: a geometric progression from `1:1`
/// up to `max_files` files and `max_patterns` patterns.
fn scale_points(max_files: usize, max_patterns: usize) -> Vec<(usize, usize)> {
    let mut points = Vec::new();
    let mut step = 1usize;
    while step < max_files || step < max_patterns {
        points.push((step.min(max_files), step.min(max_patterns)));
        step = step.saturating_mul(10);
    }
    points.push((max_files, max_patterns));
    points
}

/// Runs an operation in `dir` and returns its wall-clock duration.
fn time_cmd(dir: &Path, args: &[String]) -> Duration {
    let start = Instant::now();
    let output = Command::new(binary())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to spawn git-overlay");
    let elapsed = start.elapsed();
    assert!(
        output.status.success(),
        "`git-overlay {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    elapsed
}

/// Builds an initialized fixture: an empty git repo, an empty overlay, and
/// `files` files in the repo plus `patterns` files in the overlay. `init` is
/// timed separately.
fn fixture(
    dir: &TestDir,
    files: usize,
    patterns: usize,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");
    let init = time_cmd(&repo, &["init".into(), overlay.to_str().unwrap().into()]);
    println!("  init:            {init:?}");
    dir.write_many(&repo, "f", files);
    dir.write_many(&overlay, "g", patterns);
    (repo, overlay)
}

/// Adds `patterns` distinct patterns, each matching one repo file named
/// `v<i>`, in a single `add` invocation.
fn bench_add(repo: &Path, patterns: usize) -> Duration {
    let mut args: Vec<String> = vec!["add".into()];
    args.extend((0..patterns).map(|i| format!("v{i}")));
    time_cmd(repo, &args)
}

/// Removes `patterns` distinct patterns in a single `remove` invocation.
fn bench_remove(repo: &Path, patterns: usize) -> Duration {
    let mut args: Vec<String> = vec!["remove".into()];
    args.extend((0..patterns).map(|i| format!("v{i}")));
    time_cmd(repo, &args)
}

/// Measures `sync` at each sweep point: it must fold every overlay file into
/// the exclude list and sweep over the whole repository.
#[test]
#[ignore = "optional scaling benchmark"]
fn sync_scales() {
    let max_files = param("SCALE_MAX_FILES", 10_000);
    let max_patterns = param("SCALE_MAX_PATTERNS", 100);

    println!("sync scaling -> {max_files} files / {max_patterns} patterns");
    for (files, patterns) in scale_points(max_files, max_patterns) {
        let dir = TestDir::new();
        let (repo, overlay) = fixture(&dir, files, patterns);
        let t = time_cmd(&repo, &["sync".into()]);
        println!("  files={files:>6}  patterns={patterns:>6}  sync={t:?}");
        let _ = overlay;
    }
}

/// Measures `add` at each sweep point: the number of patterns and the number
/// of repo files it must walk both grow.
#[test]
#[ignore = "optional scaling benchmark"]
fn add_scales() {
    let max_files = param("SCALE_MAX_FILES", 10_000);
    let max_patterns = param("SCALE_MAX_PATTERNS", 100);

    println!("add scaling -> {max_files} files / {max_patterns} patterns");
    for (files, patterns) in scale_points(max_files, max_patterns) {
        let dir = TestDir::new();
        let (repo, _overlay) = fixture(&dir, files, 0);
        // Each pattern `v<i>` matches a file the fixture must also contain,
        // so add those to the repo before timing `add`.
        dir.write_many(&repo, "v", patterns);
        let t = bench_add(&repo, patterns);
        println!("  files={files:>6}  patterns={patterns:>6}  add={t:?}");
    }
}

/// Measures `remove` at each sweep point: after adding `patterns` patterns,
/// remove them all in one call.
#[test]
#[ignore = "optional scaling benchmark"]
fn remove_scales() {
    let max_files = param("SCALE_MAX_FILES", 10_000);
    let max_patterns = param("SCALE_MAX_PATTERNS", 100);

    println!("remove scaling -> {max_files} files / {max_patterns} patterns");
    for (files, patterns) in scale_points(max_files, max_patterns) {
        let dir = TestDir::new();
        let (repo, _overlay) = fixture(&dir, files, 0);
        dir.write_many(&repo, "v", patterns);
        bench_add(&repo, patterns);
        let t = bench_remove(&repo, patterns);
        println!("  files={files:>6}  patterns={patterns:>6}  remove={t:?}");
    }
}