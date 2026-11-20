use crate::bench::ScrollBench;
use crate::graph::{CommitRow, GraphState, ROW_HEIGHT, lane_cell};
use chrono::{DateTime, Local};
use gitcore::refs::{RefKind, RefLabel, by_commit, list_refs};
use gitcore::repo::locate_repository;
use gitcore::runner::Runner;
use gitcore::stream::LogStream;
use gpui::{
    AnyElement, Context, IntoElement, ParentElement, PathPromptOptions, Render, Styled, Window,
    div, px, rgb, uniform_list, UniformListScrollHandle,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use std::ops::Range;
use std::time::Instant;

const BATCH_SIZE: usize = 500;
const PREFETCH_ROWS: usize = 100;
const SHORT_ID_LENGTH: usize = 7;
const AUTHOR_WIDTH: f32 = 140.0;
const DATE_WIDTH: f32 = 150.0;
const ID_WIDTH: f32 = 70.0;
const LABEL_MAX_WIDTH: f32 = 180.0;

enum Screen {
    Empty,
    Loading,
    Loaded(Box<GraphState>),
    Failed(String),
}

pub struct Home {
    screen: Screen,
    scroll: UniformListScrollHandle,
    bench: Option<ScrollBench>,
}

impl Home {
    pub fn new() -> Self {
        Self {
            screen: Screen::Empty,
            scroll: UniformListScrollHandle::new(),
            bench: ScrollBench::from_env(),
        }
    }

    pub fn open(&mut self, path: String, cx: &mut Context<Self>) {
        if let Some(bench) = &mut self.bench {
            bench.start();
        }
        self.screen = Screen::Loading;
        cx.notify();
        let task = cx.background_executor().spawn(async move {
            let location = locate_repository(&path).map_err(|error| error.to_string())?;
            let stream =
                LogStream::start(&location, BATCH_SIZE).map_err(|error| error.to_string())?;
            let refs = by_commit(list_refs(&location).map_err(|error| error.to_string())?);
            Ok::<_, String>((location, stream, refs))
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| {
                this.screen = match result {
                    Ok((location, stream, refs)) => {
                        Screen::Loaded(Box::new(GraphState::new(location, stream, refs)))
                    }
                    Err(message) => Screen::Failed(message),
                };
                this.load_more(cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn load_more(&mut self, cx: &mut Context<Self>) {
        let Screen::Loaded(graph) = &mut self.screen else {
            return;
        };
        if graph.reading.is_some() {
            return;
        }
        let Some(mut stream) = graph.stream.take() else {
            return;
        };
        let task = cx.background_executor().spawn(async move {
            let batch = stream.next_batch();
            (stream, batch)
        });
        graph.reading = Some(cx.spawn(async move |this, cx| {
            let (stream, batch) = task.await;
            this.update(cx, |this, cx| {
                if let Some(bench) = &mut this.bench {
                    bench.on_batch();
                }
                if let Screen::Loaded(graph) = &mut this.screen {
                    graph.finish_read(stream, batch);
                }
                cx.notify();
            })
            .ok();
        }));
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

    fn rows_in(&mut self, range: Range<usize>, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let Screen::Loaded(graph) = &self.screen else {
            return Vec::new();
        };
        let started = Instant::now();
        let rows = range
            .clone()
            .filter_map(|index| graph.rows.get(index))
            .map(|row| {
                let labels = graph.refs.get(&row.commit.id).map(Vec::as_slice);
                commit_row(row, labels.unwrap_or_default()).into_any_element()
            })
            .collect();
        if let Some(bench) = &mut self.bench {
            bench.record_row_build(started.elapsed());
        }
        if range.end + PREFETCH_ROWS >= graph.rows.len() {
            self.load_more(cx);
        }
        rows
    }

    fn graph_view(&self, graph: &GraphState, cx: &mut Context<Self>) -> AnyElement {
        let status = match (&graph.error, graph.rows.len()) {
            (Some(error), _) => error.clone(),
            (None, 0) => "Loading commits...".to_owned(),
            (None, count) => format!("{} - {count} commits loaded", describe(graph)),
        };
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .gap_1()
            .child(Label::new(status))
            .child(
                uniform_list(
                    "graph",
                    graph.rows.len(),
                    cx.processor(|this, range, _, cx| this.rows_in(range, cx)),
                )
                .flex_1()
                .min_h_0()
                .track_scroll(&self.scroll),
            )
            .into_any_element()
    }
}

impl Render for Home {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let (Screen::Loaded(graph), Some(bench)) = (&self.screen, &mut self.bench) {
            let finished = graph.stream.is_none() && graph.reading.is_none();
            match bench.on_frame(graph.rows.len(), finished, &self.scroll) {
                Some(report) => {
                    println!("{report}");
                    cx.quit();
                }
                None => window.request_animation_frame(),
            }
        }
        let body = match &self.screen {
            Screen::Empty => Label::new("Open a repository to begin").into_any_element(),
            Screen::Loading => Label::new("Opening...").into_any_element(),
            Screen::Failed(message) => Label::new(message.clone()).into_any_element(),
            Screen::Loaded(graph) => self.graph_view(graph, cx),
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

fn describe(graph: &GraphState) -> String {
    match &graph.location.runner {
        Runner::Windows => graph.location.path.clone(),
        Runner::Wsl { distro } => format!("wsl:{distro}:{}", graph.location.path),
    }
}

fn commit_row(row: &CommitRow, labels: &[RefLabel]) -> impl IntoElement {
    let commit = &row.commit;
    let short_id: String = commit.id.as_str().chars().take(SHORT_ID_LENGTH).collect();
    div()
        .w_full()
        .h(px(ROW_HEIGHT))
        .flex()
        .items_center()
        .gap_3()
        .child(lane_cell(row))
        .children(labels.iter().map(label_chip))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .child(commit.subject.clone()),
        )
        .child(fixed_column(AUTHOR_WIDTH, commit.author.clone()))
        .child(fixed_column(DATE_WIDTH, format_time(commit.time)))
        .child(fixed_column(ID_WIDTH, short_id))
}

fn label_chip(label: &RefLabel) -> impl IntoElement {
    let (background, text) = match (label.kind, label.is_head) {
        (RefKind::Branch, true) => (0x1f6feb, 0xffffff),
        (RefKind::Branch, false) => (0x1d3b5c, 0xdbe9ff),
        (RefKind::RemoteBranch, _) => (0x3a3a3a, 0xe0e0e0),
        (RefKind::Tag, _) => (0x5c4a12, 0xfff2c2),
    };
    div()
        .flex_none()
        .max_w(px(LABEL_MAX_WIDTH))
        .px_1()
        .rounded_sm()
        .text_xs()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .bg(rgb(background))
        .text_color(rgb(text))
        .child(label.name.clone())
}

fn fixed_column(width: f32, text: String) -> impl IntoElement {
    div()
        .w(px(width))
        .flex_none()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .child(text)
}

fn format_time(unix_seconds: i64) -> String {
    DateTime::from_timestamp(unix_seconds, 0)
        .map(|utc| utc.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default()
}
