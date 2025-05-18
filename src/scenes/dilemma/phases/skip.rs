use std::{
    path::PathBuf, 
};

use bevy::prelude::*;


use crate::{
    data::states::{
            DilemmaPhase, GameState, MainState, StateVector
        }, entities::train::Train, systems::{
        audio::{
            OneShotAudio, 
            OneShotAudioPallet
        },
        motion::PointToPointTranslation, time::Dilation
    }
};

pub struct DilemmaSkipPlugin;
impl Plugin for DilemmaSkipPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			OnEnter(DilemmaPhase::Skip),
			(
                DilemmaSkipScene::setup
            )
			.run_if(in_state(GameState::Dilemma)),
		)
		.add_systems(
			Update,
			DilemmaSkipScene::in_position
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::Skip)),
		);
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DilemmaSkipScene;

impl DilemmaSkipScene{
    fn setup(
        mut commands : Commands,
        asset_server: Res<AssetServer>,
        mut dilation: ResMut<Dilation>
    ) {      
        commands.spawn((
            Self,
            StateScoped(DilemmaPhase::Skip),
            children![
                OneShotAudioPallet::new(
                    vec![
                        OneShotAudio {
                            source : asset_server.load(
                                PathBuf::from("./audio/effects/fast_forward.ogg")
                            ),
                            persistent : false,
                            volume :1.0,
                            dilatable : false
                        }
                    ]
                )
            ]
        ));

        dilation.0 = 6.0;
    }
    
    fn in_position(
        translation_query : Single<&PointToPointTranslation, With<Train>>,
        mut dilation: ResMut<Dilation>,
        mut next_main_state: ResMut<NextState<MainState>>,
        mut next_game_state: ResMut<NextState<GameState>>,
        mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
    ) {
        if translation_query.timer.finished() {
            dilation.0 = 1.0;
            let next_state =StateVector::new(None, None, Some(DilemmaPhase::Consequence));
            next_state.set_state(                        
                &mut next_main_state,
                &mut next_game_state,
                &mut next_sub_state
            );
        }       
    }
}