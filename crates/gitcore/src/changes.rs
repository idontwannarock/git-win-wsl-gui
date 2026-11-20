use crate::exec::CommandError;
use crate::log::Oid;
use crate::runner::RepoLocation;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChange {
    Added { path: String },
    Modified { path: String },
    Deleted { path: String },
    TypeChanged { path: String },
    Renamed { from: String, to: String },
    Copied { from: String, to: String },
}

impl FileChange {
    pub fn path(&self) -> &str {
        match self {
            Self::Added { path }
            | Self::Modified { path }
            | Self::Deleted { path }
            | Self::TypeChanged { path } => path,
            Self::Renamed { to, .. } | Self::Copied { to, .. } => to,
        }
    }

    pub fn patch_paths(&self) -> Vec<&str> {
        match self {
            Self::Renamed { from, to } | Self::Copied { from, to } => vec![to, from],
            other => vec![other.path()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseChangesError {
    UnknownStatus(String),
    MissingPath(String),
}

#[derive(Debug)]
pub enum ChangesError {
    Command(CommandError),
    Parse(ParseChangesError),
}

impl fmt::Display for ChangesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(error) => write!(f, "{error}"),
            Self::Parse(_) => write!(f, "git show printed a file list that this app cannot read"),
        }
    }
}

impl std::error::Error for ChangesError {}

impl From<CommandError> for ChangesError {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

impl From<ParseChangesError> for ChangesError {
    fn from(error: ParseChangesError) -> Self {
        Self::Parse(error)
    }
}

pub fn parse_name_status(output: &[u8]) -> Result<Vec<FileChange>, ParseChangesError> {
    let text = String::from_utf8_lossy(output);
    let mut tokens = text.split_terminator('\0');
    let mut changes = Vec::new();
    while let Some(status) = tokens.next() {
        let mut path = || {
            tokens
                .next()
                .map(str::to_owned)
                .ok_or_else(|| ParseChangesError::MissingPath(status.to_owned()))
        };
        let change = match status.chars().next() {
            Some('A') => FileChange::Added { path: path()? },
            Some('M') => FileChange::Modified { path: path()? },
            Some('D') => FileChange::Deleted { path: path()? },
            Some('T') => FileChange::TypeChanged { path: path()? },
            Some('R') => FileChange::Renamed { from: path()?, to: path()? },
            Some('C') => FileChange::Copied { from: path()?, to: path()? },
            _ => return Err(ParseChangesError::UnknownStatus(status.to_owned())),
        };
        changes.push(change);
    }
    Ok(changes)
}

pub fn commit_changes(
    location: &RepoLocation,
    commit: &Oid,
) -> Result<Vec<FileChange>, ChangesError> {
    let output = location.run_checked(&[
        "show",
        "--format=",
        "--name-status",
        "-z",
        "-M",
        "-m",
        "--first-parent",
        commit.as_str(),
    ])?;
    Ok(parse_name_status(&output)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::commit_file_patch;
    use crate::fixture::commit_all;

    fn parse(tokens: &[&str]) -> Vec<FileChange> {
        let mut bytes = Vec::new();
        for token in tokens {
            bytes.extend_from_slice(token.as_bytes());
            bytes.push(0);
        }
        parse_name_status(&bytes).unwrap()
    }

    fn added(path: &str) -> FileChange {
        FileChange::Added { path: path.to_owned() }
    }

    fn renamed(from: &str, to: &str) -> FileChange {
        FileChange::Renamed { from: from.to_owned(), to: to.to_owned() }
    }

    #[test]
    fn parses_added_modified_deleted_and_type_changed_files() {
        let changes = parse(&["A", "new.txt", "M", "src/lib.rs", "D", "gone.txt", "T", "link"]);
        assert_eq!(
            changes,
            [
                added("new.txt"),
                FileChange::Modified { path: "src/lib.rs".to_owned() },
                FileChange::Deleted { path: "gone.txt".to_owned() },
                FileChange::TypeChanged { path: "link".to_owned() },
            ]
        );
    }

    #[test]
    fn a_rename_and_a_copy_carry_the_old_and_the_new_path() {
        let changes = parse(&["R100", "old.txt", "new.txt", "C75", "base.txt", "copy.txt"]);
        assert_eq!(
            changes,
            [
                renamed("old.txt", "new.txt"),
                FileChange::Copied { from: "base.txt".to_owned(), to: "copy.txt".to_owned() },
            ]
        );
    }

    #[test]
    fn paths_with_spaces_and_non_ascii_characters_stay_as_they_are() {
        let changes = parse(&["A", "my file.txt", "A", "檔案 一.txt"]);
        assert_eq!(changes, [added("my file.txt"), added("檔案 一.txt")]);
    }

    #[test]
    fn empty_output_means_no_changed_files() {
        assert!(parse_name_status(b"").unwrap().is_empty());
    }

    #[test]
    fn the_path_of_a_change_is_the_current_one() {
        assert_eq!(added("a").path(), "a");
        assert_eq!(renamed("old", "new").path(), "new");
        assert_eq!(FileChange::Deleted { path: "gone".to_owned() }.path(), "gone");
    }

    #[test]
    fn a_rename_asks_for_the_new_and_the_old_path_in_a_patch() {
        assert_eq!(renamed("old", "new").patch_paths(), ["new", "old"]);
        assert_eq!(added("a").patch_paths(), ["a"]);
    }

    #[test]
    fn rejects_a_status_it_does_not_know() {
        assert_eq!(
            parse_name_status(b"X\0file\0"),
            Err(ParseChangesError::UnknownStatus("X".to_owned()))
        );
    }

    #[test]
    fn rejects_a_status_without_its_path() {
        assert_eq!(
            parse_name_status(b"R100\0only-one\0"),
            Err(ParseChangesError::MissingPath("R100".to_owned()))
        );
        assert_eq!(
            parse_name_status(b"M\0"),
            Err(ParseChangesError::MissingPath("M".to_owned()))
        );
    }

    struct Repo {
        dir: tempfile::TempDir,
        location: RepoLocation,
    }

    impl Repo {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let location = RepoLocation::parse(dir.path().to_str().unwrap()).unwrap();
            location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
            Self { dir, location }
        }

        fn write(&self, name: &str, content: &str) {
            std::fs::write(self.dir.path().join(name), content).unwrap();
        }

        fn git(&self, args: &[&str]) {
            self.location.run_checked(args).unwrap();
        }

        fn commit(&self, message: &str) -> Oid {
            commit_all(&self.location, message).unwrap()
        }
    }

    const LINES: &str = "line 1\nline 2\nline 3\nline 4\nline 5\n";

    #[test]
    fn lists_every_kind_of_change_of_a_real_commit() {
        let repo = Repo::new();
        repo.write("keep.txt", "keep\n");
        repo.write("edit.txt", "before\n");
        repo.write("remove.txt", "remove\n");
        repo.write("old name.txt", LINES);
        repo.commit("first");
        repo.write("edit.txt", "after\n");
        repo.git(&["rm", "-q", "remove.txt"]);
        repo.git(&["mv", "old name.txt", "新名字.txt"]);
        repo.write("added.txt", "added\n");
        let second = repo.commit("second");

        let mut changes = commit_changes(&repo.location, &second).unwrap();
        changes.sort_by(|a, b| a.path().cmp(b.path()));

        assert_eq!(
            changes,
            [
                added("added.txt"),
                FileChange::Modified { path: "edit.txt".to_owned() },
                FileChange::Deleted { path: "remove.txt".to_owned() },
                renamed("old name.txt", "新名字.txt"),
            ]
        );
    }

    #[test]
    fn a_rename_is_found_even_when_the_user_turned_rename_detection_off() {
        let repo = Repo::new();
        repo.git(&["config", "diff.renames", "false"]);
        repo.write("old.txt", LINES);
        repo.commit("first");
        repo.git(&["mv", "old.txt", "new.txt"]);
        let second = repo.commit("rename");

        let changes = commit_changes(&repo.location, &second).unwrap();

        assert_eq!(changes, [renamed("old.txt", "new.txt")]);
    }

    #[test]
    fn the_first_commit_lists_all_its_files_as_added() {
        let repo = Repo::new();
        repo.write("a.txt", "a\n");
        repo.write("b.txt", "b\n");
        let first = repo.commit("first");

        let changes = commit_changes(&repo.location, &first).unwrap();

        assert_eq!(changes, [added("a.txt"), added("b.txt")]);
    }

    #[test]
    fn a_commit_without_file_changes_lists_nothing() {
        let repo = Repo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        let empty = repo.commit("empty");

        assert!(commit_changes(&repo.location, &empty).unwrap().is_empty());
    }

    #[test]
    fn a_merge_commit_lists_the_changes_against_its_first_parent() {
        let repo = Repo::new();
        repo.write("base.txt", "base\n");
        repo.commit("base");
        repo.git(&["checkout", "-q", "-b", "side"]);
        repo.write("side.txt", "side\n");
        repo.commit("side work");
        repo.git(&["checkout", "-q", "main"]);
        repo.write("main.txt", "main\n");
        repo.commit("main work");
        repo.git(&[
            "-c", "user.name=Tester", "-c", "user.email=t@example.com",
            "merge", "-q", "--no-ff", "side", "-m", "merge side",
        ]);
        let head = repo.location.run_checked(&["rev-parse", "HEAD"]).unwrap();
        let merge = Oid::parse(String::from_utf8_lossy(&head).trim()).unwrap();

        let changes = commit_changes(&repo.location, &merge).unwrap();

        assert_eq!(changes, [added("side.txt")]);
    }

    #[test]
    fn the_paths_of_each_change_give_a_patch_for_that_file() {
        let repo = Repo::new();
        repo.write("old.txt", LINES);
        repo.commit("first");
        repo.git(&["mv", "old.txt", "new.txt"]);
        repo.write("new.txt", "line 1\nline 2\nline 3\nline 4\nline FIVE\n");
        let second = repo.commit("rename");

        let changes = commit_changes(&repo.location, &second).unwrap();
        let patch = commit_file_patch(&repo.location, &second, &changes[0].patch_paths()).unwrap();

        assert!(patch.contains("rename from old.txt"), "{patch}");
    }

    #[test]
    fn an_unknown_commit_reports_the_git_error() {
        let repo = Repo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        let unknown = Oid::parse(&"1".repeat(40)).unwrap();

        let error = commit_changes(&repo.location, &unknown).unwrap_err();

        assert!(matches!(error, ChangesError::Command(_)), "{error:?}");
    }

    #[test]
    fn lists_the_changes_of_a_commit_in_a_wsl_repository() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
        std::fs::write(std::path::Path::new(&scratch.unc()).join("a.txt"), "a\n").unwrap();
        let first = commit_all(&location, "first").unwrap();

        let changes = commit_changes(&location, &first).unwrap();

        assert_eq!(changes, [added("a.txt")]);
    }
}
