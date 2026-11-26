use crate::detail::{Load, file_row, name_and_folder};
use crate::repo_view::RepoView;
use crate::keyboard::{ListNav, Step, list_region, stepped};
use gitcore::diff::worktree_file_patch;
use gitcore::exec::CommandError;
use gitcore::index::{commit, stage, stage_all, unstage};
use gitcore::runner::RepoLocation;
use gitcore::status::{
    Branch, ChangeKind, FileState, Groups, Status, StatusError, WorkingFile, read_status,
};
use gpui::{
    AnyElement, App, AppContext, Context, Entity, Hsla, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, ScrollStrategy, StatefulInteractiveElement, Styled, Subscription,
    Task, Window, div, px, uniform_list,
};
use gpui_component::ActiveTheme;
use gpui_component::Disableable;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::diff::{Diff, DiffFile, DiffState};
use gpui_component::input::{InputEvent, Textarea, TextareaState};
use gpui_component::label::Label;
use std::ops::Range;

const LIST_WIDTH: f32 = 320.0;
const COMMIT_ERROR_MAX_HEIGHT: f32 = 120.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Staged,
    Unstaged,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    History,
    Changes,
}

pub struct Working {
    pub groups: Option<Groups>,
    pub branch: Option<Branch>,
    pub selected: Option<(Side, String)>,
    pub patch: Option<Load<Entity<DiffState>>>,
    patch_text: String,
    pub error: Option<String>,
    staged_nav: ListNav,
    unstaged_nav: ListNav,
    pub message: Option<Entity<TextareaState>>,
    pub commit_error: Option<String>,
    message_subscription: Option<Subscription>,
    status_task: Option<Task<()>>,
    patch_task: Option<Task<()>>,
    pub action_task: Option<Task<()>>,
}

impl Working {
    pub fn new(cx: &App) -> Self {
        Self {
            groups: None,
            branch: None,
            selected: None,
            patch: None,
            patch_text: String::new(),
            error: None,
            staged_nav: ListNav::new(cx),
            unstaged_nav: ListNav::new(cx),
            message: None,
            commit_error: None,
            message_subscription: None,
            status_task: None,
            patch_task: None,
            action_task: None,
        }
    }

    pub fn change_count(&self) -> usize {
        self.groups
            .as_ref()
            .map_or(0, |groups| groups.staged.len() + groups.unstaged.len())
    }

    pub fn busy(&self) -> bool {
        self.action_task.is_some()
    }

    fn can_commit(&self, message: &str) -> bool {
        let has_staged = self.groups.as_ref().is_some_and(|groups| !groups.staged.is_empty());
        has_staged && !message.trim().is_empty() && !self.busy()
    }

    fn nav(&self, side: Side) -> &ListNav {
        match side {
            Side::Staged => &self.staged_nav,
            Side::Unstaged => &self.unstaged_nav,
        }
    }

    fn selected_index(&self, side: Side) -> Option<usize> {
        let (selected_side, path) = self.selected.as_ref()?;
        if *selected_side != side {
            return None;
        }
        self.list(side).iter().position(|file| &file.path == path)
    }

    fn list(&self, side: Side) -> &[WorkingFile] {
        match (&self.groups, side) {
            (None, _) => &[],
            (Some(groups), Side::Staged) => &groups.staged,
            (Some(groups), Side::Unstaged) => &groups.unstaged,
        }
    }

    fn selected_file(&self) -> Option<&WorkingFile> {
        let (side, path) = self.selected.as_ref()?;
        self.list(*side).iter().find(|file| &file.path == path)
    }

    fn next_after_selected(&self) -> Option<String> {
        let (side, path) = self.selected.as_ref()?;
        let list = self.list(*side);
        let index = list.iter().position(|file| &file.path == path)?;
        let neighbour = list
            .get(index + 1)
            .or_else(|| index.checked_sub(1).and_then(|before| list.get(before)))?;
        Some(neighbour.path.clone())
    }
}

fn badge(state: FileState, cx: &Context<RepoView>) -> (&'static str, Hsla) {
    let colors = cx.theme();
    match state {
        FileState::Changed(ChangeKind::Added) => ("A", colors.success),
        FileState::Changed(ChangeKind::Modified) => ("M", colors.warning),
        FileState::Changed(ChangeKind::Deleted) => ("D", colors.danger),
        FileState::Changed(ChangeKind::TypeChanged) => ("T", colors.muted_foreground),
        FileState::Changed(ChangeKind::Renamed) => ("R", colors.info),
        FileState::Changed(ChangeKind::Copied) => ("C", colors.info),
        FileState::Untracked => ("U", colors.success),
        FileState::Conflicted(_) => ("!", colors.danger),
    }
}

impl RepoView {
    pub fn show_history(&mut self, cx: &mut Context<Self>) {
        if let Some(graph) = self.graph_mut() {
            graph.mode = Mode::History;
            cx.notify();
        }
    }

    pub fn show_changes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        graph.mode = Mode::Changes;
        graph.working.unstaged_nav.focus.focus(window, cx);
        if graph.working.message.is_none() {
            let state = cx.new(|cx| {
                TextareaState::new(window, cx)
                    .auto_grow(3, 8)
                    .placeholder("Commit message")
            });
            graph.working.message_subscription = Some(cx.subscribe_in(
                &state,
                window,
                |_, _, event: &InputEvent, _, cx| {
                    if matches!(event, InputEvent::Change) {
                        cx.notify();
                    }
                },
            ));
            graph.working.message = Some(state);
        }
        self.refresh_working(cx);
        cx.notify();
    }

    pub fn commit_staged(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        let location = graph.location.clone();
        let working = &mut graph.working;
        let Some(state) = &working.message else {
            return;
        };
        let message = state.read(cx).value().to_string();
        if !working.can_commit(&message) {
            return;
        }
        working.commit_error = None;
        let task = cx.background_executor().spawn(async move {
            commit(&location, &message).map_err(|error| error.to_string())
        });
        working.action_task = Some(cx.spawn_in(window, async move |this, cx| {
            let result = task.await;
            this.update_in(cx, |this, window, cx| this.finish_commit(result, window, cx))
                .ok();
        }));
        cx.notify();
    }

    fn finish_commit(&mut self, result: Result<(), String>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        graph.working.action_task = None;
        match result {
            Ok(()) => {
                if let Some(state) = &graph.working.message {
                    state.update(cx, |state, cx| state.set_value("", window, cx));
                }
                self.reload_history(cx);
            }
            Err(text) => graph.working.commit_error = Some(text),
        }
        self.refresh_working(cx);
        cx.notify();
    }

    pub fn refresh_working(&mut self, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        let location = graph.location.clone();
        let task = cx
            .background_executor()
            .spawn(async move { read_status(&location) });
        graph.working.status_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| this.finish_status(result, cx)).ok();
        }));
    }

    fn finish_status(&mut self, result: Result<Status, StatusError>, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        let working = &mut graph.working;
        working.status_task = None;
        match result {
            Ok(status) => {
                working.groups = Some(status.groups());
                working.branch = Some(status.branch);
                working.error = None;
            }
            Err(error) => working.error = Some(error.to_string()),
        }
        if working.selected.is_some() && working.selected_file().is_none() {
            working.selected = None;
            working.patch = None;
            working.patch_task = None;
        }
        if working.selected.is_some() && working.patch_task.is_none() {
            self.load_working_patch(cx);
        }
        cx.notify();
    }

    pub fn poll_working(&mut self, cx: &mut Context<Self>) {
        let Some(graph) = self.graph() else {
            return;
        };
        let busy = graph.working.status_task.is_some() || graph.working.action_task.is_some();
        if !busy {
            self.refresh_working(cx);
        }
    }

    pub fn select_working(&mut self, side: Side, path: String, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        graph.working.selected = Some((side, path));
        graph.working.patch = None;
        self.load_working_patch(cx);
    }

    fn load_working_patch(&mut self, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        let location = graph.location.clone();
        let working = &mut graph.working;
        let Some(file) = working.selected_file() else {
            return;
        };
        let kind = file.diff;
        let paths: Vec<String> = file.patch_paths().into_iter().map(str::to_owned).collect();
        let task = cx.background_executor().spawn(async move {
            let paths: Vec<&str> = paths.iter().map(String::as_str).collect();
            let patch = worktree_file_patch(&location, kind, &paths).map_err(|error| error.to_string())?;
            let files = DiffFile::parse(&patch).map_err(|error| error.to_string())?;
            Ok((patch, files))
        });
        working.patch.get_or_insert(Load::Loading);
        working.patch_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| this.finish_working_patch(result, cx)).ok();
        }));
        cx.notify();
    }

    fn finish_working_patch(&mut self, result: Result<(String, Vec<DiffFile>), String>, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        let working = &mut graph.working;
        working.patch_task = None;
        let showing = matches!(working.patch, Some(Load::Ready(_)));
        match result {
            Ok((text, files)) => {
                if !showing || working.patch_text != text {
                    working.patch = Some(Load::Ready(cx.new(|cx| DiffState::new(files, cx))));
                    working.patch_text = text;
                }
            }
            Err(message) if !showing => working.patch = Some(Load::Failed(message)),
            Err(_) => {}
        }
        cx.notify();
    }

    pub fn step_working(&mut self, side: Side, step: Step, cx: &mut Context<Self>) {
        let Some(graph) = self.graph() else {
            return;
        };
        let working = &graph.working;
        let list = working.list(side);
        let Some(index) = stepped(list.len(), working.selected_index(side), step) else {
            return;
        };
        let path = list[index].path.clone();
        working.nav(side).scroll.scroll_to_item(index, ScrollStrategy::Nearest);
        self.select_working(side, path, cx);
    }

    pub fn toggle_selected(&mut self, focused: Side, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        let working = &mut graph.working;
        if working.busy() {
            return;
        }
        let Some((side, _)) = working.selected.clone() else {
            return;
        };
        if side != focused {
            return;
        }
        let Some(file) = working.selected_file() else {
            return;
        };
        let paths: Vec<String> = file.patch_paths().into_iter().map(str::to_owned).collect();
        let next = working.next_after_selected();
        working.selected = next.map(|path| (side, path));
        working.patch = None;
        working.patch_task = None;
        self.run_index_action(
            move |location| {
                let paths: Vec<&str> = paths.iter().map(String::as_str).collect();
                match side {
                    Side::Staged => unstage(location, &paths),
                    Side::Unstaged => stage(location, &paths),
                }
            },
            cx,
        );
    }

    pub fn stage_everything(&mut self, cx: &mut Context<Self>) {
        self.run_index_action(stage_all, cx);
    }

    fn run_index_action(
        &mut self,
        action: impl FnOnce(&RepoLocation) -> Result<(), CommandError> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        if graph.working.action_task.is_some() {
            return;
        }
        let location = graph.location.clone();
        let task = cx.background_executor().spawn(async move {
            action(&location).map_err(|error| error.to_string())
        });
        graph.working.action_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| this.finish_action(result, cx)).ok();
        }));
        cx.notify();
    }

    fn finish_action(&mut self, result: Result<(), String>, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        graph.working.action_task = None;
        if let Err(message) = result {
            graph.working.error = Some(message);
        }
        self.refresh_working(cx);
        cx.notify();
    }

    pub fn working_rows(&mut self, side: Side, range: Range<usize>, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let selected_background = cx.theme().list_active;
        let muted = cx.theme().muted_foreground;
        let Some(graph) = self.graph() else {
            return Vec::new();
        };
        let working = &graph.working;
        let selected_path = working
            .selected
            .as_ref()
            .filter(|(selected_side, _)| *selected_side == side)
            .map(|(_, path)| path.as_str());
        let id = match side {
            Side::Staged => "staged",
            Side::Unstaged => "unstaged",
        };
        let list = working.list(side);
        range
            .filter_map(|index| list.get(index).map(|file| (index, file)))
            .map(|(index, file)| {
                let (name, folder) = name_and_folder(&file.path, file.renamed_from.as_deref());
                let selected = (selected_path == Some(file.path.as_str())).then_some(selected_background);
                let path = file.path.clone();
                file_row((id, index), badge(file.state, cx), name, folder, selected, muted)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_working(side, path.clone(), cx);
                    }))
                    .into_any_element()
            })
            .collect()
    }
}

fn group_header(title: String, action: Option<Button>) -> impl IntoElement {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_between()
        .child(Label::new(title))
        .children(action)
}

fn group_list(side: Side, count: usize, nav: &ListNav, cx: &mut Context<RepoView>) -> AnyElement {
    if count == 0 {
        return Label::new("No files").into_any_element();
    }
    let (region, id) = match side {
        Side::Staged => ("staged-region", "staged-list"),
        Side::Unstaged => ("unstaged-region", "unstaged-list"),
    };
    list_region(region, &nav.focus, cx.theme().ring)
        .size_full()
        .on_key_down(cx.listener(move |this, event: &KeyDownEvent, _, cx| {
            if let Some(step) = Step::from_key(&event.keystroke.key) {
                this.step_working(side, step, cx);
            } else if event.keystroke.key == "space" {
                this.toggle_selected(side, cx);
            }
        }))
        .child(
            uniform_list(
                id,
                count,
                cx.processor(move |this, range, _, cx| this.working_rows(side, range, cx)),
            )
            .track_scroll(&nav.scroll)
            .size_full(),
        )
        .into_any_element()
}

pub fn changes_view(working: &Working, cx: &mut Context<RepoView>) -> AnyElement {
    let Some(groups) = &working.groups else {
        let message = working.error.clone().unwrap_or_else(|| "Reading changes...".to_owned());
        return Label::new(message).into_any_element();
    };
    let patch = match &working.patch {
        None => div().into_any_element(),
        Some(Load::Loading) => Label::new("Loading diff...").into_any_element(),
        Some(Load::Failed(message)) => Label::new(message.clone()).into_any_element(),
        Some(Load::Ready(state)) => Diff::new(state).size_full().into_any_element(),
    };
    let stage_all_button = Button::new("stage-all")
        .label("Stage all")
        .on_click(cx.listener(|this, _, _, cx| this.stage_everything(cx)));
    let text = working
        .message
        .as_ref()
        .map(|state| state.read(cx).value().to_string())
        .unwrap_or_default();
    let danger = cx.theme().danger;
    let commit_box = div()
        .flex()
        .flex_col()
        .flex_none()
        .gap_1()
        .children(working.message.as_ref().map(Textarea::new))
        .children(working.commit_error.clone().map(|error| {
            div()
                .id("commit-error")
                .max_h(px(COMMIT_ERROR_MAX_HEIGHT))
                .overflow_y_scroll()
                .text_xs()
                .text_color(danger)
                .child(error)
        }))
        .child(
            Button::new("commit")
                .primary()
                .label("Commit")
                .disabled(!working.can_commit(&text))
                .on_click(cx.listener(|this, _, window, cx| this.commit_staged(window, cx))),
        );
    let lists = div()
        .flex()
        .flex_col()
        .flex_none()
        .w(px(LIST_WIDTH))
        .h_full()
        .min_h_0()
        .gap_1()
        .child(group_header(format!("Staged ({})", groups.staged.len()), None))
        .child(
            div()
                .flex_1()
                .min_h_0()
                .child(group_list(Side::Staged, groups.staged.len(), working.nav(Side::Staged), cx)),
        )
        .child(group_header(
            format!("Unstaged ({})", groups.unstaged.len()),
            Some(stage_all_button),
        ))
        .child(
            div()
                .flex_1()
                .min_h_0()
                .child(group_list(Side::Unstaged, groups.unstaged.len(), working.nav(Side::Unstaged), cx)),
        )
        .child(commit_box);
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap_1()
        .children(working.error.clone().map(Label::new))
        .child(
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .gap_2()
                .child(lists)
                .child(div().flex_1().min_w_0().h_full().min_h_0().child(patch)),
        )
        .into_any_element()
}
