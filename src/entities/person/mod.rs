use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld}, prelude::*, camera::primitives::Aabb, sprite::Anchor
};
use enum_map::Enum;
use rand::Rng;
use rand_distr::{Distribution, UnitSphere};

use crate::{
    data::{rng::GlobalRng, states::DilemmaPhase},
    entities::text::TextSprite,
    systems::{
        audio::{DilatableAudio, TransientAudio, TransientAudioPallet}, interaction::world_aabb, motion::Bounce, physics::{ Gravity, PhysicsPlugin, Velocity, Volatile}, time::Dilation
    },
};

use super::{text::{CharacterSprite, GlyphString}, train::{Train, TrainSounds}};

// ═══════════════════════════════════════════════════════════════════════════
//  Plugin plumbing
// ═══════════════════════════════════════════════════════════════════════════
#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PersonSystemsActive {
    #[default]
    False,
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
                    Emoticon::animate,
                )
                .run_if(in_state(PersonSystemsActive::True))
                .run_if(
                    in_state(DilemmaPhase::Decision)
                        .or(in_state(DilemmaPhase::Consequence))
                        .or(in_state(DilemmaPhase::Skip)),
                ),
            )
            .add_systems(
                Update,
                (
                    PersonSprite::bounce_off_train,
                    PersonSprite::blood_soak_train,
                    PersonSprite::detect_train_collision, // New collision detection system
                )
                    .run_if(
                        in_state(DilemmaPhase::Decision)
                            .or(in_state(DilemmaPhase::DilemmaTransition))
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

// ═══════════════════════════════════════════════════════════════════════════
//  Constants & helpers
// ═══════════════════════════════════════════════════════════════════════════
const PERSON: &str = " @ \n/|\\\n/ \\";
const PERSON_IN_DANGER: &str = "\\@/\n | \n/ \\";

const EXCLAMATION: &str = "!";
const NEUTRAL: &str = "    ";

fn default_person_anchor() -> Anchor {
    Anchor::BOTTOM_CENTER
}

#[derive(Component)]
pub struct BloodSprite(pub u8);



#[derive(Component)]
#[require(Anchor = default_person_anchor(), Gravity, TextSprite, Bounce, Visibility, Volatile)]
#[component(on_insert = PersonSprite::on_insert)]
pub struct PersonSprite { 
    pub in_danger: bool 
}

impl Default for PersonSprite {
    fn default() -> Self { 
        Self { in_danger: false } 
    }
}
#[derive(Component)]
pub struct Bloodied;

#[derive(Component)]
pub struct IgnoreTrainCollision;

impl PersonSprite {
    const PERSON_DEPTH: f32 = 5.0;

    // ─────────────────────────────────────────────────────────────
    //  Spawn hook
    // ─────────────────────────────────────────────────────────────
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        // Attach the glyph string; the GlyphString::on_insert hook does the rest
        world.commands().entity(entity).insert(GlyphString {
            text: PERSON.to_string(),
            depth: Self::PERSON_DEPTH
        });
    }

    // ─────────────────────────────────────────────────────────────
    //  Train collision detection - triggers explosion
    // ─────────────────────────────────────────────────────────────
    pub fn detect_train_collision(
        mut volatile_q: Query<(&Aabb, &GlobalTransform, &mut Volatile, Option<&Velocity>)>,
        train_q: Query<(&Aabb, &GlobalTransform, &Velocity), With<Train>>,
    ) {
        for (v_box, v_tf, mut volatile, v_vel_opt) in volatile_q.iter_mut() {
            if volatile.should_explode {
                continue; // Already triggered
            }

            let (v_min, v_max) = world_aabb(v_box, v_tf);

            for (t_box, t_tf, t_vel) in train_q.iter() {
                let (t_min, t_max) = world_aabb(t_box, t_tf);

                // 2D overlap check (X/Y only)
                if v_min.x <= t_max.x && v_max.x >= t_min.x &&
                   v_min.y <= t_max.y && v_max.y >= t_min.y {
                    volatile.trigger_explosion(
                        v_tf.translation(),
                        v_vel_opt.map_or(Vec3::ZERO, |v| v.0),
                        t_vel.0,
                    );
                    break; // One collision per frame
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────
    //  Glyph animation
    // ─────────────────────────────────────────────────────────────
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
                        char_q
                            .get(*e)
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

    // ─────────────────────────────────────────────────────────────
    //  SFX + alert
    // ─────────────────────────────────────────────────────────────
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

    pub fn bounce_off_train(
        mut commands: Commands,
        mut debris_q: Query<
            (Entity, &mut Velocity, &Aabb, &GlobalTransform),
            (
                With<CharacterSprite>,
                Without<Train>,
                Without<IgnoreTrainCollision>,
            ),
        >,
        dilation: Res<Dilation>,
        mut rng: ResMut<GlobalRng>,
        mut audio_q: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
        train_q: Query<(Entity, &Aabb, &GlobalTransform, &Velocity, Option<&TransientAudioPallet<TrainSounds>>), With<Train>>,
    ) {
        const RESTITUTION: f32 = 0.8;
        const TRAIN_SHARE: f32 = 0.8;
        const PUSH_OUT: f32 = 0.05;
        const FRONT_BONUS: f32 = 0.8;
        const RANDOM_FAN: f32 = 1.0;

        let mut trains_data: Vec<(Vec3, Vec3, Vec3, Vec3, Option<&TransientAudioPallet<TrainSounds>>, Entity)> = Vec::new();
        for (train_entity, box_local, tf, vel, pallet) in train_q.iter() {
            let (min, max) = world_aabb(box_local, tf);
            trains_data.push((min, max, vel.0, tf.translation(), pallet, train_entity));
        }

        if trains_data.is_empty() {
            return;
        }

        for (debris_entity, mut vel, d_box, d_tf) in debris_q.iter_mut() {
            let (d_min, d_max) = world_aabb(d_box, d_tf);
            let mut collided_this_frame = false;

            for &(t_min, t_max, train_vel, t_centre, pallet, train_entity) in &trains_data {
                // AABB overlap check (3D)
                if d_min.x > t_max.x || d_max.x < t_min.x ||
                   d_min.y > t_max.y || d_max.y < t_min.y ||
                   d_min.z > t_max.z || d_max.z < t_min.z {
                    continue;
                }

                collided_this_frame = true;

                // Penetration on three axes
                let pen = Vec3::new(
                    (t_max.x - d_min.x).min(d_max.x - t_min.x),
                    (t_max.y - d_min.y).min(d_max.y - t_min.y),
                    (t_max.z - d_min.z).min(d_max.z - t_min.z),
                );

                // Smallest axis = bounce normal
                let debris_centre = d_tf.translation();
                let (normal, depth) = if pen.x < pen.y && pen.x < pen.z {
                    (Vec3::X * (debris_centre.x - t_centre.x).signum(), pen.x)
                } else if pen.y < pen.z {
                    (Vec3::Y * (debris_centre.y - t_centre.y).signum(), pen.y)
                } else {
                    (Vec3::Z * (debris_centre.z - t_centre.z).signum(), pen.z)
                };

                vel.0 = (vel.0 - 2.0 * vel.0.dot(normal) * normal) * RESTITUTION;
                vel.0 += train_vel * TRAIN_SHARE;
                vel.0 += normal * (depth + PUSH_OUT);

                if train_vel.length_squared() > 0.01 && vel.0.dot(normal) > 0.0 {
                    let extra_needed = train_vel * FRONT_BONUS - vel.0;
                    vel.0 += extra_needed.max(Vec3::ZERO);

                    let fan_dir: Vec3 = Vec3::from(UnitSphere.sample(&mut *rng));
                    vel.0 += fan_dir * train_vel.length() * RANDOM_FAN;
                } else if train_vel.length_squared() > 0.01 && vel.0.dot(normal) > 0.0 {
                    if let Some(pallet) = pallet {
                        TransientAudioPallet::<TrainSounds>::play_transient_audio(
                            train_entity,
                            &mut commands,
                            pallet,
                            TrainSounds::Bounce,
                            dilation.0,
                            &mut audio_q,
                        );
                    }
                }

                break;
            }

            if !collided_this_frame {
                let mut can_be_ignored_for_all_trains = true;
                for &(t_min, t_max, _train_vel, _t_centre, _, _) in &trains_data {
                    let is_outside_width = d_max.z < t_min.z || d_min.z > t_max.z;
                    let is_behind = d_max.x < t_min.x;

                    if !(is_outside_width || is_behind) {
                        can_be_ignored_for_all_trains = false;
                        break;
                    }
                }

                if can_be_ignored_for_all_trains {
                    commands.entity(debris_entity).insert(IgnoreTrainCollision);
                }
            }
        }
    }

    pub fn blood_soak_train(
        mut commands: Commands,
        mut debris_q: Query<
            (Entity, &Aabb, &GlobalTransform, &mut BloodSprite),
            Without<IgnoreTrainCollision>,
        >,
        mut rng: ResMut<GlobalRng>,
        sprite_q: Query<(Entity, &Aabb, &GlobalTransform), (With<CharacterSprite>, Without<Bloodied>, Without<Gravity>, Without<BloodSprite>)>
    ) {
        let mut part_data: Vec<(Vec3, Vec3, Vec3, Entity)> = Vec::new();
        for (entity, box_local, tf) in sprite_q.iter() {
            let (min, max) = world_aabb(box_local, tf);
            part_data.push((min, max, tf.translation(), entity));
        }

        if part_data.is_empty() {
            return;
        }

        for (debris_entity, d_box, d_tf, mut blood) in debris_q.iter_mut() {
            let (d_min, d_max) = world_aabb(d_box, d_tf);

            for &(t_min, t_max, _, entity) in &part_data {
                if d_min.x > t_max.x || d_max.x < t_min.x ||
                   d_min.y > t_max.y || d_max.y < t_min.y {
                    continue;
                }

                let tint_shade: f32 = rng.random_range(0.0..= blood.0 as f32);

                if let Ok(mut e) = commands.get_entity(entity) {
                    e.insert(TextColor(Color::srgba(3.0, tint_shade, tint_shade, 1.0)));
                    if tint_shade < 0.1 {
                        e.insert(Bloodied);
                    }
                }

                if let Ok(mut d) = commands.get_entity(debris_entity) {
                    if blood.0 < 1 {
                        d.insert(IgnoreTrainCollision);
                    } else {
                        blood.0 -= 1;
                    }
                }
                break;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Emoticon helper sprite
// ═══════════════════════════════════════════════════════════════════════════
fn default_emoticon() -> Text2d {
    Text2d::new(NEUTRAL)
}

fn default_emoticon_transform() -> Transform {
    Transform::from_xyz(0.0, 50.0, 0.0)
}

#[derive(Component)]
#[require(TextSprite, Text2d = default_emoticon(), Transform = default_emoticon_transform())]
pub struct Emoticon {
    pub initial_size: f32,
    pub current_size: f32,
    pub translation: Vec3,
}

impl Default for Emoticon {
    fn default() -> Self {
        Self {
            initial_size: 1.0,
            current_size: 1.0,
            translation: Vec3::new(0.0, 50.0, 0.0),
        }
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

// ═══════════════════════════════════════════════════════════════════════════
//  Emotion SFX enum
// ═══════════════════════════════════════════════════════════════════════════
#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmotionSounds {
    Exclaim,
    Splat,
    Pop,
}