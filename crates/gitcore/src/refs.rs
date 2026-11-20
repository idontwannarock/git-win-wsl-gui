use crate::exec::RunError;
use crate::log::{Oid, ParseError};
use crate::runner::RepoLocation;
use std::collections::HashMap;
use std::fmt;

const REFS_FORMAT: &str = "--format=%(objectname)%1f%(*objectname)%1f%(refname)%1f%(HEAD)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefKind {
    Branch,
    RemoteBranch,
    Tag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefLabel {
    pub name: String,
    pub kind: RefKind,
    pub target: Oid,
    pub is_head: bool,
}

#[derive(Debug)]
pub enum RefsError {
    Run(RunError),
    Git(String),
    Parse(ParseError),
}

impl fmt::Display for RefsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Run(error) => write!(f, "{error}"),
            Self::Git(message) => write!(f, "{message}"),
            Self::Parse(_) => write!(f, "git for-each-ref printed output that this app cannot read"),
        }
    }
}

impl std::error::Error for RefsError {}

impl From<RunError> for RefsError {
    fn from(error: RunError) -> Self {
        Self::Run(error)
    }
}

impl From<ParseError> for RefsError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}

fn classify(refname: &str) -> Option<(RefKind, &str)> {
    if let Some(name) = refname.strip_prefix("refs/heads/") {
        Some((RefKind::Branch, name))
    } else if let Some(name) = refname.strip_prefix("refs/remotes/") {
        (!name.ends_with("/HEAD")).then_some((RefKind::RemoteBranch, name))
    } else {
        refname
            .strip_prefix("refs/tags/")
            .map(|name| (RefKind::Tag, name))
    }
}

pub fn parse_refs(output: &[u8]) -> Result<Vec<RefLabel>, ParseError> {
    let mut labels = Vec::new();
    for line in String::from_utf8_lossy(output).lines().filter(|line| !line.is_empty()) {
        let fields: Vec<&str> = line.split('\u{1f}').collect();
        let [object, peeled, refname, head] = fields[..] else {
            return Err(ParseError::FieldCount {
                found: fields.len(),
            });
        };
        let target = Oid::parse(if peeled.is_empty() { object } else { peeled })?;
        if let Some((kind, name)) = classify(refname) {
            labels.push(RefLabel {
                name: name.to_owned(),
                kind,
                target,
                is_head: head == "*",
            });
        }
    }
    Ok(labels)
}

pub fn by_commit(refs: Vec<RefLabel>) -> HashMap<Oid, Vec<RefLabel>> {
    let mut grouped: HashMap<Oid, Vec<RefLabel>> = HashMap::new();
    for label in refs {
        grouped.entry(label.target.clone()).or_default().push(label);
    }
    for labels in grouped.values_mut() {
        labels.sort_by(|a, b| (!a.is_head, a.kind, &a.name).cmp(&(!b.is_head, b.kind, &b.name)));
    }
    grouped
}

pub fn list_refs(location: &RepoLocation) -> Result<Vec<RefLabel>, RefsError> {
    let output = location.run(&["for-each-ref", REFS_FORMAT])?;
    if !output.success() {
        return Err(RefsError::Git(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(parse_refs(&output.stdout)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: &str = "1111111111111111111111111111111111111111";
    const SECOND: &str = "2222222222222222222222222222222222222222";
    const TAG_OBJECT: &str = "3333333333333333333333333333333333333333";

    fn line(object: &str, peeled: &str, refname: &str, head: &str) -> String {
        [object, peeled, refname, head].join("\u{1f}")
    }

    fn parse(lines: &[String]) -> Vec<RefLabel> {
        parse_refs(lines.join("\n").as_bytes()).unwrap()
    }

    fn label(name: &str, kind: RefKind, target: &str, is_head: bool) -> RefLabel {
        RefLabel {
            name: name.to_owned(),
            kind,
            target: Oid::parse(target).unwrap(),
            is_head,
        }
    }

    #[test]
    fn parses_a_local_branch_and_marks_the_current_one() {
        let refs = parse(&[
            line(FIRST, "", "refs/heads/main", "*"),
            line(SECOND, "", "refs/heads/topic", " "),
        ]);
        assert_eq!(
            refs,
            [
                label("main", RefKind::Branch, FIRST, true),
                label("topic", RefKind::Branch, SECOND, false),
            ]
        );
    }

    #[test]
    fn keeps_the_remote_name_in_a_remote_branch_label() {
        let refs = parse(&[line(FIRST, "", "refs/remotes/origin/main", " ")]);
        assert_eq!(refs, [label("origin/main", RefKind::RemoteBranch, FIRST, false)]);
    }

    #[test]
    fn skips_the_symbolic_remote_head() {
        let refs = parse(&[line(FIRST, "", "refs/remotes/origin/HEAD", " ")]);
        assert!(refs.is_empty());
    }

    #[test]
    fn a_lightweight_tag_points_at_its_own_object() {
        let refs = parse(&[line(FIRST, "", "refs/tags/v1", " ")]);
        assert_eq!(refs, [label("v1", RefKind::Tag, FIRST, false)]);
    }

    #[test]
    fn an_annotated_tag_points_at_the_commit_it_wraps() {
        let refs = parse(&[line(TAG_OBJECT, FIRST, "refs/tags/v2", " ")]);
        assert_eq!(refs, [label("v2", RefKind::Tag, FIRST, false)]);
    }

    #[test]
    fn skips_refs_that_are_not_branches_or_tags() {
        let refs = parse(&[
            line(FIRST, "", "refs/stash", " "),
            line(FIRST, "", "refs/notes/commits", " "),
        ]);
        assert!(refs.is_empty());
    }

    #[test]
    fn empty_output_has_no_refs() {
        assert!(parse_refs(b"").unwrap().is_empty());
    }

    #[test]
    fn rejects_a_line_with_the_wrong_field_count() {
        assert_eq!(
            parse_refs(b"only\x1ftwo"),
            Err(ParseError::FieldCount { found: 2 })
        );
    }

    #[test]
    fn rejects_an_object_id_that_is_not_hex() {
        let bad = line("zzz", "", "refs/heads/main", " ");
        assert_eq!(
            parse_refs(bad.as_bytes()),
            Err(ParseError::BadId("zzz".to_owned()))
        );
    }

    #[test]
    fn groups_labels_by_commit_with_the_current_branch_first_then_kind_then_name() {
        let grouped = by_commit(vec![
            label("v1", RefKind::Tag, FIRST, false),
            label("origin/main", RefKind::RemoteBranch, FIRST, false),
            label("zeta", RefKind::Branch, FIRST, false),
            label("main", RefKind::Branch, FIRST, true),
            label("alpha", RefKind::Branch, FIRST, false),
            label("other", RefKind::Branch, SECOND, false),
        ]);
        let names: Vec<&str> = grouped[&Oid::parse(FIRST).unwrap()]
            .iter()
            .map(|label| label.name.as_str())
            .collect();
        assert_eq!(names, ["main", "alpha", "zeta", "origin/main", "v1"]);
        assert_eq!(grouped[&Oid::parse(SECOND).unwrap()].len(), 1);
    }

    #[test]
    fn lists_the_refs_of_a_real_repository() {
        let dir = tempfile::tempdir().unwrap();
        let location = RepoLocation::parse(dir.path().to_str().unwrap()).unwrap();
        let identity = ["-c", "user.name=Tester", "-c", "user.email=t@example.com"];
        let run = |args: &[&str]| {
            let mut full = identity.to_vec();
            full.extend_from_slice(args);
            let output = location.run(&full).unwrap();
            assert!(output.success(), "{args:?}: {}", String::from_utf8_lossy(&output.stderr));
        };
        run(&["init", "-q", "-b", "main"]);
        run(&["commit", "-q", "--allow-empty", "-m", "one"]);
        run(&["branch", "topic"]);
        run(&["tag", "light"]);
        run(&["tag", "-a", "annotated", "-m", "release"]);

        let refs = list_refs(&location).unwrap();

        let head = location.run(&["rev-parse", "HEAD"]).unwrap();
        let head = Oid::parse(String::from_utf8_lossy(&head.stdout).trim()).unwrap();
        let mut found: Vec<(&str, RefKind, bool)> = refs
            .iter()
            .map(|label| (label.name.as_str(), label.kind, label.is_head))
            .collect();
        found.sort();
        assert_eq!(
            found,
            [
                ("annotated", RefKind::Tag, false),
                ("light", RefKind::Tag, false),
                ("main", RefKind::Branch, true),
                ("topic", RefKind::Branch, false),
            ]
        );
        assert!(refs.iter().all(|label| label.target == head));
    }

    #[test]
    fn lists_the_refs_of_a_repository_inside_wsl() {
        let Some(scratch) = crate::testutil::WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        let identity = ["-c", "user.name=T", "-c", "user.email=t@example.com"];
        for args in [
            vec!["init", "-q", "-b", "main"],
            vec!["commit", "-q", "--allow-empty", "-m", "one"],
            vec!["tag", "-a", "v1", "-m", "release"],
        ] {
            let mut full = identity.to_vec();
            full.extend(args);
            let output = location.run(&full).unwrap();
            assert!(output.success(), "{}", String::from_utf8_lossy(&output.stderr));
        }

        let refs = list_refs(&location).unwrap();

        let mut names: Vec<&str> = refs.iter().map(|label| label.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["main", "v1"]);
        assert_eq!(refs[0].target, refs[1].target);
    }

    #[test]
    fn a_folder_that_is_not_a_repository_reports_the_git_error() {
        let dir = tempfile::tempdir().unwrap();
        let location = RepoLocation::parse(dir.path().to_str().unwrap()).unwrap();
        let error = list_refs(&location).unwrap_err();
        assert!(matches!(error, RefsError::Git(_)), "{error:?}");
        assert!(error.to_string().contains("not a git repository"));
    }
}
