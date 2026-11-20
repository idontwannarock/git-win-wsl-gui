use crate::runner::{RepoLocation, Runner};
use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug)]
pub struct GitOutput {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl GitOutput {
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

#[derive(Debug)]
pub enum RunError {
    Spawn { program: String, source: io::Error },
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { program, source } => write!(f, "cannot start {program}: {source}"),
        }
    }
}

impl std::error::Error for RunError {}

#[derive(Debug)]
pub enum CommandError {
    Run(RunError),
    Failed(String),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Run(error) => write!(f, "{error}"),
            Self::Failed(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for CommandError {}

impl From<RunError> for CommandError {
    fn from(error: RunError) -> Self {
        Self::Run(error)
    }
}

const GIT_FLAVOR_FOLDERS: [&str; 3] = ["ucrt64", "mingw64", "clangarm64"];

fn find_git(path_dirs: &[PathBuf]) -> PathBuf {
    for dir in path_dirs {
        let candidate = dir.join("git.exe");
        if !candidate.is_file() {
            continue;
        }
        let real = (dir.file_name() == Some(OsStr::new("cmd")))
            .then(|| dir.parent())
            .flatten()
            .and_then(|root| {
                GIT_FLAVOR_FOLDERS
                    .iter()
                    .map(|flavor| root.join(flavor).join("bin").join("git.exe"))
                    .find(|real| real.is_file())
            });
        return real.unwrap_or(candidate);
    }
    PathBuf::from("git.exe")
}

fn windows_git() -> &'static Path {
    static GIT: OnceLock<PathBuf> = OnceLock::new();
    GIT.get_or_init(|| {
        let dirs: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|path| std::env::split_paths(&path).collect())
            .unwrap_or_default();
        find_git(&dirs)
    })
}

impl RepoLocation {
    pub fn command(&self, args: &[&str]) -> Command {
        let mut command = match &self.runner {
            Runner::Windows => {
                let mut command = Command::new(windows_git());
                command.arg("-C").arg(&self.path);
                command
            }
            Runner::Wsl { distro } => {
                let mut command = Command::new("wsl.exe");
                command.args(["-d", distro, "--cd", &self.path, "--exec", "git"]);
                command
            }
        };
        command.args(args);
        command
    }

    pub fn run(&self, args: &[&str]) -> Result<GitOutput, RunError> {
        run_command(self.command(args))
    }

    pub fn run_checked(&self, args: &[&str]) -> Result<Vec<u8>, CommandError> {
        let output = self.run(args)?;
        if output.success() {
            Ok(output.stdout)
        } else {
            Err(CommandError::Failed(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ))
        }
    }
}

pub(crate) fn hide_window(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

pub fn run_command(mut command: Command) -> Result<GitOutput, RunError> {
    hide_window(&mut command);
    let program = command.get_program().to_string_lossy().into_owned();
    let output = command
        .output()
        .map_err(|source| RunError::Spawn { program, source })?;
    Ok(GitOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    fn args_of(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    fn windows_location(dir: &std::path::Path) -> RepoLocation {
        RepoLocation::parse(dir.to_str().expect("utf-8 temp path")).expect("absolute temp path")
    }

    #[test]
    fn windows_location_builds_a_git_exe_command() {
        let location = RepoLocation::parse(r"C:\dev\repo").unwrap();
        let command = location.command(&["status", "--short"]);
        assert_eq!(
            std::path::Path::new(command.get_program()).file_name(),
            Some(OsStr::new("git.exe"))
        );
        assert_eq!(args_of(&command), ["-C", r"C:\dev\repo", "status", "--short"]);
    }

    fn touch(path: &std::path::Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"").unwrap();
    }

    #[test]
    fn skips_the_cmd_redirector_for_the_real_git() {
        let root = tempfile::tempdir().unwrap();
        let cmd = root.path().join("Git").join("cmd");
        let real = root.path().join("Git").join("ucrt64").join("bin").join("git.exe");
        touch(&cmd.join("git.exe"));
        touch(&real);

        assert_eq!(find_git(&[cmd]), real);
    }

    #[test]
    fn keeps_a_git_that_is_not_in_a_cmd_folder() {
        let root = tempfile::tempdir().unwrap();
        let tools = root.path().join("tools");
        touch(&tools.join("git.exe"));

        assert_eq!(find_git(std::slice::from_ref(&tools)), tools.join("git.exe"));
    }

    #[test]
    fn keeps_the_cmd_git_when_no_real_git_sits_next_to_it() {
        let root = tempfile::tempdir().unwrap();
        let cmd = root.path().join("Git").join("cmd");
        touch(&cmd.join("git.exe"));

        assert_eq!(find_git(std::slice::from_ref(&cmd)), cmd.join("git.exe"));
    }

    #[test]
    fn uses_the_bare_name_when_git_is_not_on_the_path() {
        let empty = tempfile::tempdir().unwrap();

        assert_eq!(find_git(&[empty.path().to_path_buf()]), PathBuf::from("git.exe"));
    }

    #[test]
    fn wsl_location_builds_a_wsl_exe_command() {
        let location = RepoLocation::parse(r"\\wsl$\Ubuntu\home\me\repo").unwrap();
        let command = location.command(&["log"]);
        assert_eq!(command.get_program(), OsStr::new("wsl.exe"));
        assert_eq!(
            args_of(&command),
            ["-d", "Ubuntu", "--cd", "/home/me/repo", "--exec", "git", "log"]
        );
    }

    #[test]
    fn windows_runner_runs_git_in_a_real_repository() {
        let dir = tempfile::tempdir().unwrap();
        let location = windows_location(dir.path());
        assert_eq!(location.runner, Runner::Windows);

        let init = location.run(&["init", "-q"]).unwrap();
        assert!(init.success());

        let inside = location.run(&["rev-parse", "--is-inside-work-tree"]).unwrap();
        assert!(inside.success());
        assert_eq!(inside.stdout, b"true\n");
    }

    #[test]
    fn failing_git_command_reports_exit_code_and_stderr() {
        let dir = tempfile::tempdir().unwrap();
        let location = windows_location(dir.path());

        let output = location.run(&["rev-parse", "--git-dir"]).unwrap();
        assert!(!output.success());
        assert_eq!(output.code, Some(128));
        assert!(String::from_utf8_lossy(&output.stderr).contains("not a git repository"));
    }

    #[test]
    fn run_checked_returns_stdout_on_success_and_stderr_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let location = windows_location(dir.path());

        let error = location.run_checked(&["rev-parse", "--git-dir"]).unwrap_err();
        assert!(matches!(&error, CommandError::Failed(message) if message.contains("not a git repository")));

        location.run_checked(&["init", "-q"]).unwrap();
        let inside = location.run_checked(&["rev-parse", "--is-inside-work-tree"]).unwrap();
        assert_eq!(inside, b"true\n");
    }

    #[test]
    fn missing_program_names_the_program() {
        let error = run_command(Command::new("gitcore-no-such-program")).unwrap_err();
        let RunError::Spawn { program, .. } = error;
        assert_eq!(program, "gitcore-no-such-program");
    }

    #[test]
    fn wsl_runner_runs_git_in_a_real_repository() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };

        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        assert_eq!(
            location.runner,
            Runner::Wsl {
                distro: scratch.distro.clone()
            }
        );
        assert_eq!(location.path, scratch.linux_dir);

        let init = location.run(&["init", "-q"]).unwrap();
        let inside = location.run(&["rev-parse", "--is-inside-work-tree"]).unwrap();

        assert!(init.success(), "{}", String::from_utf8_lossy(&init.stderr));
        assert_eq!(inside.stdout, b"true\n");
    }
}
