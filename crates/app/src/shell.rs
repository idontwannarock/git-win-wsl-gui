use crate::keyboard::{FocusNext, FocusPrevious};
use crate::motion::follow_system;
use crate::repo_view::RepoView;
use crate::tab_list::TabList;
use gpui::{
    AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement, PathPromptOptions,
    Render, Styled, Subscription, Task, Window, div,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

struct Tab {
    view: Entity<RepoView>,
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

    pub fn open(&mut self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.new(RepoView::new);
        view.update(cx, |view, cx| view.open(path, cx));
        let focus = view.read(cx).focus_handle().clone();
        window.focus(&focus, cx);
        self.tabs.open(Tab { view });
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
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = match self.active_view() {
            Some(view) => div().flex().flex_col().flex_1().min_h_0().child(view).into_any_element(),
            None => Label::new("Open a repository to begin").into_any_element(),
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
            .child(
                div().flex().flex_none().child(
                    Button::new("open")
                        .primary()
                        .label("Open repository...")
                        .on_click(cx.listener(|this, _, window, cx| this.choose_folder(window, cx))),
                ),
            )
            .child(body)
    }
}
