#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Runner {
    Windows,
    Wsl { distro: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoLocation {
    pub runner: Runner,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    NotAbsolute,
    MissingDistro,
}

impl RepoLocation {
    pub fn parse(input: &str) -> Result<Self, PathError> {
        if let Some(unc) = input.strip_prefix(r"\\") {
            return Self::parse_unc(input, unc);
        }
        if is_drive_path(input) {
            return Ok(Self::windows(input));
        }
        Err(PathError::NotAbsolute)
    }

    fn parse_unc(input: &str, unc: &str) -> Result<Self, PathError> {
        let mut segments = unc.split('\\');
        let host = segments.next().unwrap_or_default();
        if !is_wsl_host(host) {
            return Ok(Self::windows(input));
        }
        let distro = segments
            .next()
            .filter(|name| !name.is_empty())
            .ok_or(PathError::MissingDistro)?;
        let mut path = String::new();
        for segment in segments.filter(|segment| !segment.is_empty()) {
            path.push('/');
            path.push_str(segment);
        }
        if path.is_empty() {
            path.push('/');
        }
        Ok(Self {
            runner: Runner::Wsl {
                distro: distro.to_owned(),
            },
            path,
        })
    }

    fn windows(path: &str) -> Self {
        Self {
            runner: Runner::Windows,
            path: path.to_owned(),
        }
    }
}

fn is_wsl_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("wsl$") || host.eq_ignore_ascii_case("wsl.localhost")
}

fn is_drive_path(input: &str) -> bool {
    let bytes = input.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoKey(String);

impl RepoLocation {
    pub fn open_path(&self) -> String {
        match &self.runner {
            Runner::Windows => self.path.clone(),
            Runner::Wsl { distro } => {
                format!(r"\\wsl.localhost\{distro}{}", self.path.replace('/', "\\"))
            }
        }
    }

    pub fn key(&self) -> RepoKey {
        RepoKey(match &self.runner {
            Runner::Windows => {
                let path = self.path.replace('/', "\\");
                format!("windows:{}", path.trim_end_matches('\\').to_lowercase())
            }
            Runner::Wsl { distro } => {
                format!("wsl:{}:{}", distro.to_lowercase(), self.path.trim_end_matches('/'))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve_ok(input: &str) -> RepoLocation {
        RepoLocation::parse(input).expect("path should resolve")
    }

    #[test]
    fn drive_path_uses_windows_git() {
        let location = resolve_ok(r"C:\dev\repo");
        assert_eq!(location.runner, Runner::Windows);
        assert_eq!(location.path, r"C:\dev\repo");
    }

    #[test]
    fn wsl_dollar_path_uses_that_distro() {
        let location = resolve_ok(r"\\wsl$\Ubuntu\home\me\repo");
        assert_eq!(
            location.runner,
            Runner::Wsl {
                distro: "Ubuntu".into()
            }
        );
        assert_eq!(location.path, "/home/me/repo");
    }

    #[test]
    fn wsl_localhost_path_matches_the_dollar_form() {
        assert_eq!(
            resolve_ok(r"\\wsl.localhost\Ubuntu\home\me\repo"),
            resolve_ok(r"\\wsl$\Ubuntu\home\me\repo")
        );
    }

    #[test]
    fn wsl_host_is_case_insensitive_and_distro_keeps_its_case() {
        let location = resolve_ok(r"\\WSL$\Debian-12\srv\x");
        assert_eq!(
            location.runner,
            Runner::Wsl {
                distro: "Debian-12".into()
            }
        );
    }

    #[test]
    fn wsl_distro_root_maps_to_slash() {
        assert_eq!(resolve_ok(r"\\wsl$\Ubuntu").path, "/");
        assert_eq!(resolve_ok(r"\\wsl$\Ubuntu\").path, "/");
    }

    #[test]
    fn other_unc_share_uses_windows_git() {
        let location = resolve_ok(r"\\server\share\repo");
        assert_eq!(location.runner, Runner::Windows);
    }

    #[test]
    fn rejects_relative_and_empty_input() {
        assert_eq!(RepoLocation::parse(""), Err(PathError::NotAbsolute));
        assert_eq!(RepoLocation::parse(r"repo\src"), Err(PathError::NotAbsolute));
        assert_eq!(RepoLocation::parse("/home/me"), Err(PathError::NotAbsolute));
    }

    #[test]
    fn rejects_wsl_path_without_distro() {
        assert_eq!(RepoLocation::parse(r"\\wsl$"), Err(PathError::MissingDistro));
        assert_eq!(RepoLocation::parse(r"\\wsl$\"), Err(PathError::MissingDistro));
    }

    fn key_of(input: &str) -> RepoKey {
        resolve_ok(input).key()
    }

    #[test]
    fn windows_paths_ignore_case_slash_direction_and_a_trailing_separator() {
        let key = key_of(r"C:\code\repo");
        assert_eq!(key, key_of(r"c:\Code\REPO"));
        assert_eq!(key, key_of(r"C:\code\repo\"));
        assert_eq!(key, key_of("C:/code/repo"));
    }

    #[test]
    fn different_windows_paths_have_different_keys() {
        assert_ne!(key_of(r"C:\code\repo"), key_of(r"C:\code\repo2"));
        assert_ne!(key_of(r"C:\code\repo"), key_of(r"D:\code\repo"));
    }

    #[test]
    fn the_two_wsl_forms_have_the_same_key() {
        assert_eq!(
            key_of(r"\\wsl$\Ubuntu\home\me\repo"),
            key_of(r"\\wsl.localhost\Ubuntu\home\me\repo")
        );
    }

    #[test]
    fn a_wsl_key_ignores_distro_case_and_a_trailing_separator() {
        assert_eq!(
            key_of(r"\\wsl$\Ubuntu\home\me\repo"),
            key_of(r"\\wsl$\ubuntu\home\me\repo\")
        );
    }

    #[test]
    fn a_wsl_key_keeps_the_case_of_the_linux_path() {
        assert_ne!(
            key_of(r"\\wsl$\Ubuntu\home\me\Repo"),
            key_of(r"\\wsl$\Ubuntu\home\me\repo")
        );
    }

    #[test]
    fn wsl_distros_have_different_keys() {
        assert_ne!(
            key_of(r"\\wsl$\Ubuntu\home\me\repo"),
            key_of(r"\\wsl$\Debian\home\me\repo")
        );
    }

    #[test]
    fn a_windows_path_never_equals_a_wsl_path() {
        assert_ne!(key_of(r"C:\home\me\repo"), key_of(r"\\wsl$\Ubuntu\home\me\repo"));
    }

    #[test]
    fn a_windows_location_opens_with_its_own_path() {
        assert_eq!(resolve_ok("C:/code/repo").open_path(), "C:/code/repo");
    }

    #[test]
    fn a_wsl_location_opens_with_a_wsl_localhost_path() {
        assert_eq!(
            resolve_ok(r"\\wsl$\Ubuntu\home\me\repo").open_path(),
            r"\\wsl.localhost\Ubuntu\home\me\repo"
        );
    }

    #[test]
    fn the_open_path_of_a_location_parses_back_to_the_same_location() {
        for input in [r"C:\code\repo", "C:/code/repo", r"\\wsl$\Ubuntu\home\me\repo", r"\\wsl$\Ubuntu\"] {
            let location = resolve_ok(input);
            assert_eq!(resolve_ok(&location.open_path()), location, "for {input}");
        }
    }
}
