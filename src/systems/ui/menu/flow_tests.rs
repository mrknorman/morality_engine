use bevy::{ecs::system::SystemState, prelude::*};

use crate::{
    systems::{
        interaction::{
            clickable_system, hoverable_system, option_cycler_input_system, reset_hoverable_state,
            selectable_system, Clickable, Hoverable, InteractionAggregate, InteractionCapture,
            InteractionCaptureOwner, InteractionGate, OptionCycler, Selectable,
            SelectableClickActivation, SelectableMenu, SystemMenuActions,
        },
        ui::{
            dropdown::{self, DropdownAnchorState, DropdownLayerState},
            layer::{self, UiLayer, UiLayerKind},
            menu_surface::MenuSurface,
            tabs::{TabBar, TabBarState, TabItem},
        },
    },
};

use super::{
    command_reducer::reduce_menu_command,
    defs::{
        MenuCommand, MenuHost, MenuPage, MenuRoot, VideoDisplayMode, VideoSettingsState, VideoTabKind,
        VideoResolutionDropdown, VIDEO_FOOTER_OPTION_COUNT, VIDEO_FOOTER_OPTION_START_INDEX,
        VIDEO_RESOLUTION_OPTION_INDEX, VIDEO_TOP_OPTION_COUNT,
    },
    dropdown_flow::sync_video_tab_content_state,
    stack::MenuNavigationState,
    tabbed_focus::{resolve_tabbed_focus, TabbedFocusInputs, TabbedMenuFocus},
    tabbed_menu::{self, TabbedMenuConfig},
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

#[test]
fn nested_tab_interaction_surface_without_layer_keeps_menu_root_active() {
    let mut world = World::new();
    let owner = world.spawn_empty().id();
    let menu_root = world.spawn(MenuSurface::new(owner)).id();
    let tab_interaction_root = world.spawn(MenuSurface::new(owner).without_layer()).id();

    assert!(world.entity(tab_interaction_root).get::<UiLayer>().is_none());

    let mut capture_state: SystemState<Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>> =
        SystemState::new(&mut world);
    let mut layer_state: SystemState<
        Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    > = SystemState::new(&mut world);
    let capture_query = capture_state.get(&world);
    let layer_query = layer_state.get(&world);
    let active = layer::active_layers_by_owner_scoped(None, &capture_query, &layer_query);

    assert!(layer::is_active_layer_entity_for_owner(&active, owner, menu_root));
}

#[derive(Component)]
struct TestDropdownLayer;

#[test]
fn dropdown_open_select_close_flow_is_owner_scoped() {
    let mut world = World::new();
    let owner = world.spawn_empty().id();
    let parent_a = world.spawn_empty().id();
    let parent_b = world.spawn_empty().id();

    let dropdown_a = world
        .spawn((
            TestDropdownLayer,
            UiLayer::new(owner, UiLayerKind::Dropdown),
            Visibility::Hidden,
            selectable_menu_for_tests(),
        ))
        .id();
    world.entity_mut(parent_a).add_child(dropdown_a);

    let dropdown_b = world
        .spawn((
            TestDropdownLayer,
            UiLayer::new(owner, UiLayerKind::Dropdown),
            Visibility::Hidden,
            selectable_menu_for_tests(),
        ))
        .id();
    world.entity_mut(parent_b).add_child(dropdown_b);

    let mut dropdown_state = DropdownLayerState::default();
    let mut query_state: SystemState<(
        Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<TestDropdownLayer>>,
        Query<&mut SelectableMenu, With<TestDropdownLayer>>,
    )> = SystemState::new(&mut world);

    {
        let (mut dropdown_query, mut menu_query) = query_state.get_mut(&mut world);
        dropdown::open_for_parent::<TestDropdownLayer>(
            owner,
            parent_a,
            2,
            &mut dropdown_state,
            &mut dropdown_query,
            &mut menu_query,
        );
    }

    assert_eq!(dropdown_state.open_parent_for_owner(owner), Some(parent_a));
    assert_eq!(
        world.entity(dropdown_a).get::<Visibility>(),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        world.entity(dropdown_b).get::<Visibility>(),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        world.entity(dropdown_a).get::<SelectableMenu>().map(|menu| menu.selected_index),
        Some(2)
    );

    {
        let (mut dropdown_query, mut menu_query) = query_state.get_mut(&mut world);
        dropdown::open_for_parent::<TestDropdownLayer>(
            owner,
            parent_b,
            1,
            &mut dropdown_state,
            &mut dropdown_query,
            &mut menu_query,
        );
    }

    assert_eq!(dropdown_state.open_parent_for_owner(owner), Some(parent_b));
    assert_eq!(
        world.entity(dropdown_a).get::<Visibility>(),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        world.entity(dropdown_b).get::<Visibility>(),
        Some(&Visibility::Visible)
    );
    assert_eq!(
        world.entity(dropdown_b).get::<SelectableMenu>().map(|menu| menu.selected_index),
        Some(1)
    );

    {
        let (mut dropdown_query, _) = query_state.get_mut(&mut world);
        dropdown::close_for_parent::<TestDropdownLayer>(
            owner,
            parent_b,
            &mut dropdown_state,
            &mut dropdown_query,
        );
    }

    assert_eq!(dropdown_state.open_parent_for_owner(owner), None);
    assert_eq!(
        world.entity(dropdown_a).get::<Visibility>(),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        world.entity(dropdown_b).get::<Visibility>(),
        Some(&Visibility::Hidden)
    );
}

#[test]
fn owner_layer_priority_prefers_modal_then_dropdown_then_base() {
    let mut world = World::new();
    let owner = world.spawn_empty().id();
    let base = world
        .spawn((
            UiLayer::new(owner, UiLayerKind::Base),
            Visibility::Visible,
            InteractionGate::PauseMenuOnly,
        ))
        .id();
    let dropdown = world
        .spawn((
            UiLayer::new(owner, UiLayerKind::Dropdown),
            Visibility::Visible,
            InteractionGate::PauseMenuOnly,
        ))
        .id();
    let modal = world
        .spawn((
            UiLayer::new(owner, UiLayerKind::Modal),
            Visibility::Hidden,
            InteractionGate::PauseMenuOnly,
        ))
        .id();

    world.spawn((InteractionCapture, InteractionCaptureOwner::new(owner)));

    let mut capture_state: SystemState<Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>> =
        SystemState::new(&mut world);
    let mut layer_state: SystemState<
        Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    > = SystemState::new(&mut world);

    let capture_query = capture_state.get(&world);
    let layer_query = layer_state.get(&world);
    let active = layer::active_layers_by_owner_scoped(None, &capture_query, &layer_query);
    assert_eq!(layer::active_layer_kind_for_owner(&active, owner), UiLayerKind::Dropdown);
    assert!(layer::is_active_layer_entity_for_owner(&active, owner, dropdown));
    assert!(!layer::is_active_layer_entity_for_owner(&active, owner, base));

    world.entity_mut(modal).insert(Visibility::Visible);
    let capture_query = capture_state.get(&world);
    let layer_query = layer_state.get(&world);
    let active = layer::active_layers_by_owner_scoped(None, &capture_query, &layer_query);
    assert_eq!(layer::active_layer_kind_for_owner(&active, owner), UiLayerKind::Modal);
    assert!(layer::is_active_layer_entity_for_owner(&active, owner, modal));
    assert!(!layer::is_active_layer_entity_for_owner(&active, owner, dropdown));

    world.entity_mut(modal).insert(Visibility::Hidden);
    world.entity_mut(dropdown).insert(Visibility::Hidden);
    let capture_query = capture_state.get(&world);
    let layer_query = layer_state.get(&world);
    let active = layer::active_layers_by_owner_scoped(None, &capture_query, &layer_query);
    assert_eq!(layer::active_layer_kind_for_owner(&active, owner), UiLayerKind::Base);
    assert!(layer::is_active_layer_entity_for_owner(&active, owner, base));
}

#[test]
fn option_cycler_input_only_triggers_for_selected_row_and_allowed_direction() {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_systems(Update, option_cycler_input_system);

    let menu = app.world_mut().spawn(selectable_menu_for_tests()).id();
    app.world_mut()
        .entity_mut(menu)
        .insert(SelectableMenu::new(
            1,
            vec![KeyCode::ArrowUp],
            vec![KeyCode::ArrowDown],
            vec![KeyCode::Enter],
            true,
        ));

    let first = app.world_mut().spawn((
        Selectable::new(menu, 0),
        OptionCycler {
            at_min: true,
            ..default()
        },
    )).id();
    let second = app.world_mut().spawn((
        Selectable::new(menu, 1),
        OptionCycler {
            at_max: true,
            ..default()
        },
    )).id();

    {
        let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keyboard.press(KeyCode::ArrowLeft);
        keyboard.press(KeyCode::ArrowRight);
    }
    app.update();

    let first_cycler = app
        .world()
        .get::<OptionCycler>(first)
        .copied()
        .expect("first cycler");
    let second_cycler = app
        .world()
        .get::<OptionCycler>(second)
        .copied()
        .expect("second cycler");

    // Row 0 is not selected, so it should not trigger even though left is pressed.
    assert!(!first_cycler.left_triggered);
    assert!(!first_cycler.right_triggered);
    // Row 1 is selected, so left triggers (not at_min), right is blocked by at_max.
    assert!(second_cycler.left_triggered);
    assert!(!second_cycler.right_triggered);
}

#[test]
fn tabbed_focus_down_from_tabs_activates_selected_tab_and_returns_to_options() {
    let mut app = App::new();
    app.add_message::<crate::systems::ui::tabs::TabChanged>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<crate::startup::cursor::CustomCursor>();
    app.init_resource::<tabbed_menu::TabbedMenuFocusState>();
    app.add_systems(Update, tabbed_menu::sync_tabbed_menu_focus);

    let menu_entity = app
        .world_mut()
        .spawn((
            MenuRoot {
                host: MenuHost::Main,
                gate: InteractionGate::PauseMenuOnly,
            },
            SelectableMenu::new(
                3,
                vec![KeyCode::ArrowUp],
                vec![KeyCode::ArrowDown],
                vec![KeyCode::Enter],
                true,
            ),
            Visibility::Visible,
        ))
        .id();
    app.world_mut()
        .entity_mut(menu_entity)
        .insert(UiLayer::new(menu_entity, UiLayerKind::Base));

    let tab_root = app
        .world_mut()
        .spawn((
            TabBar::new(menu_entity),
            TabBarState { active_index: 0 },
            TabbedMenuConfig::new(4, 4, 3),
            SelectableMenu::new(1, vec![], vec![], vec![], true),
            UiLayer::new(menu_entity, UiLayerKind::Base),
            Visibility::Visible,
        ))
        .id();
    app.world_mut().spawn((
        TabItem { index: 0 },
        Selectable::new(tab_root, 0),
        Hoverable::default(),
        Clickable::new(vec![SystemMenuActions::Activate]),
    ));
    app.world_mut().spawn((
        TabItem { index: 1 },
        Selectable::new(tab_root, 1),
        Hoverable::default(),
        Clickable::new(vec![SystemMenuActions::Activate]),
    ));

    app.world_mut()
        .resource_mut::<tabbed_menu::TabbedMenuFocusState>()
        .by_menu
        .insert(menu_entity, TabbedMenuFocus::Tabs);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowDown);

    app.update();

    let menu = app
        .world()
        .get::<SelectableMenu>(menu_entity)
        .expect("base menu");
    assert_eq!(menu.selected_index, 0);

    let tab_state = app.world().get::<TabBarState>(tab_root).expect("tab state");
    assert_eq!(tab_state.active_index, 1);

    let focus_state = app
        .world()
        .resource::<tabbed_menu::TabbedMenuFocusState>();
    assert_eq!(
        focus_state.by_menu.get(&menu_entity).copied(),
        Some(TabbedMenuFocus::Options)
    );
}

#[test]
fn mouse_selection_then_keyboard_cycle_uses_same_selected_row() {
    let mut app = App::new();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<crate::startup::cursor::CustomCursor>();
    app.init_resource::<InteractionAggregate>();
    app.add_systems(
        Update,
        (
            reset_hoverable_state,
            hoverable_system::<SystemMenuActions>,
            clickable_system::<SystemMenuActions>,
            selectable_system::<SystemMenuActions>,
            option_cycler_input_system,
        )
            .chain(),
    );

    let menu = app
        .world_mut()
        .spawn(
            SelectableMenu::new(
                0,
                vec![KeyCode::ArrowUp],
                vec![KeyCode::ArrowDown],
                vec![KeyCode::Enter],
                true,
            )
            .with_click_activation(SelectableClickActivation::HoveredOnly),
        )
        .id();
    app.world_mut().spawn((
        Selectable::new(menu, 0),
        OptionCycler::default(),
        Clickable::with_region(vec![SystemMenuActions::Activate], Vec2::new(100.0, 30.0)),
        Hoverable::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    ));
    let second = app.world_mut().spawn((
        Selectable::new(menu, 1),
        OptionCycler::default(),
        Clickable::with_region(vec![SystemMenuActions::Activate], Vec2::new(100.0, 30.0)),
        Hoverable::default(),
        Transform::from_xyz(0.0, 40.0, 1.0),
        GlobalTransform::from_translation(Vec3::new(0.0, 40.0, 1.0)),
    )).id();

    app.world_mut().resource_mut::<crate::startup::cursor::CustomCursor>().position =
        Some(Vec2::new(0.0, 40.0));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);

    let selected_menu = app.world().get::<SelectableMenu>(menu).expect("menu");
    assert_eq!(selected_menu.selected_index, 1);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowRight);
    app.update();

    let cycler = app
        .world()
        .get::<OptionCycler>(second)
        .copied()
        .expect("second cycler");
    assert!(cycler.right_triggered);
}

#[test]
fn tab_change_closes_open_video_dropdown_for_owner() {
    let mut app = App::new();
    app.add_message::<crate::systems::ui::tabs::TabChanged>();
    app.init_resource::<DropdownLayerState>();
    app.init_resource::<DropdownAnchorState>();
    app.add_systems(Update, sync_video_tab_content_state);

    let menu_entity = app
        .world_mut()
        .spawn((
            MenuRoot {
                host: MenuHost::Pause,
                gate: InteractionGate::PauseMenuOnly,
            },
            SelectableMenu::new(
                VIDEO_RESOLUTION_OPTION_INDEX,
                vec![KeyCode::ArrowUp],
                vec![KeyCode::ArrowDown],
                vec![KeyCode::Enter],
                true,
            ),
        ))
        .id();

    let tab_root = app
        .world_mut()
        .spawn((
            TabBar::new(menu_entity),
            TabBarState { active_index: 0 },
            TabbedMenuConfig::new(
                VIDEO_TOP_OPTION_COUNT,
                VIDEO_FOOTER_OPTION_START_INDEX,
                VIDEO_FOOTER_OPTION_COUNT,
            ),
        ))
        .id();

    let dropdown_entity = app
        .world_mut()
        .spawn((
            VideoResolutionDropdown,
            UiLayer::new(menu_entity, UiLayerKind::Dropdown),
            Visibility::Hidden,
            selectable_menu_for_tests(),
        ))
        .id();
    app.world_mut()
        .entity_mut(menu_entity)
        .add_child(dropdown_entity);

    let mut dropdown_state = app
        .world_mut()
        .remove_resource::<DropdownLayerState>()
        .expect("dropdown state");
    {
        let world = app.world_mut();
        let mut query_state: SystemState<(
            Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
            Query<&mut SelectableMenu, With<VideoResolutionDropdown>>,
        )> = SystemState::new(world);
        let (mut dropdown_query, mut dropdown_menu_query) = query_state.get_mut(world);
        dropdown::open_for_parent::<VideoResolutionDropdown>(
            menu_entity,
            menu_entity,
            0,
            &mut dropdown_state,
            &mut dropdown_query,
            &mut dropdown_menu_query,
        );
        query_state.apply(world);
    }
    app.insert_resource(dropdown_state);

    assert_eq!(
        app.world()
            .resource::<DropdownLayerState>()
            .open_parent_for_owner(menu_entity),
        Some(menu_entity)
    );
    assert_eq!(
        app.world().get::<Visibility>(dropdown_entity),
        Some(&Visibility::Visible)
    );

    app.world_mut().write_message(crate::systems::ui::tabs::TabChanged {
        tab_bar: tab_root,
        index: 1,
    });
    app.update();

    assert_eq!(
        app.world()
            .resource::<DropdownLayerState>()
            .open_parent_for_owner(menu_entity),
        None
    );
    assert_eq!(
        app.world().get::<Visibility>(dropdown_entity),
        Some(&Visibility::Hidden)
    );
}
