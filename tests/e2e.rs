mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::{TestDir, binary};

/// Runs `git-overlay init <overlay>` in `repo` and returns the command
/// output, panicking if the process could not be spawned.
fn run_init(repo: &Path, overlay: &Path) -> std::process::Output {
    Command::new(binary())
        .arg("init")
        .arg(overlay)
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay init`")
}

/// Runs `git-overlay add <patterns...>` in `repo` and returns the command
/// output, panicking if the process could not be spawned.
fn run_add(repo: &Path, patterns: &[&str]) -> std::process::Output {
    Command::new(binary())
        .arg("add")
        .args(patterns)
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay add`")
}

/// Runs `git-overlay sync` in `repo` and returns the command output,
/// panicking if the process could not be spawned.
fn run_sync(repo: &Path) -> std::process::Output {
    Command::new(binary())
        .arg("sync")
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay sync`")
}

/// Runs `git-overlay sync --force <side>` in `repo` and returns the command
/// output, panicking if the process could not be spawned.
fn run_sync_force(repo: &Path, side: &str) -> std::process::Output {
    Command::new(binary())
        .arg("sync")
        .arg("--force")
        .arg(side)
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay sync --force`")
}

/// Sets up a managed repo and overlay where `foo.txt` exists in both places
/// with different contents, returning the `(repo, overlay)` paths.
fn setup_conflicting_foo(dir: &TestDir) -> (PathBuf, PathBuf) {
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");
    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    dir.write_file(&repo, "foo.txt", "repo-foo");
    dir.write_file(&overlay, "foo.txt", "overlay-foo");
    (repo, overlay)
}

/// Asserts that `a` and `b` are hard links to the same inode.
fn assert_same_inode(a: &Path, b: &Path, ctx: &str) {
    use std::os::unix::fs::MetadataExt;
    assert_eq!(
        std::fs::metadata(a).unwrap().ino(),
        std::fs::metadata(b).unwrap().ino(),
        "{ctx}: files should be hard links to the same inode"
    );
}

/// Asserts that `file` is ignored by git in `repo`.
fn assert_ignored(repo: &Path, file: &str, ctx: &str) {
    let status = Command::new("git")
        .arg("check-ignore")
        .arg("--quiet")
        .arg(file)
        .current_dir(repo)
        .status()
        .expect("failed to run `git check-ignore`");
    assert!(status.success(), "{ctx}: `{file}` should be ignored by git");
}

/// Runs `git-overlay remove <patterns...>` in `repo` and returns the command
/// output, panicking if the process could not be spawned.
fn run_remove(repo: &Path, patterns: &[&str]) -> std::process::Output {
    Command::new(binary())
        .arg("remove")
        .args(patterns)
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay remove`")
}

/// Runs `git-overlay info` in `repo` and returns the command output,
/// panicking if the process could not be spawned.
fn run_info(repo: &Path) -> std::process::Output {
    Command::new(binary())
        .arg("info")
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay info`")
}

/// Runs `git-overlay info --json` in `repo` and returns the command output,
/// panicking if the process could not be spawned.
fn run_info_json(repo: &Path) -> std::process::Output {
    Command::new(binary())
        .arg("info")
        .arg("--json")
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay info --json`")
}

/// Runs `git-overlay ignore add <patterns...>` in `repo` and returns the
/// command output, panicking if the process could not be spawned.
fn run_ignore_add(repo: &Path, patterns: &[&str]) -> std::process::Output {
    Command::new(binary())
        .arg("ignore")
        .arg("add")
        .args(patterns)
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay ignore add`")
}

/// Runs `git-overlay ignore remove <patterns...>` in `repo` and returns the
/// command output, panicking if the process could not be spawned.
fn run_ignore_remove(repo: &Path, patterns: &[&str]) -> std::process::Output {
    Command::new(binary())
        .arg("ignore")
        .arg("remove")
        .args(patterns)
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay ignore remove`")
}

/// Runs `git-overlay file add <paths...>` in `repo` and returns the command
/// output, panicking if the process could not be spawned.
fn run_file_add(repo: &Path, paths: &[&str]) -> std::process::Output {
    Command::new(binary())
        .arg("file")
        .arg("add")
        .args(paths)
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay file add`")
}

/// Runs `git-overlay file remove <paths...>` in `repo` and returns the
/// command output, panicking if the process could not be spawned.
fn run_file_remove(repo: &Path, paths: &[&str]) -> std::process::Output {
    Command::new(binary())
        .arg("file")
        .arg("remove")
        .args(paths)
        .current_dir(repo)
        .output()
        .expect("failed to spawn `git-overlay file remove`")
}

#[test]
fn init_records_overlay_path_in_new_repo() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    // Initializing from inside the Git repo should point it at the overlay.
    let output = run_init(&repo, &overlay);

    assert!(
        output.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The tool writes a config recording the overlay path and saves an
    // (initially empty) private exclude file.
    assert!(
        repo.join(".git/overlay.yml").exists(),
        "config file `.git/overlay.yml` was not created"
    );
    assert!(
        repo.join(".git/info/exclude").is_file(),
        "exclude file `.git/info/exclude` was not created"
    );
}

#[test]
fn init_brings_overlay_file_into_repo_and_ignores_it() {
    let dir = TestDir::new();

    // A Git-managed repository and an overlay directory that already contains
    // a private file.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");
    dir.write_file(&overlay, "hello.txt", "world");

    let output = run_init(&repo, &overlay);

    assert!(
        output.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The overlay file should now be present in the repository...
    let linked = repo.join("hello.txt");
    assert!(
        linked.is_file(),
        "`hello.txt` was not created in the repository"
    );
    assert_eq!(
        std::fs::read_to_string(&linked).expect("failed to read `hello.txt`"),
        "world",
        "`hello.txt` content does not match the overlay file"
    );

    // ...and ignored by git, so it does not show up as untracked.
    let status = Command::new("git")
        .arg("check-ignore")
        .arg("--quiet")
        .arg("hello.txt")
        .current_dir(&repo)
        .status()
        .expect("failed to run `git check-ignore`");

    assert!(status.success(), "`hello.txt` is not ignored by git");
}

#[test]
fn info_lists_exclude_patterns_and_tracked_files() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    // Initialize first so the repo is managed.
    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Add a file so there is something to report as tracked.
    dir.write_file(&repo, "hello.txt", "world");
    let add = run_add(&repo, &["hello.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay add hello.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let output = run_info(&repo);
    assert!(
        output.status.success(),
        "`git-overlay info` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hello.txt"),
        "info did not list the pattern/file"
    );
    assert!(
        stdout.contains("exclude patterns"),
        "info should report exclude patterns"
    );
    assert!(
        stdout.contains("tracked files"),
        "info should report tracked files"
    );
}

#[test]
fn info_lists_files_managed_by_sync() {
    let dir = TestDir::new();

    // A Git-managed repository and an overlay directory, both containing
    // `hello.txt` and `.hello.txt` files with identical content.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");
    dir.write_file(&repo, "hello.txt", "world");
    dir.write_file(&overlay, "hello.txt", "world");
    dir.write_file(&repo, ".hello.txt", "world");
    dir.write_file(&overlay, ".hello.txt", "world");

    // Initializing creates the overlay link in the repository,and `sync`
    // folds the overlay files into the ignore rules as managed patterns.
    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    let sync = run_sync(&repo);
    assert!(
        sync.status.success(),
        "`git-overlay sync` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    // `info` should now report `hello.txt` as a managed (tracked) file.
    let output = run_info_json(&repo);
    assert!(
        output.status.success(),
        "`git-overlay info --json` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("info --json did not print valid JSON: {e}"));

    // The managed files are read from the `tracked_files` JSON key.
    let tracked = value["tracked_files"]
        .as_array()
        .unwrap_or_else(|| panic!("info should report tracked files"));

    let files: Vec<&str> = tracked.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        files.contains(&"hello.txt"),
        "info should list `hello.txt` as tracked after `sync`"
    );
    assert!(
        files.contains(&".hello.txt"),
        "info should list `.hello.txt` as tracked after `sync`"
    );
}

#[test]
fn info_on_uninitialized_repo_only_reports_not_initialized() {
    let dir = TestDir::new();

    // A Git-managed repository that has never been `init`-ed.
    let repo = dir.create_git_repo("repo");

    let output = run_info(&repo);
    assert!(
        output.status.success(),
        "`git-overlay info` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("not initialized"),
        "info should report the repository is not initialized"
    );
    assert!(
        !stdout.contains("tracked files"),
        "info should not report tracked files for an uninitialized repository"
    );
    assert!(
        !stdout.contains("exclude patterns"),
        "info should not report exclude patterns for an uninitialized repository"
    );
}

#[test]
fn info_json_prints_valid_json() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    // Initialize first so the repo is managed.
    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Add a file so there is something to report as tracked.
    dir.write_file(&repo, "hello.txt", "world");
    let add = run_add(&repo, &["hello.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay add hello.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let output = run_info_json(&repo);
    assert!(
        output.status.success(),
        "`git-overlay info --json` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The output must parse as a single JSON object.
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("info --json did not print valid JSON: {e}"));
    let obj = value.as_object().unwrap();
    assert!(obj.contains_key("path"), "json info should report the path");
    assert!(
        obj.contains_key("initialized"),
        "json info should report initialization state"
    );
    assert!(
        obj.contains_key("exclude_patterns"),
        "json info should report exclude patterns"
    );
    assert!(
        obj.contains_key("tracked_files"),
        "json info should report tracked files"
    );
    assert!(
        obj.contains_key("ignore_patterns"),
        "json info should report ignore patterns"
    );
}

#[test]
fn info_json_on_uninitialized_repo_prints_initialized_false() {
    let dir = TestDir::new();

    // A Git-managed repository that has never been `init`-ed.
    let repo = dir.create_git_repo("repo");

    let output = run_info_json(&repo);
    assert!(
        output.status.success(),
        "`git-overlay info --json` failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("info --json did not print valid JSON: {e}"));
    assert!(
        value["initialized"] == serde_json::Value::Bool(false),
        "json info should report an uninitialized repository as `initialized: false`"
    );
}

#[test]
fn add_moves_git_file_into_overlay() {
    let dir = TestDir::new();

    // An empty Git-managed repository, an empty overlay directory.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    // Initialize first so the repo is managed.
    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Now add a file to the repository that we want to make private.
    dir.write_file(&repo, "hello.txt", "world");

    let add = run_add(&repo, &["hello.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay add hello.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    // The file should now be present in the overlay directory, with its
    // content preserved.
    let moved = overlay.join("hello.txt");
    assert!(
        moved.is_file(),
        "`hello.txt` was not added to the overlay directory"
    );
    assert_eq!(
        std::fs::read_to_string(&moved).expect("failed to read overlay `hello.txt`"),
        "world",
        "overlay `hello.txt` content does not match the original file"
    );
}

#[test]
fn add_pattern_moves_matching_git_file_into_overlay() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    // Initialize first so the repo is managed.
    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Now add a file to the repository that matches the pattern.
    dir.write_file(&repo, "hello.txt", "world");

    let add = run_add(&repo, &["*.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay add *.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    // The matching file should be moved into the overlay directory, with its
    // content preserved.
    let moved = overlay.join("hello.txt");
    assert!(
        moved.is_file(),
        "`hello.txt` was not added to the overlay directory via `*.txt`"
    );
    assert_eq!(
        std::fs::read_to_string(&moved).expect("failed to read overlay `hello.txt`"),
        "world",
        "overlay `hello.txt` content does not match the original file"
    );
}

#[test]
fn add_does_not_sync_overlay_files_but_sync_does() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory,
    // initialized so the repo is managed.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Place a file by hand in the overlay, and another in the repository that
    // we will make managed via `add`.
    dir.write_file(&overlay, "bar.txt", "bar");
    dir.write_file(&repo, "foo.txt", "foo");

    let add = run_add(&repo, &["foo.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay add foo.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    // `add` only moves the matched repository file into the overlay; it does
    // not pull overlay files back into the repo.
    assert!(
        overlay.join("foo.txt").is_file(),
        "`foo.txt` was not added to the overlay directory"
    );
    assert!(
        !repo.join("bar.txt").exists(),
        "`bar.txt` should not be in the repository before `sync`"
    );

    // A subsequent `sync` brings the hand-placed overlay file back into the
    // repository.
    let sync = run_sync(&repo);
    assert!(
        sync.status.success(),
        "`git-overlay sync` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    assert!(
        repo.join("bar.txt").is_file(),
        "`bar.txt` was not added to the repository by `sync`"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("bar.txt")).expect("failed to read repo `bar.txt`"),
        "bar",
        "repo `bar.txt` content does not match the overlay file"
    );
}

#[test]
fn sync_after_deleting_repo_file_removes_it_from_overlay() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an overlay directory already
    // holding a single private file.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");
    dir.write_file(&overlay, "hello.txt", "world");

    // Initialize the repo with the overlay and sync so the file is pulled
    // into the repository (and registered as managed).
    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let sync = run_sync(&repo);
    assert!(
        sync.status.success(),
        "`git-overlay sync` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    assert!(
        repo.join("hello.txt").is_file(),
        "`hello.txt` was not synced into the repository"
    );

    // Delete the synced file from the repository, then sync again.
    std::fs::remove_file(repo.join("hello.txt")).expect("failed to remove repo `hello.txt`");

    let sync = run_sync(&repo);
    assert!(
        sync.status.success(),
        "`git-overlay sync` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    // Because the managed file is no longer in the repository, the sync
    // removes its copy from the overlay directory as well.
    assert!(
        !overlay.join("hello.txt").exists(),
        "`hello.txt` should be removed from the overlay after `sync`"
    );

    // The overlay's own config should no longer reference the file.
    let overlay_config = std::fs::read_to_string(overlay.join(".git-overlay.yml"))
        .expect("failed to read overlay config");
    assert!(
        !overlay_config.contains("hello.txt"),
        "overlay config should no longer reference `hello.txt`, got:\n{overlay_config}"
    );
}

#[test]
fn remove_keeps_file_in_repo_but_removes_from_overlay() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory,
    // initialized so the repo is managed.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Place two files by hand in the overlay and bring them into the repo
    // via `sync`.
    dir.write_file(&overlay, "foo.txt", "foo");
    dir.write_file(&overlay, "bar.txt", "bar");

    let sync = run_sync(&repo);
    assert!(
        sync.status.success(),
        "`git-overlay sync` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    assert!(
        repo.join("foo.txt").is_file(),
        "`foo.txt` was not brought into the repository"
    );
    assert!(
        repo.join("bar.txt").is_file(),
        "`bar.txt` was not brought into the repository"
    );

    // Removing the `foo.txt` pattern stops managing it: the file stays in
    // the repository, but its copy is dropped from the overlay.
    let remove = run_remove(&repo, &["foo.txt"]);
    assert!(
        remove.status.success(),
        "`git-overlay remove foo.txt` failed: {}",
        String::from_utf8_lossy(&remove.stderr)
    );

    assert!(
        repo.join("foo.txt").is_file(),
        "`foo.txt` should still exist in the repository after `remove`"
    );
    assert!(
        !overlay.join("foo.txt").exists(),
        "`foo.txt` should be removed from the overlay after `remove`"
    );

    // The other file is untouched.
    assert!(
        overlay.join("bar.txt").is_file(),
        "`bar.txt` should still exist in the overlay"
    );
}

#[test]
fn remove_after_manually_deleting_overlay_file_succeeds() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory,
    // initialized so the repo is managed.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Add a file to the repo, move it into the overlay via `add`, and
    // bring it fully under management via `sync`.
    dir.write_file(&repo, "hello.txt", "world");
    let add = run_add(&repo, &["hello.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay add hello.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    let sync = run_sync(&repo);
    assert!(
        sync.status.success(),
        "`git-overlay sync` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    assert!(
        overlay.join("hello.txt").is_file(),
        "`hello.txt` should be in the overlay after `sync`"
    );

    // Remove the file from the overlay by hand, leaving it only in the
    // repository (as if a user or another tool removed it).
    std::fs::remove_file(overlay.join("hello.txt"))
        .expect("failed to manually remove overlay `hello.txt`");

    // `remove` should still succeed: it drops the pattern from the ignore
    // list and the stale managed record, without tripping over the already
    // -removed overlay file.
    let remove = run_remove(&repo, &["hello.txt"]);
    assert!(
        remove.status.success(),
        "`git-overlay remove hello.txt` failed: {}",
        String::from_utf8_lossy(&remove.stderr)
    );

    // The repository copy stays; nothing is left in the overlay.
    assert!(
        repo.join("hello.txt").is_file(),
        "`hello.txt` should still exist in the repository after `remove`"
    );
    assert!(
        !overlay.join("hello.txt").exists(),
        "`hello.txt` should not appear in the overlay after `remove`"
    );
}

#[test]
fn remove_one_of_overlapping_patterns_keeps_file_managed() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory,
    // initialized so the repo is managed.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // A file in the repository made private by two overlapping patterns:
    // an explicit name and a glob that also matches it.
    dir.write_file(&repo, "foo.txt", "foo");

    let add = run_add(&repo, &["foo.txt", "*.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay add foo.txt *.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    assert!(
        overlay.join("foo.txt").is_file(),
        "`foo.txt` was not added to the overlay directory"
    );

    // Removing only the explicit `foo.txt` pattern must not unmanage the
    // file, because `*.txt` still covers it.
    let remove = run_remove(&repo, &["foo.txt"]);
    assert!(
        remove.status.success(),
        "`git-overlay remove foo.txt` failed: {}",
        String::from_utf8_lossy(&remove.stderr)
    );

    assert!(
        repo.join("foo.txt").is_file(),
        "`foo.txt` should remain in the repository"
    );
    assert!(
        overlay.join("foo.txt").is_file(),
        "`foo.txt` should remain in the overlay"
    );
}

#[test]
fn sync_with_file_in_both_repo_and_overlay_keeps_both() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory,
    // initialized so the repo is managed.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // A file that exists (independently) both in the repository and in the
    // overlay, but is not registered as a managed pattern.
    dir.write_file(&repo, "foo.txt", "repo-foo");
    dir.write_file(&overlay, "foo.txt", "overlay-foo");

    let sync = run_sync(&repo);
    assert!(
        sync.status.success(),
        "`git-overlay sync` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );
    assert!(
        repo.join("foo.txt").is_file(),
        "`foo.txt` should remain in the repository"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("foo.txt")).expect("failed to read repo `foo.txt`"),
        "repo-foo",
        "repository copy of `foo.txt` was modified by `sync`"
    );
    assert!(
        overlay.join("foo.txt").is_file(),
        "`foo.txt` should remain in the overlay"
    );
    assert_eq!(
        std::fs::read_to_string(overlay.join("foo.txt")).expect("failed to read overlay `foo.txt`"),
        "overlay-foo",
        "overlay copy of `foo.txt` was modified by `sync`"
    );

    // `sync` pins the pre-existing overlay file into the ignore rules, so it
    // no longer shows up as untracked.
    let status = Command::new("git")
        .arg("check-ignore")
        .arg("--quiet")
        .arg("foo.txt")
        .current_dir(&repo)
        .status()
        .expect("failed to run `git check-ignore`");
    assert!(
        status.success(),
        "`foo.txt` should be ignored by git after `sync`"
    );
}

#[test]
fn sync_with_identical_file_in_both_repo_and_overlay_links_them() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory,
    // initialized so the repo is managed.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // A file that exists (independently) both in the repository and in the
    // overlay with the same content, but is not yet managed.
    dir.write_file(&repo, "foo.txt", "same");
    dir.write_file(&overlay, "foo.txt", "same");

    let sync = run_sync(&repo);
    assert!(
        sync.status.success(),
        "`git-overlay sync` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    // Both copies remain, with identical content, and are now hard links to
    // the same inode (the overlay file is the source of truth).
    use std::os::unix::fs::MetadataExt;
    let repo_ino = std::fs::metadata(repo.join("foo.txt"))
        .expect("failed to stat repo `foo.txt`")
        .ino();
    let overlay_ino = std::fs::metadata(overlay.join("foo.txt"))
        .expect("failed to stat overlay `foo.txt`")
        .ino();
    assert!(
        repo_ino == overlay_ino,
        "repo `foo.txt` (ino {repo_ino}) and overlay `foo.txt` (ino {overlay_ino}) \
         should be hard links to the same inode after `sync`"
    );

    // Content is preserved on both sides.
    assert_eq!(
        std::fs::read_to_string(repo.join("foo.txt")).expect("failed to read repo `foo.txt`"),
        "same",
        "repository copy of `foo.txt` was changed by `sync`"
    );
    assert_eq!(
        std::fs::read_to_string(overlay.join("foo.txt")).expect("failed to read overlay `foo.txt`"),
        "same",
        "overlay copy of `foo.txt` was changed by `sync`"
    );

    // The file is registered as managed and ignored by git.
    let status = Command::new("git")
        .arg("check-ignore")
        .arg("--quiet")
        .arg("foo.txt")
        .current_dir(&repo)
        .status()
        .expect("failed to run `git check-ignore`");
    assert!(
        status.success(),
        "`foo.txt` should be ignored by git after `sync`"
    );
}

#[test]
fn sync_force_overlay_keeps_overlay_copy_when_conflict() {
    let dir = TestDir::new();
    let (repo, overlay) = setup_conflicting_foo(&dir);

    // `--force overlay` keeps the overlay copy and updates the repo to match.
    let sync = run_sync_force(&repo, "overlay");
    assert!(
        sync.status.success(),
        "`git-overlay sync --force overlay` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    assert_eq!(
        std::fs::read_to_string(repo.join("foo.txt")).expect("failed to read repo `foo.txt`"),
        "overlay-foo",
        "`--force overlay` should update the repository copy to the overlay's content"
    );
    assert_eq!(
        std::fs::read_to_string(overlay.join("foo.txt")).expect("failed to read overlay `foo.txt`"),
        "overlay-foo",
        "`--force overlay` should keep the overlay copy"
    );

    // The two files are now hard links to the same (overlay) inode.
    assert_same_inode(
        &repo.join("foo.txt"),
        &overlay.join("foo.txt"),
        "`--force overlay`",
    );

    // And the file is managed / ignored by git.
    assert_ignored(&repo, "foo.txt", "`--force overlay`");
}

#[test]
fn sync_force_repository_keeps_repo_copy_when_conflict() {
    let dir = TestDir::new();
    let (repo, overlay) = setup_conflicting_foo(&dir);

    // `--force repository` keeps the repository copy and updates the overlay
    // to match.
    let sync = run_sync_force(&repo, "repository");
    assert!(
        sync.status.success(),
        "`git-overlay sync --force repository` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    assert_eq!(
        std::fs::read_to_string(repo.join("foo.txt")).expect("failed to read repo `foo.txt`"),
        "repo-foo",
        "`--force repository` should keep the repository copy"
    );
    assert_eq!(
        std::fs::read_to_string(overlay.join("foo.txt")).expect("failed to read overlay `foo.txt`"),
        "repo-foo",
        "`--force repository` should update the overlay copy to the repository's content"
    );

    // The two files are now hard links to the same (repository) inode.
    assert_same_inode(
        &repo.join("foo.txt"),
        &overlay.join("foo.txt"),
        "`--force repository`",
    );

    // And the file is managed / ignored by git.
    assert_ignored(&repo, "foo.txt", "`--force repository`");
}

#[test]
fn sync_force_this_alias_keeps_repo_copy() {
    let dir = TestDir::new();
    let (repo, overlay) = setup_conflicting_foo(&dir);

    // `this` is an alias for keeping the repository copy.
    let sync = run_sync_force(&repo, "this");
    assert!(
        sync.status.success(),
        "`git-overlay sync --force this` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    assert_eq!(
        std::fs::read_to_string(repo.join("foo.txt")).expect("failed to read repo `foo.txt`"),
        "repo-foo",
        "`--force this` should keep the repository copy"
    );
    assert_eq!(
        std::fs::read_to_string(overlay.join("foo.txt")).expect("failed to read overlay `foo.txt`"),
        "repo-foo",
        "`--force this` should update the overlay copy to the repository's content"
    );
    assert_same_inode(
        &repo.join("foo.txt"),
        &overlay.join("foo.txt"),
        "`--force this` alias",
    );
}

#[test]
fn sync_force_that_alias_keeps_overlay_copy() {
    let dir = TestDir::new();
    let (repo, overlay) = setup_conflicting_foo(&dir);

    // `that` is an alias for keeping the overlay copy.
    let sync = run_sync_force(&repo, "that");
    assert!(
        sync.status.success(),
        "`git-overlay sync --force that` failed: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    assert_eq!(
        std::fs::read_to_string(repo.join("foo.txt")).expect("failed to read repo `foo.txt`"),
        "overlay-foo",
        "`--force that` should keep the overlay copy"
    );
    assert_eq!(
        std::fs::read_to_string(overlay.join("foo.txt")).expect("failed to read overlay `foo.txt`"),
        "overlay-foo",
        "`--force that` should update the repository copy to the overlay's content"
    );
    assert_same_inode(
        &repo.join("foo.txt"),
        &overlay.join("foo.txt"),
        "`--force that` alias",
    );
}

#[test]
fn add_pattern_does_not_clobber_conflicting_overlay_file() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory,
    // initialized so the repo is managed.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // A file exists in both places with different contents: the repo copy
    // matches the pattern we are about to add, and the overlay already has
    // its own conflicting copy.
    dir.write_file(&repo, "foo.txt", "repo-foo");
    dir.write_file(&overlay, "foo.txt", "overlay-foo");

    let add = run_add(&repo, &["foo.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay add foo.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("foo.txt")).expect("failed to read repo `foo.txt`"),
        "repo-foo",
        "repository copy of `foo.txt` was modified by `add`"
    );
    assert_eq!(
        std::fs::read_to_string(overlay.join("foo.txt")).expect("failed to read overlay `foo.txt`"),
        "overlay-foo",
        "overlay copy of `foo.txt` was clobbered by `add`"
    );
}

#[test]
fn ignore_add_puts_pattern_outside_managed_block_and_removes_it() {
    let dir = TestDir::new();

    // An empty Git-managed repository and an empty overlay directory,
    // initialized so the repo is managed.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");

    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let exclude_path = repo.join(".git/info/exclude");
    let config_path = overlay.join(".git-overlay.yml");

    // Add an ignore pattern and confirm it lands both in the private exclude
    // file OUTSIDE the managed guard block and in the directory config.
    let add = run_ignore_add(&repo, &["foo.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay ignore add foo.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let exclude = std::fs::read_to_string(&exclude_path).expect("failed to read exclude file");
    let close_guard_idx = exclude
        .find("# <<< managed by git-overlay")
        .expect("exclude file should contain the closing guard");
    assert!(
        exclude[close_guard_idx..].contains("foo.txt"),
        "ignore pattern should be written outside the managed block, got:\n{exclude}"
    );

    let config = std::fs::read_to_string(&config_path).expect("failed to read overlay config");
    assert!(
        config.contains("ignore_patterns") && config.contains("foo.txt"),
        "directory config should record the ignore pattern, got:\n{config}"
    );

    // Remove the ignore pattern and confirm it is gone from both places.
    let remove = run_ignore_remove(&repo, &["foo.txt"]);
    assert!(
        remove.status.success(),
        "`git-overlay ignore remove foo.txt` failed: {}",
        String::from_utf8_lossy(&remove.stderr)
    );

    let exclude = std::fs::read_to_string(&exclude_path).expect("failed to read exclude file");
    assert!(
        !exclude.contains("foo.txt"),
        "ignore pattern should be gone from the exclude file, got:\n{exclude}"
    );

    let config = std::fs::read_to_string(&config_path).expect("failed to read overlay config");
    assert!(
        !config.contains("foo.txt"),
        "ignore pattern should be gone from the directory config, got:\n{config}"
    );
}

#[test]
fn file_add_manages_only_the_specific_path() {
    let dir = TestDir::new();

    // A Git-managed repository with the same file name in two directories,
    // and an empty overlay directory.
    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");
    dir.write_file(&repo, "top.txt", "top");
    dir.write_file(&repo, "sub/top.txt", "sub");

    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Add only the nested file, by its repository-relative path.
    let add = run_file_add(&repo, &["sub/top.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay file add sub/top.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    // The pattern recorded must be anchored at the repository root, so it
    // targets exactly that file and no other file of the same name.
    let exclude =
        std::fs::read_to_string(repo.join(".git/info/exclude")).expect("failed to read exclude");
    assert!(
        exclude.contains("/sub/top.txt"),
        "expected a root-anchored `/sub/top.txt` pattern, got:\n{exclude}"
    );

    // Only the specific file moved into the overlay; the root `top.txt` was
    // left alone even though it shares the same name.
    assert!(
        overlay.join("sub/top.txt").is_file(),
        "`sub/top.txt` should be managed in the overlay"
    );
    assert!(
        !overlay.join("top.txt").exists(),
        "root `top.txt` should not be managed by `/sub/top.txt`"
    );

    // Nor is the root file ignored by git.
    let status = Command::new("git")
        .arg("check-ignore")
        .arg("--quiet")
        .arg("top.txt")
        .current_dir(&repo)
        .status()
        .expect("failed to run `git check-ignore`");
    assert!(
        !status.success(),
        "root `top.txt` should not be ignored by `/sub/top.txt`"
    );
}

#[test]
fn file_add_normalizes_relative_path() {
    let dir = TestDir::new();

    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");
    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    // Adding a file by a path that goes up through `..` should still be
    // recorded against its clean repository-relative path.
    dir.write_file(&repo, "sub/top.txt", "sub");
    let add = run_file_add(&repo, &["sub/./../sub/top.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay file add sub/./../sub/top.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

    let exclude =
        std::fs::read_to_string(repo.join(".git/info/exclude")).expect("failed to read exclude");
    assert!(
        exclude.contains("/sub/top.txt"),
        "expected a normalized `/sub/top.txt` pattern, got:\n{exclude}"
    );
    assert!(
        !exclude.contains(".."),
        "no `..` should remain in the pattern, got:\n{exclude}"
    );
}

#[test]
fn file_remove_unmanages_the_specific_path() {
    let dir = TestDir::new();

    let repo = dir.create_git_repo("repo");
    let overlay = dir.create_dir("overlay");
    let init = run_init(&repo, &overlay);
    assert!(
        init.status.success(),
        "`git-overlay init` failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    dir.write_file(&repo, "sub/top.txt", "sub");
    let add = run_file_add(&repo, &["sub/top.txt"]);
    assert!(
        add.status.success(),
        "`git-overlay file add sub/top.txt` failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );
    assert!(
        overlay.join("sub/top.txt").is_file(),
        "`sub/top.txt` should be managed after `file add`"
    );

    let remove = run_file_remove(&repo, &["sub/top.txt"]);
    assert!(
        remove.status.success(),
        "`git-overlay file remove sub/top.txt` failed: {}",
        String::from_utf8_lossy(&remove.stderr)
    );

    // The file stays in the repository but is dropped from the overlay, and
    // its anchored pattern is gone from the exclude file.
    assert!(
        repo.join("sub/top.txt").is_file(),
        "`sub/top.txt` should stay in the repository after `file remove`"
    );
    assert!(
        !overlay.join("sub/top.txt").exists(),
        "`sub/top.txt` should be removed from the overlay after `file remove`"
    );
    let exclude =
        std::fs::read_to_string(repo.join(".git/info/exclude")).expect("failed to read exclude");
    assert!(
        !exclude.contains("/sub/top.txt"),
        "`/sub/top.txt` should be removed from the exclude file, got:\n{exclude}"
    );
}
