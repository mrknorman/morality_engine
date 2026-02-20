use bevy::{ecs::system::SystemState, prelude::*};

use crate::{
    systems::{
        interaction::{InteractionCapture, InteractionCaptureOwner, InteractionGate, SelectableMenu},
        ui::{
            dropdown::DropdownLayerState,
            layer::{self, UiLayer, UiLayerKind},
        },
    },
};

use super::{
    command_reducer::reduce_menu_command,
    defs::{
        MenuCommand, MenuPage, VideoDisplayMode, VideoSettingsState, VideoTabKind,
        VIDEO_RESOLUTION_OPTION_INDEX,
    },
    stack::MenuNavigationState,
    tabbed_focus::{resolve_tabbed_focus, TabbedFocusInputs, TabbedMenuFocus},
    MenuStack,
};

fn selectable_menu_for_tests() -> SelectableMenu {
    SelectableMenu::new(
        0,
        vec![KeyCode::ArrowUp],
        vec![KeyCode::ArrowDown],
        vec![KeyCode::Enter],
        true,
    )
}

#[test]
fn menu_stack_dropdown_and_modal_flow_remains_consistent() {
    let menu_entity = Entity::from_bits(101);
    let mut menu_stack = MenuStack::new(MenuPage::Options);
    let mut selectable_menu = selectable_menu_for_tests();
    let mut settings = VideoSettingsState::default();
    settings.initialized = true;
    let mut dropdown_state = DropdownLayerState::default();
    let mut navigation_state = MenuNavigationState::default();

    let push_result = reduce_menu_command(
        MenuCommand::Push(MenuPage::Video),
        menu_entity,
        menu_stack.current_page(),
        VideoTabKind::Display,
        &mut menu_stack,
        &mut selectable_menu,
        &mut settings,
        &mut dropdown_state,
        &mut navigation_state,
    );
    assert!(push_result.dirty_menu);
    assert_eq!(menu_stack.current_page(), Some(MenuPage::Video));

    let dropdown_result = reduce_menu_command(
        MenuCommand::ToggleVideoTopOption(VIDEO_RESOLUTION_OPTION_INDEX),
        menu_entity,
        menu_stack.current_page(),
        VideoTabKind::Display,
        &mut menu_stack,
        &mut selectable_menu,
        &mut settings,
        &mut dropdown_state,
        &mut navigation_state,
    );
    assert_eq!(
        dropdown_result.open_dropdown.map(|(row, _)| row),
        Some(VIDEO_RESOLUTION_OPTION_INDEX)
    );

    // Simulate unsaved settings before leaving Video/Options flow.
    settings.pending.display_mode = VideoDisplayMode::Borderless;
    let pop_result = reduce_menu_command(
        MenuCommand::Pop,
        menu_entity,
        menu_stack.current_page(),
        VideoTabKind::Display,
        &mut menu_stack,
        &mut selectable_menu,
        &mut settings,
        &mut dropdown_state,
        &mut navigation_state,
    );
    assert!(pop_result.spawn_exit_unsaved_modal);
    assert!(pop_result.close_dropdown_for_menu);
    assert_eq!(navigation_state.exit_prompt_target_menu, Some(menu_entity));
}

#[test]
fn tabs_and_layer_gating_flow_remain_owner_scoped() {
    let transition = resolve_tabbed_focus(TabbedFocusInputs {
        previous_focus: TabbedMenuFocus::Tabs,
        selected_option_index: 0,
        previous_selected_index: 0,
        active_tab_index: 1,
        selected_tab_index: 2,
        option_lock: None,
        top_option_count: 4,
        footer_start_index: 4,
        footer_count: 3,
        tab_pressed: false,
        up_pressed: false,
        down_pressed: true,
        left_pressed: false,
        right_pressed: false,
        keyboard_focus_navigation: true,
        clicked_tab_index: None,
        clicked_option_index: None,
        hovered_tab_index: None,
        hovered_option_index: None,
    });
    assert_eq!(transition.focus, TabbedMenuFocus::Options);
    assert_eq!(transition.pending_tab_activation, Some(2));
    assert_eq!(transition.selected_option_index, 0);

    let mut world = World::new();
    let owner = world.spawn_empty().id();
    let base = world
        .spawn((
            UiLayer::new(owner, UiLayerKind::Base),
            Visibility::Visible,
            InteractionGate::PauseMenuOnly,
        ))
        .id();
    let modal = world
        .spawn((
            UiLayer::new(owner, UiLayerKind::Modal),
            Visibility::Visible,
            InteractionGate::PauseMenuOnly,
        ))
        .id();
    world.spawn((
        InteractionCapture,
        InteractionCaptureOwner::new(owner),
    ));

    let mut capture_state: SystemState<Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>> =
        SystemState::new(&mut world);
    let mut layer_state: SystemState<
        Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    > = SystemState::new(&mut world);
    let capture_query = capture_state.get(&world);
    let layer_query = layer_state.get(&world);
    let active = layer::active_layers_by_owner_scoped(None, &capture_query, &layer_query);

    assert_eq!(layer::active_layer_kind_for_owner(&active, owner), UiLayerKind::Modal);
    assert!(layer::is_active_layer_entity_for_owner(&active, owner, modal));
    assert!(!layer::is_active_layer_entity_for_owner(&active, owner, base));
}
