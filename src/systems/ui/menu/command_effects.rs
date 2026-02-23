use super::command_reducer::{MenuReducerResult, MenuStateTransition};
use super::debug_showcase;
use super::level_select;
use super::modal_flow::{spawn_apply_confirm_modal, spawn_exit_unsaved_modal};
use super::*;
use crate::data::stats::GameStats;
use crate::scenes::dilemma::content::DilemmaScene;
use crate::scenes::runtime::SceneNavigator;

const MAIN_MENU_OVERLAY_DIM_ALPHA: f32 = 0.8;
const MAIN_MENU_OVERLAY_DIM_SIZE: f32 = 6000.0;
const MAIN_MENU_OVERLAY_DIM_Z: f32 = -5.0;

fn request_application_exit(
    primary_window: &Query<
        Entity,
        (
            With<bevy::window::Window>,
            With<PrimaryWindow>,
            Without<ClosingWindow>,
        ),
    >,
    close_requests: &mut MessageWriter<WindowCloseRequested>,
    app_exit: &mut MessageWriter<AppExit>,
) {
    if let Ok(window) = primary_window.single() {
        close_requests.write(WindowCloseRequested { window });
    } else {
        app_exit.write(AppExit::Success);
    }
}

fn handle_apply_video_settings_command(
    commands: &mut Commands,
    menu_entity: Entity,
    menu_root: &MenuRoot,
    asset_server: &Res<AssetServer>,
    settings: &mut VideoSettingsState,
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut VideoDropdownVisibilityQuery,
    crt_settings: &mut CrtSettings,
    screen_shake: &mut ScreenShakeState,
    main_camera_query: &mut Query<
        (
            &mut Bloom,
            &mut Tonemapping,
            &mut DebandDither,
            &mut Fxaa,
            &mut ContrastAdaptiveSharpening,
            &mut ChromaticAberration,
            &mut Msaa,
        ),
        With<MainCamera>,
    >,
    window_exit: &mut WindowExitContext,
) {
    if !settings.initialized || !video_settings_dirty(settings) {
        return;
    }

    close_dropdowns_for_menu(menu_entity, dropdown_state, dropdown_query);
    settings.revert_snapshot = Some(settings.saved);
    if let Ok(mut window) = window_exit.primary_window_queries.p1().single_mut() {
        apply_snapshot_to_window(&mut window, settings.pending);
    }
    apply_snapshot_to_post_processing(
        settings.pending,
        crt_settings,
        screen_shake,
        main_camera_query,
    );
    settings.apply_timer = Some(Timer::from_seconds(30.0, TimerMode::Once));
    spawn_apply_confirm_modal(commands, menu_entity, asset_server, menu_root.gate);
}

fn handle_exit_application_command(window_exit: &mut WindowExitContext) {
    let primary_window = window_exit.primary_window_queries.p0();
    request_application_exit(
        &primary_window,
        &mut window_exit.close_requests,
        &mut window_exit.app_exit,
    );
}

fn handle_open_main_menu_options_overlay_command(
    commands: &mut Commands,
    menu_root: &MenuRoot,
    asset_server: &Res<AssetServer>,
    existing_overlay_query: &Query<(), With<MainMenuOptionsOverlay>>,
    offscreen_camera_query: &Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_transform_query: &Query<&GlobalTransform, With<MainCamera>>,
) {
    if menu_root.host != MenuHost::Main || !existing_overlay_query.is_empty() {
        return;
    }

    let Some(camera_translation) =
        super::camera::menu_camera_center(offscreen_camera_query, main_camera_transform_query)
    else {
        return;
    };

    let overlay_entity = spawn_menu_root(
        commands,
        asset_server,
        MenuHost::Main,
        "main_menu_options_overlay",
        Vec3::new(
            camera_translation.x,
            camera_translation.y,
            system_menu::MENU_Z,
        ),
        MenuPage::Options,
        UiInputPolicy::CapturedOnly,
    );
    commands
        .entity(overlay_entity)
        .insert((MainMenuOptionsOverlay, DespawnOnExit(MainState::Menu)));

    commands.entity(overlay_entity).with_children(|parent| {
        parent.spawn((
            Name::new("main_menu_options_dimmer"),
            UiInputCaptureToken,
            UiInputCaptureOwner::new(overlay_entity),
            Sprite::from_color(
                Color::srgba(0.0, 0.0, 0.0, MAIN_MENU_OVERLAY_DIM_ALPHA),
                Vec2::splat(MAIN_MENU_OVERLAY_DIM_SIZE),
            ),
            Transform::from_xyz(0.0, 0.0, MAIN_MENU_OVERLAY_DIM_Z),
        ));
    });
}

fn handle_next_scene_command(
    scene_queue: &mut ResMut<SceneQueue>,
    next_main_state: &mut ResMut<NextState<MainState>>,
    next_game_state: &mut ResMut<NextState<GameState>>,
    next_sub_state: &mut ResMut<NextState<DilemmaPhase>>,
) {
    SceneNavigator::next_state_vector_or_fallback(scene_queue)
        .set_state(next_main_state, next_game_state, next_sub_state);
}

fn handle_start_single_level_command(
    scene: DilemmaScene,
    stats: &mut ResMut<GameStats>,
    scene_queue: &mut ResMut<SceneQueue>,
    next_main_state: &mut ResMut<NextState<MainState>>,
    next_game_state: &mut ResMut<NextState<GameState>>,
    next_sub_state: &mut ResMut<NextState<DilemmaPhase>>,
) {
    **stats = GameStats::default();
    scene_queue.configure_single_level(scene);
    handle_next_scene_command(
        scene_queue,
        next_main_state,
        next_game_state,
        next_sub_state,
    );
}

pub(super) fn apply_menu_reducer_result(
    result: MenuReducerResult,
    menu_entity: Entity,
    menu_root: &MenuRoot,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    settings: &mut VideoSettingsState,
    dropdown_state: &mut DropdownLayerState,
    dropdown_query: &mut VideoDropdownVisibilityQuery,
    crt_settings: &mut CrtSettings,
    screen_shake: &mut ScreenShakeState,
    main_camera_query: &mut Query<
        (
            &mut Bloom,
            &mut Tonemapping,
            &mut DebandDither,
            &mut Fxaa,
            &mut ContrastAdaptiveSharpening,
            &mut ChromaticAberration,
            &mut Msaa,
        ),
        With<MainCamera>,
    >,
    window_exit: &mut WindowExitContext,
    next_pause_state: &mut ResMut<NextState<PauseState>>,
    next_main_state: &mut ResMut<NextState<MainState>>,
    next_game_state: &mut ResMut<NextState<GameState>>,
    next_sub_state: &mut ResMut<NextState<DilemmaPhase>>,
    scene_queue: &mut ResMut<SceneQueue>,
    stats: &mut ResMut<GameStats>,
    existing_overlay_query: &Query<(), With<MainMenuOptionsOverlay>>,
    existing_level_select_overlay_query: &Query<(), With<level_select::LevelSelectOverlay>>,
    offscreen_camera_query: &Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_transform_query: &Query<&GlobalTransform, With<MainCamera>>,
    showcase_root_query: &Query<Entity, With<debug_showcase::DebugUiShowcaseRoot>>,
    dirty_menus: &mut HashSet<Entity>,
    closed_menus: &mut HashSet<Entity>,
    pending_dropdown_open: &mut Vec<(Entity, usize, usize)>,
) {
    if result.close_dropdown_for_menu {
        close_dropdowns_for_menu(menu_entity, dropdown_state, dropdown_query);
    }
    if let Some((row, selected_index)) = result.open_dropdown {
        pending_dropdown_open.push((menu_entity, row, selected_index));
    }
    if result.spawn_exit_unsaved_modal {
        spawn_exit_unsaved_modal(commands, menu_entity, asset_server, menu_root.gate);
    }
    if result.open_debug_ui_showcase {
        debug_showcase::rebuild_debug_ui_showcase(commands, showcase_root_query);
    }
    if result.open_main_menu_options_overlay {
        handle_open_main_menu_options_overlay_command(
            commands,
            menu_root,
            asset_server,
            existing_overlay_query,
            offscreen_camera_query,
            main_camera_transform_query,
        );
    }
    if result.open_level_select_overlay {
        level_select::spawn_level_select_overlay(
            commands,
            menu_root,
            asset_server,
            existing_level_select_overlay_query,
            offscreen_camera_query,
            main_camera_transform_query,
        );
    }
    if result.advance_to_next_scene {
        handle_next_scene_command(
            scene_queue,
            next_main_state,
            next_game_state,
            next_sub_state,
        );
    }
    if let Some(scene) = result.start_single_level_scene {
        handle_start_single_level_command(
            scene,
            stats,
            scene_queue,
            next_main_state,
            next_game_state,
            next_sub_state,
        );
    }
    if result.apply_video_settings {
        handle_apply_video_settings_command(
            commands,
            menu_entity,
            menu_root,
            asset_server,
            settings,
            dropdown_state,
            dropdown_query,
            crt_settings,
            screen_shake,
            main_camera_query,
            window_exit,
        );
    }
    if let Some(state_transition) = result.state_transition {
        match state_transition {
            MenuStateTransition::Pause(state) => next_pause_state.set(state),
            MenuStateTransition::Main(state) => next_main_state.set(state),
            MenuStateTransition::PauseAndMain(pause, main) => {
                next_pause_state.set(pause);
                next_main_state.set(main);
            }
        }
    }
    if result.exit_application {
        handle_exit_application_command(window_exit);
    }
    if result.dirty_menu {
        dirty_menus.insert(menu_entity);
    }
    if result.close_menu {
        closed_menus.insert(menu_entity);
    }
}
