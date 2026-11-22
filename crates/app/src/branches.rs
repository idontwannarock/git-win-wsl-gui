use crate::graph::GraphState;
use crate::home::Home;
use gitcore::branch::{checkout_local, checkout_remote};
use gitcore::log::Oid;
use gitcore::refs::{RefKind, RefLabel};
use gpui::{
    AnyElement, ClickEvent, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px, uniform_list,
};
use gpui_component::ActiveTheme;
use gpui_component::Disableable;
use gpui_component::button::Button;
use gpui_component::label::Label;
use std::collections::HashMap;
use std::ops::Range;

const PANEL_WIDTH: f32 = 220.0;
const ROW_HEIGHT: f32 = 22.0;
const ERROR_MAX_HEIGHT: f32 = 120.0;

pub enum Entry {
    Header(&'static str),
    Branch(RefLabel),
}

pub struct BranchPanel {
    entries: Vec<Entry>,
    selected: Option<(RefKind, String)>,
    error: Option<String>,
}

impl BranchPanel {
    pub fn new(refs: &HashMap<Oid, Vec<RefLabel>>) -> Self {
        Self {
            entries: entries(refs),
            selected: None,
            error: None,
        }
    }

    pub fn show_refs(&mut self, refs: &HashMap<Oid, Vec<RefLabel>>) {
        self.entries = entries(refs);
    }

    fn selected_branch(&self) -> Option<&RefLabel> {
        let (kind, name) = self.selected.as_ref()?;
        self.entries.iter().find_map(|entry| match entry {
            Entry::Branch(label) if label.kind == *kind && label.name == *name => Some(label),
            _ => None,
        })
    }

    fn can_check_out(&self) -> bool {
        self.selected_branch().is_some_and(|label| !label.is_head)
    }
}

fn entries(refs: &HashMap<Oid, Vec<RefLabel>>) -> Vec<Entry> {
    let mut local = Vec::new();
    let mut remote = Vec::new();
    for label in refs.values().flatten() {
        match label.kind {
            RefKind::Branch => local.push(label),
            RefKind::RemoteBranch => remote.push(label),
            RefKind::Tag => {}
        }
    }
    let mut entries = Vec::new();
    for (title, mut group) in [("Local", local), ("Remote", remote)] {
        if group.is_empty() {
            continue;
        }
        group.sort_by(|a, b| a.name.cmp(&b.name));
        entries.push(Entry::Header(title));
        entries.extend(group.into_iter().map(|label| Entry::Branch(label.clone())));
    }
    entries
}

impl Home {
    pub fn select_branch(&mut self, kind: RefKind, name: String, cx: &mut Context<Self>) {
        if let Some(graph) = self.graph_mut() {
            graph.branches.selected = Some((kind, name));
            cx.notify();
        }
    }

    pub fn checkout_selected(&mut self, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        if graph.working.busy() || !graph.branches.can_check_out() {
            return;
        }
        let Some((kind, name)) = graph.branches.selected.clone() else {
            return;
        };
        let location = graph.location.clone();
        graph.branches.error = None;
        let task = cx.background_executor().spawn(async move {
            let result = match kind {
                RefKind::RemoteBranch => checkout_remote(&location, &name),
                _ => checkout_local(&location, &name),
            };
            result.map_err(|error| error.to_string())
        });
        graph.working.action_task = Some(cx.spawn(async move |this, cx| {
            let result = task.await;
            this.update(cx, |this, cx| this.finish_checkout(result, cx)).ok();
        }));
        cx.notify();
    }

    fn finish_checkout(&mut self, result: Result<(), String>, cx: &mut Context<Self>) {
        let Some(graph) = self.graph_mut() else {
            return;
        };
        graph.working.action_task = None;
        match result {
            Ok(()) => self.reload_history(cx),
            Err(text) => graph.branches.error = Some(text),
        }
        self.refresh_working(cx);
        cx.notify();
    }

    pub fn branch_rows(&mut self, range: Range<usize>, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let selected_background = cx.theme().list_active;
        let muted = cx.theme().muted_foreground;
        let accent = cx.theme().primary;
        let Some(graph) = self.graph() else {
            return Vec::new();
        };
        let panel = &graph.branches;
        range
            .filter_map(|index| panel.entries.get(index).map(|entry| (index, entry)))
            .map(|(index, entry)| match entry {
                Entry::Header(title) => div()
                    .h(px(ROW_HEIGHT))
                    .flex()
                    .items_center()
                    .px_2()
                    .text_xs()
                    .text_color(muted)
                    .child(*title)
                    .into_any_element(),
                Entry::Branch(label) => {
                    let selected = panel
                        .selected
                        .as_ref()
                        .is_some_and(|(kind, name)| *kind == label.kind && *name == label.name);
                    let kind = label.kind;
                    let name = label.name.clone();
                    div()
                        .id(("branch", index))
                        .w_full()
                        .h(px(ROW_HEIGHT))
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2()
                        .when(selected, |row| row.bg(selected_background))
                        .child(
                            div()
                                .flex_none()
                                .w(px(10.0))
                                .text_color(accent)
                                .child(if label.is_head { "●" } else { "" }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(label.name.clone()),
                        )
                        .on_click(cx.listener(move |this, event: &ClickEvent, _, cx| {
                            this.select_branch(kind, name.clone(), cx);
                            if event.click_count() >= 2 {
                                this.checkout_selected(cx);
                            }
                        }))
                        .into_any_element()
                }
            })
            .collect()
    }
}

pub fn branches_view(graph: &GraphState, cx: &mut Context<Home>) -> AnyElement {
    let panel = &graph.branches;
    let danger = cx.theme().danger;
    let can_check_out = panel.can_check_out() && !graph.working.busy();
    div()
        .flex()
        .flex_col()
        .flex_none()
        .w(px(PANEL_WIDTH))
        .h_full()
        .min_h_0()
        .gap_1()
        .child(Label::new("Branches"))
        .child(
            div().flex_1().min_h_0().child(
                uniform_list(
                    "branches",
                    panel.entries.len(),
                    cx.processor(|this, range, _, cx| this.branch_rows(range, cx)),
                )
                .size_full(),
            ),
        )
        .children(panel.error.clone().map(|error| {
            div()
                .id("checkout-error")
                .max_h(px(ERROR_MAX_HEIGHT))
                .overflow_y_scroll()
                .text_xs()
                .text_color(danger)
                .child(error)
        }))
        .child(
            Button::new("checkout")
                .label("Checkout")
                .disabled(!can_check_out)
                .on_click(cx.listener(|this, _, _, cx| this.checkout_selected(cx))),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(name: &str, kind: RefKind, is_head: bool) -> RefLabel {
        RefLabel {
            name: name.to_owned(),
            kind,
            target: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
            is_head,
        }
    }

    fn describe(entries: &[Entry]) -> Vec<String> {
        entries
            .iter()
            .map(|entry| match entry {
                Entry::Header(title) => format!("# {title}"),
                Entry::Branch(label) => label.name.clone(),
            })
            .collect()
    }

    fn by_commit(labels: Vec<RefLabel>) -> HashMap<Oid, Vec<RefLabel>> {
        gitcore::refs::by_commit(labels)
    }

    #[test]
    fn lists_local_then_remote_branches_by_name_and_skips_tags() {
        let refs = by_commit(vec![
            label("v1", RefKind::Tag, false),
            label("origin/main", RefKind::RemoteBranch, false),
            label("topic", RefKind::Branch, false),
            label("main", RefKind::Branch, true),
        ]);
        assert_eq!(
            describe(&entries(&refs)),
            ["# Local", "main", "topic", "# Remote", "origin/main"]
        );
    }

    #[test]
    fn a_repository_without_remotes_has_no_remote_header() {
        let refs = by_commit(vec![label("main", RefKind::Branch, true)]);
        assert_eq!(describe(&entries(&refs)), ["# Local", "main"]);
    }

    #[test]
    fn the_current_branch_cannot_be_checked_out_again() {
        let refs = by_commit(vec![
            label("main", RefKind::Branch, true),
            label("topic", RefKind::Branch, false),
        ]);
        let mut panel = BranchPanel::new(&refs);
        assert!(!panel.can_check_out());
        panel.selected = Some((RefKind::Branch, "main".to_owned()));
        assert!(!panel.can_check_out());
        panel.selected = Some((RefKind::Branch, "topic".to_owned()));
        assert!(panel.can_check_out());
    }
}
