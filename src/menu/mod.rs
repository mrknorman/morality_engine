use bevy::{
    prelude::*,
    audio::Volume
};

use crate::{
    audio::{
        continuous_audio,
        ContinuousAudioPallet,
        MusicAudio,
        BackgroundAudio,
        TransientAudioPallet,
        TransientAudio
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
        TextButton, 
        TextRaw, 
        TextTitle
    }, 
    track::Track, 
    train::{
        TrainPlugin,
        Train, 
        STEAM_TRAIN
    },
    io::IOPlugin,
    common_ui::NextButton
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
     ___________  __    __    _______     ___________  _______     ______    ___      ___       _______  ___  ___ 
    ("     _   ")/" |  | "\  /"     "|   ("     _   ")/"      \   /    " \  |"  |    |"  |     /"     "||"  \/"  |
     )__/  \\__/(:  (__)  :)(: ______)    )__/  \\__/|:        | // ____  \ ||  |    ||  |    (: ______) \   \  / 
        \\_ /    \/      \/  \/    |         \\_ /   |_____/   )/  /    ) :)|:  |    |:  |     \/    |    \\  \/  
        |.  |    //  __  \\  // ___)_        |.  |    //      /(: (____/ //  \  |___  \  |___  // ___)_   /   /   
        \:  |   (:  (  )  :)(:      "|       \:  |   |:  __   \ \        /  ( \_|:  \( \_|:  \(:      "| /   /    
         \__|    \__|  |__/  \_______)        \__|   |__|  \___) \"_____/    \_______)\_______)\_______)|___/     
                                                                                                                  
              __      ___       _______     ______     _______    __  ___________  __    __   ___      ___            
             /""\    |"  |     /" _   "|   /    " \   /"      \  |" \("     _   ")/" |  | "\ |"  \    /"  |           
            /    \   ||  |    (: ( \___)  // ____  \ |:        | ||  |)__/  \\__/(:  (__)  :) \   \  //   |           
           /' /\  \  |:  |     \/ \      /  /    ) :)|_____/   ) |:  |   \\_ /    \/      \/  /\\  \/.    |           
          //  __'  \  \  |___  //  \ ___(: (____/ //  //      /  |.  |   |.  |    //  __  \\ |: \.        |           
         /   /  \\  \( \_|:  \(:   _(  _|\        /  |:  __   \  /\  |\  \:  |   (:  (  )  :)|.  \    /:  |           
        (___/    \___)\_______)\_______)  \"_____/   |__|  \___)(__\_|_)  \__|    \__|  |__/ |___|\__/|___|           
"#;

    let menu_translation : Vec3 = Vec3::new(0.0, 0.0, 0.0);
    let title_translation : Vec3 = Vec3::new(0.0, 150.0, 1.0);
    let train_translation: Vec3 = Vec3::new(110.0, -35.0, 1.0);
    let track_displacement: Vec3 = Vec3::new(-120.0, -30.0, 1.0);
    let track_translation: Vec3 = train_translation + track_displacement;
    let signature_translation : Vec3 = Vec3::new(0.0, -100.0, 1.0);

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
                            AudioPlayer::<AudioSource>(asset_server.load(
                                "./sounds/static.ogg"
                            )),
                            PlaybackSettings{
                                volume : Volume::new(0.1),
                                ..continuous_audio()
                            }
                        ),
                        (
                            "office".to_string(),
                            AudioPlayer::<AudioSource>(asset_server.load(
                                "./sounds/office.ogg"
                            )),
                            PlaybackSettings{
                                volume : Volume::new(0.5),
                                ..continuous_audio()
                            }
                        )
                    ]
                )
            ));

            parent.spawn((
                MusicAudio,
                AudioPlayer::<AudioSource>(asset_server.load(
                    "./music/the_last_decision.ogg", 
                )),
                PlaybackSettings{
                    volume : Volume::new(0.3),
                    ..continuous_audio()
                }
            ));

            parent.spawn(
                (
                    TextTitle,
                    Text2d::new(text),
                    Transform::from_translation(title_translation)
                )
            );
            parent.spawn(
                Train::init(
                    &asset_server,
                    STEAM_TRAIN,
                    train_translation,
                    0.0
                )
            );
            parent.spawn((
                Track::new(50),
                Transform::from_translation(track_translation)
            ));
            parent.spawn(
                (
                    TextRaw,
                    Text2d::new("A game by Michael Norman"),
                    Transform::from_translation(signature_translation)
                )
            );
            parent.spawn(
                (
                    NextButton,
                    TextButton::new(
                        vec![
                            InputAction::PlaySound(String::from("click")),
                            InputAction::ChangeState(next_state_vector)
                        ],
                        vec![KeyCode::Enter],
                        "[Click Here or Press Enter to Begin]",
                    ),
                    TransientAudioPallet::new(
                        vec![(
                            "click".to_string(),
                            vec![
                                TransientAudio::new(
                                    asset_server.load("sounds/mech_click.ogg"), 
                                    0.1, 
                                    true,
                                    1.0
                                )
                            ]
                        )]
                    ),
                    NextButton::transform(&windows)
                )
            );
        }
    );
}