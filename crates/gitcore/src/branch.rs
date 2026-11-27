use crate::exec::CommandError;
use crate::log::Oid;
use crate::runner::RepoLocation;

pub fn checkout_local(location: &RepoLocation, branch: &str) -> Result<(), CommandError> {
    location.run_checked(&["switch", "--", branch]).map(drop)
}

pub fn checkout_remote(location: &RepoLocation, remote_branch: &str) -> Result<(), CommandError> {
    location.run_checked(&["switch", "--track", "--", remote_branch]).map(drop)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartPoint {
    Commit(Oid),
    Local(String),
    Remote(String),
}

impl StartPoint {
    fn revision(&self) -> String {
        match self {
            StartPoint::Commit(oid) => oid.as_str().to_owned(),
            StartPoint::Local(name) => format!("refs/heads/{name}"),
            StartPoint::Remote(name) => format!("refs/remotes/{name}"),
        }
    }
}

pub fn create(
    location: &RepoLocation,
    name: &str,
    start: Option<&StartPoint>,
) -> Result<(), CommandError> {
    let revision = start.map(StartPoint::revision);
    let mut args = vec!["switch", "-c", name];
    args.extend(revision.as_deref());
    location.run_checked(&args).map(drop)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeleteMode {
    IfMerged,
    Force,
}

impl DeleteMode {
    fn flag(self) -> &'static str {
        match self {
            DeleteMode::IfMerged => "-d",
            DeleteMode::Force => "-D",
        }
    }
}

pub fn delete(location: &RepoLocation, name: &str, mode: DeleteMode) -> Result<(), CommandError> {
    location.run_checked(&["branch", mode.flag(), "--", name]).map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refs::list_refs;
    use crate::testutil::TempRepo;

    fn current_branch(location: &RepoLocation) -> String {
        let refs = list_refs(location).unwrap();
        refs.into_iter().find(|label| label.is_head).unwrap().name
    }

    fn branch_names(location: &RepoLocation) -> Vec<String> {
        let mut names: Vec<String> = list_refs(location)
            .unwrap()
            .into_iter()
            .filter(|label| label.kind == crate::refs::RefKind::Branch)
            .map(|label| label.name)
            .collect();
        names.sort();
        names
    }

    fn repo_with_one_commit() -> TempRepo {
        let repo = TempRepo::new();
        repo.write("a.txt", "a\n");
        repo.commit("one");
        repo
    }

    #[test]
    fn creating_a_branch_starts_at_head_and_checks_it_out() {
        let repo = repo_with_one_commit();
        let head = repo.location.run_checked(&["rev-parse", "HEAD"]).unwrap();

        create(&repo.location, "topic", None).unwrap();

        assert_eq!(current_branch(&repo.location), "topic");
        assert_eq!(branch_names(&repo.location), ["main", "topic"]);
        let topic = repo.location.run_checked(&["rev-parse", "topic"]).unwrap();
        assert_eq!(head, topic);
    }

    fn head_oid(location: &RepoLocation, rev: &str) -> Oid {
        let out = location.run_checked(&["rev-parse", rev]).unwrap();
        Oid::parse(String::from_utf8_lossy(&out).trim()).unwrap()
    }

    fn repo_with_two_commits() -> TempRepo {
        let repo = repo_with_one_commit();
        repo.write("b.txt", "b\n");
        repo.commit("two");
        repo
    }

    #[test]
    fn creating_a_branch_at_a_commit_checks_it_out_there() {
        let repo = repo_with_two_commits();
        let first = head_oid(&repo.location, "HEAD~1");

        create(&repo.location, "topic", Some(&StartPoint::Commit(first.clone()))).unwrap();

        assert_eq!(current_branch(&repo.location), "topic");
        assert_eq!(head_oid(&repo.location, "topic"), first);
    }

    #[test]
    fn creating_a_branch_at_a_local_branch_starts_at_its_commit() {
        let repo = repo_with_one_commit();
        repo.git(&["branch", "other"]);
        repo.write("b.txt", "b\n");
        repo.commit("two");

        create(&repo.location, "topic", Some(&StartPoint::Local("other".to_owned()))).unwrap();

        assert_eq!(current_branch(&repo.location), "topic");
        assert_eq!(head_oid(&repo.location, "topic"), head_oid(&repo.location, "other"));
        assert_ne!(head_oid(&repo.location, "topic"), head_oid(&repo.location, "main"));
    }

    #[test]
    fn creating_a_branch_at_a_remote_branch_starts_at_its_commit() {
        let (_origin, local) = clone_with_remote_branch();

        create(&local.location, "topic", Some(&StartPoint::Remote("origin/feature".to_owned()))).unwrap();

        assert_eq!(current_branch(&local.location), "topic");
        assert_eq!(head_oid(&local.location, "topic"), head_oid(&local.location, "origin/feature"));
        let upstream = local.location.run_checked(&["rev-parse", "--abbrev-ref", "topic@{upstream}"]).unwrap();
        assert_eq!(String::from_utf8_lossy(&upstream).trim(), "origin/feature");
    }

    #[test]
    fn a_start_branch_named_like_an_option_is_read_as_a_branch() {
        let repo = repo_with_two_commits();
        let first = head_oid(&repo.location, "HEAD~1");
        repo.git(&["update-ref", "refs/heads/-Cmain", first.as_str()]);

        create(&repo.location, "topic", Some(&StartPoint::Local("-Cmain".to_owned()))).unwrap();

        assert_eq!(head_oid(&repo.location, "topic"), first);
        assert_eq!(current_branch(&repo.location), "topic");
    }

    #[test]
    fn a_missing_start_branch_reports_the_git_error_and_changes_nothing() {
        let repo = repo_with_one_commit();

        let error = create(&repo.location, "topic", Some(&StartPoint::Local("nope".to_owned()))).unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
        assert_eq!(branch_names(&repo.location), ["main"]);
        assert_eq!(current_branch(&repo.location), "main");
    }

    #[test]
    fn creating_a_branch_keeps_the_changes_of_the_work_tree() {
        let repo = repo_with_one_commit();
        repo.write("a.txt", "local edit\n");

        create(&repo.location, "topic", None).unwrap();

        let content = std::fs::read_to_string(repo.dir.path().join("a.txt")).unwrap();
        assert_eq!(content, "local edit\n");
    }

    #[test]
    fn creating_a_branch_that_exists_reports_the_git_error_and_keeps_head() {
        let repo = repo_with_one_commit();
        repo.git(&["branch", "topic"]);

        let error = create(&repo.location, "topic", None).unwrap_err();

        assert!(error.to_string().contains("already exists"), "{error}");
        assert_eq!(current_branch(&repo.location), "main");
    }

    #[test]
    fn a_name_that_git_rejects_creates_nothing() {
        let repo = repo_with_one_commit();

        for name in ["has space", "a..b", "-x", "-Cmain", ""] {
            let error = create(&repo.location, name, None).unwrap_err();
            assert!(matches!(error, CommandError::Failed(_)), "{name}: {error:?}");
        }

        assert_eq!(branch_names(&repo.location), ["main"]);
        assert_eq!(current_branch(&repo.location), "main");
    }

    #[test]
    fn deleting_a_merged_branch_removes_it() {
        let repo = repo_with_one_commit();
        repo.git(&["branch", "topic"]);

        delete(&repo.location, "topic", DeleteMode::IfMerged).unwrap();

        assert_eq!(branch_names(&repo.location), ["main"]);
    }

    #[test]
    fn deleting_an_unmerged_branch_reports_the_git_error_and_keeps_it() {
        let repo = repo_with_one_commit();
        repo.git(&["switch", "-q", "-c", "topic"]);
        repo.write("b.txt", "b\n");
        repo.commit("two");
        repo.git(&["switch", "-q", "main"]);

        let error = delete(&repo.location, "topic", DeleteMode::IfMerged).unwrap_err();

        assert!(error.to_string().contains("not fully merged"), "{error}");
        assert_eq!(branch_names(&repo.location), ["main", "topic"]);
    }

    #[test]
    fn a_forced_delete_removes_an_unmerged_branch() {
        let repo = repo_with_one_commit();
        repo.git(&["switch", "-q", "-c", "topic"]);
        repo.write("b.txt", "b\n");
        repo.commit("two");
        repo.git(&["switch", "-q", "main"]);

        delete(&repo.location, "topic", DeleteMode::Force).unwrap();

        assert_eq!(branch_names(&repo.location), ["main"]);
    }

    #[test]
    fn a_forced_delete_of_the_current_branch_reports_the_git_error_and_keeps_it() {
        let repo = repo_with_one_commit();

        let error = delete(&repo.location, "main", DeleteMode::Force).unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
        assert_eq!(branch_names(&repo.location), ["main"]);
    }

    #[test]
    fn deleting_the_current_branch_reports_the_git_error_and_keeps_it() {
        let repo = repo_with_one_commit();

        let error = delete(&repo.location, "main", DeleteMode::IfMerged).unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
        assert_eq!(branch_names(&repo.location), ["main"]);
    }

    #[test]
    fn deleting_a_missing_branch_reports_the_git_error() {
        let repo = repo_with_one_commit();

        let error = delete(&repo.location, "nope", DeleteMode::IfMerged).unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
    }

    #[test]
    fn a_branch_name_that_looks_like_an_option_is_deleted_as_a_name() {
        let repo = repo_with_one_commit();
        repo.git(&["update-ref", "refs/heads/-Cmain", "HEAD"]);

        delete(&repo.location, "-Cmain", DeleteMode::IfMerged).unwrap();

        assert_eq!(branch_names(&repo.location), ["main"]);
    }

    #[test]
    fn checking_out_a_local_branch_moves_head() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a");
        repo.commit("one");
        repo.git(&["branch", "topic"]);

        checkout_local(&repo.location, "topic").unwrap();

        assert_eq!(current_branch(&repo.location), "topic");
    }

    #[test]
    fn a_branch_name_that_looks_like_an_option_is_checked_out_as_a_name() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a");
        repo.commit("one");
        repo.git(&["update-ref", "refs/heads/-Cmain", "HEAD"]);
        let main_before = repo.location.run_checked(&["rev-parse", "main"]).unwrap();

        checkout_local(&repo.location, "-Cmain").unwrap();

        assert_eq!(current_branch(&repo.location), "-Cmain");
        let main_after = repo.location.run_checked(&["rev-parse", "main"]).unwrap();
        assert_eq!(main_before, main_after);
    }

    #[test]
    fn a_refused_checkout_reports_the_git_error_and_keeps_head_and_files() {
        let repo = TempRepo::new();
        repo.write("a.txt", "base\n");
        repo.commit("one");
        repo.git(&["switch", "-q", "-c", "topic"]);
        repo.write("a.txt", "topic\n");
        repo.commit("two");
        repo.git(&["switch", "-q", "main"]);
        repo.write("a.txt", "local edit\n");

        let error = checkout_local(&repo.location, "topic").unwrap_err();

        assert!(error.to_string().contains("would be overwritten"), "{error}");
        assert_eq!(current_branch(&repo.location), "main");
        let content = std::fs::read_to_string(repo.dir.path().join("a.txt")).unwrap();
        assert_eq!(content, "local edit\n");
    }

    #[test]
    fn a_missing_branch_reports_the_git_error() {
        let repo = TempRepo::new();
        repo.write("a.txt", "a");
        repo.commit("one");

        let error = checkout_local(&repo.location, "nope").unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
        assert_eq!(current_branch(&repo.location), "main");
    }

    fn clone_with_remote_branch() -> (TempRepo, TempRepo) {
        let origin = TempRepo::new();
        origin.write("a.txt", "a");
        origin.commit("one");
        let local = TempRepo::new();
        let url = origin.dir.path().to_str().unwrap();
        local.git(&["remote", "add", "origin", url]);
        local.git(&["pull", "-q", "origin", "main"]);
        origin.git(&["switch", "-q", "-c", "feature"]);
        origin.write("b.txt", "b");
        origin.commit("two");
        local.git(&["fetch", "-q", "origin"]);
        (origin, local)
    }

    #[test]
    fn checking_out_a_remote_branch_creates_a_tracking_local_branch() {
        let (_origin, local) = clone_with_remote_branch();

        checkout_remote(&local.location, "origin/feature").unwrap();

        assert_eq!(current_branch(&local.location), "feature");
        let upstream = local
            .location
            .run_checked(&["rev-parse", "--abbrev-ref", "feature@{upstream}"])
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&upstream).trim(), "origin/feature");
        assert!(local.dir.path().join("b.txt").is_file());
    }

    #[test]
    fn checking_out_a_remote_branch_whose_local_name_exists_reports_the_git_error() {
        let (_origin, local) = clone_with_remote_branch();
        local.git(&["branch", "feature"]);

        let error = checkout_remote(&local.location, "origin/feature").unwrap_err();

        assert!(error.to_string().contains("already exists"), "{error}");
        assert_eq!(current_branch(&local.location), "main");
    }

    #[test]
    fn checks_out_a_branch_inside_wsl() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        for args in [
            ["init", "-q", "-b", "main"].as_slice(),
            &["-c", "user.name=T", "-c", "user.email=t@example.com", "commit", "-q", "--allow-empty", "-m", "one"],
            &["branch", "topic"],
        ] {
            location.run_checked(args).unwrap();
        }

        checkout_local(&location, "topic").unwrap();

        assert_eq!(current_branch(&location), "topic");
    }

    #[test]
    fn creates_and_deletes_a_branch_inside_wsl() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        for args in [
            ["init", "-q", "-b", "main"].as_slice(),
            &["-c", "user.name=T", "-c", "user.email=t@example.com", "commit", "-q", "--allow-empty", "-m", "one"],
        ] {
            location.run_checked(args).unwrap();
        }

        create(&location, "topic", None).unwrap();
        assert_eq!(current_branch(&location), "topic");
        checkout_local(&location, "main").unwrap();
        delete(&location, "topic", DeleteMode::Force).unwrap();

        let first = head_oid(&location, "main");
        create(&location, "again", Some(&StartPoint::Commit(first.clone()))).unwrap();
        assert_eq!(current_branch(&location), "again");
        assert_eq!(head_oid(&location, "again"), first);
        checkout_local(&location, "main").unwrap();
        delete(&location, "again", DeleteMode::Force).unwrap();

        assert_eq!(branch_names(&location), ["main"]);
    }
}
