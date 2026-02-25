use super::*;
use crate::scenes::{dialogue::content::DialogueScene, dilemma::content::DilemmaScene};

fn apply_push_transition(
    menu_stack: &mut MenuStack,
    selectable_menu: &mut SelectableMenu,
    page: MenuPage,
) {
    menu_stack.push(page);
    selectable_menu.selected_index = menu_stack.current_selected_index();
}

fn apply_pop_transition(menu_stack: &mut MenuStack, selectable_menu: &mut SelectableMenu) -> bool {
    if let Some(selected_index) = menu_stack.pop() {
        selectable_menu.selected_index = selected_index;
        true
    } else {
        false
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MenuStateTransition {
    Pause(PauseState),
    Main(MainState),
    PauseAndMain(PauseState, MainState),
}

#[derive(Clone, Debug, Default)]
pub(super) struct MenuReducerResult {
    pub(super) close_dropdown_for_menu: bool,
    pub(super) open_dropdown: Option<(usize, usize)>,
    pub(super) dirty_menu: bool,
    pub(super) close_menu: bool,
    pub(super) open_main_menu_options_overlay: bool,
    pub(super) open_level_select_overlay: bool,
    pub(super) advance_to_next_scene: bool,
    pub(super) spawn_exit_unsaved_modal: bool,
    pub(super) open_debug_ui_showcase: bool,
    pub(super) apply_video_settings: bool,
    pub(super) open_level_select_launch_modal_scene: Option<DilemmaScene>,
    pub(super) start_single_level_scene: Option<DilemmaScene>,
    pub(super) start_single_dialogue_scene: Option<DialogueScene>,
    pub(super) state_transition: Option<MenuStateTransition>,
    pub(super) exit_application: bool,
}

fn reduce_push_menu_command(
    page: MenuPage,
    menu_stack: &mut MenuStack,
    selectable_menu: &mut SelectableMenu,
) -> MenuReducerResult {
    apply_push_transition(menu_stack, selectable_menu, page);
    MenuReducerResult {
        close_dropdown_for_menu: true,
        dirty_menu: true,
        ..MenuReducerResult::default()
    }
}

fn reduce_pop_menu_command(
    menu_entity: Entity,
    current_page: Option<MenuPage>,
    menu_stack: &mut MenuStack,
    selectable_menu: &mut SelectableMenu,
    settings: &VideoSettingsState,
    navigation_state: &mut MenuNavigationState,
) -> MenuReducerResult {
    let leaving_video_options =
        current_page.is_some_and(|page| matches!(page, MenuPage::Video | MenuPage::Options));
    let mut result = MenuReducerResult {
        close_dropdown_for_menu: true,
        ..MenuReducerResult::default()
    };

    if settings.initialized && video_settings_dirty(settings) && leaving_video_options {
        navigation_state.exit_prompt_target_menu = Some(menu_entity);
        navigation_state.exit_prompt_closes_menu_system = false;
        result.spawn_exit_unsaved_modal = true;
        return result;
    }

    if apply_pop_transition(menu_stack, selectable_menu) {
        result.dirty_menu = true;
    } else {
        result.close_menu = true;
    }
    result
}

fn reduce_toggle_video_top_option_command(
    command: &MenuCommand,
    menu_entity: Entity,
    active_tab: VideoTabKind,
    settings: &mut VideoSettingsState,
    dropdown_state: &mut DropdownLayerState,
) -> MenuReducerResult {
    if !settings.initialized {
        return MenuReducerResult::default();
    }
    let Some(row) = video_top_row_for_command(command) else {
        return MenuReducerResult::default();
    };
    let choice_count = video_top_option_choice_count(active_tab, row);
    if video_top_option_uses_dropdown(active_tab, row) {
        if dropdown_state.take_suppress_toggle_once(menu_entity) {
            return MenuReducerResult::default();
        }
        if dropdown_state.is_parent_open_for_owner(menu_entity, menu_entity) {
            MenuReducerResult {
                close_dropdown_for_menu: true,
                ..MenuReducerResult::default()
            }
        } else {
            video_top_option_selected_index(settings.pending, active_tab, row)
                .map(|selected_index| MenuReducerResult {
                    open_dropdown: Some((row, selected_index.min(choice_count - 1))),
                    ..MenuReducerResult::default()
                })
                .unwrap_or_default()
        }
    } else {
        let _ = step_video_top_option_for_input(&mut settings.pending, active_tab, row, true);
        MenuReducerResult {
            close_dropdown_for_menu: true,
            ..MenuReducerResult::default()
        }
    }
}

fn reduce_reset_video_defaults_command(settings: &mut VideoSettingsState) -> MenuReducerResult {
    if !settings.initialized {
        return MenuReducerResult::default();
    }

    settings.pending = default_video_settings();
    MenuReducerResult {
        close_dropdown_for_menu: true,
        ..MenuReducerResult::default()
    }
}

pub(super) fn reduce_menu_command(
    command: MenuCommand,
    menu_entity: Entity,
    current_page: Option<MenuPage>,
    active_tab: VideoTabKind,
    menu_stack: &mut MenuStack,
    selectable_menu: &mut SelectableMenu,
    settings: &mut VideoSettingsState,
    dropdown_state: &mut DropdownLayerState,
    navigation_state: &mut MenuNavigationState,
) -> MenuReducerResult {
    match command {
        MenuCommand::None => MenuReducerResult::default(),
        MenuCommand::Push(page) => reduce_push_menu_command(page, menu_stack, selectable_menu),
        MenuCommand::Pop => reduce_pop_menu_command(
            menu_entity,
            current_page,
            menu_stack,
            selectable_menu,
            settings,
            navigation_state,
        ),
        MenuCommand::SetPause(state) => MenuReducerResult {
            state_transition: Some(MenuStateTransition::Pause(state)),
            ..MenuReducerResult::default()
        },
        MenuCommand::OpenMainMenuOptionsOverlay => MenuReducerResult {
            open_main_menu_options_overlay: true,
            ..MenuReducerResult::default()
        },
        MenuCommand::OpenLevelSelectOverlay => MenuReducerResult {
            open_level_select_overlay: true,
            ..MenuReducerResult::default()
        },
        MenuCommand::LaunchDilemmaFromLevelSelect(scene) => {
            if cfg!(debug_assertions) {
                MenuReducerResult {
                    open_level_select_launch_modal_scene: Some(scene),
                    ..MenuReducerResult::default()
                }
            } else {
                MenuReducerResult {
                    start_single_level_scene: Some(scene),
                    close_menu: true,
                    ..MenuReducerResult::default()
                }
            }
        }
        MenuCommand::StartSingleDialogue(scene) => MenuReducerResult {
            start_single_dialogue_scene: Some(scene),
            close_menu: true,
            ..MenuReducerResult::default()
        },
        MenuCommand::NextScene => MenuReducerResult {
            advance_to_next_scene: true,
            ..MenuReducerResult::default()
        },
        command @ MenuCommand::ToggleVideoTopOption(_) => reduce_toggle_video_top_option_command(
            &command,
            menu_entity,
            active_tab,
            settings,
            dropdown_state,
        ),
        MenuCommand::OpenDebugUiShowcase => MenuReducerResult {
            open_debug_ui_showcase: true,
            close_menu: true,
            ..MenuReducerResult::default()
        },
        MenuCommand::ApplyVideoSettings => MenuReducerResult {
            apply_video_settings: true,
            ..MenuReducerResult::default()
        },
        MenuCommand::ResetVideoDefaults => reduce_reset_video_defaults_command(settings),
        MenuCommand::SetMain(state) => MenuReducerResult {
            state_transition: Some(MenuStateTransition::Main(state)),
            ..MenuReducerResult::default()
        },
        MenuCommand::SetPauseAndMain(pause_state, main_state) => MenuReducerResult {
            state_transition: Some(MenuStateTransition::PauseAndMain(pause_state, main_state)),
            ..MenuReducerResult::default()
        },
        MenuCommand::ExitApplication => MenuReducerResult {
            exit_application: true,
            ..MenuReducerResult::default()
        },
        MenuCommand::CloseMenu => MenuReducerResult {
            close_menu: true,
            ..MenuReducerResult::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_menu_entity() -> Entity {
        Entity::from_bits(1)
    }

    fn test_selectable_menu() -> SelectableMenu {
        SelectableMenu::new(
            0,
            vec![KeyCode::ArrowUp],
            vec![KeyCode::ArrowDown],
            vec![KeyCode::Enter],
            true,
        )
    }

    #[test]
    fn push_marks_menu_dirty_and_updates_stack() {
        let menu_entity = test_menu_entity();
        let mut menu_stack = MenuStack::new(MenuPage::PauseRoot);
        let mut selectable_menu = test_selectable_menu();
        let mut settings = VideoSettingsState::default();
        let mut dropdown_state = DropdownLayerState::default();
        let mut navigation_state = MenuNavigationState::default();

        let result = reduce_menu_command(
            MenuCommand::Push(MenuPage::Options),
            menu_entity,
            menu_stack.current_page(),
            VideoTabKind::Display,
            &mut menu_stack,
            &mut selectable_menu,
            &mut settings,
            &mut dropdown_state,
            &mut navigation_state,
        );

        assert!(result.dirty_menu);
        assert!(result.close_dropdown_for_menu);
        assert_eq!(menu_stack.current_page(), Some(MenuPage::Options));
    }

    #[test]
    fn pop_on_root_closes_menu() {
        let menu_entity = test_menu_entity();
        let mut menu_stack = MenuStack::new(MenuPage::Options);
        let mut selectable_menu = test_selectable_menu();
        let mut settings = VideoSettingsState::default();
        settings.initialized = true;
        let mut dropdown_state = DropdownLayerState::default();
        let mut navigation_state = MenuNavigationState::default();

        let result = reduce_menu_command(
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

        assert!(result.close_menu);
        assert!(!result.dirty_menu);
    }

    #[test]
    fn pop_with_unsaved_video_changes_spawns_exit_modal() {
        let menu_entity = test_menu_entity();
        let mut menu_stack = MenuStack::new(MenuPage::Options);
        menu_stack.push(MenuPage::Video);
        let mut selectable_menu = test_selectable_menu();
        let mut settings = VideoSettingsState::default();
        settings.initialized = true;
        settings.saved = default_video_settings();
        settings.pending = settings.saved;
        settings.pending.display_mode = VideoDisplayMode::Borderless;
        let mut dropdown_state = DropdownLayerState::default();
        let mut navigation_state = MenuNavigationState::default();

        let result = reduce_menu_command(
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

        assert!(result.spawn_exit_unsaved_modal);
        assert_eq!(navigation_state.exit_prompt_target_menu, Some(menu_entity));
        assert!(!result.close_menu);
        assert_eq!(menu_stack.current_page(), Some(MenuPage::Video));
    }

    #[test]
    fn toggle_resolution_dropdown_opens_dropdown_for_multi_choice_option() {
        let menu_entity = test_menu_entity();
        let mut menu_stack = MenuStack::new(MenuPage::Options);
        menu_stack.push(MenuPage::Video);
        let mut selectable_menu = test_selectable_menu();
        let mut settings = VideoSettingsState::default();
        settings.initialized = true;
        settings.pending = default_video_settings();
        let mut dropdown_state = DropdownLayerState::default();
        let mut navigation_state = MenuNavigationState::default();

        let result = reduce_menu_command(
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
            result.open_dropdown.map(|(row, _)| row),
            Some(VIDEO_RESOLUTION_OPTION_INDEX)
        );
        assert!(!result.close_menu);
    }

    #[test]
    fn open_main_menu_options_overlay_sets_overlay_flag() {
        let menu_entity = test_menu_entity();
        let mut menu_stack = MenuStack::new(MenuPage::PauseRoot);
        let mut selectable_menu = test_selectable_menu();
        let mut settings = VideoSettingsState::default();
        let mut dropdown_state = DropdownLayerState::default();
        let mut navigation_state = MenuNavigationState::default();

        let result = reduce_menu_command(
            MenuCommand::OpenMainMenuOptionsOverlay,
            menu_entity,
            menu_stack.current_page(),
            VideoTabKind::Display,
            &mut menu_stack,
            &mut selectable_menu,
            &mut settings,
            &mut dropdown_state,
            &mut navigation_state,
        );

        assert!(result.open_main_menu_options_overlay);
        assert!(!result.advance_to_next_scene);
    }

    #[test]
    fn open_level_select_overlay_sets_overlay_flag() {
        let menu_entity = test_menu_entity();
        let mut menu_stack = MenuStack::new(MenuPage::PauseRoot);
        let mut selectable_menu = test_selectable_menu();
        let mut settings = VideoSettingsState::default();
        let mut dropdown_state = DropdownLayerState::default();
        let mut navigation_state = MenuNavigationState::default();

        let result = reduce_menu_command(
            MenuCommand::OpenLevelSelectOverlay,
            menu_entity,
            menu_stack.current_page(),
            VideoTabKind::Display,
            &mut menu_stack,
            &mut selectable_menu,
            &mut settings,
            &mut dropdown_state,
            &mut navigation_state,
        );

        assert!(result.open_level_select_overlay);
        assert!(!result.advance_to_next_scene);
    }

    #[test]
    fn launch_dilemma_from_level_select_routes_by_build_mode() {
        let menu_entity = test_menu_entity();
        let mut menu_stack = MenuStack::new(MenuPage::PauseRoot);
        let mut selectable_menu = test_selectable_menu();
        let mut settings = VideoSettingsState::default();
        let mut dropdown_state = DropdownLayerState::default();
        let mut navigation_state = MenuNavigationState::default();
        let scene =
            DilemmaScene::Lab0(crate::scenes::dilemma::content::Lab0Dilemma::IncompetentBandit);

        let result = reduce_menu_command(
            MenuCommand::LaunchDilemmaFromLevelSelect(scene),
            menu_entity,
            menu_stack.current_page(),
            VideoTabKind::Display,
            &mut menu_stack,
            &mut selectable_menu,
            &mut settings,
            &mut dropdown_state,
            &mut navigation_state,
        );

        if cfg!(debug_assertions) {
            assert_eq!(result.open_level_select_launch_modal_scene, Some(scene));
            assert_eq!(result.start_single_level_scene, None);
            assert!(!result.close_menu);
        } else {
            assert_eq!(result.start_single_level_scene, Some(scene));
            assert!(result.close_menu);
            assert_eq!(result.open_level_select_launch_modal_scene, None);
        }
    }

    #[test]
    fn start_single_dialogue_requests_scene_start_and_closes_menu() {
        let menu_entity = test_menu_entity();
        let mut menu_stack = MenuStack::new(MenuPage::PauseRoot);
        let mut selectable_menu = test_selectable_menu();
        let mut settings = VideoSettingsState::default();
        let mut dropdown_state = DropdownLayerState::default();
        let mut navigation_state = MenuNavigationState::default();
        let scene = DialogueScene::Lab0(crate::scenes::dialogue::content::Lab0Dialogue::Intro);

        let result = reduce_menu_command(
            MenuCommand::StartSingleDialogue(scene),
            menu_entity,
            menu_stack.current_page(),
            VideoTabKind::Display,
            &mut menu_stack,
            &mut selectable_menu,
            &mut settings,
            &mut dropdown_state,
            &mut navigation_state,
        );

        assert_eq!(result.start_single_dialogue_scene, Some(scene));
        assert!(result.close_menu);
    }

    #[test]
    fn next_scene_command_sets_advance_flag() {
        let menu_entity = test_menu_entity();
        let mut menu_stack = MenuStack::new(MenuPage::PauseRoot);
        let mut selectable_menu = test_selectable_menu();
        let mut settings = VideoSettingsState::default();
        let mut dropdown_state = DropdownLayerState::default();
        let mut navigation_state = MenuNavigationState::default();

        let result = reduce_menu_command(
            MenuCommand::NextScene,
            menu_entity,
            menu_stack.current_page(),
            VideoTabKind::Display,
            &mut menu_stack,
            &mut selectable_menu,
            &mut settings,
            &mut dropdown_state,
            &mut navigation_state,
        );

        assert!(result.advance_to_next_scene);
        assert!(!result.open_main_menu_options_overlay);
    }
}
