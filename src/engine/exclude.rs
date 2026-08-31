use std::path::{Path, PathBuf};

/// Opening guard clause. Everything between this line and [`CLOSE_GUARD`] is
/// managed by this class; anything else in the file is left untouched.
const OPEN_GUARD: &str = "# >>> managed by git-overlay";
/// Closing guard clause. Everything between [`OPEN_GUARD`] and this line is
/// managed by this class.
const CLOSE_GUARD: &str = "# <<< managed by git-overlay";

/// Manages the repo's private Git ignore file, i.e. `.git/info/exclude`.
///
/// Unlike `.gitignore`, files listed here are private to a single working
/// copy: they are never tracked or shared with other clones of the repository.
///
/// A list of patterns lives between two guard clauses in the file. Only that
/// region is managed; the rest of the file is left as the user wrote it.
pub struct ExcludeFile {
    /// The repository root this manager belongs to.
    root: PathBuf,
    /// Location of `.git/info/exclude` under `root`.
    path: PathBuf,
    /// The patterns between the guard clauses, loaded by [`Self::load`].
    patterns: Vec<String>,
    /// Everything up to (but not including) the opening guard, preserved
    /// verbatim across writes. Empty if there is no such content.
    head: String,
    /// Everything after (but not including) the closing guard, preserved
    /// verbatim across writes. Empty if there is no such content.
    tail: String,
}

impl ExcludeFile {
    /// Loads the patterns from the guard-clause region of `.git/info/exclude`
    /// and returns a manager loaded for them.
    pub fn load(root: &Path) -> Result<Self, String> {
        let root = root.to_path_buf();
        let path = root.join(".git").join("info").join("exclude");
        let mut file = Self {
            root,
            path,
            patterns: Vec::new(),
            head: String::new(),
            tail: String::new(),
        };
        file.read()?;
        Ok(file)
    }

    /// Reads the patterns between the guard clauses from disk, keeping the
    /// surrounding (non-managed) content in [`Self::head`] and [`Self::tail`]
    /// so later writes only touch the managed region.
    fn read(&mut self) -> Result<(), String> {
        let content = std::fs::read_to_string(&self.path)
            .map_err(|e| format!("failed to read {}: {e}", self.path.display()))?;
        let lines: Vec<&str> = content.lines().collect();

        let open = lines.iter().position(|l| l.trim() == OPEN_GUARD);
        let close = lines.iter().position(|l| l.trim() == CLOSE_GUARD);

        match (open, close) {
            (Some(o), Some(c)) if o < c => {
                // `head`/`tail` exclude the guard lines themselves; [`Self::write`]
                // re-emits them on save.
                self.head = join_lines(&lines[..o]);
                self.tail = join_lines(&lines[c + 1..]);
                self.patterns = lines[o + 1..c].iter().map(|s| uncomment(s)).collect();
            }
            _ => {
                // No (valid) guards yet: nothing managed, keep the whole file
                // as surrounding content.
                self.head = content.to_string();
                self.tail = String::new();
                self.patterns = Vec::new();
            }
        }

        Ok(())
    }

    /// Writes `lines` into the managed region, wrapping them in the guard
    /// clauses and leaving the surrounding content (`head`/`tail`) intact.
    fn write(&self, lines: &[String]) -> Result<(), String> {
        let mut out = String::new();
        out.push_str(&self.head);
        if !self.head.is_empty() && !self.head.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(OPEN_GUARD);
        out.push('\n');
        for line in lines {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(CLOSE_GUARD);
        out.push('\n');
        out.push_str(&self.tail);

        std::fs::write(&self.path, out)
            .map_err(|e| format!("failed to write {}: {e}", self.path.display()))
    }

    /// Writes each current pattern to the file commented out, effectively
    /// disabling the ignores while preserving the list. The surrounding
    /// content is left unchanged.
    pub fn freeze(&self) -> Result<(), String> {
        let commented: Vec<String> = self.patterns.iter().map(|p| format!("# {p}")).collect();
        self.write(&commented)
    }

    /// Writes each pattern to the file as-is (no comments), effectively
    /// restoring the active ignores. Only the managed region is written; the
    /// surrounding content is left unchanged. If the in-memory patterns were
    /// loaded from a frozen (commented) file they are already uncommented by
    /// [`Self::load`], so this writes the plain patterns.
    pub fn unfreeze(&self) -> Result<(), String> {
        self.write(&self.patterns)
    }

    /// Appends a pattern to the in-memory list. Does not touch the file until
    /// [`Self::save`] is called; duplicates are not filtered.
    pub fn add(&mut self, pattern: impl Into<String>) {
        self.patterns.push(pattern.into());
    }

    /// Removes all occurrences equal to `pattern` from the in-memory list. Does
    /// not touch the file until [`Self::save`] is called.
    pub fn remove(&mut self, pattern: &str) {
        self.patterns.retain(|p| p != pattern);
    }

    /// Clears the in-memory list of patterns. Does not touch the file until
    /// [`Self::save`] is called.
    pub fn clear(&mut self) {
        self.patterns.clear();
    }

    /// Writes the current in-memory patterns to the managed region (as plain,
    /// uncommented patterns), leaving the surrounding content intact.
    pub fn save(&self) -> Result<(), String> {
        self.write(&self.patterns)
    }

    /// Returns the repository root this file belongs to.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the patterns between the guard clauses.
    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }
}

/// Strips a leading comment marker and surrounding whitespace from a pattern
/// line so `# foo` and `#foo` are both stored as `foo`. Patterns that were not
/// commented are stored unchanged.
fn uncomment(line: &str) -> String {
    let line = line.trim();
    match line.strip_prefix('#') {
        Some(rest) => rest.trim().to_string(),
        None => line.to_string(),
    }
}

/// Joins `lines` into a single string, each followed by a newline. Returns an
/// empty string for an empty slice.
fn join_lines(lines: &[&str]) -> String {
    let mut s = String::new();
    for line in lines {
        s.push_str(line);
        s.push('\n');
    }
    s
}
