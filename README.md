# Git Overlay

**Git Overlay** is a Rust command-line tool for managing a private layer of
personal files inside a Git repository without contributing them upstream.

It is built for workflows where you want to customize a public or shared
repository with your own environment. For example:

- Inject values from your password manager into environment variables via
  `direnv`.
- Test-drive agent skill files without fear of accidental modifications.
- Back up, version, and synchronize personal configuration alongside the
  repository that uses it.

Git Overlay bridges the gap between a shared repository and your personal
files by **hard-linking** them together. Files in your overlay directory are
linked into the repository, and are listed in the repository's *private*
ignore file so they are never staged, committed, or shared with other clones.

## Features

- **Hard-link based** — files in the overlay directory and their counterparts
  in the repository share the same inode, so edits on either side are always
  in sync without copying.
- **Private to your clone** — managed files are recorded in `.git/info/exclude`,
  which is never tracked or propagated upstream, so your personal files stay
  out of the shared history.
- **Pattern-based management** — add files individually or with gitignore-style
  patterns, so new matching files are picked up automatically.
- **Bidirectional sync** — new overlay files are linked into the repository,
  and newly excluded repository files are moved into the overlay.
- **Personal ignore patterns** — exclude files (e.g. `.venv` directories)
  from synchronization entirely.
- **Machine-readable output** — `info --json` emits the managed and ignored
  patterns as JSON for scripting.

## Installation

Build from source with Cargo:

```bash
cargo build --release
# binary is emitted to target/release/git-overlay
```

The project also ships a Nix flake (see `flake.nix`) for reproducible builds.

## Quick Start

### 1. Initialize

Run `init` from inside a Git repository, pointing at the directory that will
hold your personal files:

```bash
cd ~/public-project
git-overlay init ~/personal-files
```

This records the overlay location, initializes the private ignore file, and
links any existing overlay files into the repository.

### 2. Sync from the overlay directory

Any file you place in the overlay directory is linked into the repository on
the next `sync`:

```bash
cd ~/public-project
touch ~/personal-files/hello.txt
git-overlay sync
```

### 3. Add repository files to the overlay

Bring existing files into the overlay, either individually or by pattern:

```bash
cd ~/public-project

# Add a single file
touch hello-world.txt
git-overlay add hello-world.txt

# Add a single file, matched only by its exact repository path
git-overlay file add hello-world.txt

# Add a pattern; every matching file (now and in the future) is managed
git-overlay add '*.txt'
```

The difference between `add` and `file add` is intent: `add` stores the
pattern literally, so adding `hello.txt` matches *any* file of that name
anywhere in the repository. `file add` anchors the pattern at the repository
root (storing, e.g., `/config/settings.toml`), so it targets only that one
file at that exact path—a same-named file in another directory is left
alone. Use it when you want to manage a specific file now without also
claiming future files that happen to share its name.

A file is managed as long as it matches at least one active pattern. This
makes it safe to migrate from explicit files to pattern-based matching:

```bash
touch a.txt b.txt
git-overlay add a.txt b.txt     # track explicitly
git-overlay add '*.txt'         # switch to pattern matching
git-overlay remove a.txt b.txt  # drop the now-redundant explicit entries
```

### 4. Exclude personal directories

Skip generated files, such as `.venv` directories created by your scripts:

```bash
git-overlay ignore add .venv
```

Ignore patterns are written *outside* the managed region of the exclude file
and stored in the overlay configuration, so they are easy to back up and
reuse across machines.

## Command Reference

Run `git-overlay --help` for the complete list of commands and options.

| Command | Description |
| --- | --- |
| `init <path>` | Initialize the current repository, pointing it at the overlay directory. |
| `sync` | Synchronize the repository and the overlay directory. |
| `add <pattern>...` | Add patterns to the private ignore list and manage matching files. |
| `remove <pattern>...` | Remove patterns from the private ignore list. |
| `file add <path>...` | Add individual files via their exact repository-relative path (root-anchored pattern). |
| `file remove <path>...` | Remove individual files via their exact repository-relative path. |
| `info [--json]` | Show the active exclude patterns and the tracked (managed) files. |
| `ignore add <pattern>...` | Add patterns to the overlay ignore list. |
| `ignore remove <pattern>...` | Remove patterns from the overlay ignore list. |

Global options:

- `-v, --verbose` — increase logging verbosity.

## How It Works

The tool manages two pieces of state per repository:

1. **Private ignore file** (`.git/info/exclude`)

   This is Git's per-clone ignore list—unlike `.gitignore`, it is never
   tracked or shared. Git Overlay maintains a *managed region* of this file
   delimited by guard clauses:

   ```gitignore
   # >>> managed by git-overlay
   your-pattern
   # <<< managed by git-overlay
   ```

   Anything outside the guard clauses is left exactly as you wrote it.

2. **Configuration files**

   - `.git/overlay.yml` — repository-scoped config recording the overlay
     directory path and the currently managed files.
   - `.git-overlay.yml` — overlay-scoped config, stored in the overlay
     directory, holding the managed and ignored patterns. It lets you back up
     or synchronize the overlay configuration itself.

### Sync semantics

When `sync` runs, Git Overlay:

1. Walks the repository and finds files matching the active exclude patterns.
2. For each matched file not yet managed, moves it into the overlay directory
   and hard-links it back into the repository.
3. Removes overlay files that are recorded as managed but no longer match any
   active pattern.
4. Folds the overlay's managed and ignore patterns into the private exclude
   file, and links any overlay files missing from the repository back into it.

If a file exists in **both** the repository and the overlay with *different*
contents, Git Overlay deliberately leaves both in place and emits a warning,
rather than silently destroying either copy.

## Requirements

- Rust (for building from source)
- `git` available on the `PATH`
- An existing Git working tree (commands fail outside one)
