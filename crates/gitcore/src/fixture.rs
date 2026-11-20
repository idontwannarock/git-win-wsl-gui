use crate::exec::hide_window;
use crate::log::Oid;
use crate::runner::RepoLocation;
use std::io::Write;
use std::process::Stdio;

pub fn generate_history(
    location: &RepoLocation,
    count: usize,
    merge_every: usize,
) -> Result<(), String> {
    let init = location
        .run(&["init", "-q", "-b", "main"])
        .map_err(|error| error.to_string())?;
    if !init.success() {
        return Err(String::from_utf8_lossy(&init.stderr).into_owned());
    }

    let mut command = location.command(&["fast-import", "--quiet"]);
    hide_window(&mut command);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let mut stdin = child.stdin.take().expect("stdin is piped");
    let script = fast_import_script(count, merge_every);
    let writer = std::thread::spawn(move || stdin.write_all(script.as_bytes()));
    let output = child.wait_with_output().map_err(|error| error.to_string())?;
    writer
        .join()
        .expect("writer thread")
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

pub fn commit_all(location: &RepoLocation, message: &str) -> Result<Oid, String> {
    let identity = ["-c", "user.name=Tester", "-c", "user.email=t@example.com"];
    let mut commit = identity.to_vec();
    commit.extend(["commit", "-q", "--allow-empty", "-m", message]);
    for args in [&["add", "-A"][..], &commit[..]] {
        location
            .run_checked(args)
            .map_err(|error| error.to_string())?;
    }
    let head = location
        .run_checked(&["rev-parse", "HEAD"])
        .map_err(|error| error.to_string())?;
    Oid::parse(String::from_utf8_lossy(&head).trim()).map_err(|error| format!("{error:?}"))
}

fn fast_import_script(count: usize, merge_every: usize) -> String {
    let mut script = String::new();
    let mut main_tip = 0;
    for number in 1..=count {
        let message = format!("commit {number}");
        let is_side = merge_every > 0 && number > 2 && number % merge_every == 0;
        let is_merge = merge_every > 0 && number > 3 && (number - 1) % merge_every == 0;
        let branch = if is_side { "side" } else { "main" };
        script.push_str(&format!(
            "commit refs/heads/{branch}\nmark :{number}\ncommitter Tester <t@example.com> {} +0000\ndata {}\n{message}\n",
            1_700_000_000 + number,
            message.len(),
        ));
        if is_side {
            script.push_str(&format!("from :{main_tip}\n"));
        } else {
            if is_merge {
                script.push_str(&format!("merge :{}\n", number - 1));
            }
            main_tip = number;
        }
    }
    script
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::parse_log;

    fn location_of(dir: &tempfile::TempDir) -> RepoLocation {
        RepoLocation::parse(dir.path().to_str().unwrap()).unwrap()
    }

    fn history(location: &RepoLocation) -> Vec<crate::log::Commit> {
        let output = location
            .run(&["log", "--all", "--topo-order", "-z", crate::log::LOG_FORMAT])
            .unwrap();
        assert!(output.success(), "{}", String::from_utf8_lossy(&output.stderr));
        parse_log(&output.stdout).unwrap()
    }

    #[test]
    fn a_linear_history_has_one_parent_per_commit() {
        let dir = tempfile::tempdir().unwrap();
        let location = location_of(&dir);

        generate_history(&location, 30, 0).unwrap();

        let commits = history(&location);
        assert_eq!(commits.len(), 30);
        assert_eq!(commits[0].subject, "commit 30");
        assert!(commits.iter().all(|commit| commit.parents.len() <= 1));
    }

    #[test]
    fn a_branchy_history_has_the_requested_count_and_some_merges() {
        let dir = tempfile::tempdir().unwrap();
        let location = location_of(&dir);

        generate_history(&location, 100, 10).unwrap();

        let commits = history(&location);
        assert_eq!(commits.len(), 100);
        let merges = commits.iter().filter(|commit| commit.parents.len() == 2).count();
        assert!(merges >= 5, "only {merges} merges");
    }
}
