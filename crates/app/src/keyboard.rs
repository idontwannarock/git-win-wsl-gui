use gpui::{
    App, Div, ElementId, FocusHandle, Hsla, InteractiveElement, KeyBinding, Stateful, Styled,
    UniformListScrollHandle, actions, div, transparent_black,
};

actions!(keyboard, [FocusNext, FocusPrevious]);

pub fn bind(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, None),
        KeyBinding::new("shift-tab", FocusPrevious, None),
    ]);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    Previous,
    Next,
}

impl Step {
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "up" => Some(Self::Previous),
            "down" => Some(Self::Next),
            _ => None,
        }
    }
}

pub fn stepped(len: usize, current: Option<usize>, step: Step) -> Option<usize> {
    let last = len.checked_sub(1)?;
    Some(match (current, step) {
        (None, Step::Next) => 0,
        (None, Step::Previous) => last,
        (Some(index), Step::Next) => (index + 1).min(last),
        (Some(index), Step::Previous) => index.min(last).saturating_sub(1),
    })
}

pub struct ListNav {
    pub focus: FocusHandle,
    pub scroll: UniformListScrollHandle,
}

impl ListNav {
    pub fn new(cx: &App) -> Self {
        Self {
            focus: cx.focus_handle().tab_stop(true),
            scroll: UniformListScrollHandle::new(),
        }
    }
}

pub fn list_region(id: impl Into<ElementId>, focus: &FocusHandle, ring: Hsla) -> Stateful<Div> {
    div()
        .id(id)
        .track_focus(focus)
        .border_1()
        .border_color(transparent_black())
        .rounded_md()
        .focus_visible(move |style| style.border_color(ring))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_up_and_down_are_steps() {
        assert_eq!(Step::from_key("up"), Some(Step::Previous));
        assert_eq!(Step::from_key("down"), Some(Step::Next));
        assert_eq!(Step::from_key("space"), None);
    }

    #[test]
    fn an_empty_list_has_no_step() {
        assert_eq!(stepped(0, None, Step::Next), None);
        assert_eq!(stepped(0, Some(0), Step::Previous), None);
    }

    #[test]
    fn with_nothing_selected_next_picks_the_first_and_previous_the_last() {
        assert_eq!(stepped(5, None, Step::Next), Some(0));
        assert_eq!(stepped(5, None, Step::Previous), Some(4));
    }

    #[test]
    fn steps_move_by_one_and_stop_at_the_ends() {
        assert_eq!(stepped(5, Some(2), Step::Next), Some(3));
        assert_eq!(stepped(5, Some(2), Step::Previous), Some(1));
        assert_eq!(stepped(5, Some(4), Step::Next), Some(4));
        assert_eq!(stepped(5, Some(0), Step::Previous), Some(0));
    }

    #[test]
    fn a_selection_past_the_end_is_pulled_back_into_the_list() {
        assert_eq!(stepped(3, Some(9), Step::Previous), Some(1));
        assert_eq!(stepped(3, Some(9), Step::Next), Some(2));
    }
}
