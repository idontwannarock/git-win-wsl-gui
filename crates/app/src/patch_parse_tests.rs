use gitcore::diff::{WorktreeDiff, commit_file_patch, worktree_file_patch};
use gitcore::fixture::commit_all;
use gitcore::log::Oid;
use gitcore::runner::RepoLocation;
use gpui_component::diff::{DiffFile, DiffFileStatus};

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

    fn write(&self, name: &str, content: &[u8]) {
        std::fs::write(self.dir.path().join(name), content).unwrap();
    }

    fn commit(&self, message: &str) -> Oid {
        commit_all(&self.location, message).unwrap()
    }

    fn parsed(&self, commit: &Oid, paths: &[&str]) -> DiffFile {
        let patch = commit_file_patch(&self.location, commit, paths).unwrap();
        let mut files = DiffFile::parse(&patch).unwrap_or_else(|error| panic!("{error:?}\n{patch}"));
        assert_eq!(files.len(), 1, "{patch}");
        files.remove(0)
    }
}

#[test]
fn a_modified_file_parses_with_its_added_and_removed_lines() {
    let repo = Repo::new();
    repo.write("a.txt", b"one\ntwo\nthree\n");
    repo.commit("first");
    repo.write("a.txt", b"one\nTWO\nthree\n");
    let second = repo.commit("second");

    let file = repo.parsed(&second, &["a.txt"]);

    assert_eq!(file.status(), DiffFileStatus::Modified);
    assert_eq!((file.additions(), file.deletions()), (1, 1));
    assert!(!file.is_binary());
}

#[test]
fn a_binary_file_parses_as_binary() {
    let repo = Repo::new();
    repo.write("image.bin", &[0, 1, 2, 0, 255]);
    let first = repo.commit("first");

    let file = repo.parsed(&first, &["image.bin"]);

    assert!(file.is_binary());
}

#[test]
fn a_renamed_file_parses_with_its_old_and_new_path() {
    let repo = Repo::new();
    repo.write("old.txt", b"line 1\nline 2\nline 3\nline 4\nline 5\n");
    repo.commit("first");
    repo.location.run_checked(&["mv", "old.txt", "new.txt"]).unwrap();
    repo.write("new.txt", b"line 1\nline 2\nline 3\nline 4\nline FIVE\n");
    let second = repo.commit("rename");

    let file = repo.parsed(&second, &["new.txt", "old.txt"]);

    assert_eq!(file.status(), DiffFileStatus::Renamed);
    assert_eq!(file.path().as_ref(), "new.txt");
    assert_eq!(file.original_path().map(|path| path.as_ref()), Some("old.txt"));
}

#[test]
fn a_file_without_a_final_newline_parses() {
    let repo = Repo::new();
    repo.write("a.txt", b"one\ntwo");
    let first = repo.commit("first");

    let file = repo.parsed(&first, &["a.txt"]);

    assert_eq!(file.status(), DiffFileStatus::Added);
    assert_eq!(file.additions(), 2);
}

#[test]
fn a_path_with_non_ascii_characters_parses() {
    let repo = Repo::new();
    repo.write("檔案.txt", "內容\n".as_bytes());
    let first = repo.commit("first");

    let file = repo.parsed(&first, &["檔案.txt"]);

    assert_eq!(file.path().as_ref(), "檔案.txt");
}

fn parse_worktree(repo: &Repo, kind: WorktreeDiff, paths: &[&str]) -> DiffFile {
    let patch = worktree_file_patch(&repo.location, kind, paths).unwrap();
    let mut files = DiffFile::parse(&patch).unwrap_or_else(|error| panic!("{error:?}\n{patch}"));
    assert_eq!(files.len(), 1, "{patch}");
    files.remove(0)
}

#[test]
fn an_untracked_file_parses_as_an_added_file() {
    let repo = Repo::new();
    repo.write("新 檔.txt", "one\ntwo\n".as_bytes());

    let file = parse_worktree(&repo, WorktreeDiff::Untracked, &["新 檔.txt"]);

    assert_eq!(file.status(), DiffFileStatus::Added);
    assert_eq!(file.path().as_ref(), "新 檔.txt");
    assert_eq!(file.additions(), 2);
}

#[test]
fn a_staged_rename_parses_with_its_old_and_new_path() {
    let repo = Repo::new();
    repo.write("old.txt", b"line 1\nline 2\nline 3\nline 4\nline 5\n");
    repo.commit("first");
    repo.location.run_checked(&["mv", "old.txt", "new.txt"]).unwrap();

    let file = parse_worktree(&repo, WorktreeDiff::Staged, &["new.txt", "old.txt"]);

    assert_eq!(file.status(), DiffFileStatus::Renamed);
    assert_eq!(file.original_path().map(|path| path.as_ref()), Some("old.txt"));
}
