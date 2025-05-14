use bevy::{
    ecs::{component::HookContext, world::DeferredWorld}, prelude::*, render::primitives::Aabb, sprite::Anchor
};
use enum_map::Enum;
use crate::{
    data::{rng::GlobalRng, states::DilemmaPhase}, entities::text::TextSprite, systems::{
        audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
        motion::Bounce,
        physics::{Friction, Gravity, PhysicsPlugin, Velocity},
        time::Dilation,
    }
};
use super::train::{Train, TrainCarriage};
use rand::Rng;

/// ─────────────────────────────────────────────────────────────
///  General plugin plumbing (unchanged except for imports)
/// ─────────────────────────────────────────────────────────────
#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PersonSystemsActive {
    #[default] False,
    True,
}

pub struct PersonPlugin;

impl Plugin for PersonPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PersonSystemsActive>()
            .add_systems(Update, activate_systems)
            .add_systems(
                Update,
                (
                    PersonSprite::animate,
                    PersonSprite::scream,
                    PersonSprite::alert,
                    PersonSprite::explode,
                    Emoticon::animate,
                )
                    .run_if(in_state(PersonSystemsActive::True))
                    .run_if(
                        in_state(DilemmaPhase::Decision)
                            .or(in_state(DilemmaPhase::Consequence)),
                    ),
            );

        if !app.is_plugin_added::<PhysicsPlugin>() {
            app.add_plugins(PhysicsPlugin);
        }
    }
}

fn activate_systems(
    mut person_state: ResMut<NextState<PersonSystemsActive>>,
    person_query: Query<&PersonSprite>,
) {
    person_state.set(if person_query.is_empty() {
        PersonSystemsActive::False
    } else {
        PersonSystemsActive::True
    });
}

/// ─────────────────────────────────────────────────────────────
///  Constants & helper structs
/// ─────────────────────────────────────────────────────────────
const PERSON: &str = " @ \n/|\\\n/ \\";
const PERSON_IN_DANGER: &str = "\\@/\n | \n/ \\";

const EXCLAMATION: &str = "!";
const NEUTRAL: &str = "    ";

/// Hard-coded layout for the 3 × 3 ASCII art.
const COLS: usize = 3;
const LINES: usize = 3;
const CHAR_WIDTH: f32 = 9.0;
const LINE_HEIGHT: f32 = 16.0;

/// Each visible (or space) glyph in the figure.
#[derive(Component)]
pub struct CharacterSprite {
    row: usize,
    col: usize,
}

impl CharacterSprite {
    fn offset(&self) -> Vec3 {
        // Bottom line sits at y = 0, figure centred on parent’s X axis
        let x = (self.col as f32 - (COLS as f32 - 1.) * 0.5) * CHAR_WIDTH;
        let y = ((LINES - 1 - self.row) as f32) * LINE_HEIGHT;
        Vec3::new(x, y, 0.)
    }
}

fn default_person_anchor() -> Anchor {
    Anchor::BottomCenter
}

/// ─────────────────────────────────────────────────────────────
///  PersonSprite  (parent entity)
/// ─────────────────────────────────────────────────────────────
#[derive(Component)]
#[component(on_insert = PersonSprite::on_insert)]
#[require(Anchor = default_person_anchor(), Gravity, TextSprite, Bounce, Visibility)]
pub struct PersonSprite {
    pub in_danger: bool,
}

impl Default for PersonSprite {
    fn default() -> Self {
        Self { in_danger: false }
    }
}

impl PersonSprite {
    // -----------------------------------------------------------------------------------------
    // Spawning
    // -----------------------------------------------------------------------------------------
    /// Helper: `PersonSprite::spawn(commands, position)` creates the parent and every child glyph.

	fn on_insert(
        mut world: DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {

        for (row, line) in PERSON.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                world.commands().entity(entity).with_children(|p| {
                    p.spawn((
                        CharacterSprite { row, col },
                        TextSprite,
                        Text2d::new(ch.to_string()),
                        Transform::from_translation(CharacterSprite { row, col }.offset()),
                        GlobalTransform::default()
                    ));
                });
            }
        }
    }

    // -----------------------------------------------------------------------------------------
    // Animation     (updates the glyphs instead of one big string)
    // -----------------------------------------------------------------------------------------
    pub fn animate(
        mut parent_q: Query<(&Children, &mut Bounce), With<PersonSprite>>,
        mut char_q: Query<(&mut Text2d, &CharacterSprite)>,
    ) {
        for (children, bounce) in parent_q.iter_mut() {
            let template = if bounce.enacting { PERSON_IN_DANGER } else { PERSON };

            for (row, line) in template.lines().enumerate() {
                for (col, ch) in line.chars().enumerate() {
                    let new_char = ch.to_string();

                    if let Some(child) = children.iter().find(|e| {
                        char_q.get(*e)
                            .map(|(_, c)| c.row == row && c.col == col)
                            .unwrap_or(false)
                    }) {
                        if let Ok((mut text, _)) = char_q.get_mut(child) {
                            if text.0 != new_char {
                                text.0 = new_char;
                            }
                        }
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------------------------
    // The rest of the original behavior is unchanged
    // -----------------------------------------------------------------------------------------
    pub fn scream(
        mut query: Query<(
            Entity,
            &TransientAudioPallet<EmotionSounds>,
            &mut PersonSprite,
            &mut Bounce,
        )>,
        dilation: Res<Dilation>,
        mut commands: Commands,
        mut audio_q: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    ) {
        for (e, pallet, person, bounce) in query.iter_mut() {
            if person.in_danger && bounce.timer.just_finished() {
                TransientAudioPallet::<EmotionSounds>::play_transient_audio(
                    e,
                    &mut commands,
                    pallet,
                    EmotionSounds::Exclaim,
                    dilation.0,
                    &mut audio_q,
                );
            }
        }
    }

    pub fn alert(mut q: Query<(&mut PersonSprite, &mut Bounce)>) {
        for (p, mut b) in q.iter_mut() {
            b.active = p.in_danger;
        }
    }

    /// Despawns the whole figure when **any** glyph collides with a train.
    /// How fast the parts shoot away from the figure’s centre (in units · s-¹)
	const EXPLOSION_SPEED: f32 = 250.0;

	pub fn explode(
		// glyph, its AABB, world-transform and parent PersonSprite
		char_q : Query<(Entity, &Aabb, &GlobalTransform, &ChildOf), With<CharacterSprite>>,
		// PersonSprite world-transform *and* optional velocity
		person_q : Query<(&GlobalTransform, Option<&Velocity>), With<PersonSprite>>,
		// carriage AABB, world-transform and parent Train
		carriage_q : Query<(&Aabb, &GlobalTransform, &ChildOf), With<TrainCarriage>>,
		// velocity of each Train root
		train_q : Query<&Velocity, With<Train>>,
		mut rng: ResMut<GlobalRng>,
		mut commands : Commands,
	) {
		// ─────────────────────────────────────────
		// 1. detect the very first collision
		// ─────────────────────────────────────────
	
		for (char_e, char_aabb, char_tf, char_parent) in char_q.iter() {
			let c_min = char_tf.transform_point(Vec3::from(char_aabb.center - char_aabb.half_extents));
			let c_max = char_tf.transform_point(Vec3::from(char_aabb.center + char_aabb.half_extents));
	
			for (train_aabb, train_tf, carriage_parent) in carriage_q.iter() {
				let t_min = train_tf.transform_point(Vec3::from(train_aabb.center - train_aabb.half_extents));
				let t_max = train_tf.transform_point(Vec3::from(train_aabb.center + train_aabb.half_extents));
	
				if c_min.x <= t_max.x
					&& c_max.x >= t_min.x
					&& c_min.y <= t_max.y
					&& c_max.y >= t_min.y
				{
					// ─────────────────────────────────────────
					// 2. gather context for this particular pair
					// ─────────────────────────────────────────
					let person_ent  = char_parent.parent();
					let train_ent   = carriage_parent.parent();
	
					let (person_tf, person_vel_opt) = person_q
						.get(person_ent)
						.unwrap_or((&GlobalTransform::IDENTITY, None));
					let person_ctr  = person_tf.translation();
					let person_vel  = person_vel_opt.map_or(Vec3::ZERO, |v| v.0);
	
					let train_vel   = train_q.get(train_ent).map_or(Vec3::ZERO, |v| v.0);
	
					// ─────────────────────────────────────────
					// 3. apply impulse to every glyph of *this* person
					// ─────────────────────────────────────────
					for (glyph_e, _aabb, glyph_tf, glyph_parent) in char_q.iter() {
						if glyph_parent.parent() == person_ent {
							// base direction: centre → glyph
							let mut dir = (glyph_tf.translation() - person_ctr)
								.try_normalize()
								.unwrap_or(Vec3::X);
	
							if dir == Vec3::ZERO {
								dir = Vec3::X;
							}
	
							// ────── randomness ──────
							// a tiny angular nudge (-0.2 … 0.2 on each axis then re-normalise)
							let jitter = Vec3::new(
								rng.uniform.gen_range(-0.2..0.2),
								rng.uniform.gen_range(-0.2..0.2),
								rng.uniform.gen_range(-0.2..0.2),
							);
							dir = (dir + jitter).try_normalize().unwrap_or(dir);
	
							// speed multiplier 0.9 … 1.1 (±10 %)
							let speed_scale: f32 = rng.uniform.gen_range(0.9..1.1);
	
							let velocity = dir * Self::EXPLOSION_SPEED * speed_scale   // blast + jitter
											+ train_vel                                   // impart train momentum
											+ person_vel;                                 // keep parent momentum
	
							// world-space transform so the glyph stays where it is
							let world_tf = Transform::from_matrix(glyph_tf.compute_matrix());
	
							commands.entity(glyph_e)
								.remove::<ChildOf>()   // detach
								.insert((
									world_tf,          // keep position / rotation / scale
									Gravity::default(),
									Friction::default(),
									Velocity(velocity),
								));
						}
					}
	
					// ─────────────────────────────────────────
					// 4. remove the now-empty PersonSprite
					// ─────────────────────────────────────────
					commands.entity(person_ent).despawn();
	
					return; // prevent double explosions this frame
				}
			}
		}
	}
}

/// ─────────────────────────────────────────────────────────────
///  Emoticon (unchanged)
/// ─────────────────────────────────────────────────────────────
fn default_emoticon() -> Text2d { Text2d::new(NEUTRAL) }
fn default_emoticon_transform() -> Transform { Transform::from_xyz(0.0, 50.0, 0.0) }

#[derive(Component)]
#[require(TextSprite, Text2d = default_emoticon(), Transform = default_emoticon_transform())]
pub struct Emoticon {
    pub initial_size: f32,
    pub current_size: f32,
    pub translation: Vec3,
}

impl Default for Emoticon {
    fn default() -> Self {
        Self { initial_size: 1.0, current_size: 1.0, translation: Vec3::new(0.0, 50.0, 0.0) }
    }
}

impl Emoticon {
    pub fn animate(
        time: Res<Time>,
        dilation: Res<Dilation>,
        person_q: Query<&mut Bounce, With<PersonSprite>>,
        mut emoticon_q: Query<(&ChildOf, &mut Emoticon, &mut Transform, &mut Text2d)>,
    ) {
        let dt = time.delta_secs() * dilation.0;
        for (parent, mut sprite, mut tf, mut text) in emoticon_q.iter_mut() {
            if let Ok(bounce) = person_q.get(parent.parent()) {
                if bounce.enacting {
                    sprite.current_size += dt * 2.0;
                    tf.translation.y += dt * 15.0;
                    tf.scale = Vec3::splat(sprite.current_size);
                    text.0 = EXCLAMATION.to_string();
                } else {
                    sprite.current_size = sprite.initial_size;
                    tf.translation.y = sprite.translation.y;
                    tf.scale = Vec3::ONE;
                    text.0 = NEUTRAL.to_string();
                }
            }
        }
    }
}

/// ─────────────────────────────────────────────────────────────
///  Enum for the scream sfx (unchanged)
/// ─────────────────────────────────────────────────────────────
#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmotionSounds { Exclaim }
