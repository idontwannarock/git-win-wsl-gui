use crate::exec::CommandError;
use crate::log::Oid;
use crate::runner::RepoLocation;

const SHOW_ARGS: [&str; 10] = [
    "-c",
    "core.quotePath=false",
    "show",
    "--format=",
    "--patch",
    "-M",
    "--no-color",
    "--no-textconv",
    "-m",
    "--first-parent",
];

pub fn commit_file_patch(
    location: &RepoLocation,
    commit: &Oid,
    paths: &[&str],
) -> Result<String, CommandError> {
    let mut args = SHOW_ARGS.to_vec();
    args.push(commit.as_str());
    args.push("--");
    args.extend_from_slice(paths);
    let patch = location.run_checked(&args)?;
    Ok(String::from_utf8_lossy(&patch).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::commit_all;
    use std::path::Path;

    struct Repo {
        dir: tempfile::TempDir,
        location: RepoLocation,
    }

    impl Repo {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let location = RepoLocation::parse(dir.path().to_str().unwrap()).unwrap();
            location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
            Self { dir, location }
        }

        fn write(&self, name: &str, content: &[u8]) {
            std::fs::write(self.dir.path().join(name), content).unwrap();
        }

        fn git(&self, args: &[&str]) {
            self.location.run_checked(args).unwrap();
        }

        fn commit(&self, message: &str) -> Oid {
            commit_all(&self.location, message).unwrap()
        }

        fn patch(&self, commit: &Oid, paths: &[&str]) -> String {
            commit_file_patch(&self.location, commit, paths).unwrap()
        }
    }

    fn has_escape(text: &str) -> bool {
        text.contains('\u{1b}')
    }

    #[test]
    fn a_modified_file_gives_a_plain_unified_patch() {
        let repo = Repo::new();
        repo.write("a.txt", b"one\ntwo\nthree\n");
        repo.commit("first");
        repo.write("a.txt", b"one\nTWO\nthree\n");
        let second = repo.commit("second");

        let patch = repo.patch(&second, &["a.txt"]);

        assert!(patch.starts_with("diff --git a/a.txt b/a.txt\n"), "{patch}");
        assert!(patch.contains("\n@@ "));
        assert!(patch.contains("\n-two\n"));
        assert!(patch.contains("\n+TWO\n"));
        assert!(!has_escape(&patch));
    }

    #[test]
    fn the_patch_holds_only_the_requested_file() {
        let repo = Repo::new();
        repo.write("a.txt", b"a\n");
        repo.write("b.txt", b"b\n");
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["b.txt"]);

        assert!(patch.contains("b/b.txt"));
        assert!(!patch.contains("a.txt"));
    }

    #[test]
    fn a_binary_file_gives_a_binary_marker_and_no_hunks() {
        let repo = Repo::new();
        repo.write("image.bin", &[0, 1, 2, 0, 255]);
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["image.bin"]);

        assert!(patch.contains("Binary files"), "{patch}");
        assert!(!patch.contains("\n@@ "));
    }

    #[test]
    fn a_renamed_file_names_both_paths_when_both_are_given() {
        let repo = Repo::new();
        repo.git(&["config", "diff.renames", "false"]);
        repo.write("old.txt", b"line 1\nline 2\nline 3\nline 4\nline 5\n");
        repo.commit("first");
        repo.git(&["mv", "old.txt", "new.txt"]);
        repo.write("new.txt", b"line 1\nline 2\nline 3\nline 4\nline FIVE\n");
        let second = repo.commit("rename");

        let patch = repo.patch(&second, &["new.txt", "old.txt"]);

        assert!(patch.contains("rename from old.txt\n"), "{patch}");
        assert!(patch.contains("rename to new.txt\n"));
        assert!(patch.contains("\n-line 5\n"));
    }

    #[test]
    fn a_missing_newline_at_the_end_is_marked() {
        let repo = Repo::new();
        repo.write("a.txt", b"one\ntwo");
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["a.txt"]);

        assert!(patch.contains("\\ No newline at end of file"), "{patch}");
    }

    #[test]
    fn the_first_commit_shows_its_files_as_new() {
        let repo = Repo::new();
        repo.write("a.txt", b"one\n");
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["a.txt"]);

        assert!(patch.contains("new file mode"), "{patch}");
        assert!(patch.contains("\n+one\n"));
    }

    #[test]
    fn a_merge_commit_shows_the_change_against_its_first_parent() {
        let repo = Repo::new();
        repo.write("base.txt", b"base\n");
        repo.commit("base");
        repo.git(&["checkout", "-q", "-b", "side"]);
        repo.write("side.txt", b"side\n");
        repo.commit("side work");
        repo.git(&["checkout", "-q", "main"]);
        repo.write("main.txt", b"main\n");
        repo.commit("main work");
        repo.git(&[
            "-c", "user.name=Tester", "-c", "user.email=t@example.com",
            "merge", "-q", "--no-ff", "side", "-m", "merge side",
        ]);
        let merge = Oid::parse(
            String::from_utf8_lossy(&repo.location.run_checked(&["rev-parse", "HEAD"]).unwrap())
                .trim(),
        )
        .unwrap();

        let patch = repo.patch(&merge, &["side.txt"]);

        assert!(patch.contains("new file mode"), "{patch}");
        assert!(patch.contains("\n+side\n"));
    }

    #[test]
    fn a_file_changed_on_both_sides_of_a_merge_gives_one_patch() {
        let repo = Repo::new();
        repo.write("shared.txt", b"1\n2\n3\n4\n5\n6\n7\n8\n9\n");
        repo.commit("base");
        repo.git(&["checkout", "-q", "-b", "side"]);
        repo.write("shared.txt", b"SIDE\n2\n3\n4\n5\n6\n7\n8\n9\n");
        repo.commit("side edit");
        repo.git(&["checkout", "-q", "main"]);
        repo.write("shared.txt", b"1\n2\n3\n4\n5\n6\n7\n8\nMAIN\n");
        repo.commit("main edit");
        repo.git(&[
            "-c", "user.name=Tester", "-c", "user.email=t@example.com",
            "merge", "-q", "--no-ff", "side", "-m", "merge side",
        ]);
        let head = repo.location.run_checked(&["rev-parse", "HEAD"]).unwrap();
        let merge = Oid::parse(String::from_utf8_lossy(&head).trim()).unwrap();

        let patch = repo.patch(&merge, &["shared.txt"]);

        assert_eq!(patch.matches("diff --git").count(), 1, "{patch}");
        assert!(patch.contains("\n+SIDE\n"));
    }

    #[test]
    fn a_path_with_non_ascii_characters_is_not_escaped() {
        let repo = Repo::new();
        repo.write("檔案.txt", "內容\n".as_bytes());
        let first = repo.commit("first");

        let patch = repo.patch(&first, &["檔案.txt"]);

        assert!(patch.contains("b/檔案.txt"), "{patch}");
        assert!(patch.contains("+內容"));
    }

    #[test]
    fn user_settings_for_color_and_text_conversion_do_not_change_the_patch() {
        let repo = Repo::new();
        repo.git(&["config", "color.ui", "always"]);
        repo.git(&["config", "diff.upper.textconv", "tr a-z A-Z"]);
        repo.write(".gitattributes", b"a.txt diff=upper\n");
        repo.write("a.txt", b"one\n");
        repo.commit("first");
        repo.write("a.txt", b"two\n");
        let second = repo.commit("second");

        let patch = repo.patch(&second, &["a.txt"]);

        assert!(patch.contains("\n-one\n"), "{patch}");
        assert!(patch.contains("\n+two\n"));
        assert!(!has_escape(&patch));
    }

    #[test]
    fn an_unknown_commit_reports_the_git_error() {
        let repo = Repo::new();
        repo.write("a.txt", b"one\n");
        repo.commit("first");
        let unknown = Oid::parse(&"1".repeat(40)).unwrap();

        let error = commit_file_patch(&repo.location, &unknown, &["a.txt"]).unwrap_err();

        assert!(matches!(error, CommandError::Failed(_)), "{error:?}");
    }

    #[test]
    fn gives_the_patch_of_a_file_in_a_wsl_repository() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
        let linux_file = Path::new(&scratch.unc()).join("a.txt");
        std::fs::write(&linux_file, b"one\n").unwrap();
        let first = commit_all(&location, "first").unwrap();

        let patch = commit_file_patch(&location, &first, &["a.txt"]).unwrap();

        assert!(patch.contains("+one"), "{patch}");
    }
}
