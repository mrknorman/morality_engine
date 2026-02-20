use super::*;
use crate::systems::{
    colors::ColorAnchor,
    interaction::InteractionVisualPalette,
};

#[derive(Clone, Debug)]
pub struct MainMenuEntry {
    pub name: String,
    pub label: String,
    pub y: f32,
    pub command: MenuCommand,
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
