use crate::keyboard::{FocusNext, FocusPrevious};
use crate::motion::follow_system;
use crate::repo_view::{Opened, RepoView};
use crate::tab_list::TabList;
use crate::tab_session;
use crate::tab_title::titles;
use gpui::{
    AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, MouseButton, ParentElement,
    PathPromptOptions, Render, StatefulInteractiveElement, Styled, Subscription, Task, Window, div,
};
use gitcore::runner::{RepoKey, RepoLocation};
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
    key: Option<RepoKey>,
    _opened: Subscription,
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

    fn find(&self, key: &RepoKey, except: Option<&Entity<RepoView>>) -> Option<usize> {
        self.tabs
            .entries()
            .iter()
            .position(|tab| tab.key.as_ref() == Some(key) && Some(&tab.view) != except)
    }

    fn add(&mut self, path: String, view: Entity<RepoView>, window: &mut Window, cx: &mut Context<Self>) {
        let opened = cx.subscribe_in(&view, window, Self::on_opened);
        let key = key_of(&path);
        self.tabs.open(Tab { view, source: path, key, _opened: opened });
    }

    pub fn open(&mut self, path: String, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(existing) = key_of(&path).and_then(|key| self.find(&key, None)) {
            if let Some(tab) = self.tabs.entries().get(existing) {
                tab.view.update(cx, |view, cx| {
                    if view.is_failed() {
                        view.open(path, cx);
                    }
                });
            }
            self.activate(existing, window, cx);
            return;
        }
        let view = cx.new(RepoView::new);
        view.update(cx, |view, cx| view.open(path.clone(), cx));
        self.add(path, view, window, cx);
        self.focus_active(window, cx);
        self.changed(cx);
    }

    pub fn restore(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (paths, active) = tab_session::load();
        for path in paths {
            if key_of(&path).is_some_and(|key| self.find(&key, None).is_some()) {
                continue;
            }
            let view = cx.new(|cx| RepoView::pending(path.clone(), cx));
            self.add(path, view, window, cx);
        }
        self.activate(active, window, cx);
    }

    fn changed(&self, cx: &mut Context<Self>) {
        let sources: Vec<String> = self.tabs.entries().iter().map(|tab| tab.source.clone()).collect();
        tab_session::save(&sources, self.tabs.active().unwrap_or(0));
        cx.notify();
    }

    fn show_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_active(window, cx);
        if let Some(view) = self.active_view() {
            view.update(cx, |view, cx| view.activate(cx));
        }
        self.changed(cx);
    }

    fn on_opened(&mut self, view: &Entity<RepoView>, event: &Opened, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.tabs.entries().iter().position(|tab| &tab.view == view) else {
            return;
        };
        let key = event.0.key();
        if let Some(existing) = self.find(&key, Some(view)) {
            if self.tabs.close_in_favor_of(index, existing) {
                self.show_active(window, cx);
            }
            return;
        }
        if let Some(tab) = self.tabs.entry_mut(index) {
            tab.key = Some(key);
            tab.source = event.0.open_path();
        }
        self.changed(cx);
    }

    fn activate(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.tabs.activate(index) {
            self.show_active(window, cx);
        }
    }

    fn close(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let was_active = self.tabs.active() == Some(index);
        if self.tabs.close(index).is_none() {
            return;
        }
        if was_active {
            self.show_active(window, cx);
        } else {
            self.changed(cx);
        }
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

fn key_of(path: &str) -> Option<RepoKey> {
    RepoLocation::parse(path).ok().map(|location| location.key())
}
