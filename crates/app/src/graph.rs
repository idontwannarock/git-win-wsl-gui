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

fn lane_fit(width: usize) -> LaneFit {
    let drawn = if width <= CELL_LANES { width } else { CELL_LANES - MARKER_LANES };
    LaneFit { drawn, hidden: width - drawn }
}

pub fn lane_cell(row: &CommitRow) -> impl IntoElement {
    let lanes = row.lanes.clone();
    let fit = lane_fit(lanes.width);
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
                move |bounds, _, window, _| paint_lanes(window, bounds, &lanes, fit.drawn),
            )
            .size_full(),
        )
        .children(marker)
}

fn paint_lanes(window: &mut Window, bounds: Bounds<Pixels>, row: &GraphRow, drawn: usize) {
    let top = bounds.origin.y;
    let middle = top + bounds.size.height / 2.0;
    let bottom = top + bounds.size.height;
    let x = |lane: usize| bounds.origin.x + px(LANE_WIDTH) * (lane as f32 + 0.5);

    let mut segment = |lane: usize, from: Point<Pixels>, to: Point<Pixels>| {
        if lane >= drawn {
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

    if row.column < drawn {
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

    #[test]
    fn up_to_eight_lanes_every_lane_is_drawn() {
        assert_eq!(lane_fit(0), LaneFit { drawn: 0, hidden: 0 });
        assert_eq!(lane_fit(1), LaneFit { drawn: 1, hidden: 0 });
        assert_eq!(lane_fit(8), LaneFit { drawn: 8, hidden: 0 });
    }

    #[test]
    fn nine_lanes_draw_six_and_hide_three() {
        assert_eq!(lane_fit(9), LaneFit { drawn: 6, hidden: 3 });
    }

    #[test]
    fn many_lanes_draw_six_and_hide_the_rest() {
        assert_eq!(lane_fit(40), LaneFit { drawn: 6, hidden: 34 });
    }
}
