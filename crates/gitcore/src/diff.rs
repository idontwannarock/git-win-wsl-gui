use crate::exec::CommandError;
use crate::log::Oid;
use crate::runner::RepoLocation;

const SHOW_ARGS: [&str; 11] = [
    "-c",
    "core.quotePath=false",
    "--literal-pathspecs",
    "show",
    "--format=",
    "--patch",
    "-M",
    "--no-color",
    "--no-textconv",
    "-m",
    "--first-parent",
];

pub fn commit_file_patch(
    location: &RepoLocation,
    commit: &Oid,
    paths: &[&str],
) -> Result<String, CommandError> {
    let mut args = SHOW_ARGS.to_vec();
    args.push(commit.as_str());
    args.push("--");
    args.extend_from_slice(paths);
    let patch = location.run_checked(&args)?;
    Ok(String::from_utf8_lossy(&patch).into_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorktreeDiff {
    Staged,
    Unstaged,
    Untracked,
}

pub fn worktree_file_patch(
    location: &RepoLocation,
    kind: WorktreeDiff,
    paths: &[&str],
) -> Result<String, CommandError> {
    let mut args = vec!["-c", "core.quotePath=false", "--literal-pathspecs", "diff"];
    args.extend(["-M", "--no-color", "--no-textconv", "--no-ext-diff"]);
    match kind {
        WorktreeDiff::Staged => args.push("--cached"),
        WorktreeDiff::Unstaged => {}
        WorktreeDiff::Untracked => args.extend(["--no-index", "--", "/dev/null"]),
    }
    if kind != WorktreeDiff::Untracked {
        args.push("--");
    }
    args.extend_from_slice(paths);
    let output = location.run(&args)?;
    let found_a_difference =
        kind == WorktreeDiff::Untracked && output.code == Some(1) && !output.stdout.is_empty();
    if output.success() || found_a_difference {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    Err(CommandError::Failed(
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::commit_all;
    use crate::testutil::TempRepo;
    use std::path::Path;

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

        fn git(&self, args: &[&str]) {
            self.location.run_checked(args).unwrap();
        }

        fn commit(&self, message: &str) -> Oid {
            commit_all(&self.location, message).unwrap()
        }

        fn patch(&self, commit: &Oid, paths: &[&str]) -> String {
            commit_file_patch(&self.location, commit, paths).unwrap()
        }
    }

    fn has_escape(text: &str) -> bool {
        text.contains('\u{1b}')
    }

    #[test]
    fn a_modified_file_gives_a_plain_unified_patch() {
        let repo = Repo::new();
        repo.write("a.txt", b"one\ntwo\nthree\n");
        repo.commit("first");
        repo.write("a.txt", b"one\nTWO\nthree\n");
        let second = repo.commit("second");

        let patch = repo.patch(&second, &["a.txt"]);

        assert!(patch.starts_with("diff --git a/a.txt b/a.txt\n"), "{patch}");
        assert!(patch.contains("\n@@ "));
        assert!(patch.contains("\n-two\n"));
        assert!(patch.contains("\n+TWO\n"));
        assert!(!has_escape(&patch));
    }

    #[test]
    fn the_patch_holds_only_the_requested_file() {
        let repo = Repo::new();
        repo.write("a.txt", b"a\n");
        repo.write("b.txt", b"b\n");
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["b.txt"]);

        assert!(patch.contains("b/b.txt"));
        assert!(!patch.contains("a.txt"));
    }

    #[test]
    fn a_binary_file_gives_a_binary_marker_and_no_hunks() {
        let repo = Repo::new();
        repo.write("image.bin", &[0, 1, 2, 0, 255]);
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["image.bin"]);

        assert!(patch.contains("Binary files"), "{patch}");
        assert!(!patch.contains("\n@@ "));
    }

    #[test]
    fn a_renamed_file_names_both_paths_when_both_are_given() {
        let repo = Repo::new();
        repo.git(&["config", "diff.renames", "false"]);
        repo.write("old.txt", b"line 1\nline 2\nline 3\nline 4\nline 5\n");
        repo.commit("first");
        repo.git(&["mv", "old.txt", "new.txt"]);
        repo.write("new.txt", b"line 1\nline 2\nline 3\nline 4\nline FIVE\n");
        let second = repo.commit("rename");

        let patch = repo.patch(&second, &["new.txt", "old.txt"]);

        assert!(patch.contains("rename from old.txt\n"), "{patch}");
        assert!(patch.contains("rename to new.txt\n"));
        assert!(patch.contains("\n-line 5\n"));
    }

    #[test]
    fn a_missing_newline_at_the_end_is_marked() {
        let repo = Repo::new();
        repo.write("a.txt", b"one\ntwo");
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["a.txt"]);

        assert!(patch.contains("\\ No newline at end of file"), "{patch}");
    }

    #[test]
    fn the_first_commit_shows_its_files_as_new() {
        let repo = Repo::new();
        repo.write("a.txt", b"one\n");
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["a.txt"]);

        assert!(patch.contains("new file mode"), "{patch}");
        assert!(patch.contains("\n+one\n"));
    }

    #[test]
    fn a_merge_commit_shows_the_change_against_its_first_parent() {
        let repo = Repo::new();
        repo.write("base.txt", b"base\n");
        repo.commit("base");
        repo.git(&["checkout", "-q", "-b", "side"]);
        repo.write("side.txt", b"side\n");
        repo.commit("side work");
        repo.git(&["checkout", "-q", "main"]);
        repo.write("main.txt", b"main\n");
        repo.commit("main work");
        repo.git(&[
            "-c", "user.name=Tester", "-c", "user.email=t@example.com",
            "merge", "-q", "--no-ff", "side", "-m", "merge side",
        ]);
        let merge = Oid::parse(
            String::from_utf8_lossy(&repo.location.run_checked(&["rev-parse", "HEAD"]).unwrap())
                .trim(),
        )
        .unwrap();

        let patch = repo.patch(&merge, &["side.txt"]);

        assert!(patch.contains("new file mode"), "{patch}");
        assert!(patch.contains("\n+side\n"));
    }

    #[test]
    fn a_file_changed_on_both_sides_of_a_merge_gives_one_patch() {
        let repo = Repo::new();
        repo.write("shared.txt", b"1\n2\n3\n4\n5\n6\n7\n8\n9\n");
        repo.commit("base");
        repo.git(&["checkout", "-q", "-b", "side"]);
        repo.write("shared.txt", b"SIDE\n2\n3\n4\n5\n6\n7\n8\n9\n");
        repo.commit("side edit");
        repo.git(&["checkout", "-q", "main"]);
        repo.write("shared.txt", b"1\n2\n3\n4\n5\n6\n7\n8\nMAIN\n");
        repo.commit("main edit");
        repo.git(&[
            "-c", "user.name=Tester", "-c", "user.email=t@example.com",
            "merge", "-q", "--no-ff", "side", "-m", "merge side",
        ]);
        let head = repo.location.run_checked(&["rev-parse", "HEAD"]).unwrap();
        let merge = Oid::parse(String::from_utf8_lossy(&head).trim()).unwrap();

        let patch = repo.patch(&merge, &["shared.txt"]);

        assert_eq!(patch.matches("diff --git").count(), 1, "{patch}");
        assert!(patch.contains("\n+SIDE\n"));
    }

    #[test]
    fn a_path_with_non_ascii_characters_is_not_escaped() {
        let repo = Repo::new();
        repo.write("檔案.txt", "內容\n".as_bytes());
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["檔案.txt"]);

        assert!(patch.contains("b/檔案.txt"), "{patch}");
        assert!(patch.contains("+內容"));
    }

    #[test]
    fn user_settings_for_color_and_text_conversion_do_not_change_the_patch() {
        let repo = Repo::new();
        repo.git(&["config", "color.ui", "always"]);
        repo.git(&["config", "diff.upper.textconv", "tr a-z A-Z"]);
        repo.write(".gitattributes", b"a.txt diff=upper\n");
        repo.write("a.txt", b"one\n");
        repo.commit("first");
        repo.write("a.txt", b"two\n");
        let second = repo.commit("second");

        let patch = repo.patch(&second, &["a.txt"]);

        assert!(patch.contains("\n-one\n"), "{patch}");
        assert!(patch.contains("\n+two\n"));
        assert!(!has_escape(&patch));
    }

    #[test]
    fn an_unknown_commit_reports_the_git_error() {
        let repo = Repo::new();
        repo.write("a.txt", b"one\n");
        repo.commit("first");
        let unknown = Oid::parse(&"1".repeat(40)).unwrap();

        let error = commit_file_patch(&repo.location, &unknown, &["a.txt"]).unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
    }

    #[test]
    fn a_commit_patch_treats_wildcard_characters_in_a_path_as_plain_characters() {
        let repo = TempRepo::new();
        repo.write("[x].txt", "one\n");
        repo.write("x.txt", "other\n");
        let first = commit_all(&repo.location, "first").unwrap();

        let patch = commit_file_patch(&repo.location, &first, &["[x].txt"]).unwrap();

        assert!(patch.contains("[x].txt"), "{patch}");
        assert!(!patch.contains("other"), "{patch}");
    }

    fn worktree_patch(repo: &TempRepo, kind: WorktreeDiff, paths: &[&str]) -> String {
        worktree_file_patch(&repo.location, kind, paths).unwrap()
    }

    #[test]
    fn an_unstaged_change_gives_a_plain_patch_of_work_tree_against_index() {
        let repo = TempRepo::new();
        repo.write("a.txt", "one\ntwo\nthree\n");
        repo.commit("first");
        repo.git(&["config", "color.ui", "always"]);
        repo.write("a.txt", "one\nTWO\nthree\n");

        let patch = worktree_patch(&repo, WorktreeDiff::Unstaged, &["a.txt"]);

        assert!(patch.starts_with("diff --git a/a.txt b/a.txt\n"), "{patch}");
        assert!(patch.contains("\n-two\n"));
        assert!(patch.contains("\n+TWO\n"));
        assert!(!has_escape(&patch));
    }

    #[test]
    fn an_unstaged_patch_ignores_what_is_already_staged() {
        let repo = TempRepo::new();
        repo.write("a.txt", "one\n");
        repo.commit("first");
        repo.write("a.txt", "two\n");
        repo.git(&["add", "a.txt"]);
        repo.write("a.txt", "three\n");

        let patch = worktree_patch(&repo, WorktreeDiff::Unstaged, &["a.txt"]);

        assert!(patch.contains("\n-two\n+three\n"), "{patch}");
        assert!(!patch.contains("one"), "{patch}");
    }

    #[test]
    fn a_staged_patch_compares_the_index_with_head() {
        let repo = TempRepo::new();
        repo.write("a.txt", "one\n");
        repo.commit("first");
        repo.write("a.txt", "two\n");
        repo.git(&["add", "a.txt"]);
        repo.write("a.txt", "three\n");

        let patch = worktree_patch(&repo, WorktreeDiff::Staged, &["a.txt"]);

        assert!(patch.contains("\n-one\n+two\n"), "{patch}");
        assert!(!patch.contains("three"), "{patch}");
    }

    #[test]
    fn a_staged_patch_works_before_the_first_commit() {
        let repo = TempRepo::new();
        repo.write("a.txt", "one\n");
        repo.git(&["add", "a.txt"]);

        let patch = worktree_patch(&repo, WorktreeDiff::Staged, &["a.txt"]);

        assert!(patch.contains("new file mode"), "{patch}");
        assert!(patch.contains("\n+one\n"), "{patch}");
    }

    #[test]
    fn a_staged_rename_needs_both_paths_to_show_as_a_rename() {
        let repo = TempRepo::new();
        repo.write("old.txt", "line 1\nline 2\nline 3\nline 4\nline 5\n");
        repo.commit("first");
        repo.git(&["config", "diff.renames", "false"]);
        repo.git(&["mv", "old.txt", "new.txt"]);

        let both = worktree_patch(&repo, WorktreeDiff::Staged, &["new.txt", "old.txt"]);
        let only_new = worktree_patch(&repo, WorktreeDiff::Staged, &["new.txt"]);

        assert!(both.contains("rename from old.txt\nrename to new.txt"), "{both}");
        assert!(only_new.contains("new file mode"), "{only_new}");
    }

    #[test]
    fn an_untracked_file_gives_a_patch_of_a_new_file() {
        let repo = TempRepo::new();
        repo.write("sub dir/新檔.txt", "hello\nworld\n");

        let patch = worktree_patch(&repo, WorktreeDiff::Untracked, &["sub dir/新檔.txt"]);

        assert!(patch.contains("new file mode"), "{patch}");
        assert!(patch.contains("+++ b/sub dir/新檔.txt"), "{patch}");
        assert!(patch.contains("\n+hello\n+world\n"), "{patch}");
    }

    #[test]
    fn an_untracked_file_gives_a_patch_when_git_warns_about_line_endings() {
        let repo = TempRepo::new();
        repo.git(&["config", "core.autocrlf", "true"]);
        repo.write("new.txt", "one\ntwo\n");

        let patch = worktree_patch(&repo, WorktreeDiff::Untracked, &["new.txt"]);

        assert!(patch.contains("\n+one\n+two"), "{patch}");
    }

    #[test]
    fn a_binary_untracked_file_gives_a_binary_marker() {
        let repo = TempRepo::new();
        std::fs::write(repo.dir.path().join("a.bin"), [0u8, 1, 2, 0, 255]).unwrap();

        let patch = worktree_patch(&repo, WorktreeDiff::Untracked, &["a.bin"]);

        assert!(patch.contains("Binary files"), "{patch}");
    }

    #[test]
    fn file_names_with_wildcard_characters_are_plain_names() {
        let repo = TempRepo::new();
        repo.write("[x].txt", "one\n");
        repo.write("x.txt", "other\n");
        repo.commit("first");
        repo.write("[x].txt", "two\n");
        repo.write("x.txt", "another\n");

        let patch = worktree_patch(&repo, WorktreeDiff::Unstaged, &["[x].txt"]);

        assert!(patch.contains("+two"), "{patch}");
        assert!(!patch.contains("another"), "{patch}");
    }

    #[test]
    fn an_external_diff_tool_in_the_config_does_not_replace_the_patch() {
        let repo = TempRepo::new();
        repo.write("a.txt", "one\n");
        repo.commit("first");
        repo.git(&["config", "diff.external", "false"]);
        repo.write("a.txt", "two\n");

        let patch = worktree_patch(&repo, WorktreeDiff::Unstaged, &["a.txt"]);

        assert!(patch.contains("\n+two\n"), "{patch}");
    }

    #[test]
    fn a_file_without_changes_gives_an_empty_patch() {
        let repo = TempRepo::new();
        repo.write("a.txt", "one\n");
        repo.commit("first");

        assert_eq!(worktree_patch(&repo, WorktreeDiff::Unstaged, &["a.txt"]), "");
        assert_eq!(worktree_patch(&repo, WorktreeDiff::Staged, &["a.txt"]), "");
    }

    #[test]
    fn a_missing_untracked_file_reports_the_git_error() {
        let repo = TempRepo::new();

        let error = worktree_file_patch(&repo.location, WorktreeDiff::Untracked, &["missing.txt"])
            .unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
    }

    #[test]
    fn gives_work_tree_patches_in_a_wsl_repository() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
        let root = Path::new(&scratch.unc()).to_path_buf();
        std::fs::write(root.join("a.txt"), b"one\n").unwrap();
        commit_all(&location, "first").unwrap();
        std::fs::write(root.join("a.txt"), b"two\n").unwrap();
        std::fs::write(root.join("new.txt"), b"fresh\n").unwrap();

        let unstaged = worktree_file_patch(&location, WorktreeDiff::Unstaged, &["a.txt"]).unwrap();
        let untracked = worktree_file_patch(&location, WorktreeDiff::Untracked, &["new.txt"]).unwrap();

        assert!(unstaged.contains("\n+two\n"), "{unstaged}");
        assert!(untracked.contains("\n+fresh\n"), "{untracked}");
    }

    #[test]
    fn gives_the_patch_of_a_file_in_a_wsl_repository() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
        let linux_file = Path::new(&scratch.unc()).join("a.txt");
        std::fs::write(&linux_file, b"one\n").unwrap();
        let first = commit_all(&location, "first").unwrap();

        let patch = commit_file_patch(&location, &first, &["a.txt"]).unwrap();

        assert!(patch.contains("+one"), "{patch}");
    }
}
