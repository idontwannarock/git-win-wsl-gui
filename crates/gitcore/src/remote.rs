use crate::exec::{CommandError, run_command};
use crate::runner::{RepoLocation, Runner};
use std::process::Command;

const NO_PROMPT: &str = "GIT_TERMINAL_PROMPT";

fn network_command(location: &RepoLocation, args: &[&str]) -> Command {
    let mut command = location.command(args);
    command.env(NO_PROMPT, "0");
    if matches!(location.runner, Runner::Wsl { .. }) {
        let shared = match std::env::var("WSLENV") {
            Ok(existing) if !existing.is_empty() => format!("{existing}:{NO_PROMPT}"),
            _ => NO_PROMPT.to_owned(),
        };
        command.env("WSLENV", shared);
    }
    command
}

fn run_network(location: &RepoLocation, args: &[&str]) -> Result<(), CommandError> {
    let output = run_command(network_command(location, args))?;
    if output.success() {
        Ok(())
    } else {
        Err(CommandError::Failed(error_text(&output.stderr)))
    }
}

fn error_text(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let cause: Vec<&str> = text.lines().filter(|line| !line.starts_with("hint:")).collect();
    let cause = cause.join("\n");
    if cause.trim().is_empty() { text.trim().to_owned() } else { cause.trim().to_owned() }
}

pub fn fetch(location: &RepoLocation) -> Result<(), CommandError> {
    run_network(location, &["fetch", "--all", "--prune"])
}

pub fn pull(location: &RepoLocation) -> Result<(), CommandError> {
    run_network(location, &["pull", "--ff-only"])
}

pub fn push(location: &RepoLocation) -> Result<(), CommandError> {
    let upstream = location.run(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])?;
    if upstream.success() {
        run_network(location, &["push"])
    } else {
        run_network(location, &["push", "--set-upstream", "origin", "HEAD"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempRepo;

    struct Pair {
        _bare: tempfile::TempDir,
        bare_path: String,
        mine: TempRepo,
        other: TempRepo,
    }

    fn pair() -> Pair {
        let bare = tempfile::tempdir().unwrap();
        let bare_path = bare.path().to_str().unwrap().to_owned();
        RepoLocation::parse(&bare_path)
            .unwrap()
            .run_checked(&["init", "--bare", "-q", "-b", "main"])
            .unwrap();
        let mine = TempRepo::new();
        mine.write("a.txt", "one");
        mine.commit("one");
        mine.git(&["remote", "add", "origin", &bare_path]);
        mine.git(&["push", "-q", "-u", "origin", "main"]);
        let other = TempRepo::new();
        other.git(&["remote", "add", "origin", &bare_path]);
        other.git(&["pull", "-q", "origin", "main"]);
        other.git(&["branch", "--set-upstream-to", "origin/main"]);
        Pair { _bare: bare, bare_path, mine, other }
    }

    fn rev(repo: &TempRepo, name: &str) -> String {
        let out = repo.location.run_checked(&["rev-parse", name]).unwrap();
        String::from_utf8_lossy(&out).trim().to_owned()
    }

    fn remote_branches(repo: &TempRepo) -> String {
        let out = repo.location.run_checked(&["branch", "-r"]).unwrap();
        String::from_utf8_lossy(&out).into_owned()
    }

    #[test]
    fn fetch_brings_a_new_remote_commit_and_leaves_the_files() {
        let pair = pair();
        pair.other.write("b.txt", "two");
        pair.other.commit("two");
        pair.other.git(&["push", "-q"]);

        fetch(&pair.mine.location).unwrap();

        assert_eq!(rev(&pair.mine, "origin/main"), rev(&pair.other, "HEAD"));
        assert_ne!(rev(&pair.mine, "main"), rev(&pair.other, "HEAD"));
        assert!(!pair.mine.dir.path().join("b.txt").exists());
    }

    #[test]
    fn fetch_removes_a_branch_deleted_on_the_remote() {
        let pair = pair();
        pair.other.git(&["push", "-q", "origin", "main:topic"]);
        fetch(&pair.mine.location).unwrap();
        assert!(remote_branches(&pair.mine).contains("origin/topic"));

        pair.other.git(&["push", "-q", "origin", "--delete", "topic"]);
        fetch(&pair.mine.location).unwrap();

        assert!(!remote_branches(&pair.mine).contains("origin/topic"));
    }

    #[test]
    fn fetch_from_a_missing_remote_reports_the_git_error() {
        let pair = pair();
        let missing = pair.mine.dir.path().join("nowhere");
        pair.mine.git(&["remote", "set-url", "origin", missing.to_str().unwrap()]);

        let error = fetch(&pair.mine.location).unwrap_err();

        assert!(matches!(&error, CommandError::Failed(text) if !text.is_empty()), "{error:?}");
    }

    #[test]
    fn fetch_in_a_repository_with_no_remote_succeeds() {
        let repo = TempRepo::new();
        assert!(fetch(&repo.location).is_ok());
    }

    #[test]
    fn a_remote_command_never_asks_at_a_terminal() {
        let windows = RepoLocation::parse(r"C:\dev\repo").unwrap();
        let command = network_command(&windows, &["fetch"]);
        let value = command
            .get_envs()
            .find(|(name, _)| *name == "GIT_TERMINAL_PROMPT")
            .and_then(|(_, value)| value);
        assert_eq!(value, Some(std::ffi::OsStr::new("0")));
    }

    #[test]
    fn a_wsl_command_passes_the_no_prompt_variable_through_wsl() {
        let wsl = RepoLocation::parse(r"\\wsl$\Ubuntu\home\me\repo").unwrap();
        let command = network_command(&wsl, &["fetch"]);
        let wslenv = command
            .get_envs()
            .find(|(name, _)| *name == "WSLENV")
            .and_then(|(_, value)| value)
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default();
        assert!(wslenv.split(':').any(|name| name == "GIT_TERMINAL_PROMPT"), "{wslenv}");
    }

    fn remote_rev(pair: &Pair, name: &str) -> String {
        let bare = RepoLocation::parse(&pair.bare_path).unwrap();
        let out = bare.run_checked(&["rev-parse", name]).unwrap();
        String::from_utf8_lossy(&out).trim().to_owned()
    }

    fn other_pushes_a_commit(pair: &Pair) {
        pair.other.write("b.txt", "two");
        pair.other.commit("two");
        pair.other.git(&["push", "-q"]);
    }

    fn failed_with_text(result: Result<(), CommandError>) -> bool {
        matches!(&result, Err(CommandError::Failed(text)) if !text.is_empty())
    }

    #[test]
    fn pull_fast_forwards_the_branch_and_the_files() {
        let pair = pair();
        other_pushes_a_commit(&pair);

        pull(&pair.mine.location).unwrap();

        assert_eq!(rev(&pair.mine, "main"), rev(&pair.other, "HEAD"));
        assert_eq!(std::fs::read_to_string(pair.mine.dir.path().join("b.txt")).unwrap(), "two");
    }

    #[test]
    fn pull_on_diverged_branches_reports_the_git_error_and_changes_nothing() {
        let pair = pair();
        other_pushes_a_commit(&pair);
        pair.mine.write("c.txt", "local");
        pair.mine.commit("local");
        let before = rev(&pair.mine, "HEAD");

        let result = pull(&pair.mine.location);

        let Err(CommandError::Failed(text)) = result else {
            panic!("pull should fail");
        };
        assert!(text.contains("fast-forward"), "{text}");
        assert!(!text.contains("hint:"), "{text}");
        assert_eq!(rev(&pair.mine, "HEAD"), before);
        assert!(!pair.mine.dir.path().join("b.txt").exists());
    }

    #[test]
    fn an_error_keeps_the_cause_and_drops_the_hints() {
        let stderr = b"hint: one\nhint:\nhint: two\nfatal: the cause\n";
        assert_eq!(error_text(stderr), "fatal: the cause");
        assert_eq!(error_text(b"hint: only a hint\n"), "hint: only a hint");
    }

    #[test]
    fn pull_when_up_to_date_succeeds() {
        let pair = pair();
        assert!(pull(&pair.mine.location).is_ok());
    }

    #[test]
    fn pull_on_a_branch_with_no_upstream_reports_the_git_error() {
        let pair = pair();
        pair.mine.git(&["switch", "-q", "-c", "solo"]);
        assert!(failed_with_text(pull(&pair.mine.location)));
    }

    #[test]
    fn push_sends_the_commit_to_the_upstream() {
        let pair = pair();
        pair.mine.write("b.txt", "two");
        pair.mine.commit("two");

        push(&pair.mine.location).unwrap();

        assert_eq!(remote_rev(&pair, "main"), rev(&pair.mine, "HEAD"));
    }

    #[test]
    fn push_of_a_branch_with_no_upstream_creates_it_and_sets_the_upstream() {
        let pair = pair();
        pair.mine.git(&["switch", "-q", "-c", "feature"]);
        pair.mine.write("b.txt", "two");
        pair.mine.commit("two");

        push(&pair.mine.location).unwrap();

        assert_eq!(remote_rev(&pair, "feature"), rev(&pair.mine, "HEAD"));
        assert_eq!(rev(&pair.mine, "feature@{u}"), rev(&pair.mine, "origin/feature"));
    }

    #[test]
    fn a_refused_push_reports_the_git_error_and_leaves_the_remote() {
        let pair = pair();
        other_pushes_a_commit(&pair);
        pair.mine.write("c.txt", "local");
        pair.mine.commit("local");

        let result = push(&pair.mine.location);

        assert!(failed_with_text(result));
        assert_eq!(remote_rev(&pair, "main"), rev(&pair.other, "HEAD"));
    }

    #[test]
    fn a_wsl_repository_pushes_fetches_and_pulls() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let root = RepoLocation::parse(&scratch.unc()).unwrap();
        root.run_checked(&["init", "--bare", "-q", "-b", "main", "remote.git"]).unwrap();
        root.run_checked(&["clone", "-q", "remote.git", "work"]).unwrap();
        root.run_checked(&["clone", "-q", "remote.git", "peer"]).unwrap();
        let work = RepoLocation::parse(&format!(r"{}\work", scratch.unc())).unwrap();
        let peer = RepoLocation::parse(&format!(r"{}\peer", scratch.unc())).unwrap();
        for repo in [&work, &peer] {
            repo.run_checked(&["config", "user.name", "Tester"]).unwrap();
            repo.run_checked(&["config", "user.email", "t@example.com"]).unwrap();
        }
        let write = |name: &str, repo: &str| {
            std::fs::write(std::path::Path::new(&scratch.unc()).join(repo).join(name), "x\n").unwrap();
        };
        write("a.txt", "work");
        work.run_checked(&["add", "-A"]).unwrap();
        work.run_checked(&["commit", "-q", "-m", "one"]).unwrap();

        push(&work).unwrap();
        peer.run_checked(&["pull", "-q", "origin", "main"]).unwrap();
        peer.run_checked(&["branch", "--set-upstream-to", "origin/main"]).unwrap();
        write("b.txt", "peer");
        peer.run_checked(&["add", "-A"]).unwrap();
        peer.run_checked(&["commit", "-q", "-m", "two"]).unwrap();
        push(&peer).unwrap();
        fetch(&work).unwrap();
        pull(&work).unwrap();

        let head = |repo: &RepoLocation| repo.run_checked(&["rev-parse", "HEAD"]).unwrap();
        assert_eq!(head(&work), head(&peer));
    }

    #[test]
    fn push_in_a_repository_with_no_origin_reports_the_git_error() {
        let repo = TempRepo::new();
        repo.write("a.txt", "one");
        repo.commit("one");
        assert!(failed_with_text(push(&repo.location)));
    }
}
