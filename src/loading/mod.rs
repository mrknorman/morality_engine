use std::vec;
use bevy::prelude::*;
use crate::{
    audio::{
        ContinuousAudio, 
        ContinuousAudioBundle,
        ContinuousAudioPallet, 
        MusicAudio, 
        NarrationAudio, 
        OneShotAudio, 
        OneShotAudioBundle,
        OneShotAudioPallet,
        NarrationAudioConfig
    }, game_states::{
        GameState, 
        MainState,
        StateVector, 
        SubState
    }, interaction::{
        InputAction,
        InteractionPlugin, 
        Clickable
    }, io::{
        BottomAnchor, IOPlugin
    }, text::TextButtonBundle, timing::{
        TimerPallet, TimingPlugin
    }
};

mod loading_bar;
use loading_bar::LoadingBarBundle;

pub struct LoadingPlugin;
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Loading), setup_loading
        ).insert_resource(
            NextStateButton::empty()
        )
        .add_systems(
            Update,
            (
                spawn_delayed_children,
                update_button_text,
                LoadingBarBundle::fill,
            )
                .run_if(in_state(GameState::Loading)),
        );

        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }
        if !app.is_plugin_added::<TimingPlugin>() {
            app.add_plugins(TimingPlugin);
        }
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
        }
    }
}

#[derive(Resource)]
struct NextStateButton{
    entity : Option<Entity>
}

impl NextStateButton {
    fn empty() -> Self {
        Self {
            entity : None
        }
    }
}

#[derive(Component)]
pub struct LoadingRoot;

pub fn setup_loading(
        mut commands: Commands,
        asset_server: Res<AssetServer>
    ) {
    
    commands.spawn((
        LoadingRoot,
        StateScoped(GameState::Loading),
        TimerPallet::new(
            vec![
                ("music".to_string(), 2.0),
                ("narration".to_string(), 3.0),
                ("post_narration".to_string(), 10.0),
                ("button".to_string(), 5.0)
            ]
        ),
        TransformBundle::from_transform(
            Transform::from_xyz(0.0, 0.0, 0.0)
        ),
        VisibilityBundle::default(),
        ContinuousAudioPallet::new(
            vec![
                (
                    "hum".to_string(),
                    ContinuousAudio::new(
                        &asset_server, 
                        "./sounds/hum.ogg", 
                        0.1
                    ),
                ),
                (
                    "office".to_string(),
                    ContinuousAudio::new(
                        &asset_server, 
                        "./sounds/office.ogg", 
                        0.5
                    ),
                )
            ]
        ),
        OneShotAudioPallet::new(
            vec![
                OneShotAudio::new(
                    &asset_server, 
                    "./sounds/startup_beep.ogg",
                    true,
                    1.0
                )
            ]
        )
    )).with_children(
        |parent| {
            parent.spawn(LoadingBarBundle::new(
                "text/loading_messages/loading.json"
            ));
        }
    );
}

pub fn spawn_delayed_children(
    mut commands: Commands,
    loading_query: Query<(Entity, &TimerPallet), With<LoadingRoot>>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>
) {

    for (entity, timers) in loading_query.iter() {
        // Handle narration timer
        if let Some(narration_timer) = timers.timers.get("narration") {
            if narration_timer.just_finished() {
                commands.entity(entity).with_children(
                    |parent| {
                    parent.spawn((
                        NarrationAudio,
                        OneShotAudioBundle::new(
                            &asset_server,
                            "./sounds/intro_lilly_elvenlabs.ogg",
                            false,
                            1.0,
                        )
                    ));
                });
            }
        } else {
            warn!("Entity {:?} is missing the 'narration' timer", entity);
        }

        if let Some(music_timer) = timers.timers.get("music") {
            if music_timer.just_finished() {
                commands.entity(entity).with_children(
                    |parent| {
                    parent.spawn((
                        MusicAudio,
                        ContinuousAudioBundle::new(
                            &asset_server,
                            "./music/a_friend.ogg",
                            0.3,
                        )
                    ));
                });
            }
        } else {
            warn!("Entity {:?} is missing the 'music' timer", entity);
        }

        // Handle button timer
        if let Some(button_timer) = timers.timers.get("button") {
            if button_timer.just_finished() {
                let next_state_vector = StateVector::new(
                    Some(MainState::InGame),
                    Some(GameState::Dialogue),
                    Some(SubState::Intro),
                );

                let button_distance = 100.0;
                let window = windows.get_single().unwrap();
                let screen_height = window.height();
                let button_y = -screen_height / 2.0 + button_distance; 
                let button_translation: Vec3 = Vec3::new(0.0, button_y, 1.0);

                let mut child_entity_id = None;
                commands.entity(entity).with_children(|parent| {
                    let child_entity = parent.spawn((
                        BottomAnchor::new(button_distance),
                        TextButtonBundle::new(
                            &asset_server,
                            vec![
                                InputAction::PlaySound(String::from("click")),
                                InputAction::ChangeState(next_state_vector),
                            ],
                            vec![KeyCode::Enter],
                            "[Click Here or Press Enter to Ignore Your Inner Dialogue]",
                            button_translation,
                        ),
                    ))
                    .id(); // Capture the entity ID of the spawned child

                    // Assign the child entity ID to the mutable variable
                    child_entity_id = Some(child_entity);
                });

                // Insert the child entity ID as a resource outside the closure
                if let Some(child_entity) = child_entity_id {
                    commands.insert_resource(NextStateButton{entity : Some(child_entity)});
                }
            }
        } else {
            warn!("Entity {:?} is missing the 'button' timer", entity);
        }
    }
}

fn update_button_text(
    next_state_button : Res<NextStateButton>,
    narration_audio_config : Res<NarrationAudioConfig>,
    mut loading_query: Query<(&mut TimerPallet), With<LoadingRoot>>,
    mut text_query : Query<(&mut Text, &mut Clickable)>
) {

    if let Some(entity) = next_state_button.entity {
        if let Ok((text, clickable)) = text_query.get_mut(entity) {
            if narration_audio_config.finished() {

                TextButtonBundle::change_text(
                    "[Click to Continue After Patiently Absorbing the Dialogue]",
                    text,
                    clickable
                );
            }
        }
    }

    for timers in loading_query.iter_mut() {
        // Handle narration timer
        if let Some(narration_timer) = timers.timers.get("post_narration") {

        }
    }
}