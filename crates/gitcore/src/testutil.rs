use std::process::Command;

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
