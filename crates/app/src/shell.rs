use crate::keyboard::{FocusNext, FocusPrevious};
use crate::motion::follow_system;
use crate::repo_view::RepoView;
use crate::tab_list::TabList;
use crate::tab_title::titles;
use gpui::{
    AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, MouseButton, ParentElement,
    PathPromptOptions, Render, StatefulInteractiveElement, Styled, Subscription, Task, Window, div,
};
use gpui_component::Sizable;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use gpui_component::tab::{Tab as TabItem, TabBar};
use gpui_component::tooltip::Tooltip;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

struct Tab {
    view: Entity<RepoView>,
    source: String,
}

pub struct Shell {
    focus: FocusHandle,
    tabs: TabList<Tab>,
    _activation: Subscription,
    _poll: Task<()>,
}

impl Shell {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let activation = cx.observe_window_activation(window, |this, window, cx| {
            if window.is_window_active() {
                follow_system(cx);
                if let Some(view) = this.active_view() {
                    view.update(cx, |view, cx| view.refresh_working(cx));
                }
            }
        });
        let poll = cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor().timer(POLL_INTERVAL).await;
                let alive = this.update_in(cx, |this, window, cx| {
                    if window.is_window_active()
                        && let Some(view) = this.active_view()
                    {
                        view.update(cx, |view, cx| view.poll_working(cx));
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self { focus, tabs: TabList::new(), _activation: activation, _poll: poll }
    }

    fn active_view(&self) -> Option<Entity<RepoView>> {
        let index = self.tabs.active()?;
        self.tabs.entries().get(index).map(|tab| tab.view.clone())
    }

    fn focus_active(&self, window: &mut Window, cx: &mut Context<Self>) {
        let handle = match self.active_view() {
            Some(view) => view.read(cx).focus_handle().clone(),
            None => self.focus.clone(),
        };
        window.focus(&handle, cx);
    }

    pub fn open(&mut self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.new(RepoView::new);
        view.update(cx, |view, cx| view.open(path.clone(), cx));
        self.tabs.open(Tab { view, source: path });
        self.focus_active(window, cx);
        cx.notify();
    }

    fn activate(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.tabs.activate(index) {
            return;
        }
        self.focus_active(window, cx);
        if let Some(view) = self.active_view() {
            view.update(cx, |view, cx| view.refresh_working(cx));
        }
        cx.notify();
    }

    fn close(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.tabs.close(index).is_none() {
            return;
        }
        self.focus_active(window, cx);
        cx.notify();
    }

    fn choose_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = picked.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            this.update_in(cx, |this, window, cx| {
                this.open(path.to_string_lossy().into_owned(), window, cx)
            })
            .ok();
        })
        .detach();
    }

    fn tab_bar(&self, cx: &mut Context<Self>) -> TabBar {
        let sources: Vec<String> = self.tabs.entries().iter().map(|tab| tab.source.clone()).collect();
        let items = titles(&sources).into_iter().zip(sources).enumerate().map(|(index, (title, source))| {
            TabItem::new()
                .label(title)
                .suffix(
                    Button::new(("close", index))
                        .ghost()
                        .small()
                        .label("×")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            this.close(index, window, cx);
                        })),
                )
                .on_mouse_up(
                    MouseButton::Middle,
                    cx.listener(move |this, _, window, cx| this.close(index, window, cx)),
                )
                .tooltip(move |window, cx| Tooltip::new(source.clone()).build(window, cx))
        });
        TabBar::new("tabs")
            .underline()
            .selected_index(self.tabs.active().unwrap_or(usize::MAX))
            .on_click(cx.listener(|this, index: &usize, window, cx| this.activate(*index, window, cx)))
            .children(items)
            .suffix(
                Button::new("open")
                    .ghost()
                    .label("+")
                    .on_click(cx.listener(|this, _, window, cx| this.choose_folder(window, cx))),
            )
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match self.active_view() {
            Some(view) => div().flex().flex_col().flex_1().min_h_0().child(view).into_any_element(),
            None => div()
                .flex()
                .flex_col()
                .items_start()
                .gap_2()
                .child(Label::new("Open a repository to begin"))
                .child(
                    Button::new("open-empty")
                        .primary()
                        .label("Open repository...")
                        .on_click(cx.listener(|this, _, window, cx| this.choose_folder(window, cx))),
                )
                .into_any_element(),
        };
        div()
            .size_full()
            .track_focus(&self.focus)
            .on_action(cx.listener(|_, _: &FocusNext, window, cx| window.focus_next(cx)))
            .on_action(cx.listener(|_, _: &FocusPrevious, window, cx| window.focus_prev(cx)))
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .child(self.tab_bar(cx))
            .child(body)
    }
}
