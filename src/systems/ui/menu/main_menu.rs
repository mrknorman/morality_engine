use super::*;
use crate::systems::{
    colors::ColorAnchor,
    interaction::InteractionVisualPalette,
};
use crate::{
    data::states::PauseState,
    startup::render::{MainCamera, OffscreenCamera},
    systems::{
        audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
        interaction::{
            InteractionCapture, InteractionCaptureOwner, InteractionGate,
        },
        time::Dilation,
    },
};

#[derive(Clone, Debug)]
pub struct MainMenuEntry {
    pub name: String,
    pub label: String,
    pub y: f32,
    pub command: MenuCommand,
}

#[derive(Component)]
pub struct MainMenuOptionList;

pub fn resolve_main_menu_command_id(command_id: &str) -> Result<MenuCommand, String> {
    match command_id {
        "enter_game" => Ok(MenuCommand::NextScene),
        "open_options" => Ok(MenuCommand::OpenMainMenuOptionsOverlay),
        "exit_desktop" => Ok(MenuCommand::ExitApplication),
        _ => Err(format!("unknown main menu command `{command_id}`")),
    }
}

pub fn spawn_main_menu_option_list(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    translation: Vec3,
    options: &[MainMenuEntry],
    palette: InteractionVisualPalette,
) -> Entity {
    let menu_entity = system_menu::spawn_selectable_root(
        commands,
        asset_server,
        "menu_selectable_list",
        translation,
        SystemMenuSounds::Switch,
        SelectableMenu::new(
            0,
            vec![KeyCode::ArrowUp],
            vec![KeyCode::ArrowDown],
            vec![KeyCode::Enter, KeyCode::ArrowRight],
            true,
        )
        .with_click_activation(SelectableClickActivation::HoveredOnly),
    );

    commands.entity(menu_entity).insert((
        Name::new("main_menu_selectable_list"),
        MainMenuOptionList,
        MenuRoot {
            host: MenuHost::Main,
            gate: InteractionGate::GameplayOnly,
        },
        MenuStack::new(MenuPage::PauseRoot),
    ));

    commands.entity(menu_entity).with_children(|parent| {
        for (index, option) in options.iter().enumerate() {
            let option_entity = system_menu::spawn_option(
                parent,
                option.label.clone(),
                0.0,
                option.y,
                menu_entity,
                index,
                system_menu::SystemMenuOptionVisualStyle::default().without_selection_indicators(),
            );

            let mut child_commands = parent.commands();
            child_commands.entity(option_entity).insert((
                Name::new(option.name.clone()),
                Clickable::new(vec![SystemMenuActions::Activate]),
                MenuOptionCommand(option.command.clone()),
                system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click),
                TextColor(palette.idle_color),
                ColorAnchor(palette.idle_color),
                palette,
            ));
        }
    });

    menu_entity
}

fn menu_camera_center(
    offscreen_camera_query: &Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: &Query<&GlobalTransform, With<MainCamera>>,
) -> Option<Vec3> {
    if let Ok(camera) = offscreen_camera_query.single() {
        Some(camera.translation())
    } else if let Ok(camera) = main_camera_query.single() {
        Some(camera.translation())
    } else {
        None
    }
}

pub(super) fn sync_main_menu_options_overlay_position(
    offscreen_camera_query: Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_query: Query<&GlobalTransform, With<MainCamera>>,
    mut overlay_query: Query<&mut Transform, With<MainMenuOptionsOverlay>>,
) {
    let Some(camera_translation) =
        menu_camera_center(&offscreen_camera_query, &main_camera_query)
    else {
        return;
    };

    for mut overlay_transform in &mut overlay_query {
        overlay_transform.translation.x = camera_translation.x;
        overlay_transform.translation.y = camera_translation.y;
    }
}

pub(super) fn play_main_menu_navigation_sound(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    menu_query: Query<
        (
            Entity,
            &SelectableMenu,
            &TransientAudioPallet<SystemMenuSounds>,
            Option<&InteractionGate>,
        ),
        With<MainMenuOptionList>,
    >,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    system_menu::play_navigation_sound_owner_scoped(
        &mut commands,
        &keyboard_input,
        pause_state.as_ref(),
        &capture_query,
        &menu_query,
        &mut audio_query,
        SystemMenuSounds::Switch,
        dilation.0,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_main_menu_command_id_maps_known_commands() {
        assert_eq!(
            resolve_main_menu_command_id("enter_game").expect("enter_game should resolve"),
            MenuCommand::NextScene
        );
        assert_eq!(
            resolve_main_menu_command_id("open_options").expect("open_options should resolve"),
            MenuCommand::OpenMainMenuOptionsOverlay
        );
        assert_eq!(
            resolve_main_menu_command_id("exit_desktop").expect("exit_desktop should resolve"),
            MenuCommand::ExitApplication
        );
    }

    #[test]
    fn resolve_main_menu_command_id_rejects_unknown_commands() {
        let error = resolve_main_menu_command_id("unknown").expect_err("unknown should fail");
        assert!(error.contains("unknown main menu command"));
    }
}
