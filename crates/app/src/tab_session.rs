use gitcore::trace;
use std::path::PathBuf;

fn file_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(base).join("git-win-wsl-gui").join("tabs.txt"))
}

pub fn load() -> (Vec<String>, usize) {
    let Some(path) = file_path() else {
        return (Vec::new(), 0);
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let session = decode(&text);
            if session.0.is_empty() && !text.trim().is_empty() {
                trace::record(|| "tabs.txt has no usable tab list".to_owned());
            }
            session
        }
        Err(error) => {
            if error.kind() != std::io::ErrorKind::NotFound {
                trace::record(|| format!("cannot read tabs: {error}"));
            }
            (Vec::new(), 0)
        }
    }
}

pub fn save(paths: &[String], active: usize) {
    let Some(path) = file_path() else {
        return;
    };
    let written = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&path, encode(paths, active)));
    if let Err(error) = written {
        trace::record(|| format!("cannot save tabs: {error}"));
    }
}

pub fn encode(paths: &[String], active: usize) -> String {
    let mut text = format!("{active}\n");
    for path in paths {
        text.push_str(path);
        text.push('\n');
    }
    text
}

pub fn decode(text: &str) -> (Vec<String>, usize) {
    let mut lines = text.lines();
    let Some(Ok(active)) = lines.next().map(|line| line.trim().parse::<usize>()) else {
        return (Vec::new(), 0);
    };
    let paths: Vec<String> = lines
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect();
    let active = if active < paths.len() { active } else { 0 };
    (paths, active)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths_of(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|path| path.to_string()).collect()
    }

    #[test]
    fn encode_writes_the_active_index_then_one_path_per_line() {
        assert_eq!(
            encode(&paths_of(&[r"C:\a", r"\\wsl$\Ubuntu\home\me\b"]), 1),
            "1\nC:\\a\n\\\\wsl$\\Ubuntu\\home\\me\\b\n"
        );
    }

    #[test]
    fn decode_reads_back_what_encode_wrote() {
        let paths = paths_of(&[r"C:\a", r"\\wsl$\Ubuntu\home\me\b", r"D:\c d\e"]);
        assert_eq!(decode(&encode(&paths, 2)), (paths, 2));
    }

    #[test]
    fn an_empty_text_gives_no_tabs() {
        assert_eq!(decode(""), (Vec::new(), 0));
    }

    #[test]
    fn a_first_line_that_is_not_a_number_gives_no_tabs() {
        assert_eq!(decode("C:\\a\nC:\\b\n"), (Vec::new(), 0));
    }

    #[test]
    fn an_index_out_of_range_falls_back_to_the_first_tab() {
        assert_eq!(decode("5\nC:\\a\nC:\\b\n"), (paths_of(&[r"C:\a", r"C:\b"]), 0));
    }

    #[test]
    fn a_negative_index_gives_no_tabs() {
        assert_eq!(decode("-1\nC:\\a\n"), (Vec::new(), 0));
    }

    #[test]
    fn blank_lines_are_skipped_and_windows_line_endings_work() {
        assert_eq!(
            decode("1\r\nC:\\a\r\n\r\nC:\\b\r\n"),
            (paths_of(&[r"C:\a", r"C:\b"]), 1)
        );
    }

    #[test]
    fn an_index_with_no_paths_gives_no_tabs() {
        assert_eq!(decode("0\n"), (Vec::new(), 0));
    }

    #[test]
    fn encode_of_no_tabs_decodes_to_no_tabs() {
        assert_eq!(decode(&encode(&[], 0)), (Vec::new(), 0));
    }
}
