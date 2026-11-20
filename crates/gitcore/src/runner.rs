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
}
