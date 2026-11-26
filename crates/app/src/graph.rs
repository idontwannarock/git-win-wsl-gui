use crate::branches::BranchPanel;
use crate::detail::Detail;
use crate::keyboard::ListNav;
use crate::theme::{MUTED, lane_color};
use crate::working::{Mode, Working};
use gitcore::lanes::{GraphRow, LaneLayout};
use gitcore::log::{Commit, Oid};
use gitcore::refs::RefLabel;
use gitcore::repo::OpenedRepo;
use gitcore::runner::RepoLocation;
use gitcore::stream::{LogStream, StreamError};
use gpui::{
    App, Bounds, FocusHandle, IntoElement, ParentElement, PathBuilder, Pixels, Point, Styled, Task, Window,
    canvas, div, fill, point, px,
};
use std::collections::HashMap;

pub const ROW_HEIGHT: f32 = 24.0;
const LANE_WIDTH: f32 = 14.0;
const CELL_LANES: usize = 8;
const MARKER_LANES: usize = 2;
const LINE_WIDTH: f32 = 2.0;
const DOT_SIZE: f32 = 8.0;

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
    pub history_focus: FocusHandle,
    pub files_nav: ListNav,
    pub mode: Mode,
    pub working: Working,
    pub branches: BranchPanel,
    pub branches_nav: ListNav,
    pub error: Option<String>,
    pub reading: Option<Task<()>>,
    pub stream: Option<LogStream>,
    layout: LaneLayout,
}

impl GraphState {
    pub fn replace_history(&mut self, opened: OpenedRepo) {
        self.location = opened.location;
        self.branches.show_refs(&opened.refs);
        self.refs = opened.refs;
        self.stream = Some(opened.stream);
        self.rows.clear();
        self.layout = LaneLayout::new();
        self.detail = None;
        self.reading = None;
        self.error = None;
    }

    pub fn new(opened: OpenedRepo, source: String, cx: &App) -> Self {
        Self {
            source,
            location: opened.location,
            branches: BranchPanel::new(&opened.refs),
            branches_nav: ListNav::new(cx),
            refs: opened.refs,
            detail: None,
            history_focus: cx.focus_handle().tab_stop(true),
            files_nav: ListNav::new(cx),
            mode: Mode::History,
            working: Working::new(cx),
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

#[derive(Debug, PartialEq, Eq)]
struct LaneFit {
    drawn: usize,
    hidden: usize,
}

impl LaneFit {
    fn shows(&self, lane: usize, column: usize) -> bool {
        lane < self.drawn && column < self.drawn
    }
}

fn lane_fit(row: &GraphRow) -> LaneFit {
    let drawn = if row.width <= CELL_LANES { row.width } else { CELL_LANES - MARKER_LANES };
    let mut hidden: Vec<usize> = row
        .passthrough
        .iter()
        .chain(&row.merges_in)
        .chain(&row.parents_out)
        .chain([&row.column])
        .copied()
        .filter(|&lane| lane >= drawn)
        .collect();
    hidden.sort_unstable();
    hidden.dedup();
    LaneFit { drawn, hidden: hidden.len() }
}

pub fn lane_cell(row: &CommitRow) -> impl IntoElement {
    let lanes = row.lanes.clone();
    let fit = lane_fit(&lanes);
    let marker = (fit.hidden > 0).then(|| {
        div()
            .absolute()
            .top_0()
            .right_0()
            .w(px(LANE_WIDTH * MARKER_LANES as f32))
            .h_full()
            .flex()
            .items_center()
            .justify_end()
            .text_xs()
            .text_color(MUTED.hsla())
            .child(format!("+{}", fit.hidden))
    });
    div()
        .relative()
        .w(px(LANE_WIDTH * CELL_LANES as f32))
        .h(px(ROW_HEIGHT))
        .flex_none()
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| paint_lanes(window, bounds, &lanes, &fit),
            )
            .size_full(),
        )
        .children(marker)
}

fn paint_lanes(window: &mut Window, bounds: Bounds<Pixels>, row: &GraphRow, fit: &LaneFit) {
    let top = bounds.origin.y;
    let middle = top + bounds.size.height / 2.0;
    let bottom = top + bounds.size.height;
    let x = |lane: usize| bounds.origin.x + px(LANE_WIDTH) * (lane as f32 + 0.5);

    let mut segment = |lane: usize, column: usize, from: Point<Pixels>, to: Point<Pixels>| {
        if !fit.shows(lane, column) {
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
        segment(lane, lane, point(x(lane), top), point(x(lane), bottom));
    }
    for &lane in &row.merges_in {
        segment(lane, row.column, point(x(lane), top), point(x(row.column), middle));
    }
    for &lane in &row.parents_out {
        segment(lane, row.column, point(x(row.column), middle), point(x(lane), bottom));
    }

    if row.column < fit.drawn {
        let size = px(DOT_SIZE);
        let dot = Bounds::new(
            point(x(row.column) - size / 2.0, middle - size / 2.0),
            gpui::size(size, size),
        );
        window.paint_quad(fill(dot, lane_color(row.column)).corner_radii(size / 2.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(
        column: usize,
        passthrough: &[usize],
        merges_in: &[usize],
        parents_out: &[usize],
        width: usize,
    ) -> GraphRow {
        GraphRow {
            column,
            passthrough: passthrough.to_vec(),
            merges_in: merges_in.to_vec(),
            parents_out: parents_out.to_vec(),
            width,
        }
    }

    #[test]
    fn up_to_eight_lanes_every_lane_is_drawn() {
        assert_eq!(lane_fit(&row(0, &[], &[], &[0], 1)), LaneFit { drawn: 1, hidden: 0 });
        assert_eq!(
            lane_fit(&row(0, &[1, 2, 3, 4, 5, 6, 7], &[], &[0], 8)),
            LaneFit { drawn: 8, hidden: 0 }
        );
    }

    #[test]
    fn twelve_open_lanes_draw_six_and_hide_six() {
        let open: Vec<usize> = (1..12).collect();
        assert_eq!(lane_fit(&row(0, &open, &[], &[0], 12)), LaneFit { drawn: 6, hidden: 6 });
    }

    #[test]
    fn hidden_counts_open_lanes_not_empty_slots() {
        assert_eq!(lane_fit(&row(0, &[1, 11], &[], &[0], 12)), LaneFit { drawn: 6, hidden: 1 });
    }

    #[test]
    fn a_commit_in_a_hidden_lane_counts_as_hidden() {
        assert_eq!(lane_fit(&row(11, &[0, 1], &[], &[1], 12)), LaneFit { drawn: 6, hidden: 1 });
    }

    #[test]
    fn a_line_is_shown_only_when_both_ends_are_drawn() {
        let fit = LaneFit { drawn: 6, hidden: 6 };
        assert!(fit.shows(2, 3));
        assert!(!fit.shows(2, 12));
        assert!(!fit.shows(6, 3));
    }
}
