use crate::detail::Detail;
use crate::working::{Mode, Working};
use gitcore::lanes::{GraphRow, LaneLayout};
use gitcore::log::{Commit, Oid};
use gitcore::refs::RefLabel;
use gitcore::repo::OpenedRepo;
use gitcore::runner::RepoLocation;
use gitcore::stream::{LogStream, StreamError};
use gpui::{
    Bounds, FocusHandle, IntoElement, PathBuilder, Pixels, Point, Rgba, Styled, Task, Window, canvas, fill,
    point, px, rgb,
};
use std::collections::HashMap;

pub const ROW_HEIGHT: f32 = 24.0;
const LANE_WIDTH: f32 = 14.0;
const VISIBLE_LANES: usize = 8;
const LINE_WIDTH: f32 = 2.0;
const DOT_SIZE: f32 = 8.0;
const LANE_COLORS: [u32; 6] = [0x4fc1ff, 0xc586c0, 0xdcdcaa, 0x6a9955, 0xce9178, 0x9cdcfe];

pub struct CommitRow {
    pub commit: Commit,
    lanes: GraphRow,
}

pub struct GraphState {
    pub source: String,
    pub location: RepoLocation,
    pub rows: Vec<CommitRow>,
    pub refs: HashMap<Oid, Vec<RefLabel>>,
    pub detail: Option<Detail>,
    pub mode: Mode,
    pub working: Working,
    pub error: Option<String>,
    pub reading: Option<Task<()>>,
    pub stream: Option<LogStream>,
    layout: LaneLayout,
}

impl GraphState {
    pub fn replace_history(&mut self, opened: OpenedRepo) {
        self.location = opened.location;
        self.refs = opened.refs;
        self.stream = Some(opened.stream);
        self.rows.clear();
        self.layout = LaneLayout::new();
        self.detail = None;
        self.reading = None;
        self.error = None;
    }

    pub fn new(opened: OpenedRepo, source: String, focus: FocusHandle) -> Self {
        Self {
            source,
            location: opened.location,
            refs: opened.refs,
            detail: None,
            mode: Mode::History,
            working: Working::new(focus),
            rows: Vec::new(),
            error: None,
            reading: None,
            stream: Some(opened.stream),
            layout: LaneLayout::new(),
        }
    }

    pub fn finish_read(
        &mut self,
        stream: LogStream,
        batch: Option<Result<Vec<Commit>, StreamError>>,
    ) {
        self.reading = None;
        match batch {
            Some(Ok(commits)) => {
                for commit in commits {
                    let lanes = self.layout.next_row(&commit.id, &commit.parents);
                    self.rows.push(CommitRow { commit, lanes });
                }
                self.stream = Some(stream);
            }
            Some(Err(error)) => self.error = Some(error.to_string()),
            None => {}
        }
    }
}

pub fn lane_cell(row: &CommitRow) -> impl IntoElement {
    let lanes = row.lanes.clone();
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| paint_lanes(window, bounds, &lanes),
    )
    .w(px(LANE_WIDTH * VISIBLE_LANES as f32))
    .h(px(ROW_HEIGHT))
    .flex_none()
}

fn lane_color(lane: usize) -> Rgba {
    rgb(LANE_COLORS[lane % LANE_COLORS.len()])
}

fn paint_lanes(window: &mut Window, bounds: Bounds<Pixels>, row: &GraphRow) {
    let top = bounds.origin.y;
    let middle = top + bounds.size.height / 2.0;
    let bottom = top + bounds.size.height;
    let x = |lane: usize| bounds.origin.x + px(LANE_WIDTH) * (lane as f32 + 0.5);

    let mut segment = |lane: usize, from: Point<Pixels>, to: Point<Pixels>| {
        if lane >= VISIBLE_LANES {
            return;
        }
        let mut path = PathBuilder::stroke(px(LINE_WIDTH));
        path.move_to(from);
        path.line_to(to);
        if let Ok(path) = path.build() {
            window.paint_path(path, lane_color(lane));
        }
    };
    for &lane in &row.passthrough {
        segment(lane, point(x(lane), top), point(x(lane), bottom));
    }
    for &lane in &row.merges_in {
        segment(lane, point(x(lane), top), point(x(row.column), middle));
    }
    for &lane in &row.parents_out {
        segment(lane, point(x(row.column), middle), point(x(lane), bottom));
    }

    if row.column < VISIBLE_LANES {
        let size = px(DOT_SIZE);
        let dot = Bounds::new(
            point(x(row.column) - size / 2.0, middle - size / 2.0),
            gpui::size(size, size),
        );
        window.paint_quad(fill(dot, lane_color(row.column)).corner_radii(size / 2.0));
    }
}
