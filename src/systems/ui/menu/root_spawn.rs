use super::*;

pub fn spawn_menu_root(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    host: MenuHost,
    name: &str,
    translation: Vec3,
    initial_page: MenuPage,
    gate: InteractionGate,
) -> Entity {
    let menu_entity = system_menu::spawn_root(
        commands,
        asset_server,
        name,
        translation,
        SystemMenuSounds::Switch,
    );
    commands.entity(menu_entity).insert((
        MenuRoot { host, gate },
        MenuStack::new(initial_page),
        gate,
    ));
    let click_activation = if host == MenuHost::Debug {
        SelectableClickActivation::HoveredOnly
    } else {
        SelectableClickActivation::SelectedOnAnyClick
    };
    commands.entity(menu_entity).insert(
        MenuSurface::new(menu_entity)
            .with_layer(UiLayerKind::Base)
            .with_click_activation(click_activation),
    );

    spawn_page_content(commands, asset_server, menu_entity, initial_page, gate);

    if host == MenuHost::Pause {
        commands.entity(menu_entity).with_children(|parent| {
            parent.spawn((
                Name::new("pause_menu_music"),
                PauseMenuAudio,
                AudioPlayer::<AudioSource>(asset_server.load(PAUSE_MENU_MUSIC_PATH)),
                PlaybackSettings {
                    volume: Volume::Linear(0.25),
                    ..continuous_audio()
                },
            ));
        });
    }

    menu_entity
}
