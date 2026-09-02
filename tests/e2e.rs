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