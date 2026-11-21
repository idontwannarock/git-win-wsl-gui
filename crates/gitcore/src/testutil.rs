use crate::fixture::commit_all;
use crate::runner::RepoLocation;
use crate::status::{Status, read_status};
use std::process::Command;

pub struct TempRepo {
    pub dir: tempfile::TempDir,
    pub location: RepoLocation,
}

impl TempRepo {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let location = RepoLocation::parse(dir.path().to_str().unwrap()).unwrap();
        location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
        for setting in [["user.name", "Tester"], ["user.email", "t@example.com"]] {
            location.run_checked(&["config", setting[0], setting[1]]).unwrap();
        }
        Self { dir, location }
    }

    pub fn write(&self, name: &str, content: &str) {
        let path = self.dir.path().join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    pub fn git(&self, args: &[&str]) {
        self.location.run_checked(args).unwrap();
    }

    pub fn commit(&self, message: &str) {
        commit_all(&self.location, message).unwrap();
    }

    pub fn status(&self) -> Status {
        read_status(&self.location).unwrap()
    }
}

pub struct WslScratch {
    pub distro: String,
    pub linux_dir: String,
}

impl WslScratch {
    pub fn create() -> Option<Self> {
        let distro = std::env::var("GITCORE_WSL_DISTRO").unwrap_or_else(|_| "Ubuntu".into());
        let made = Command::new("wsl.exe")
            .args(["-d", &distro, "--exec", "mktemp", "-d"])
            .output();
        match made {
            Ok(out) if out.status.success() => Some(Self {
                linux_dir: String::from_utf8_lossy(&out.stdout).trim().to_owned(),
                distro,
            }),
            other => {
                eprintln!("SKIPPED wsl test: no usable distro {distro:?}: {other:?}");
                None
            }
        }
    }

    pub fn unc(&self) -> String {
        format!(r"\\wsl$\{}{}", self.distro, self.linux_dir.replace('/', r"\"))
    }
}

impl Drop for WslScratch {
    fn drop(&mut self) {
        let _ = Command::new("wsl.exe")
            .args(["-d", &self.distro, "--exec", "rm", "-rf", &self.linux_dir])
            .output();
    }
}
