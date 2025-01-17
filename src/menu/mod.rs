use bevy::{
    audio::Volume, prelude::*
};
use enum_map::{
    Enum, 
    enum_map
};

use crate::{
    ascii_fonts::{
        AsciiPlugin, 
        AsciiString
    }, 
    audio::{
        continuous_audio, BackgroundAudio, ContinuousAudioPallet, DilatableAudio, MusicAudio, TransientAudio, TransientAudioPallet
    }, 
    background::{
        Background, 
        BackgroundPlugin
    }, 
    colors::{
        DIM_BACKGROUND_COLOR, 
        MENU_COLOR
    }, 
    common_ui::NextButton, 
    game_states::{
        MainState, 
        StateVector
    }, 
    interaction::{ 
        ActionPallet, InputAction, InteractionPlugin
    }, 
    io::IOPlugin, 
    text::{
        TextButton, 
        TextRaw
    }, 
    track::Track, 
    train::{
        Train, 
        TrainPlugin, 
        STEAM_TRAIN
    }
};

#[derive(Enum, Debug, Clone, Copy)]
enum MenuButtonActions {
    EnterGame
}

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
        if !app.is_plugin_added::<AsciiPlugin>() {
            app.add_plugins(AsciiPlugin);
        }
        if !app.is_plugin_added::<BackgroundPlugin>() {
			app.add_plugins(BackgroundPlugin);
		}
    }
}

fn setup_menu(
        mut commands: Commands, 
        asset_server: Res<AssetServer>
    ) {

    let menu_translation : Vec3 = Vec3::new(0.0, 0.0, 0.0);
    let title_translation : Vec3 = Vec3::new(-380.0, 225.0, 1.0);
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
            Visibility::default(),
            Transform::from_translation(menu_translation),
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
                            },
                            None
                        ),
                        (
                            "office".to_string(),
                            AudioPlayer::<AudioSource>(asset_server.load(
                                "./sounds/office.ogg"
                            )),
                            PlaybackSettings{
                                volume : Volume::new(0.5),
                                ..continuous_audio()
                            },
                            Some(DilatableAudio)
                        )
                    ]
                )
            ));

            parent.spawn((         
                Background::load_from_json(
                    "text/backgrounds/desert.json",	
                    0.000001,
                    -0.5
                ),
                TextColor(DIM_BACKGROUND_COLOR)
                )
            );
            parent.spawn((
                Track::new(600),
                TextColor(DIM_BACKGROUND_COLOR),
                Transform::from_translation(track_translation)
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
                    AsciiString("THE TROLLEY\n  ALGORITHM".to_string()),
                    TextColor(MENU_COLOR),
                    Transform::from_translation(title_translation)
                )
            );

            parent.spawn(
                (                    
                    Train::init(
                        &asset_server,
                        STEAM_TRAIN,
                        0.0
                    ),
                    Transform::from_translation(train_translation),
                    TextColor(MENU_COLOR),
                )  
            );

            parent.spawn(
                (
                    TextRaw,
                    TextColor(MENU_COLOR),
                    Text2d::new("A game by Michael Norman"),
                    Transform::from_translation(signature_translation)
                )
            );
            parent.spawn(
                (
                    NextButton,
                    TextColor(MENU_COLOR),
                    TextButton::new(
                        vec![
                            InputAction::PlaySound(String::from("click")),
                            InputAction::ChangeState(next_state_vector.clone())
                        ],
                        vec![KeyCode::Enter],
                        "[Click Here or Press Enter to Begin]",
                    ),
                    ActionPallet::<MenuButtonActions>(
                        enum_map!(
                            MenuButtonActions::EnterGame => vec![
                                InputAction::PlaySound(String::from("click")),
                                InputAction::ChangeState(next_state_vector.clone())
                            ]
                        )
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
                            ],
                            Some(DilatableAudio)
                        )]
                    ),
                )
            );
        }
    );
}