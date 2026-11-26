use crate::exec::CommandError;
use crate::runner::RepoLocation;
use crate::status::{FileState, WorkingFile};

fn run_on_paths(
    location: &RepoLocation,
    command: &str,
    paths: &[&str],
) -> Result<(), CommandError> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut args = vec!["--literal-pathspecs", command, "--"];
    args.extend_from_slice(paths);
    location.run_checked(&args).map(drop)
}

pub fn stage(location: &RepoLocation, paths: &[&str]) -> Result<(), CommandError> {
    run_on_paths(location, "add", paths)
}

pub fn unstage(location: &RepoLocation, paths: &[&str]) -> Result<(), CommandError> {
    run_on_paths(location, "reset", paths)
}

pub fn discard(location: &RepoLocation, file: &WorkingFile) -> Result<(), CommandError> {
    if !file.can_discard() {
        return Err(CommandError::Failed(format!("cannot discard {}", file.path)));
    }
    let command: &[&str] = match file.state {
        FileState::Untracked => &["clean", "-f"],
        _ => &["restore", "--worktree"],
    };
    let mut args = vec!["--literal-pathspecs"];
    args.extend(command);
    args.extend(["--", file.path.as_str()]);
    location.run_checked(&args).map(drop)
}

pub fn stage_all(location: &RepoLocation) -> Result<(), CommandError> {
    location.run_checked(&["add", "-A"]).map(drop)
}

pub fn commit(location: &RepoLocation, message: &str) -> Result<(), CommandError> {
    run_commit(location, &["commit", "-m", message])
}

pub fn amend(location: &RepoLocation, message: &str) -> Result<(), CommandError> {
    run_commit(location, &["commit", "--amend", "-m", message])
}

pub fn last_message(location: &RepoLocation) -> Result<String, CommandError> {
    let output = location.run_checked(&["log", "-1", "--format=%B"])?;
    Ok(String::from_utf8_lossy(&output).trim_end().to_owned())
}

fn run_commit(location: &RepoLocation, args: &[&str]) -> Result<(), CommandError> {
    let output = location.run(args)?;
    if output.success() {
        return Ok(());
    }
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Err(CommandError::Failed(text.trim().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::{StatusEntry, WorkingFile};
    use crate::testutil::{TempRepo, WslScratch};

    fn staged(repo: &TempRepo) -> Vec<String> {
        let mut paths: Vec<String> = repo
            .status()
            .entries
            .into_iter()
            .filter_map(|entry| match entry {
                StatusEntry::Tracked { path, staged: Some(_), .. } => Some(path),
                _ => None,
            })
            .collect();
        paths.sort();
        paths
    }

    fn unstaged(repo: &TempRepo) -> Vec<String> {
        let mut paths: Vec<String> = repo
            .status()
            .entries
            .into_iter()
            .filter_map(|entry| match entry {
                StatusEntry::Tracked { path, unstaged: Some(_), .. } | StatusEntry::Untracked { path } => {
                    Some(path)
                }
                _ => None,
            })
            .collect();
        paths.sort();
        paths
    }

    fn head_subject(repo: &TempRepo) -> String {
        let output = repo.location.run_checked(&["log", "-1", "--format=%B"]).unwrap();
        String::from_utf8(output).unwrap().trim_end().to_owned()
    }

    fn commit_count(repo: &TempRepo) -> usize {
        repo.location
            .run(&["rev-list", "--count", "HEAD"])
            .ok()
            .filter(|output| output.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().parse().unwrap())
            .unwrap_or(0)
    }

    #[test]
    fn staging_moves_an_untracked_file_and_a_modified_file_to_the_staged_group() {
        let repo = TempRepo::new();
        repo.write("tracked.txt", "a\n");
        repo.commit("first");
        repo.write("tracked.txt", "b\n");
        repo.write("fresh.txt", "new\n");
        repo.write("other.txt", "other\n");

        stage(&repo.location, &["tracked.txt", "fresh.txt"]).unwrap();

        assert_eq!(staged(&repo), ["fresh.txt", "tracked.txt"]);
        assert_eq!(unstaged(&repo), ["other.txt"]);
    }

    #[test]
    fn staging_a_deleted_file_stages_the_deletion() {
        let repo = TempRepo::new();
        repo.write("gone.txt", "a\n");
        repo.commit("first");
        std::fs::remove_file(repo.dir.path().join("gone.txt")).unwrap();

        stage(&repo.location, &["gone.txt"]).unwrap();

        assert_eq!(staged(&repo), ["gone.txt"]);
        assert!(unstaged(&repo).is_empty());
    }

    #[test]
    fn staging_treats_wildcards_and_leading_dashes_as_plain_file_names() {
        let repo = TempRepo::new();
        repo.write("-dash.txt", "dash\n");
        repo.write("[x].txt", "bracket\n");
        repo.write("x.txt", "x\n");

        stage(&repo.location, &["-dash.txt", "[x].txt"]).unwrap();

        assert_eq!(staged(&repo), ["-dash.txt", "[x].txt"]);
        assert_eq!(unstaged(&repo), ["x.txt"]);
    }

    #[test]
    fn staging_with_no_paths_stages_nothing() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");

        stage(&repo.location, &[]).unwrap();

        assert!(staged(&repo).is_empty());
    }

    #[test]
    fn staging_a_missing_file_reports_the_git_error() {
        let repo = TempRepo::new();

        let error = stage(&repo.location, &["missing.txt"]).unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
    }

    #[test]
    fn unstaging_moves_a_file_back_to_the_unstaged_group() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.write("b.txt", "b\n");
        repo.commit("first");
        repo.write("a.txt", "a2\n");
        repo.write("b.txt", "b2\n");
        repo.git(&["add", "-A"]);

        unstage(&repo.location, &["a.txt"]).unwrap();

        assert_eq!(staged(&repo), ["b.txt"]);
        assert_eq!(unstaged(&repo), ["a.txt"]);
    }

    #[test]
    fn unstaging_works_before_the_first_commit() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.git(&["add", "a.txt"]);

        unstage(&repo.location, &["a.txt"]).unwrap();

        assert!(staged(&repo).is_empty());
        assert_eq!(unstaged(&repo), ["a.txt"]);
    }

    #[test]
    fn unstaging_both_paths_of_a_rename_leaves_nothing_staged() {
        let repo = TempRepo::new();
        repo.write("old.txt", "line 1\nline 2\nline 3\nline 4\nline 5\n");
        repo.commit("first");
        repo.git(&["mv", "old.txt", "new.txt"]);

        unstage(&repo.location, &["new.txt", "old.txt"]).unwrap();

        assert!(staged(&repo).is_empty());
        assert_eq!(unstaged(&repo), ["new.txt", "old.txt"]);
        assert!(repo.dir.path().join("new.txt").is_file());
    }

    #[test]
    fn unstaging_only_the_new_path_of_a_rename_keeps_the_deletion_staged() {
        let repo = TempRepo::new();
        repo.write("old.txt", "line 1\nline 2\nline 3\nline 4\nline 5\n");
        repo.commit("first");
        repo.git(&["mv", "old.txt", "new.txt"]);

        unstage(&repo.location, &["new.txt"]).unwrap();

        assert_eq!(staged(&repo), ["old.txt"]);
    }

    #[test]
    fn unstaging_with_no_paths_unstages_nothing() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.git(&["add", "a.txt"]);

        unstage(&repo.location, &[]).unwrap();

        assert_eq!(staged(&repo), ["a.txt"]);
    }

    #[test]
    fn stage_all_stages_modified_deleted_and_untracked_files() {
        let repo = TempRepo::new();
        repo.write("edit.txt", "a\n");
        repo.write("gone.txt", "a\n");
        repo.commit("first");
        repo.write("edit.txt", "b\n");
        std::fs::remove_file(repo.dir.path().join("gone.txt")).unwrap();
        repo.write("nested/fresh.txt", "new\n");

        stage_all(&repo.location).unwrap();

        assert_eq!(staged(&repo), ["edit.txt", "gone.txt", "nested/fresh.txt"]);
        assert!(unstaged(&repo).is_empty());
    }

    #[test]
    fn committing_creates_a_commit_from_the_staged_files_only() {
        let repo = TempRepo::new();
        repo.write("staged.txt", "a\n");
        repo.write("left.txt", "b\n");
        repo.git(&["add", "staged.txt"]);

        commit(&repo.location, "add staged file").unwrap();

        assert_eq!(head_subject(&repo), "add staged file");
        assert_eq!(commit_count(&repo), 1);
        assert!(staged(&repo).is_empty());
        assert_eq!(unstaged(&repo), ["left.txt"]);
    }

    #[test]
    fn committing_keeps_a_multi_line_message_with_quotes_and_non_ascii_text() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.git(&["add", "a.txt"]);
        let message = "fix \"quoted\" 'single' 修正\n\nsecond paragraph\n  indented line";

        commit(&repo.location, message).unwrap();

        assert_eq!(head_subject(&repo), message);
    }

    #[test]
    fn committing_with_nothing_staged_reports_the_git_message() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");

        let error = commit(&repo.location, "empty").unwrap_err();

        assert!(error.to_string().contains("nothing to commit"), "{error}");
        assert_eq!(commit_count(&repo), 1);
    }

    #[test]
    fn a_failing_pre_commit_hook_shows_its_output_and_keeps_the_staged_files() {
        let repo = TempRepo::new();
        let hooks = repo.dir.path().join("hooks");
        std::fs::create_dir_all(&hooks).unwrap();
        std::fs::write(
            hooks.join("pre-commit"),
            "#!/bin/sh\necho 'stdout from hook'\necho 'stderr from hook' >&2\nexit 1\n",
        )
        .unwrap();
        repo.git(&["config", "core.hooksPath", hooks.to_str().unwrap()]);
        repo.write("a.txt", "a\n");
        repo.git(&["add", "a.txt"]);

        let error = commit(&repo.location, "blocked").unwrap_err();

        let text = error.to_string();
        assert!(text.contains("stdout from hook"), "{text}");
        assert!(text.contains("stderr from hook"), "{text}");
        assert_eq!(commit_count(&repo), 0);
        assert_eq!(staged(&repo), ["a.txt"]);
    }

    fn files_in_head(repo: &TempRepo) -> Vec<String> {
        let output = repo.location.run_checked(&["ls-tree", "-r", "--name-only", "HEAD"]).unwrap();
        String::from_utf8(output).unwrap().lines().map(str::to_owned).collect()
    }

    #[test]
    fn amending_adds_the_staged_files_and_replaces_the_message() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        repo.write("b.txt", "b\n");
        repo.git(&["add", "b.txt"]);

        amend(&repo.location, "fixed message").unwrap();

        assert_eq!(head_subject(&repo), "fixed message");
        assert_eq!(commit_count(&repo), 1);
        assert_eq!(files_in_head(&repo), ["a.txt", "b.txt"]);
        assert!(staged(&repo).is_empty());
    }

    #[test]
    fn amending_with_nothing_staged_changes_only_the_message() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        repo.write("left.txt", "left\n");

        amend(&repo.location, "only the message").unwrap();

        assert_eq!(head_subject(&repo), "only the message");
        assert_eq!(commit_count(&repo), 1);
        assert_eq!(files_in_head(&repo), ["a.txt"]);
        assert_eq!(unstaged(&repo), ["left.txt"]);
    }

    #[test]
    fn the_last_message_keeps_every_line_of_the_message() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.git(&["add", "a.txt"]);
        let message = "subject 修正\n\nsecond paragraph\n  indented line";
        commit(&repo.location, message).unwrap();

        assert_eq!(last_message(&repo.location).unwrap(), message);
    }

    #[test]
    fn amending_a_repository_with_no_commit_reports_the_git_error() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.git(&["add", "a.txt"]);

        let error = amend(&repo.location, "nothing to amend").unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
        assert_eq!(commit_count(&repo), 0);
        assert_eq!(staged(&repo), ["a.txt"]);
        assert!(last_message(&repo.location).is_err());
    }

    #[test]
    fn a_failing_hook_blocks_an_amend_and_keeps_the_last_commit() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        let hooks = repo.dir.path().join("hooks");
        std::fs::create_dir_all(&hooks).unwrap();
        std::fs::write(hooks.join("pre-commit"), "#!/bin/sh\necho 'hook says no'\nexit 1\n").unwrap();
        repo.git(&["config", "core.hooksPath", hooks.to_str().unwrap()]);
        repo.write("b.txt", "b\n");
        repo.git(&["add", "b.txt"]);

        let error = amend(&repo.location, "blocked").unwrap_err();

        assert!(error.to_string().contains("hook says no"), "{error}");
        assert_eq!(head_subject(&repo), "first");
        assert_eq!(staged(&repo), ["b.txt"]);
    }

    fn unstaged_file(repo: &TempRepo, path: &str) -> WorkingFile {
        repo.status()
            .groups()
            .unstaged
            .into_iter()
            .find(|file| file.path == path)
            .unwrap()
    }

    fn content(repo: &TempRepo, path: &str) -> String {
        let text = std::fs::read_to_string(repo.dir.path().join(path)).unwrap();
        text.replace("\r\n", "\n")
    }

    #[test]
    fn discarding_a_modified_file_brings_back_the_content_of_the_index() {
        let repo = TempRepo::new();
        repo.write("a.txt", "one\n");
        repo.write("b.txt", "keep\n");
        repo.commit("first");
        repo.write("a.txt", "edited\n");
        repo.write("b.txt", "also edited\n");

        discard(&repo.location, &unstaged_file(&repo, "a.txt")).unwrap();

        assert_eq!(content(&repo, "a.txt"), "one\n");
        assert_eq!(content(&repo, "b.txt"), "also edited\n");
        assert_eq!(unstaged(&repo), ["b.txt"]);
    }

    #[test]
    fn discarding_a_deleted_file_brings_it_back() {
        let repo = TempRepo::new();
        repo.write("gone.txt", "text\n");
        repo.commit("first");
        std::fs::remove_file(repo.dir.path().join("gone.txt")).unwrap();

        discard(&repo.location, &unstaged_file(&repo, "gone.txt")).unwrap();

        assert_eq!(content(&repo, "gone.txt"), "text\n");
        assert!(unstaged(&repo).is_empty());
    }

    #[test]
    fn discarding_an_untracked_file_removes_only_that_file() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        repo.write("nested/fresh.txt", "new\n");
        repo.write("nested/other.txt", "other\n");

        discard(&repo.location, &unstaged_file(&repo, "nested/fresh.txt")).unwrap();

        assert!(!repo.dir.path().join("nested/fresh.txt").exists());
        assert_eq!(unstaged(&repo), ["nested/other.txt"]);
    }

    #[test]
    fn discarding_the_unstaged_part_keeps_the_staged_content() {
        let repo = TempRepo::new();
        repo.write("a.txt", "v1\n");
        repo.commit("first");
        repo.write("a.txt", "v2\n");
        repo.git(&["add", "a.txt"]);
        repo.write("a.txt", "v3\n");

        discard(&repo.location, &unstaged_file(&repo, "a.txt")).unwrap();

        assert_eq!(content(&repo, "a.txt"), "v2\n");
        assert_eq!(staged(&repo), ["a.txt"]);
        assert!(unstaged(&repo).is_empty());
    }

    #[test]
    fn discarding_treats_wildcards_and_leading_dashes_as_plain_file_names() {
        let repo = TempRepo::new();
        repo.write("-dash.txt", "dash\n");
        repo.commit("first");
        repo.write("-dash.txt", "edited\n");
        repo.write("[x].txt", "bracket\n");
        repo.write("x.txt", "x\n");

        discard(&repo.location, &unstaged_file(&repo, "-dash.txt")).unwrap();
        discard(&repo.location, &unstaged_file(&repo, "[x].txt")).unwrap();

        assert_eq!(content(&repo, "-dash.txt"), "dash\n");
        assert_eq!(unstaged(&repo), ["x.txt"]);
    }

    #[test]
    fn a_file_that_cannot_be_discarded_reports_an_error_and_changes_nothing() {
        let repo = TempRepo::new();
        repo.write("a.txt", "v1\n");
        repo.commit("first");
        repo.write("a.txt", "v2\n");
        repo.git(&["add", "a.txt"]);
        let staged_file = repo.status().groups().staged.into_iter().next().unwrap();

        let error = discard(&repo.location, &staged_file).unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
        assert_eq!(content(&repo, "a.txt"), "v2\n");
        assert_eq!(staged(&repo), ["a.txt"]);
    }

    #[test]
    fn stages_and_commits_in_a_wsl_repository() {
        let Some(scratch) = WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
        location.run_checked(&["config", "user.name", "Tester"]).unwrap();
        location.run_checked(&["config", "user.email", "t@example.com"]).unwrap();
        std::fs::write(std::path::Path::new(&scratch.unc()).join("a b.txt"), "a\n").unwrap();
        let message = "wsl \"quoted\" 修正\n\nsecond paragraph";

        stage(&location, &["a b.txt"]).unwrap();
        commit(&location, message).unwrap();

        let log = location.run_checked(&["log", "-1", "--format=%B"]).unwrap();
        assert_eq!(String::from_utf8(log).unwrap().trim_end(), message);
    }

    #[test]
    fn amends_the_last_commit_in_a_wsl_repository() {
        let Some(scratch) = WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
        location.run_checked(&["config", "user.name", "Tester"]).unwrap();
        location.run_checked(&["config", "user.email", "t@example.com"]).unwrap();
        let root = std::path::Path::new(&scratch.unc()).to_path_buf();
        std::fs::write(root.join("a.txt"), "a\n").unwrap();
        stage(&location, &["a.txt"]).unwrap();
        commit(&location, "first").unwrap();
        std::fs::write(root.join("b.txt"), "b\n").unwrap();
        stage(&location, &["b.txt"]).unwrap();

        assert_eq!(last_message(&location).unwrap(), "first");
        amend(&location, "wsl 修正\n\nbody").unwrap();

        assert_eq!(last_message(&location).unwrap(), "wsl 修正\n\nbody");
        let count = location.run_checked(&["rev-list", "--count", "HEAD"]).unwrap();
        assert_eq!(String::from_utf8(count).unwrap().trim(), "1");
    }

    #[test]
    fn discards_a_modified_and_an_untracked_file_in_a_wsl_repository() {
        let Some(scratch) = WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
        location.run_checked(&["config", "user.name", "Tester"]).unwrap();
        location.run_checked(&["config", "user.email", "t@example.com"]).unwrap();
        let root = std::path::Path::new(&scratch.unc()).to_path_buf();
        std::fs::write(root.join("a b.txt"), "one\n").unwrap();
        location.run_checked(&["add", "a b.txt"]).unwrap();
        location.run_checked(&["commit", "-q", "-m", "first"]).unwrap();
        std::fs::write(root.join("a b.txt"), "edited\n").unwrap();
        std::fs::write(root.join("new.txt"), "new\n").unwrap();

        let groups = crate::status::read_status(&location).unwrap().groups();
        for file in &groups.unstaged {
            discard(&location, file).unwrap();
        }

        assert_eq!(std::fs::read_to_string(root.join("a b.txt")).unwrap(), "one\n");
        assert!(!root.join("new.txt").exists());
    }
}
