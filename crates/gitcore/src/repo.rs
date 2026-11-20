use crate::exec::RunError;
use crate::log::Oid;
use crate::refs::{RefLabel, RefsError, by_commit, list_refs};
use crate::runner::{PathError, RepoLocation, Runner};
use crate::stream::LogStream;
use std::collections::HashMap;
use std::fmt;
use std::thread;

pub struct OpenedRepo {
    pub location: RepoLocation,
    pub refs: HashMap<Oid, Vec<RefLabel>>,
    pub stream: LogStream,
}

#[derive(Debug)]
pub enum OpenError {
    Path(PathError),
    Run(RunError),
    NotRepository(String),
    Refs(RefsError),
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(PathError::NotAbsolute) => write!(f, "the path is not absolute"),
            Self::Path(PathError::MissingDistro) => write!(f, "the WSL path names no distro"),
            Self::Run(error) => write!(f, "{error}"),
            Self::NotRepository(message) => write!(f, "{message}"),
            Self::Refs(error) => write!(f, "{error}"),
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

impl From<RefsError> for OpenError {
    fn from(error: RefsError) -> Self {
        Self::Refs(error)
    }
}

pub fn open_repository(path: &str, batch_size: usize) -> Result<OpenedRepo, OpenError> {
    let probe = RepoLocation::parse(path)?;
    let (location, refs, stream) = thread::scope(|scope| {
        let refs = scope.spawn(|| list_refs(&probe));
        let stream = scope.spawn(|| LogStream::start(&probe, batch_size));
        let location = locate_repository(path);
        (
            location,
            refs.join().expect("refs thread"),
            stream.join().expect("stream thread"),
        )
    });
    Ok(OpenedRepo {
        location: location?,
        refs: by_commit(refs?),
        stream: stream?,
    })
}

pub fn locate_repository(path: &str) -> Result<RepoLocation, OpenError> {
    let probe = RepoLocation::parse(path)?;
    let toplevel = probe.run(&["rev-parse", "--show-toplevel"])?;
    if !toplevel.success() {
        return Err(OpenError::NotRepository(
            String::from_utf8_lossy(&toplevel.stderr).trim().to_owned(),
        ));
    }
    let top_path = String::from_utf8_lossy(&toplevel.stdout).trim().to_owned();
    Ok(RepoLocation {
        path: match probe.runner {
            Runner::Windows => top_path.replace('/', "\\"),
            Runner::Wsl { .. } => top_path,
        },
        runner: probe.runner,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::generate_history;
    use std::path::Path;

    fn init(dir: &Path) {
        let location = RepoLocation::parse(dir.to_str().unwrap()).unwrap();
        let output = location.run(&["init", "-q", "-b", "main"]).unwrap();
        assert!(output.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }

    fn path_text(path: &Path) -> &str {
        path.to_str().unwrap()
    }

    #[test]
    fn locates_the_top_level_of_a_repository() {
        let dir = tempfile::tempdir().unwrap();
        init(dir.path());
        let location = locate_repository(path_text(dir.path())).unwrap();
        assert_eq!(location.runner, Runner::Windows);
        let again = locate_repository(&location.path).unwrap();
        assert_eq!(again, location);
    }

    #[test]
    fn a_subfolder_moves_to_the_top_level() {
        let dir = tempfile::tempdir().unwrap();
        init(dir.path());
        let top = locate_repository(path_text(dir.path())).unwrap();
        let sub = dir.path().join("nested").join("deeper");
        std::fs::create_dir_all(&sub).unwrap();
        assert_eq!(locate_repository(path_text(&sub)).unwrap(), top);
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let error = locate_repository(path_text(dir.path())).unwrap_err();
        assert!(matches!(error, OpenError::NotRepository(_)), "{error:?}");
        assert!(error.to_string().contains("not a git repository"));
    }

    #[test]
    fn a_repository_inside_wsl_gets_the_distro_runner() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let probe = RepoLocation::parse(&scratch.unc()).unwrap();
        let output = probe.run(&["init", "-q", "-b", "main"]).unwrap();
        assert!(output.success(), "{}", String::from_utf8_lossy(&output.stderr));

        let location = locate_repository(&scratch.unc()).unwrap();
        assert_eq!(location.runner, probe.runner);
        assert_eq!(location.path, scratch.linux_dir);
    }

    fn subjects(opened: &mut OpenedRepo) -> Vec<String> {
        std::iter::from_fn(|| opened.stream.next_batch())
            .flat_map(|batch| batch.unwrap())
            .map(|commit| commit.subject)
            .collect()
    }

    #[test]
    fn opening_gives_the_location_the_labels_and_the_history() {
        let dir = tempfile::tempdir().unwrap();
        generate_history(&RepoLocation::parse(path_text(dir.path())).unwrap(), 3, 0).unwrap();

        let mut opened = open_repository(path_text(dir.path()), 10).unwrap();

        assert_eq!(opened.location, locate_repository(path_text(dir.path())).unwrap());
        let labels: Vec<&str> = opened
            .refs
            .values()
            .flatten()
            .map(|label| label.name.as_str())
            .collect();
        assert_eq!(labels, ["main"]);
        assert_eq!(subjects(&mut opened), ["commit 3", "commit 2", "commit 1"]);
    }

    #[test]
    fn opening_a_subfolder_reads_the_whole_repository() {
        let dir = tempfile::tempdir().unwrap();
        generate_history(&RepoLocation::parse(path_text(dir.path())).unwrap(), 2, 0).unwrap();
        let sub = dir.path().join("nested");
        std::fs::create_dir_all(&sub).unwrap();

        let mut opened = open_repository(path_text(&sub), 10).unwrap();

        assert_eq!(opened.location, locate_repository(path_text(dir.path())).unwrap());
        assert_eq!(subjects(&mut opened), ["commit 2", "commit 1"]);
    }

    #[test]
    fn opening_a_folder_that_is_not_a_repository_names_the_git_error() {
        let dir = tempfile::tempdir().unwrap();

        let error = open_repository(path_text(dir.path()), 10).err().unwrap();

        assert!(matches!(error, OpenError::NotRepository(_)), "{error:?}");
        assert!(error.to_string().contains("not a git repository"));
    }

    #[test]
    fn opening_a_repository_inside_wsl_uses_the_distro_git() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        generate_history(&RepoLocation::parse(&scratch.unc()).unwrap(), 4, 0).unwrap();

        let mut opened = open_repository(&scratch.unc(), 10).unwrap();

        assert_eq!(opened.location.path, scratch.linux_dir);
        assert_eq!(subjects(&mut opened).len(), 4);
    }

    #[test]
    fn a_relative_path_is_rejected_before_git_runs() {
        let error = locate_repository("some\\relative").unwrap_err();
        assert!(matches!(error, OpenError::Path(PathError::NotAbsolute)));
    }
}
