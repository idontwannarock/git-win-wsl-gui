use crate::exec::{RunError, hide_window};
use crate::log::{Commit, LOG_FORMAT, ParseError};
use crate::runner::RepoLocation;
use std::fmt;
use std::io::{self, BufRead, BufReader, Read};
use std::process::{Child, ChildStdout, Stdio};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread;

const BATCHES_IN_FLIGHT: usize = 2;

#[derive(Debug)]
pub enum StreamError {
    Git(String),
    Parse(ParseError),
    Read(io::Error),
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git(message) => write!(f, "{message}"),
            Self::Parse(_) => write!(f, "git log printed output that this app cannot read"),
            Self::Read(error) => write!(f, "cannot read git log output: {error}"),
        }
    }
}

impl std::error::Error for StreamError {}

type Batch = Result<Vec<Commit>, StreamError>;

pub struct LogStream {
    child: Child,
    batches: Receiver<Batch>,
    finished: bool,
}

impl LogStream {
    pub fn start(location: &RepoLocation, batch_size: usize) -> Result<Self, RunError> {
        let mut command = location.command(&["log", "--all", "--topo-order", "-z", LOG_FORMAT]);
        hide_window(&mut command);
        let program = command.get_program().to_string_lossy().into_owned();
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| RunError::Spawn { program, source })?;
        let stdout = child.stdout.take().expect("stdout is piped");
        let (sender, batches) = sync_channel(BATCHES_IN_FLIGHT);
        thread::spawn(move || read_batches(stdout, batch_size, sender));
        Ok(Self {
            child,
            batches,
            finished: false,
        })
    }

    pub fn next_batch(&mut self) -> Option<Batch> {
        if self.finished {
            return None;
        }
        match self.batches.recv() {
            Ok(Ok(batch)) => Some(Ok(batch)),
            Ok(Err(error)) => {
                self.finished = true;
                Some(Err(error))
            }
            Err(_) => {
                self.finished = true;
                self.exit_failure().map(|message| Err(StreamError::Git(message)))
            }
        }
    }

    fn exit_failure(&mut self) -> Option<String> {
        let status = self.child.wait().ok()?;
        if status.success() {
            return None;
        }
        let mut message = String::new();
        if let Some(mut stderr) = self.child.stderr.take() {
            let _ = stderr.read_to_string(&mut message);
        }
        Some(message.trim().to_owned())
    }
}

impl Drop for LogStream {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read_batches(stdout: ChildStdout, batch_size: usize, sender: SyncSender<Batch>) {
    let mut reader = BufReader::new(stdout);
    let mut record = Vec::new();
    let mut batch = Vec::with_capacity(batch_size);
    loop {
        record.clear();
        match reader.read_until(0, &mut record) {
            Ok(0) => break,
            Ok(_) => {
                if record.last() == Some(&0) {
                    record.pop();
                }
                if record.is_empty() {
                    continue;
                }
                match Commit::parse_record(&String::from_utf8_lossy(&record)) {
                    Ok(commit) => batch.push(commit),
                    Err(error) => {
                        let _ = sender.send(Err(StreamError::Parse(error)));
                        return;
                    }
                }
                if batch.len() == batch_size
                    && sender.send(Ok(std::mem::take(&mut batch))).is_err()
                {
                    return;
                }
            }
            Err(error) => {
                let _ = sender.send(Err(StreamError::Read(error)));
                return;
            }
        }
    }
    if !batch.is_empty() {
        let _ = sender.send(Ok(batch));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{WslScratch, linear_history};
    use std::time::Duration;

    fn location_of(dir: &tempfile::TempDir) -> RepoLocation {
        RepoLocation::parse(dir.path().to_str().unwrap()).unwrap()
    }

    fn drain(stream: &mut LogStream) -> Vec<Vec<Commit>> {
        std::iter::from_fn(|| stream.next_batch())
            .map(|batch| batch.unwrap())
            .collect()
    }

    fn process_is_running(pid: u32) -> bool {
        let output = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
    }

    #[test]
    fn sends_the_history_newest_first_in_batches_of_the_requested_size() {
        let dir = tempfile::tempdir().unwrap();
        let location = location_of(&dir);
        linear_history(&location, 5);

        let mut stream = LogStream::start(&location, 2).unwrap();
        let batches = drain(&mut stream);

        let sizes: Vec<usize> = batches.iter().map(Vec::len).collect();
        assert_eq!(sizes, [2, 2, 1]);
        let subjects: Vec<&str> = batches
            .iter()
            .flatten()
            .map(|commit| commit.subject.as_str())
            .collect();
        assert_eq!(
            subjects,
            ["commit 5", "commit 4", "commit 3", "commit 2", "commit 1"]
        );
    }

    #[test]
    fn a_repository_without_commits_streams_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let location = location_of(&dir);
        assert!(location.run(&["init", "-q"]).unwrap().success());

        let mut stream = LogStream::start(&location, 10).unwrap();
        assert!(stream.next_batch().is_none());
    }

    #[test]
    fn a_folder_that_is_not_a_repository_reports_the_git_error() {
        let dir = tempfile::tempdir().unwrap();
        let mut stream = LogStream::start(&location_of(&dir), 10).unwrap();

        let error = stream.next_batch().unwrap().unwrap_err();

        assert!(matches!(error, StreamError::Git(_)), "{error:?}");
        assert!(error.to_string().contains("not a git repository"));
        assert!(stream.next_batch().is_none());
    }

    #[test]
    fn git_pauses_when_nobody_reads_and_resumes_when_reading_starts() {
        let dir = tempfile::tempdir().unwrap();
        let location = location_of(&dir);
        linear_history(&location, 20_000);

        let mut stream = LogStream::start(&location, 100).unwrap();
        thread::sleep(Duration::from_millis(1500));
        assert!(
            stream.child.try_wait().unwrap().is_none(),
            "git finished although nobody read its output"
        );

        let total: usize = drain(&mut stream).iter().map(Vec::len).sum();
        assert_eq!(total, 20_000);
    }

    #[test]
    fn dropping_the_stream_stops_the_git_process() {
        let dir = tempfile::tempdir().unwrap();
        let location = location_of(&dir);
        linear_history(&location, 20_000);

        let mut stream = LogStream::start(&location, 100).unwrap();
        stream.next_batch().unwrap().unwrap();
        let pid = stream.child.id();
        assert!(process_is_running(pid));

        drop(stream);

        assert!(!process_is_running(pid));
    }

    fn wsl_git_log_is_running(distro: &str, marker: &str) -> bool {
        let output = std::process::Command::new("wsl.exe")
            .args(["-d", distro, "--exec", "pgrep", "-f", marker])
            .output()
            .unwrap();
        output.status.success()
    }

    #[test]
    fn dropping_the_stream_stops_git_inside_wsl() {
        let Some(scratch) = WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        linear_history(&location, 20_000);

        let mut stream = LogStream::start(&location, 100).unwrap();
        stream.next_batch().unwrap().unwrap();
        thread::sleep(Duration::from_millis(500));
        assert!(wsl_git_log_is_running(&scratch.distro, "topo-order"));

        drop(stream);

        let stopped = (0..30).any(|_| {
            thread::sleep(Duration::from_millis(100));
            !wsl_git_log_is_running(&scratch.distro, "topo-order")
        });
        assert!(stopped, "git is still running inside WSL after the stream closed");
    }

    #[test]
    fn streams_a_repository_inside_wsl() {
        let Some(scratch) = WslScratch::create() else {
            return;
        };
        let location = RepoLocation::parse(&scratch.unc()).unwrap();
        linear_history(&location, 1_200);

        let mut stream = LogStream::start(&location, 500).unwrap();
        let sizes: Vec<usize> = drain(&mut stream).iter().map(Vec::len).collect();

        assert_eq!(sizes, [500, 500, 200]);
    }
}
