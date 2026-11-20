pub const LOG_FORMAT: &str = "--format=%H%x1f%P%x1f%an%x1f%at%x1f%s%x1f%b";

const FIELD_SEPARATOR: char = '\u{1f}';

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Oid(String);

impl Oid {
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let valid_length = matches!(text.len(), 40 | 64);
        if valid_length && text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            Ok(Self(text.to_owned()))
        } else {
            Err(ParseError::BadId(text.to_owned()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub id: Oid,
    pub parents: Vec<Oid>,
    pub author: String,
    pub time: i64,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    FieldCount { found: usize },
    BadId(String),
    BadTime(String),
}

impl Commit {
    pub fn parse_record(record: &str) -> Result<Self, ParseError> {
        let fields: Vec<&str> = record.split(FIELD_SEPARATOR).collect();
        let [id, parents, author, time, subject, body] = fields[..] else {
            return Err(ParseError::FieldCount {
                found: fields.len(),
            });
        };
        Ok(Self {
            id: Oid::parse(id)?,
            parents: parents
                .split_whitespace()
                .map(Oid::parse)
                .collect::<Result<_, _>>()?,
            author: author.to_owned(),
            time: time
                .parse()
                .map_err(|_| ParseError::BadTime(time.to_owned()))?,
            subject: subject.to_owned(),
            body: body.trim_end().to_owned(),
        })
    }
}

pub fn parse_log(output: &[u8]) -> Result<Vec<Commit>, ParseError> {
    String::from_utf8_lossy(output)
        .split('\0')
        .filter(|record| !record.is_empty())
        .map(Commit::parse_record)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::RepoLocation;

    const ROOT: &str = "1111111111111111111111111111111111111111";
    const SECOND: &str = "2222222222222222222222222222222222222222";
    const SIDE: &str = "3333333333333333333333333333333333333333";
    const MERGE: &str = "4444444444444444444444444444444444444444";

    fn record(id: &str, parents: &str, author: &str, time: &str, subject: &str, body: &str) -> String {
        [id, parents, author, time, subject, body].join("\u{1f}")
    }

    fn stream(records: &[String]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for record in records {
            bytes.extend_from_slice(record.as_bytes());
            bytes.push(0);
        }
        bytes
    }

    #[test]
    fn parses_a_root_commit_with_an_empty_body() {
        let input = stream(&[record(ROOT, "", "Ada", "1700000000", "first", "")]);
        let commits = parse_log(&input).unwrap();
        assert_eq!(commits.len(), 1);
        let commit = &commits[0];
        assert_eq!(commit.id.as_str(), ROOT);
        assert!(commit.parents.is_empty());
        assert_eq!(commit.author, "Ada");
        assert_eq!(commit.time, 1_700_000_000);
        assert_eq!(commit.subject, "first");
        assert_eq!(commit.body, "");
    }

    #[test]
    fn parses_a_merge_commit_with_two_parents() {
        let parents = format!("{SECOND} {SIDE}");
        let input = stream(&[record(MERGE, &parents, "Ada", "1700000300", "merge side", "")]);
        let commits = parse_log(&input).unwrap();
        let ids: Vec<&str> = commits[0].parents.iter().map(|id| id.as_str()).collect();
        assert_eq!(ids, [SECOND, SIDE]);
    }

    #[test]
    fn keeps_a_multi_line_body_without_the_trailing_newline() {
        let body = "line one\n\nline three\n";
        let input = stream(&[record(SECOND, ROOT, "Bo", "1700000100", "second", body)]);
        let commits = parse_log(&input).unwrap();
        assert_eq!(commits[0].body, "line one\n\nline three");
    }

    #[test]
    fn keeps_the_order_of_the_input() {
        let input = stream(&[
            record(MERGE, &format!("{SECOND} {SIDE}"), "Ada", "4", "merge", ""),
            record(SIDE, ROOT, "Bo", "3", "side", ""),
            record(SECOND, ROOT, "Ada", "2", "second", ""),
            record(ROOT, "", "Ada", "1", "root", ""),
        ]);
        let subjects: Vec<String> = parse_log(&input)
            .unwrap()
            .into_iter()
            .map(|commit| commit.subject)
            .collect();
        assert_eq!(subjects, ["merge", "side", "second", "root"]);
    }

    #[test]
    fn empty_input_is_an_empty_history() {
        assert!(parse_log(b"").unwrap().is_empty());
    }

    #[test]
    fn rejects_a_record_with_the_wrong_field_count() {
        let input = stream(&["only\u{1f}three\u{1f}fields".to_owned()]);
        assert_eq!(
            parse_log(&input),
            Err(ParseError::FieldCount { found: 3 })
        );
    }

    #[test]
    fn rejects_an_id_that_is_not_hex() {
        let input = stream(&[record("not-a-hash", "", "Ada", "1", "s", "")]);
        assert_eq!(
            parse_log(&input),
            Err(ParseError::BadId("not-a-hash".to_owned()))
        );
    }

    #[test]
    fn rejects_a_time_that_is_not_a_number() {
        let input = stream(&[record(ROOT, "", "Ada", "yesterday", "s", "")]);
        assert_eq!(
            parse_log(&input),
            Err(ParseError::BadTime("yesterday".to_owned()))
        );
    }

    fn git(location: &RepoLocation, args: &[&str]) {
        let output = location.run(args).unwrap();
        assert!(
            output.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn parses_the_log_of_a_real_repository_with_a_merge() {
        let dir = tempfile::tempdir().unwrap();
        let location = RepoLocation::parse(dir.path().to_str().unwrap()).unwrap();
        let identity = ["-c", "user.name=Tester", "-c", "user.email=t@example.com"];
        let commit = |message: &str| {
            let mut args = identity.to_vec();
            args.extend(["commit", "-q", "--allow-empty", "-m", message]);
            git(&location, &args);
        };

        git(&location, &["init", "-q", "-b", "main"]);
        commit("root");
        git(&location, &["checkout", "-q", "-b", "side"]);
        commit("side work\n\nwith a body");
        git(&location, &["checkout", "-q", "main"]);
        commit("main work");
        let mut merge = identity.to_vec();
        merge.extend(["merge", "-q", "--no-ff", "side", "-m", "merge side"]);
        git(&location, &merge);

        let mut args = vec!["log", "--all", "--topo-order", "-z"];
        args.push(LOG_FORMAT);
        let output = location.run(&args).unwrap();
        assert!(output.success());

        let commits = parse_log(&output.stdout).unwrap();
        let subjects: Vec<&str> = commits.iter().map(|c| c.subject.as_str()).collect();
        assert_eq!(subjects.len(), 4);
        assert_eq!(subjects[0], "merge side");
        assert_eq!(subjects[3], "root");

        let merge_commit = &commits[0];
        assert_eq!(merge_commit.parents.len(), 2);
        assert!(commits.iter().all(|c| c.author == "Tester"));
        let side = commits.iter().find(|c| c.subject == "side work").unwrap();
        assert_eq!(side.body, "with a body");
    }
}
