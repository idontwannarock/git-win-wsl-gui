use gitcore::fixture::generate_history;
use gitcore::repo::open_repository;
use gitcore::runner::RepoLocation;
use std::time::{Duration, Instant};

const BATCH_SIZE: usize = 500;
const RUNS: usize = 5;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["generate", path, count, merge_every] => generate(path, count, merge_every),
        ["measure", path] => measure(path),
        _ => {
            eprintln!("usage: first_page generate <path> <count> <merge_every> | measure <path>");
            std::process::exit(2);
        }
    }
}

fn generate(path: &str, count: &str, merge_every: &str) {
    std::fs::create_dir_all(path).expect("create the repository folder");
    let location = RepoLocation::parse(path).expect("absolute path");
    let started = Instant::now();
    generate_history(
        &location,
        count.parse().expect("commit count"),
        merge_every.parse().expect("merge interval"),
    )
    .expect("generate history");
    println!("generated {count} commits in {:?}", started.elapsed());
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn measure(path: &str) {
    println!("run  open  first_batch  first_page  drain_all  commits");
    for run in 1..=RUNS {
        let started = Instant::now();
        let mut opened = open_repository(path, BATCH_SIZE).expect("open");
        let opened_at = started.elapsed();

        let first = opened
            .stream
            .next_batch()
            .expect("a first batch")
            .expect("a readable batch");
        let first_page = started.elapsed();

        let mut commits = first.len();
        while let Some(batch) = opened.stream.next_batch() {
            commits += batch.expect("a readable batch").len();
        }
        let all = started.elapsed();

        println!(
            "{run}  {:.0}  {:.0}  {:.0}  {:.0}  {commits}",
            millis(opened_at),
            millis(first_page - opened_at),
            millis(first_page),
            millis(all),
        );
    }
}
