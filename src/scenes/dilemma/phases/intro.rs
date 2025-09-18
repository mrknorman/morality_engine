use bevy::{
	prelude::*,
	audio::Volume,
};
use enum_map::{
    Enum,
    enum_map
};

use crate::{
    data::states::{
        DilemmaPhase,
		GameState, 
		MainState, 
		StateVector
    }, entities::text::TextButton, scenes::dilemma::{DilemmaSounds, dilemma::{Dilemma, DilemmaStage}, lever::{Lever, LeverState}}, style::common_ui::NextButton, systems::{
		audio::{
			NarrationAudio, TransientAudio, TransientAudioPallet, one_shot_audio 
    	}, colors::ColorTranslation, inheritance::BequeathTextColor, interaction::{
			ActionPallet, 
			InputAction
		}, motion::Bounce, scheduling::{
			TimerConfig, 
			TimerPallet, 
			TimerStartCondition
		}
	}
};

pub struct DilemmaIntroPlugin;
impl Plugin for DilemmaIntroPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			OnEnter(DilemmaPhase::Intro), 
			DilemmaIntroScene::setup
			.run_if(in_state(GameState::Dilemma))
		)
		.add_systems(
			Update,
			DilemmaIntroScene::spawn_delayed_children
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::Intro))
		)
		.add_systems(
			OnExit(DilemmaPhase::Intro),
			DilemmaIntroScene::on_exit
		);
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaIntroActions {
    StartDilemma
}

impl std::fmt::Display for DilemmaIntroActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaIntroEvents {
    Narration,
	Button
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct DilemmaIntroScene;

impl DilemmaIntroScene {

	fn setup(
		mut commands : Commands,
		stage: Res<DilemmaStage>,
	) {
		let state= match stage.default_option {
			None => LeverState::Random,
			Some(ref option) if *option == 0 => LeverState::Left,
			Some(_) => LeverState::Right
		};
		commands.insert_resource(Lever(state));
		
		commands.spawn(
			(
				DilemmaIntroScene,
				StateScoped(DilemmaPhase::Intro),
				TimerPallet::new(
					vec![
						(
							DilemmaIntroEvents::Narration,
							TimerConfig::new(
								TimerStartCondition::Immediate, 
								1.0,
								None
							)
						),
						(
							DilemmaIntroEvents::Button,
							TimerConfig::new(
								TimerStartCondition::Immediate, 
								2.0,
								None
							)
						)
					]
				)
			)
		);
	}

	fn spawn_delayed_children(
		mut commands: Commands,
		dilemma : Res<Dilemma>,
		loading_query: Query<(Entity, &TimerPallet<DilemmaIntroEvents>), With<DilemmaIntroScene>>,
		asset_server: Res<AssetServer>
	) {
		for (entity, timers) in loading_query.iter() {

			if timers.0[DilemmaIntroEvents::Narration].just_finished() {
				commands.entity(entity).with_children(
					|parent| {
						parent.spawn((
							NarrationAudio,
							AudioPlayer::<AudioSource>(asset_server.load(
								dilemma.narration_path.clone(),
							)),
							PlaybackSettings{
								paused : false,
								volume : Volume::Linear(1.0),
								..one_shot_audio()
							}
						));
				});
			}

			// Handle narration timer
			if timers.0[DilemmaIntroEvents::Button].just_finished() {          
				commands.entity(entity).with_children(|parent| {
					parent.spawn((
						NextButton,
						TextButton::new(
							vec![DilemmaIntroActions::StartDilemma],
							vec![KeyCode::Enter],
							"[ Click here or Press Enter to Test Your Morality ]",
						),
						ActionPallet::<DilemmaIntroActions, DilemmaSounds>(
							enum_map!(
								DilemmaIntroActions::StartDilemma => vec![
									InputAction::PlaySound(DilemmaSounds::Click),
									InputAction::ChangeState(
										StateVector::new(
											Some(MainState::InGame),
											Some(GameState::Dilemma),
											Some(DilemmaPhase::DilemmaTransition),
										)
									),
									InputAction::Despawn(None)
									]
								)
						),
						TransientAudioPallet::new(
							vec![(
								DilemmaSounds::Click,
								vec![
									TransientAudio::new(
										asset_server.load("./audio/effects/mech_click.ogg"), 
										0.1, 
										true,
										1.0,
										true
									)
								]
							)]
						)
					));
				});
			}
		}
	}

	fn on_exit(
        mut commands : Commands,
        mut title_query: Query<(Entity, &mut ColorTranslation)>
    ) {
		for (entity, mut color) in title_query.iter_mut() {
			commands.entity(entity).insert(BequeathTextColor);
			commands.entity(entity).remove::<Bounce>();

			color.start()
		}
	}
}