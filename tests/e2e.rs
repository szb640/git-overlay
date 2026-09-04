mod common;

use std::path::Path;
use std::process::Command;

use common::{binary, TestDir};

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

    assert!(
        status.success(),
        "`hello.txt` is not ignored by git"
    );
}

#[test]
fn info_lists_ignore_patterns_and_tracked_files() {
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
    assert!(stdout.contains("hello.txt"), "info did not list the pattern/file");
    assert!(stdout.contains("ignore patterns"), "info should report ignore patterns");
    assert!(stdout.contains("tracked files"), "info should report tracked files");
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
        !stdout.contains("ignore patterns"),
        "info should not report ignore patterns for an uninitialized repository"
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
