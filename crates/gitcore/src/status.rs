use crate::exec::CommandError;
use crate::log::Oid;
use crate::runner::RepoLocation;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    BothModified,
    BothAdded,
    BothDeleted,
    AddedByUs,
    AddedByThem,
    DeletedByUs,
    DeletedByThem,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusEntry {
    Tracked {
        path: String,
        renamed_from: Option<String>,
        staged: Option<ChangeKind>,
        unstaged: Option<ChangeKind>,
    },
    Untracked {
        path: String,
    },
    Conflicted {
        path: String,
        kind: ConflictKind,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Branch {
    pub head: Option<String>,
    pub oid: Option<Oid>,
    pub upstream: Option<String>,
    pub ahead_behind: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Status {
    pub branch: Branch,
    pub entries: Vec<StatusEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseStatusError {
    UnknownRecord(String),
    BadRecord(String),
}

#[derive(Debug)]
pub enum StatusError {
    Command(CommandError),
    Parse(ParseStatusError),
}

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(error) => write!(f, "{error}"),
            Self::Parse(_) => write!(f, "git status printed output that this app cannot read"),
        }
    }
}

impl std::error::Error for StatusError {}

impl From<CommandError> for StatusError {
    fn from(error: CommandError) -> Self {
        Self::Command(error)
    }
}

impl From<ParseStatusError> for StatusError {
    fn from(error: ParseStatusError) -> Self {
        Self::Parse(error)
    }
}

fn bad(line: &str) -> ParseStatusError {
    ParseStatusError::BadRecord(line.to_owned())
}

fn change_kind(letter: char) -> Option<Option<ChangeKind>> {
    match letter {
        '.' => Some(None),
        'M' => Some(Some(ChangeKind::Modified)),
        'A' => Some(Some(ChangeKind::Added)),
        'D' => Some(Some(ChangeKind::Deleted)),
        'R' => Some(Some(ChangeKind::Renamed)),
        'C' => Some(Some(ChangeKind::Copied)),
        'T' => Some(Some(ChangeKind::TypeChanged)),
        _ => None,
    }
}

fn conflict_kind(letters: &str) -> Option<ConflictKind> {
    match letters {
        "UU" => Some(ConflictKind::BothModified),
        "AA" => Some(ConflictKind::BothAdded),
        "DD" => Some(ConflictKind::BothDeleted),
        "AU" => Some(ConflictKind::AddedByUs),
        "UA" => Some(ConflictKind::AddedByThem),
        "DU" => Some(ConflictKind::DeletedByUs),
        "UD" => Some(ConflictKind::DeletedByThem),
        _ => None,
    }
}

fn tracked_entry(
    line: &str,
    letters: &str,
    path: &str,
    renamed_from: Option<String>,
) -> Result<StatusEntry, ParseStatusError> {
    let mut chars = letters.chars();
    let (Some(x), Some(y), None) = (chars.next(), chars.next(), chars.next()) else {
        return Err(bad(line));
    };
    let (Some(staged), Some(unstaged)) = (change_kind(x), change_kind(y)) else {
        return Err(bad(line));
    };
    if staged.is_none() && unstaged.is_none() {
        return Err(bad(line));
    }
    Ok(StatusEntry::Tracked {
        path: path.to_owned(),
        renamed_from,
        staged,
        unstaged,
    })
}

fn apply_header(line: &str, branch: &mut Branch) -> Result<(), ParseStatusError> {
    let header = line.strip_prefix("# ").ok_or_else(|| bad(line))?;
    if let Some(value) = header.strip_prefix("branch.oid ") {
        branch.oid = match value {
            "(initial)" => None,
            hash => Some(Oid::parse(hash).map_err(|_| bad(line))?),
        };
    } else if let Some(value) = header.strip_prefix("branch.head ") {
        branch.head = (value != "(detached)").then(|| value.to_owned());
    } else if let Some(value) = header.strip_prefix("branch.upstream ") {
        branch.upstream = Some(value.to_owned());
    } else if let Some(value) = header.strip_prefix("branch.ab ") {
        let counts = value
            .split_once(' ')
            .and_then(|(ahead, behind)| Some((ahead.strip_prefix('+')?, behind.strip_prefix('-')?)))
            .and_then(|(ahead, behind)| Some((ahead.parse().ok()?, behind.parse().ok()?)));
        branch.ahead_behind = Some(counts.ok_or_else(|| bad(line))?);
    }
    Ok(())
}

pub fn parse_status(output: &[u8]) -> Result<Status, ParseStatusError> {
    let text = String::from_utf8_lossy(output);
    let mut tokens = text.split_terminator('\0');
    let mut status = Status::default();
    while let Some(line) = tokens.next() {
        let entry = match line.chars().next() {
            Some('#') => {
                apply_header(line, &mut status.branch)?;
                continue;
            }
            Some('!') => continue,
            Some('?') => StatusEntry::Untracked {
                path: line.strip_prefix("? ").ok_or_else(|| bad(line))?.to_owned(),
            },
            Some('1') => {
                let fields: Vec<&str> = line.splitn(9, ' ').collect();
                let [_, letters, _, _, _, _, _, _, path] = fields[..] else {
                    return Err(bad(line));
                };
                tracked_entry(line, letters, path, None)?
            }
            Some('2') => {
                let fields: Vec<&str> = line.splitn(10, ' ').collect();
                let [_, letters, _, _, _, _, _, _, _, path] = fields[..] else {
                    return Err(bad(line));
                };
                let from = tokens.next().ok_or_else(|| bad(line))?;
                tracked_entry(line, letters, path, Some(from.to_owned()))?
            }
            Some('u') => {
                let fields: Vec<&str> = line.splitn(11, ' ').collect();
                let [_, letters, _, _, _, _, _, _, _, _, path] = fields[..] else {
                    return Err(bad(line));
                };
                StatusEntry::Conflicted {
                    path: path.to_owned(),
                    kind: conflict_kind(letters).ok_or_else(|| bad(line))?,
                }
            }
            _ => return Err(ParseStatusError::UnknownRecord(line.to_owned())),
        };
        status.entries.push(entry);
    }
    Ok(status)
}

pub fn read_status(location: &RepoLocation) -> Result<Status, StatusError> {
    let output = location.run_checked(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v2",
        "-z",
        "--branch",
        "--untracked-files=all",
    ])?;
    Ok(parse_status(&output)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempRepo as Repo;

    const OID: &str = "1111111111111111111111111111111111111111";
    const HASH: &str = "2222222222222222222222222222222222222222";

    fn records(lines: &[&str]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for line in lines {
            bytes.extend_from_slice(line.as_bytes());
            bytes.push(0);
        }
        bytes
    }

    fn parse(lines: &[&str]) -> Status {
        parse_status(&records(lines)).unwrap()
    }

    fn entries(lines: &[&str]) -> Vec<StatusEntry> {
        parse(lines).entries
    }

    fn ordinary(xy: &str, path: &str) -> String {
        format!("1 {xy} N... 100644 100644 100644 {HASH} {HASH} {path}")
    }

    fn tracked(path: &str, staged: Option<ChangeKind>, unstaged: Option<ChangeKind>) -> StatusEntry {
        StatusEntry::Tracked {
            path: path.to_owned(),
            renamed_from: None,
            staged,
            unstaged,
        }
    }

    #[test]
    fn reads_the_branch_with_its_upstream_and_counts() {
        let status = parse(&[
            &format!("# branch.oid {OID}"),
            "# branch.head main",
            "# branch.upstream origin/main",
            "# branch.ab +3 -1",
        ]);
        assert_eq!(
            status.branch,
            Branch {
                head: Some("main".to_owned()),
                oid: Some(Oid::parse(OID).unwrap()),
                upstream: Some("origin/main".to_owned()),
                ahead_behind: Some((3, 1)),
            }
        );
        assert!(status.entries.is_empty());
    }

    #[test]
    fn a_new_repository_has_no_commit_and_no_upstream() {
        let status = parse(&["# branch.oid (initial)", "# branch.head main"]);
        assert_eq!(status.branch.oid, None);
        assert_eq!(status.branch.head.as_deref(), Some("main"));
        assert_eq!(status.branch.upstream, None);
        assert_eq!(status.branch.ahead_behind, None);
    }

    #[test]
    fn a_detached_head_has_no_branch_name() {
        let status = parse(&[&format!("# branch.oid {OID}"), "# branch.head (detached)"]);
        assert_eq!(status.branch.head, None);
        assert!(status.branch.oid.is_some());
    }

    #[test]
    fn splits_the_two_status_letters_into_staged_and_unstaged() {
        let found = entries(&[
            &ordinary("M.", "staged.txt"),
            &ordinary(".M", "unstaged.txt"),
            &ordinary("MM", "both.txt"),
            &ordinary("A.", "added.txt"),
            &ordinary(".D", "deleted.txt"),
            &ordinary("T.", "type.txt"),
        ]);
        assert_eq!(
            found,
            [
                tracked("staged.txt", Some(ChangeKind::Modified), None),
                tracked("unstaged.txt", None, Some(ChangeKind::Modified)),
                tracked("both.txt", Some(ChangeKind::Modified), Some(ChangeKind::Modified)),
                tracked("added.txt", Some(ChangeKind::Added), None),
                tracked("deleted.txt", None, Some(ChangeKind::Deleted)),
                tracked("type.txt", Some(ChangeKind::TypeChanged), None),
            ]
        );
    }

    #[test]
    fn a_renamed_file_carries_its_old_path() {
        let found = entries(&[
            &format!("2 R. N... 100644 100644 100644 {HASH} {HASH} R100 new name.txt"),
            "old name.txt",
        ]);
        assert_eq!(
            found,
            [StatusEntry::Tracked {
                path: "new name.txt".to_owned(),
                renamed_from: Some("old name.txt".to_owned()),
                staged: Some(ChangeKind::Renamed),
                unstaged: None,
            }]
        );
    }

    #[test]
    fn untracked_files_keep_spaces_and_non_ascii_characters() {
        let found = entries(&["? my file.txt", "? 資料夾/檔案.txt"]);
        assert_eq!(
            found,
            [
                StatusEntry::Untracked { path: "my file.txt".to_owned() },
                StatusEntry::Untracked { path: "資料夾/檔案.txt".to_owned() },
            ]
        );
    }

    #[test]
    fn every_conflict_letter_pair_has_a_kind() {
        let pairs = [
            ("UU", ConflictKind::BothModified),
            ("AA", ConflictKind::BothAdded),
            ("DD", ConflictKind::BothDeleted),
            ("AU", ConflictKind::AddedByUs),
            ("UA", ConflictKind::AddedByThem),
            ("DU", ConflictKind::DeletedByUs),
            ("UD", ConflictKind::DeletedByThem),
        ];
        for (xy, kind) in pairs {
            let line = format!("u {xy} N... 100644 100644 100644 100644 {HASH} {HASH} {HASH} f.txt");
            assert_eq!(
                entries(&[&line]),
                [StatusEntry::Conflicted { path: "f.txt".to_owned(), kind }],
                "{xy}"
            );
        }
    }

    #[test]
    fn ignored_files_are_skipped() {
        assert!(entries(&["! target/"]).is_empty());
    }

    #[test]
    fn empty_output_is_a_status_with_nothing() {
        assert_eq!(parse_status(b"").unwrap(), Status::default());
    }

    #[test]
    fn rejects_a_record_type_it_does_not_know() {
        assert_eq!(
            parse_status(&records(&["9 what"])),
            Err(ParseStatusError::UnknownRecord("9 what".to_owned()))
        );
    }

    #[test]
    fn rejects_records_it_cannot_read() {
        for line in [
            ordinary("XX", "a.txt"),
            ordinary("..", "a.txt"),
            "1 M.".to_owned(),
            "# branch.ab +x -y".to_owned(),
            "# branch.oid nothex".to_owned(),
        ] {
            assert_eq!(
                parse_status(&records(&[&line])),
                Err(ParseStatusError::BadRecord(line.clone())),
                "{line}"
            );
        }
    }

    #[test]
    fn rejects_a_rename_without_its_old_path() {
        let line = format!("2 R. N... 100644 100644 100644 {HASH} {HASH} R100 new.txt");
        assert!(matches!(
            parse_status(&records(&[&line])),
            Err(ParseStatusError::BadRecord(_))
        ));
    }

    fn sorted(mut status: Status) -> Vec<StatusEntry> {
        status.entries.sort_by_key(|entry| match entry {
            StatusEntry::Tracked { path, .. }
            | StatusEntry::Untracked { path }
            | StatusEntry::Conflicted { path, .. } => path.clone(),
        });
        status.entries
    }

    const LINES: &str = "line 1\nline 2\nline 3\nline 4\nline 5\n";

    #[test]
    fn reads_every_kind_of_change_in_a_real_repository() {
        let repo = Repo::new();
        repo.write("keep.txt", "keep\n");
        repo.write("edit staged.txt", "a\n");
        repo.write("edit unstaged.txt", "a\n");
        repo.write("remove.txt", "a\n");
        repo.write("old name.txt", LINES);
        repo.commit("first");
        repo.write("edit staged.txt", "b\n");
        repo.git(&["add", "edit staged.txt"]);
        repo.write("edit unstaged.txt", "b\n");
        repo.git(&["rm", "-q", "remove.txt"]);
        repo.git(&["mv", "old name.txt", "新名字.txt"]);
        repo.write("added.txt", "new\n");
        repo.git(&["add", "added.txt"]);
        repo.write("nested/untracked.txt", "u\n");

        assert_eq!(
            sorted(repo.status()),
            [
                tracked("added.txt", Some(ChangeKind::Added), None),
                tracked("edit staged.txt", Some(ChangeKind::Modified), None),
                tracked("edit unstaged.txt", None, Some(ChangeKind::Modified)),
                StatusEntry::Untracked { path: "nested/untracked.txt".to_owned() },
                tracked("remove.txt", Some(ChangeKind::Deleted), None),
                StatusEntry::Tracked {
                    path: "新名字.txt".to_owned(),
                    renamed_from: Some("old name.txt".to_owned()),
                    staged: Some(ChangeKind::Renamed),
                    unstaged: None,
                },
            ]
        );
    }

    #[test]
    fn a_clean_repository_has_no_entries_and_names_its_branch() {
        let repo = Repo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");

        let status = repo.status();

        assert!(status.entries.is_empty());
        assert_eq!(status.branch.head.as_deref(), Some("main"));
        assert!(status.branch.oid.is_some());
    }

    #[test]
    fn a_repository_without_commits_lists_its_untracked_files() {
        let repo = Repo::new();
        repo.write("a.txt", "a\n");

        let status = repo.status();

        assert_eq!(status.branch.oid, None);
        assert_eq!(status.entries, [StatusEntry::Untracked { path: "a.txt".to_owned() }]);
    }

    #[test]
    fn a_detached_head_is_reported() {
        let repo = Repo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        repo.git(&["checkout", "-q", "--detach"]);

        assert_eq!(repo.status().branch.head, None);
    }

    #[test]
    fn a_merge_conflict_is_listed_as_conflicted() {
        let repo = Repo::new();
        repo.write("f.txt", "base\n");
        repo.commit("base");
        repo.git(&["checkout", "-q", "-b", "side"]);
        repo.write("f.txt", "side\n");
        repo.commit("side");
        repo.git(&["checkout", "-q", "main"]);
        repo.write("f.txt", "main\n");
        repo.commit("main");
        let merge = repo.location.run(&[
            "-c", "user.name=T", "-c", "user.email=t@example.com", "merge", "side",
        ]);
        assert!(!merge.unwrap().success());

        assert_eq!(
            repo.status().entries,
            [StatusEntry::Conflicted {
                path: "f.txt".to_owned(),
                kind: ConflictKind::BothModified
            }]
        );
    }

    #[test]
    fn counts_the_commits_ahead_of_the_upstream() {
        let remote = tempfile::tempdir().unwrap();
        RepoLocation::parse(remote.path().to_str().unwrap())
            .unwrap()
            .run_checked(&["init", "-q", "--bare", "-b", "main"])
            .unwrap();
        let repo = Repo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        repo.git(&["remote", "add", "origin", remote.path().to_str().unwrap()]);
        repo.git(&["push", "-q", "-u", "origin", "main"]);
        repo.write("a.txt", "b\n");
        repo.commit("second");

        let branch = repo.status().branch;

        assert_eq!(branch.upstream.as_deref(), Some("origin/main"));
        assert_eq!(branch.ahead_behind, Some((1, 0)));
    }

    #[test]
    fn reading_the_status_does_not_rewrite_the_index() {
        let repo = Repo::new();
        repo.write("a.txt", "a\n");
        repo.commit("first");
        std::thread::sleep(std::time::Duration::from_millis(50));
        repo.write("a.txt", "a\n");
        let index = repo.dir.path().join(".git").join("index");
        let before = std::fs::metadata(&index).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));

        repo.status();

        assert_eq!(std::fs::metadata(&index).unwrap().modified().unwrap(), before);
    }

    #[test]
    fn a_folder_that_is_not_a_repository_reports_the_git_error() {
        let dir = tempfile::tempdir().unwrap();
        let location = RepoLocation::parse(dir.path().to_str().unwrap()).unwrap();

        let error = read_status(&location).unwrap_err();

        assert!(matches!(error, StatusError::Command(_)), "{error:?}");
    }

    #[test]
    fn reads_the_status_of_a_wsl_repository() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        location.run_checked(&["init", "-q", "-b", "main"]).unwrap();
        std::fs::write(std::path::Path::new(&scratch.unc()).join("a.txt"), "a\n").unwrap();

        let status = read_status(&location).unwrap();

        assert_eq!(status.entries, [StatusEntry::Untracked { path: "a.txt".to_owned() }]);
    }
}
