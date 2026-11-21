use crate::home::{Home, format_time};
use gitcore::changes::FileChange;
use gitcore::log::Commit;
use gpui::{
    AnyElement, Context, Div, ElementId, Entity, Hsla, InteractiveElement, IntoElement,
    ParentElement, Stateful, StatefulInteractiveElement, Styled, Task, div,
    prelude::FluentBuilder, px, uniform_list,
};
use gpui_component::ActiveTheme;
use gpui_component::diff::{Diff, DiffState};
use gpui_component::label::Label;
use std::ops::Range;

const FILE_LIST_WIDTH: f32 = 320.0;
const FILE_ROW_HEIGHT: f32 = 22.0;
const MESSAGE_MAX_HEIGHT: f32 = 96.0;

pub enum Load<T> {
    Loading,
    Ready(T),
    Failed(String),
}

pub struct Detail {
    pub row: usize,
    pub commit: Commit,
    pub files: Load<Vec<FileChange>>,
    pub selected_file: Option<usize>,
    pub patch: Option<Load<Entity<DiffState>>>,
    pub files_task: Option<Task<()>>,
    pub patch_task: Option<Task<()>>,
}

impl Detail {
    pub fn new(row: usize, commit: Commit) -> Self {
        Self {
            row,
            commit,
            files: Load::Loading,
            selected_file: None,
            patch: None,
            files_task: None,
            patch_task: None,
        }
    }
}

fn status_badge(change: &FileChange, cx: &Context<Home>) -> (&'static str, Hsla) {
    let colors = cx.theme();
    match change {
        FileChange::Added { .. } => ("A", colors.success),
        FileChange::Modified { .. } => ("M", colors.warning),
        FileChange::Deleted { .. } => ("D", colors.danger),
        FileChange::TypeChanged { .. } => ("T", colors.muted_foreground),
        FileChange::Renamed { .. } => ("R", colors.info),
        FileChange::Copied { .. } => ("C", colors.info),
    }
}

pub fn name_and_folder(path: &str, from: Option<&str>) -> (String, String) {
    let (folder, name) = path.rsplit_once('/').unwrap_or(("", path));
    let folder = match from {
        Some(from) => format!("{folder}  (from {from})"),
        None => folder.to_owned(),
    };
    (name.to_owned(), folder)
}

pub fn file_row(
    id: impl Into<ElementId>,
    badge: (&'static str, Hsla),
    name: String,
    folder: String,
    selected: Option<Hsla>,
    muted: Hsla,
) -> Stateful<Div> {
    let (letter, color) = badge;
    div()
        .id(id)
        .w_full()
        .h(px(FILE_ROW_HEIGHT))
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .when_some(selected, |row, background| row.bg(background))
        .child(div().flex_none().w(px(14.0)).text_color(color).child(letter))
        .child(div().flex_none().whitespace_nowrap().child(name))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .text_xs()
                .text_color(muted)
                .child(folder),
        )
}

pub fn file_rows(detail: &Detail, range: Range<usize>, cx: &mut Context<Home>) -> Vec<AnyElement> {
    let Load::Ready(files) = &detail.files else {
        return Vec::new();
    };
    let selected_background = cx.theme().list_active;
    let muted = cx.theme().muted_foreground;
    range
        .filter_map(|index| files.get(index).map(|change| (index, change)))
        .map(|(index, change)| {
            let from = match change {
                FileChange::Renamed { from, .. } | FileChange::Copied { from, .. } => Some(from.as_str()),
                _ => None,
            };
            let (name, folder) = name_and_folder(change.path(), from);
            let selected = (detail.selected_file == Some(index)).then_some(selected_background);
            file_row(("file", index), status_badge(change, cx), name, folder, selected, muted)
                .on_click(cx.listener(move |this, _, _, cx| this.select_file(index, cx)))
                .into_any_element()
        })
        .collect()
}

pub fn detail_view(detail: &Detail, cx: &mut Context<Home>) -> AnyElement {
    let commit = &detail.commit;
    let header = div()
        .flex()
        .flex_col()
        .flex_none()
        .gap_1()
        .child(Label::new(commit.subject.clone()))
        .child(Label::new(format!(
            "{}  {}  {}",
            commit.id.as_str(),
            commit.author,
            format_time(commit.time)
        )))
        .when(!commit.body.is_empty(), |header| {
            header.child(
                div()
                    .id("message-body")
                    .max_h(px(MESSAGE_MAX_HEIGHT))
                    .overflow_y_scroll()
                    .child(commit.body.clone()),
            )
        });

    let files = match &detail.files {
        Load::Loading => Label::new("Loading files...").into_any_element(),
        Load::Failed(message) => Label::new(message.clone()).into_any_element(),
        Load::Ready(list) if list.is_empty() => Label::new("No file changes").into_any_element(),
        Load::Ready(list) => uniform_list(
            "files",
            list.len(),
            cx.processor(|this, range, _, cx| this.file_rows(range, cx)),
        )
        .size_full()
        .into_any_element(),
    };

    let patch = match &detail.patch {
        None => div().into_any_element(),
        Some(Load::Loading) => Label::new("Loading diff...").into_any_element(),
        Some(Load::Failed(message)) => Label::new(message.clone()).into_any_element(),
        Some(Load::Ready(state)) => Diff::new(state).size_full().into_any_element(),
    };

    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap_1()
        .child(header)
        .child(
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .gap_2()
                .child(
                    div()
                        .flex_none()
                        .w(px(FILE_LIST_WIDTH))
                        .h_full()
                        .min_h_0()
                        .child(files),
                )
                .child(div().flex_1().min_w_0().h_full().min_h_0().child(patch)),
        )
        .into_any_element()
}
