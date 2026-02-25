use super::footer_nav::FooterNavConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabbedMenuFocus {
    Options,
    Tabs,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TabbedFocusInputs {
    pub previous_focus: TabbedMenuFocus,
    pub selected_option_index: usize,
    pub previous_selected_index: usize,
    pub active_tab_index: usize,
    pub selected_tab_index: usize,
    pub option_lock: Option<usize>,
    pub top_option_count: usize,
    pub footer_start_index: usize,
    pub footer_count: usize,
    pub tab_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub keyboard_focus_navigation: bool,
    pub clicked_tab_index: Option<usize>,
    pub clicked_option_index: Option<usize>,
    pub hovered_tab_index: Option<usize>,
    pub hovered_option_index: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TabbedFocusTransition {
    pub focus: TabbedMenuFocus,
    pub selected_option_index: usize,
    pub option_lock: Option<usize>,
    pub tab_selection_target: Option<usize>,
    pub pending_tab_activation: Option<usize>,
    pub pointer_activity_for_menu: bool,
}

fn top_last_index(top_option_count: usize) -> usize {
    top_option_count.saturating_sub(1)
}

pub(super) fn resolve_tabbed_focus(input: TabbedFocusInputs) -> TabbedFocusTransition {
    let footer = FooterNavConfig::new(input.footer_start_index, input.footer_count);
    let mut focus = input.previous_focus;
    let mut selected_option_index = input.selected_option_index;
    let mut option_lock = input.option_lock;
    let mut pointer_tab_target = None;
    let mut pending_tab_activation = None;
    let pointer_activity_for_menu = input.clicked_tab_index.is_some()
        || input.clicked_option_index.is_some()
        || input.hovered_tab_index.is_some()
        || input.hovered_option_index.is_some();

    if input.tab_pressed
        || (focus == TabbedMenuFocus::Options
        && input.up_pressed
        && selected_option_index == 0
        && input.previous_selected_index == 0)
    {
        focus = TabbedMenuFocus::Tabs;
        option_lock = None;
    } else if focus == TabbedMenuFocus::Tabs && input.down_pressed {
        if input.selected_tab_index != input.active_tab_index {
            pending_tab_activation = Some(input.selected_tab_index);
        }
        focus = TabbedMenuFocus::Options;
        selected_option_index = 0;
        option_lock = Some(0);
    } else if focus == TabbedMenuFocus::Options
        && footer.contains(selected_option_index)
        && (input.left_pressed ^ input.right_pressed)
    {
        selected_option_index = footer
            .cycle(selected_option_index, input.right_pressed)
            .unwrap_or(selected_option_index);
        option_lock = Some(selected_option_index);
    } else if focus == TabbedMenuFocus::Options
        && footer.contains(selected_option_index)
        && input.up_pressed
    {
        selected_option_index = top_last_index(input.top_option_count);
        option_lock = None;
    } else if let Some(clicked_tab_index) = input.clicked_tab_index {
        focus = TabbedMenuFocus::Tabs;
        pointer_tab_target = Some(clicked_tab_index);
        option_lock = None;
    } else if let Some(clicked_option_index) = input.clicked_option_index {
        focus = TabbedMenuFocus::Options;
        selected_option_index = clicked_option_index;
        option_lock = None;
    } else if !input.keyboard_focus_navigation {
        if let Some(hovered_tab_index) = input.hovered_tab_index {
            focus = TabbedMenuFocus::Tabs;
            pointer_tab_target = Some(hovered_tab_index);
            option_lock = None;
        } else if let Some(hovered_option_index) = input.hovered_option_index {
            focus = TabbedMenuFocus::Options;
            selected_option_index = hovered_option_index;
            option_lock = None;
        }
    } else if focus == TabbedMenuFocus::Options && (input.up_pressed || input.down_pressed) {
        option_lock = None;
    }

    let entering_tab_focus =
        input.previous_focus != TabbedMenuFocus::Tabs && focus == TabbedMenuFocus::Tabs;
    let tab_selection_target = if focus == TabbedMenuFocus::Tabs {
        if pointer_tab_target.is_some() {
            pointer_tab_target
        } else if entering_tab_focus {
            Some(input.active_tab_index)
        } else {
            None
        }
    } else {
        None
    };

    TabbedFocusTransition {
        focus,
        selected_option_index,
        option_lock,
        tab_selection_target,
        pending_tab_activation,
        pointer_activity_for_menu,
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_tabbed_focus, TabbedFocusInputs, TabbedMenuFocus};

    fn base_input() -> TabbedFocusInputs {
        TabbedFocusInputs {
            previous_focus: TabbedMenuFocus::Options,
            selected_option_index: 0,
            previous_selected_index: 0,
            active_tab_index: 1,
            selected_tab_index: 1,
            option_lock: None,
            top_option_count: 3,
            footer_start_index: 3,
            footer_count: 3,
            tab_pressed: false,
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
            keyboard_focus_navigation: false,
            clicked_tab_index: None,
            clicked_option_index: None,
            hovered_tab_index: None,
            hovered_option_index: None,
        }
    }

    #[test]
    fn down_from_tabs_commits_selected_tab_and_returns_to_options() {
        let mut input = base_input();
        input.previous_focus = TabbedMenuFocus::Tabs;
        input.down_pressed = true;
        input.selected_tab_index = 2;
        let transition = resolve_tabbed_focus(input);
        assert_eq!(transition.focus, TabbedMenuFocus::Options);
        assert_eq!(transition.selected_option_index, 0);
        assert_eq!(transition.option_lock, Some(0));
        assert_eq!(transition.pending_tab_activation, Some(2));
    }

    #[test]
    fn footer_left_right_cycles_selection() {
        let mut input = base_input();
        input.selected_option_index = 3;
        input.right_pressed = true;
        let transition = resolve_tabbed_focus(input);
        assert_eq!(transition.selected_option_index, 4);

        let mut input = base_input();
        input.selected_option_index = 3;
        input.left_pressed = true;
        let transition = resolve_tabbed_focus(input);
        assert_eq!(transition.selected_option_index, 5);
    }

    #[test]
    fn clicking_option_while_tabs_focused_moves_focus_without_dead_zone() {
        let mut input = base_input();
        input.previous_focus = TabbedMenuFocus::Tabs;
        input.selected_option_index = 0;
        input.clicked_option_index = Some(2);
        let transition = resolve_tabbed_focus(input);
        assert_eq!(transition.focus, TabbedMenuFocus::Options);
        assert_eq!(transition.selected_option_index, 2);
    }

    #[test]
    fn footer_up_jumps_directly_to_last_top_option() {
        let mut input = base_input();
        input.selected_option_index = 5;
        input.up_pressed = true;
        let transition = resolve_tabbed_focus(input);
        assert_eq!(transition.focus, TabbedMenuFocus::Options);
        assert_eq!(transition.selected_option_index, 2);
    }

    #[test]
    fn hovered_option_without_mouse_delta_can_still_reclaim_focus() {
        let mut input = base_input();
        input.previous_focus = TabbedMenuFocus::Tabs;
        input.selected_option_index = 0;
        input.hovered_option_index = Some(1);
        let transition = resolve_tabbed_focus(input);
        assert_eq!(transition.focus, TabbedMenuFocus::Options);
        assert_eq!(transition.selected_option_index, 1);
    }

    #[test]
    fn keyboard_navigation_ignores_hover_targets_for_focus_arbitration() {
        let mut input = base_input();
        input.previous_focus = TabbedMenuFocus::Options;
        input.selected_option_index = 2;
        input.keyboard_focus_navigation = true;
        input.left_pressed = true;
        input.hovered_tab_index = Some(1);
        input.hovered_option_index = Some(0);

        let transition = resolve_tabbed_focus(input);
        assert_eq!(transition.focus, TabbedMenuFocus::Options);
        assert_eq!(transition.selected_option_index, 2);
        assert_eq!(transition.tab_selection_target, None);
    }

    #[test]
    fn click_target_has_priority_over_hover_target() {
        let mut input = base_input();
        input.previous_focus = TabbedMenuFocus::Options;
        input.clicked_option_index = Some(2);
        input.hovered_tab_index = Some(1);
        input.hovered_option_index = Some(0);

        let transition = resolve_tabbed_focus(input);
        assert_eq!(transition.focus, TabbedMenuFocus::Options);
        assert_eq!(transition.selected_option_index, 2);
        assert_eq!(transition.tab_selection_target, None);
    }
}
