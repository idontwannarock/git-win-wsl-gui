use crate::graph::GraphState;
use crate::repo_view::RepoView;
use gitcore::remote::{fetch, pull, push};
use gitcore::status::Branch;
use gpui::{Context, Div, IntoElement, ParentElement, Styled, div, px};
use gpui::{InteractiveElement, StatefulInteractiveElement};
use gpui_component::ActiveTheme;
use gpui_component::Disableable;
use gpui_component::button::Button;

const NOTE_MAX_HEIGHT: f32 = 120.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Remote {
    Fetch,
    Pull,
    Push,
}

pub enum SyncNote {
    Running(Remote),
    Done(Remote),
    Failed(String),
}

impl SyncNote {
    fn text(&self) -> String {
        match self {
            Self::Running(Remote::Fetch) => "Fetching...".to_owned(),
            Self::Running(Remote::Pull) => "Pulling...".to_owned(),
            Self::Running(Remote::Push) => "Pushing...".to_owned(),
            Self::Done(Remote::Fetch) => "Fetched".to_owned(),
            Self::Done(Remote::Pull) => "Pulled".to_owned(),
            Self::Done(Remote::Push) => "Pushed".to_owned(),
            Self::Failed(text) => text.clone(),
        }
    }
}

pub struct SyncLabels {
    pub pull: String,
    pub push: String,
}

pub fn sync_labels(branch: Option<&Branch>) -> SyncLabels {
    let (ahead, behind) = branch.and_then(|branch| branch.ahead_behind).unwrap_or((0, 0));
    let with = |name: &str, arrow: char, count: u32| {
        if count == 0 { name.to_owned() } else { format!("{name} {arrow}{count}") }
    };
    SyncLabels { pull: with("Pull", '↓', behind), push: with("Push", '↑', ahead) }
}

impl RepoView {
    pub fn sync(&mut self, remote: Remote, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        if graph.working.busy() {
            return;
        }
        let location = graph.location.clone();
        graph.sync_note = Some(SyncNote::Running(remote));
        let task = cx.background_executor().spawn(async move {
            let result = match remote {
                Remote::Fetch => fetch(&location),
                Remote::Pull => pull(&location),
                Remote::Push => push(&location),
            };
            result.map_err(|error| error.to_string())
        });
        graph.working.action_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| this.finish_sync(remote, result, cx)).ok();
        }));
        cx.notify();
    }

    fn finish_sync(&mut self, remote: Remote, result: Result<(), String>, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        graph.working.action_task = None;
        match result {
            Ok(()) => {
                graph.sync_note = Some(SyncNote::Done(remote));
                self.reload_history(cx);
            }
            Err(text) => graph.sync_note = Some(SyncNote::Failed(text)),
        }
        self.refresh_working(cx);
        cx.notify();
    }
}

pub fn sync_buttons(graph: &GraphState, cx: &mut Context<RepoView>) -> Div {
    let labels = sync_labels(graph.working.branch.as_ref());
    let idle = !graph.working.busy();
    div()
        .flex()
        .flex_none()
        .gap_2()
        .child(
            Button::new("fetch")
                .label("Fetch")
                .disabled(!idle)
                .on_click(cx.listener(|this, _, _, cx| this.sync(Remote::Fetch, cx))),
        )
        .child(
            Button::new("pull")
                .label(labels.pull)
                .disabled(!idle)
                .on_click(cx.listener(|this, _, _, cx| this.sync(Remote::Pull, cx))),
        )
        .child(
            Button::new("push")
                .label(labels.push)
                .disabled(!idle)
                .on_click(cx.listener(|this, _, _, cx| this.sync(Remote::Push, cx))),
        )
}

pub fn sync_note_line(graph: &GraphState, cx: &mut Context<RepoView>) -> Option<impl IntoElement> {
    let note = graph.sync_note.as_ref()?;
    let color = match note {
        SyncNote::Failed(_) => cx.theme().danger,
        _ => cx.theme().muted_foreground,
    };
    Some(
        div()
            .id("sync-note")
            .max_h(px(NOTE_MAX_HEIGHT))
            .overflow_y_scroll()
            .text_xs()
            .text_color(color)
            .child(note.text()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn branch(ahead_behind: Option<(u32, u32)>) -> Branch {
        Branch {
            head: Some("main".to_owned()),
            upstream: Some("origin/main".to_owned()),
            ahead_behind,
            ..Branch::default()
        }
    }

    fn labels(branch: Option<&Branch>) -> (String, String) {
        let labels = sync_labels(branch);
        (labels.pull, labels.push)
    }

    #[test]
    fn behind_shows_on_pull_and_ahead_shows_on_push() {
        assert_eq!(
            labels(Some(&branch(Some((2, 1))))),
            ("Pull ↓1".to_owned(), "Push ↑2".to_owned())
        );
    }

    #[test]
    fn a_zero_count_shows_no_number() {
        assert_eq!(labels(Some(&branch(Some((0, 0))))), ("Pull".to_owned(), "Push".to_owned()));
        assert_eq!(labels(Some(&branch(Some((3, 0))))), ("Pull".to_owned(), "Push ↑3".to_owned()));
        assert_eq!(labels(Some(&branch(Some((0, 4))))), ("Pull ↓4".to_owned(), "Push".to_owned()));
    }

    #[test]
    fn no_upstream_or_no_status_shows_no_number() {
        assert_eq!(labels(Some(&branch(None))), ("Pull".to_owned(), "Push".to_owned()));
        assert_eq!(labels(None), ("Pull".to_owned(), "Push".to_owned()));
    }
}
