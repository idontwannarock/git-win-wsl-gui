use chrono::{DateTime, Local};
use gitcore::log::Commit;
use gitcore::repo::{OpenedRepo, open_repository};
use gitcore::runner::Runner;
use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, PathPromptOptions,
    Render, StatefulInteractiveElement, Styled, Window, div,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;

const COMMIT_COUNT: usize = 100;
const SHORT_ID_LENGTH: usize = 7;

enum Screen {
    Empty,
    Loading,
    Loaded(OpenedRepo),
    Failed(String),
}

pub struct Home {
    screen: Screen,
}

impl Home {
    pub fn new() -> Self {
        Self {
            screen: Screen::Empty,
        }
    }

    pub fn open(&mut self, path: String, cx: &mut Context<Self>) {
        self.screen = Screen::Loading;
        cx.notify();
        let task = cx
            .background_executor()
            .spawn(async move { open_repository(&path, COMMIT_COUNT) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                this.screen = match result {
                    Ok(opened) => Screen::Loaded(opened),
                    Err(error) => Screen::Failed(error.to_string()),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn choose_folder(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update(cx, |this, cx| {
                this.open(path.to_string_lossy().into_owned(), cx)
            })
            .ok();
        })
        .detach();
    }
}

impl Render for Home {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match &self.screen {
            Screen::Empty => Label::new("Open a repository to begin").into_any_element(),
            Screen::Loading => Label::new("Opening...").into_any_element(),
            Screen::Failed(message) => Label::new(message.clone()).into_any_element(),
            Screen::Loaded(opened) => commit_list(opened),
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .child(
                Button::new("open")
                    .primary()
                    .label("Open repository...")
                    .on_click(cx.listener(|this, _, _, cx| this.choose_folder(cx))),
            )
            .child(body)
    }
}

fn commit_list(opened: &OpenedRepo) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .gap_1()
        .child(Label::new(describe(opened)))
        .child(
            div()
                .id("commits")
                .flex_1()
                .overflow_y_scroll()
                .children(opened.commits.iter().map(commit_row)),
        )
        .into_any_element()
}

fn describe(opened: &OpenedRepo) -> String {
    match &opened.location.runner {
        Runner::Windows => opened.location.path.clone(),
        Runner::Wsl { distro } => format!("wsl:{distro}:{}", opened.location.path),
    }
}

fn commit_row(commit: &Commit) -> impl IntoElement {
    let short_id: String = commit.id.as_str().chars().take(SHORT_ID_LENGTH).collect();
    div()
        .flex()
        .gap_3()
        .child(Label::new(short_id))
        .child(Label::new(commit.subject.clone()))
        .child(Label::new(commit.author.clone()))
        .child(Label::new(format_time(commit.time)))
}

fn format_time(unix_seconds: i64) -> String {
    DateTime::from_timestamp(unix_seconds, 0)
        .map(|utc| utc.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default()
}
