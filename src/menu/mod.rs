use bevy::prelude::*;

use crate::{
    audio::{
        ContinuousAudio,
        ContinuousAudioBundle,
        ContinuousAudioPallet,
        MusicAudio,
        BackgroundAudio
    }, 
    game_states::{
        MainState, 
        StateVector
    }, 
    interaction::{
        InteractionPlugin,
        InputAction
    }, 
    text::{
        TextButtonBundle, 
        TextRawBundle, 
        TextTitleBundle
    }, 
    track::TrackBundle, 
    train::{
        TrainPlugin,
        TrainBundle, 
        STEAM_TRAIN
    },
    io::IOPlugin,
    common_ui::NextButtonBundle
};

pub struct MenuScreenPlugin;
impl Plugin for MenuScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MainState::Menu), setup_menu
        );
        if !app.is_plugin_added::<TrainPlugin>() {
            app.add_plugins(TrainPlugin);
        }
        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
        }
    }
}

fn setup_menu(
        mut commands: Commands, 
        asset_server: Res<AssetServer>,
        windows: Query<&Window>
    ) {
    
    let text = r#"
 ___      ___     ______     _______        __      ___        __  ___________  ___  ___       _______  _____  ___    _______   __    _____  ___    _______  
|"  \    /"  |   /    " \   /"      \      /""\    |"  |      |" \("     _   ")|"  \/"  |     /"     "|(\"   \|"  \  /" _   "| |" \  (\"   \|"  \  /"     "| 
 \   \  //   |  // ____  \ |:        |    /    \   ||  |      ||  |)__/  \\__/  \   \  /     (: ______)|.\\   \    |(: ( \___) ||  | |.\\   \    |(: ______) 
 /\\  \/.    | /  /    ) :)|_____/   )   /' /\  \  |:  |      |:  |   \\_ /      \\  \/       \/    |  |: \.   \\  | \/ \      |:  | |: \.   \\  | \/    |   
|: \.        |(: (____/ //  //      /   //  __'  \  \  |___   |.  |   |.  |      /   /        // ___)_ |.  \    \. | //  \ ___ |.  | |.  \    \. | // ___)_  
|.  \    /:  | \        /  |:  __   \  /   /  \\  \( \_|:  \  /\  |\  \:  |     /   /        (:      "||    \    \ |(:   _(  _|/\  |\|    \    \ |(:      "| 
|___|\__/|___|  \"_____/   |__|  \___)(___/    \___)\_______)(__\_|_)  \__|    |___/          \_______) \___|\____\) \_______)(__\_|_)\___|\____\) \_______) 
"#;

    let menu_translation : Vec3 = Vec3::new(0.0, 0.0, 0.0);
    let title_translation : Vec3 = Vec3::new(0.0, 150.0, 1.0);
    let train_translation: Vec3 = Vec3::new(63.0, 00.0, 1.0);
    let track_displacement: Vec3 = Vec3::new(-57.0, -30.0, 1.0);
    let track_translation: Vec3 = train_translation + track_displacement;
    let signature_translation : Vec3 = Vec3::new(0.0, -40.0, 1.0);

    let next_state_vector = StateVector::new(
        Some(MainState::InGame),
        None,
        None
    );

    commands.spawn(
        (
            StateScoped(MainState::Menu),
            Transform::from_translation(menu_translation),
            Visibility::default()
        )
    ).with_children(
        |parent| {
            parent.spawn((
                BackgroundAudio,
                ContinuousAudioPallet::new(
                    vec![
                        (
                            "static".to_string(),
                            ContinuousAudio::new(
                                &asset_server, 
                                "./sounds/static.ogg", 
                                0.1,
                                false
                            ),
                        ),
                        (
                            "office".to_string(),
                            ContinuousAudio::new(
                                &asset_server, 
                                "./sounds/office.ogg", 
                                1.0,
                                false
                            ),
                        )
                    ]
                )
            ));

            parent.spawn((
                MusicAudio,
                ContinuousAudioBundle::new(
                    &asset_server, 
                    "./music/the_last_decision.ogg", 
                    0.3,
                    false
                )
            ));

            parent.spawn(
                TextTitleBundle::new(
                    text,
                    title_translation, 
                )
            );
            parent.spawn(
                TrainBundle::new(
                    &asset_server,
                    STEAM_TRAIN,
                    train_translation,
                    0.0
                )
            );
            parent.spawn(
                TrackBundle::new(
                    50, 
                    track_translation
                )
            );
            parent.spawn(
                TextRawBundle::new(
                    "A game by Michael Norman",
                    signature_translation
                )
            );
            parent.spawn(
                (
                    NextButtonBundle::new(),
                    TextButtonBundle::new(
                        &asset_server, 
                        vec![
                            InputAction::PlaySound(String::from("click")),
                            InputAction::ChangeState(next_state_vector)
                        ],
                        vec![KeyCode::Enter],
                        "[Click Here or Press Enter to Begin]",
                        NextButtonBundle::translation(&windows)
                    )
                )
            );
        }
    );
}