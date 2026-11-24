use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_FILE_BYTES: u64 = 10_000_000;
const MAX_ARG_CHARS: usize = 80;
const MAX_STDERR_CHARS: usize = 300;

struct Current {
    file: File,
    size: u64,
}

pub struct Trace {
    path: PathBuf,
    limit: u64,
    current: Mutex<Option<Current>>,
}

impl Trace {
    pub fn open(path: &Path) -> io::Result<Self> {
        Self::open_with_limit(path, MAX_FILE_BYTES)
    }

    fn open_with_limit(path: &Path, limit: u64) -> io::Result<Self> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        if fs::metadata(path).is_ok_and(|meta| meta.len() > limit) {
            move_to_old(path)?;
        }
        let current = open_append(path)?;
        Ok(Self { path: path.to_owned(), limit, current: Mutex::new(Some(current)) })
    }

    pub fn write(&self, at: SystemTime, message: &str) {
        let line = format!("{} {}\n", timestamp(at), one_line(message));
        let Ok(mut slot) = self.current.lock() else {
            return;
        };
        let full = slot
            .as_ref()
            .is_some_and(|current| current.size > 0 && current.size + line.len() as u64 > self.limit);
        if full {
            *slot = None;
            if move_to_old(&self.path).is_err() {
                return;
            }
        }
        if slot.is_none() {
            *slot = open_append(&self.path).ok();
        }
        if let Some(current) = slot.as_mut()
            && current.file.write_all(line.as_bytes()).is_ok()
        {
            current.size += line.len() as u64;
        }
    }
}

fn move_to_old(path: &Path) -> io::Result<()> {
    fs::rename(path, path.with_extension("log.old"))
}

fn open_append(path: &Path) -> io::Result<Current> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let size = file.metadata()?.len();
    Ok(Current { file, size })
}

static GLOBAL: OnceLock<Trace> = OnceLock::new();

pub fn init(path: &Path) -> io::Result<()> {
    let trace = Trace::open(path)?;
    let _ = GLOBAL.set(trace);
    Ok(())
}

pub fn record(message: impl FnOnce() -> String) {
    if let Some(trace) = GLOBAL.get() {
        trace.write(SystemTime::now(), &message());
    }
}

pub(crate) fn describe_command(command: &Command) -> String {
    std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|part| shorten(&one_line(&part.to_string_lossy()), MAX_ARG_CHARS))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn describe_outcome(result: &io::Result<Output>, elapsed: Duration) -> String {
    let millis = elapsed.as_millis();
    match result {
        Err(error) => format!("cannot start after {millis} ms: {error}"),
        Ok(output) => {
            let code = output
                .status
                .code()
                .map_or_else(|| "no exit code".to_owned(), |code| format!("exit {code}"));
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr.trim();
            if stderr.is_empty() {
                format!("{code} in {millis} ms")
            } else {
                format!("{code} in {millis} ms: {}", shorten(&one_line(stderr), MAX_STDERR_CHARS))
            }
        }
    }
}

fn one_line(text: &str) -> String {
    text.replace("\r\n", " | ").replace(['\r', '\n'], " | ")
}

fn shorten(text: &str, max_chars: usize) -> String {
    match text.char_indices().nth(max_chars) {
        Some((end, _)) => format!("{}...", &text[..end]),
        None => text.to_owned(),
    }
}

fn timestamp(at: SystemTime) -> String {
    let since = at.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = since.as_secs() as i64;
    let (year, month, day) = civil_from_days(seconds.div_euclid(86_400));
    let in_day = seconds.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        in_day / 3600,
        in_day % 3600 / 60,
        in_day % 60,
        since.subsec_millis()
    )
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::ExitStatus;

    fn at(seconds: u64, millis: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds) + Duration::from_millis(millis)
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("gitcore-trace-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn a_timestamp_is_utc_with_milliseconds() {
        assert_eq!(timestamp(at(1_700_000_000, 123)), "2023-11-14T22:13:20.123Z");
    }

    #[test]
    fn a_timestamp_handles_a_leap_day() {
        assert_eq!(timestamp(at(951_782_400, 0)), "2000-02-29T00:00:00.000Z");
    }

    #[test]
    fn a_command_is_shown_with_its_program_and_arguments() {
        let mut command = Command::new("git.exe");
        command.args(["-C", "C:\\repo", "status"]);
        assert_eq!(describe_command(&command), "git.exe -C C:\\repo status");
    }

    #[test]
    fn a_long_or_multi_line_argument_stays_on_one_short_line() {
        let mut command = Command::new("git.exe");
        command.args(["commit", "-m", &format!("first\nsecond {}", "x".repeat(200))]);
        let shown = describe_command(&command);
        assert!(!shown.contains('\n'));
        assert!(shown.ends_with("..."));
        assert!(shown.len() < 120);
    }

    #[test]
    fn an_outcome_shows_exit_code_time_and_the_first_part_of_stderr() {
        use std::os::windows::process::ExitStatusExt;
        let output = Output {
            status: ExitStatus::from_raw(128),
            stdout: Vec::new(),
            stderr: b"fatal: not a git repository\n".to_vec(),
        };
        assert_eq!(
            describe_outcome(&Ok(output), Duration::from_millis(42)),
            "exit 128 in 42 ms: fatal: not a git repository"
        );
    }

    #[test]
    fn a_clean_outcome_has_no_stderr_part() {
        use std::os::windows::process::ExitStatusExt;
        let output = Output {
            status: ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        assert_eq!(describe_outcome(&Ok(output), Duration::from_millis(7)), "exit 0 in 7 ms");
    }

    #[test]
    fn a_failed_start_shows_the_reason() {
        let error = io::Error::new(io::ErrorKind::NotFound, "program not found");
        assert_eq!(
            describe_outcome(&Err(error), Duration::from_millis(1)),
            "cannot start after 1 ms: program not found"
        );
    }

    #[test]
    fn each_message_becomes_one_line_in_the_file() {
        let dir = temp_dir("lines");
        let path = dir.join("app.log");
        let trace = Trace::open(&path).unwrap();
        trace.write(at(1_700_000_000, 0), "first\nsecond");
        trace.write(at(1_700_000_001, 0), "third");
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(
            text,
            "2023-11-14T22:13:20.000Z first | second\n2023-11-14T22:13:21.000Z third\n"
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_over_the_size_limit_moves_to_old_and_a_new_file_starts() {
        let dir = temp_dir("rotate");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("app.log");
        fs::write(&path, vec![b'x'; MAX_FILE_BYTES as usize + 1]).unwrap();
        let trace = Trace::open(&path).unwrap();
        trace.write(at(1_700_000_000, 0), "fresh");
        assert_eq!(fs::metadata(dir.join("app.log.old")).unwrap().len(), MAX_FILE_BYTES + 1);
        assert_eq!(fs::read_to_string(&path).unwrap(), "2023-11-14T22:13:20.000Z fresh\n");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_that_grows_past_the_limit_while_open_moves_to_old() {
        let dir = temp_dir("grow");
        let path = dir.join("app.log");
        let trace = Trace::open_with_limit(&path, 100).unwrap();
        for second in 0..3 {
            trace.write(at(1_700_000_000 + second, 0), "a message of some length");
        }
        let old = fs::read_to_string(dir.join("app.log.old")).unwrap();
        let current = fs::read_to_string(&path).unwrap();
        assert_eq!((old.lines().count(), current.lines().count()), (2, 1));
        assert!(!old.is_empty() && !current.is_empty());
        assert!(current.len() <= 100);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_small_file_is_kept_and_extended() {
        let dir = temp_dir("append");
        let path = dir.join("app.log");
        Trace::open(&path).unwrap().write(at(1_700_000_000, 0), "one");
        Trace::open(&path).unwrap().write(at(1_700_000_001, 0), "two");
        assert_eq!(fs::read_to_string(&path).unwrap().lines().count(), 2);
        assert!(!dir.join("app.log.old").exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
