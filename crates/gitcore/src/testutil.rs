use crate::exec::hide_window;
use crate::runner::RepoLocation;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn linear_history(location: &RepoLocation, count: usize) {
    let init = location.run(&["init", "-q", "-b", "main"]).unwrap();
    assert!(init.success(), "{}", String::from_utf8_lossy(&init.stderr));

    let mut script = String::new();
    for number in 1..=count {
        let message = format!("commit {number}");
        script.push_str(&format!(
            "commit refs/heads/main\ncommitter Tester <t@example.com> {} +0000\ndata {}\n{message}\n",
            1_700_000_000 + number,
            message.len(),
        ));
    }

    let mut command = location.command(&["fast-import", "--quiet"]);
    hide_window(&mut command);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(script.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
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
