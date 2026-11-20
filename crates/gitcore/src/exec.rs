use crate::runner::{RepoLocation, Runner};
use std::fmt;
use std::io;
use std::os::windows::process::CommandExt;
use std::process::Command;

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

impl RepoLocation {
    pub fn command(&self, args: &[&str]) -> Command {
        let mut command = match &self.runner {
            Runner::Windows => {
                let mut command = Command::new("git.exe");
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
        assert_eq!(command.get_program(), OsStr::new("git.exe"));
        assert_eq!(args_of(&command), ["-C", r"C:\dev\repo", "status", "--short"]);
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
