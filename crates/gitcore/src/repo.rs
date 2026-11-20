use crate::exec::RunError;
use crate::runner::{PathError, RepoLocation, Runner};
use std::fmt;

#[derive(Debug)]
pub enum OpenError {
    Path(PathError),
    Run(RunError),
    NotRepository(String),
}

impl fmt::Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(PathError::NotAbsolute) => write!(f, "the path is not absolute"),
            Self::Path(PathError::MissingDistro) => write!(f, "the WSL path names no distro"),
            Self::Run(error) => write!(f, "{error}"),
            Self::NotRepository(message) => write!(f, "{message}"),
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

    #[test]
    fn a_relative_path_is_rejected_before_git_runs() {
        let error = locate_repository("some\\relative").unwrap_err();
        assert!(matches!(error, OpenError::Path(PathError::NotAbsolute)));
    }
}
