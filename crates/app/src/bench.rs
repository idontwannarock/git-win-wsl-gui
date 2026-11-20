use gpui::{ScrollStrategy, UniformListScrollHandle};
use std::time::{Duration, Instant};

const ROWS_PER_FRAME: usize = 10;
const TARGET_ROWS: usize = 30_000;
const MAX_FRAMES: usize = 20_000;
const SLOW_FRAME: Duration = Duration::from_millis(20);

pub struct ScrollBench {
    opened: Instant,
    first_page: Option<Duration>,
    batch_arrived: Option<Instant>,
    result_to_visible: Option<Duration>,
    frames_with_rows: usize,
    last_frame: Option<Instant>,
    frame_times: Vec<Duration>,
    row_build_times: Vec<Duration>,
    waiting_frames: usize,
    position: usize,
}

impl ScrollBench {
    pub fn from_env() -> Option<Self> {
        std::env::var_os("GITGUI_BENCH").map(|_| Self {
            opened: Instant::now(),
            first_page: None,
            batch_arrived: None,
            result_to_visible: None,
            frames_with_rows: 0,
            last_frame: None,
            frame_times: Vec::new(),
            row_build_times: Vec::new(),
            waiting_frames: 0,
            position: 0,
        })
    }

    pub fn start(&mut self) {
        self.opened = Instant::now();
    }

    pub fn on_batch(&mut self) {
        self.batch_arrived.get_or_insert_with(Instant::now);
    }

    pub fn record_row_build(&mut self, elapsed: Duration) {
        self.row_build_times.push(elapsed);
    }

    pub fn on_frame(
        &mut self,
        loaded_rows: usize,
        finished: bool,
        scroll: &UniformListScrollHandle,
    ) -> Option<String> {
        let now = Instant::now();
        if self.first_page.is_none() && loaded_rows > 0 {
            self.first_page = Some(self.opened.elapsed());
        }
        if loaded_rows > 0 {
            self.frames_with_rows += 1;
            if self.frames_with_rows == 2 {
                self.result_to_visible = self.batch_arrived.map(|arrived| arrived.elapsed());
            }
        }
        if self.first_page.is_some() {
            if let Some(last) = self.last_frame {
                self.frame_times.push(now - last);
            }
            self.last_frame = Some(now);
            if self.position + ROWS_PER_FRAME < loaded_rows {
                self.position += ROWS_PER_FRAME;
                scroll.scroll_to_item(self.position, ScrollStrategy::Top);
            } else {
                self.waiting_frames += 1;
            }
        }
        let reached_end = finished && self.position + ROWS_PER_FRAME >= loaded_rows;
        let done = self.position >= TARGET_ROWS
            || reached_end
            || self.frame_times.len() >= MAX_FRAMES;
        done.then(|| self.report())
    }

    fn report(&self) -> String {
        let mut frames = self.frame_times.clone();
        frames.sort();
        let at = |fraction: f64| {
            frames
                .get(((frames.len() as f64 * fraction) as usize).min(frames.len().saturating_sub(1)))
                .copied()
                .unwrap_or_default()
        };
        let slow = frames.iter().filter(|time| **time > SLOW_FRAME).count();
        let build_total: Duration = self.row_build_times.iter().sum();
        let build_max = self.row_build_times.iter().max().copied().unwrap_or_default();
        let build_avg = build_total / self.row_build_times.len().max(1) as u32;
        format!(
            "first_page_ms={:.0} result_to_visible_ms={:.0}\nrows_scrolled={} frames={} waiting_frames={}\nframe_ms p50={:.1} p95={:.1} p99={:.1} max={:.1} over_20ms={slow}\nrow_build_us avg={:.0} max={:.0}",
            self.first_page.unwrap_or_default().as_secs_f64() * 1000.0,
            self.result_to_visible.unwrap_or_default().as_secs_f64() * 1000.0,
            self.position,
            frames.len(),
            self.waiting_frames,
            at(0.50).as_secs_f64() * 1000.0,
            at(0.95).as_secs_f64() * 1000.0,
            at(0.99).as_secs_f64() * 1000.0,
            frames.last().copied().unwrap_or_default().as_secs_f64() * 1000.0,
            build_avg.as_secs_f64() * 1e6,
            build_max.as_secs_f64() * 1e6,
        )
    }
}
