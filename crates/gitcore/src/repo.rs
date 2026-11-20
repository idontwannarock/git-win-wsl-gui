use crate::exec::RunError;
use crate::log::{Commit, LOG_FORMAT, ParseError, parse_log};
use crate::runner::{PathError, RepoLocation, Runner};
use std::fmt;

#[derive(Debug)]
pub struct OpenedRepo {
    pub location: RepoLocation,
    pub commits: Vec<Commit>,
}

#[derive(Debug)]
pub enum OpenError {
    Path(PathError),
    Run(RunError),
    NotRepository(String),
    Git(String),
    Parse(ParseError),
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(PathError::NotAbsolute) => write!(f, "the path is not absolute"),
            Self::Path(PathError::MissingDistro) => write!(f, "the WSL path names no distro"),
            Self::Run(error) => write!(f, "{error}"),
            Self::NotRepository(message) | Self::Git(message) => write!(f, "{message}"),
            Self::Parse(_) => write!(f, "git log printed output that this app cannot read"),
        }
    }
}

impl std::error::Error for OpenError {}

impl From<PathError> for OpenError {
    fn from(error: PathError) -> Self {
        Self::Path(error)
    }
}

impl From<RunError> for OpenError {
    fn from(error: RunError) -> Self {
        Self::Run(error)
    }
}

impl From<ParseError> for OpenError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

pub fn open_repository(path: &str, commit_count: usize) -> Result<OpenedRepo, OpenError> {
    let probe = RepoLocation::parse(path)?;
    let toplevel = probe.run(&["rev-parse", "--show-toplevel"])?;
    if !toplevel.success() {
        return Err(OpenError::NotRepository(stderr_text(&toplevel.stderr)));
    }
    let top_path = String::from_utf8_lossy(&toplevel.stdout).trim().to_owned();
    let location = RepoLocation {
        path: match probe.runner {
            Runner::Windows => top_path.replace('/', "\\"),
            Runner::Wsl { .. } => top_path,
        },
        runner: probe.runner,
    };

    let count = commit_count.to_string();
    let log = location.run(&["log", "--all", "--topo-order", "-z", LOG_FORMAT, "-n", &count])?;
    if !log.success() {
        return Err(OpenError::Git(stderr_text(&log.stderr)));
    }
    let commits = parse_log(&log.stdout)?;
    Ok(OpenedRepo { location, commits })
}

fn stderr_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn git(dir: &Path, args: &[&str]) {
        let location = RepoLocation::parse(dir.to_str().unwrap()).unwrap();
        let mut full = vec!["-c", "user.name=Tester", "-c", "user.email=t@example.com"];
        full.extend_from_slice(args);
        let output = location.run(&full).unwrap();
        assert!(
            output.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn repo_with_commits(count: usize) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        for number in 1..=count {
            git(
                dir.path(),
                &["commit", "-q", "--allow-empty", "-m", &format!("commit {number}")],
            );
        }
        dir
    }

    fn path_text(path: &Path) -> &str {
        path.to_str().unwrap()
    }

    #[test]
    fn opens_a_repository_and_lists_the_newest_commits_first() {
        let dir = repo_with_commits(3);
        let opened = open_repository(path_text(dir.path()), 100).unwrap();
        let subjects: Vec<&str> = opened.commits.iter().map(|c| c.subject.as_str()).collect();
        assert_eq!(subjects, ["commit 3", "commit 2", "commit 1"]);
    }

    #[test]
    fn stops_at_the_requested_commit_count() {
        let dir = repo_with_commits(5);
        let opened = open_repository(path_text(dir.path()), 2).unwrap();
        assert_eq!(opened.commits.len(), 2);
        assert_eq!(opened.commits[0].subject, "commit 5");
    }

    #[test]
    fn opening_a_subfolder_moves_to_the_top_level() {
        let dir = repo_with_commits(1);
        let sub = dir.path().join("nested").join("deeper");
        std::fs::create_dir_all(&sub).unwrap();
        let opened = open_repository(path_text(&sub), 10).unwrap();
        assert!(!opened.location.path.ends_with("deeper"));
        assert!(!opened.location.path.ends_with("nested"));
        let again = open_repository(&opened.location.path, 10).unwrap();
        assert_eq!(again.location, opened.location);
    }

    #[test]
    fn a_repository_without_commits_has_an_empty_history() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        let opened = open_repository(path_text(dir.path()), 10).unwrap();
        assert!(opened.commits.is_empty());
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let error = open_repository(path_text(dir.path()), 10).unwrap_err();
        assert!(matches!(error, OpenError::NotRepository(_)), "{error:?}");
        assert!(error.to_string().contains("not a git repository"));
    }

    #[test]
    fn opens_a_repository_inside_wsl_through_the_distro_git() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        let mut commit = vec!["-c", "user.name=T", "-c", "user.email=t@example.com"];
        commit.extend(["commit", "-q", "--allow-empty", "-m", "in wsl"]);
        for args in [vec!["init", "-q", "-b", "main"], commit] {
            let output = location.run(&args).unwrap();
            assert!(output.success(), "{}", String::from_utf8_lossy(&output.stderr));
        }

        let opened = open_repository(&scratch.unc(), 10).unwrap();
        assert_eq!(opened.location.runner, location.runner);
        assert_eq!(opened.location.path, scratch.linux_dir);
        assert_eq!(opened.commits.len(), 1);
        assert_eq!(opened.commits[0].subject, "in wsl");
    }

    #[test]
    fn a_relative_path_is_rejected_before_git_runs() {
        let error = open_repository("some\\relative", 10).unwrap_err();
        assert!(matches!(error, OpenError::Path(PathError::NotAbsolute)));
    }
}
