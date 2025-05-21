use bevy::{
    ecs::{component::HookContext, world::DeferredWorld}, pbr::PreviousGlobalTransform, prelude::*, render::primitives::Aabb, sprite::Anchor
};
use enum_map::Enum;
use rand::{seq::IndexedRandom, Rng};
use rand_distr::{Distribution, LogNormal, Normal, UnitSphere};

use crate::{
    data::{rng::GlobalRng, states::DilemmaPhase},
    entities::text::TextSprite,
    systems::{
        audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
        motion::Bounce,
        physics::{AngularVelocity, Friction, Gravity, PhysicsPlugin, ScaleVelocity, Velocity},
        time::Dilation,
    },
};

use super::{text::{CharacterSprite, GlyphString}, train::{Train, TrainSounds}};

// ═══════════════════════════════════════════════════════════════════════════
//  Runtime events & system-sets
// ═══════════════════════════════════════════════════════════════════════════
#[derive(Event, Debug, Clone, Copy)]
pub struct PersonExplosionEvent {
    person_ent: Entity,
    person_ctr: Vec3,
    person_vel: Vec3,
    train_vel: Vec3,
}

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
        app.add_event::<PersonExplosionEvent>()
            .init_state::<PersonSystemsActive>()
            .add_systems(Update, activate_systems)
            // ── core behaviour (non-explosion) ──
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
                )
                    .run_if(
                        in_state(DilemmaPhase::Decision)
                            .or(in_state(DilemmaPhase::Consequence)),
                    ),
            )
            // ── explosion pipeline ──
            .add_systems(
                Update,
                (
                    PersonSprite::explode,
                    PersonSprite::play_explosion_audio,
                    PersonSprite::apply_explosion,
                )
                    .chain()                          // ← order + automatic flushes
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

// ═══════════════════════════════════════════════════════════════════════════
//  Constants & helpers
// ═══════════════════════════════════════════════════════════════════════════
const PERSON: &str = " @ \n/|\\\n/ \\";
const PERSON_IN_DANGER: &str = "\\@/\n | \n/ \\";

const EXCLAMATION: &str = "!";
const NEUTRAL: &str = "    ";

const COLS: usize = 3;
const LINES: usize = 3;


fn default_person_anchor() -> Anchor {
    Anchor::BottomCenter
}

#[derive(Component)]
pub struct BloodSprite;

#[derive(Component)]
#[require(Anchor = default_person_anchor(), Gravity, TextSprite, Bounce, Visibility)]
#[component(on_insert = PersonSprite::on_insert)]
pub struct PersonSprite { pub in_danger: bool }

impl Default for PersonSprite {
    fn default() -> Self { Self { in_danger: false } }
}

// helper: world-space AABB for any entity
fn world_aabb(local: &Aabb, tf: &GlobalTransform) -> (Vec3, Vec3) {
    let he = Vec3::from(local.half_extents);
    let c  = Vec3::from(local.center);
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for &sx in &[-1.0, 1.0] {
        for &sy in &[-1.0, 1.0] {
            for &sz in &[-1.0, 1.0] {
                let p = tf.transform_point(c + he * Vec3::new(sx, sy, sz));
                min = min.min(p);
                max = max.max(p);
            }
        }
    }
    (min, max)
}

#[derive(Component)]
pub struct Bloodied;

impl PersonSprite {

    const PERSON_DEPTH: f32 = 5.0;
    // ─────────────────────────────────────────────────────────────
    //  Spawn hook
    // ─────────────────────────────────────────────────────────────
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        // attach the glyph string; the GlyphString::on_insert hook does the rest
        world.commands().entity(entity).insert(GlyphString{
            text : PERSON.to_string(),
            depth : Self::PERSON_DEPTH
        });
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

    // ─────────────────────────────────────────────────────────────
    //  Explosion stage 1 – detect collision & emit event
    // ─────────────────────────────────────────────────────────────
   pub fn explode(
        // each person now has its own collider cube
        person_q : Query<
            (Entity, &Aabb, &GlobalTransform, Option<&Velocity>),
            With<PersonSprite>
        >,
        // each train already has a root-level Aabb and Velocity
        train_q  : Query<(&Aabb, &GlobalTransform, &Velocity), With<Train>>,
        mut ev_writer: EventWriter<PersonExplosionEvent>,
    ) {
        for (person_ent, p_box, p_tf, p_vel_opt) in person_q.iter() {
            let (p_min, p_max) = world_aabb(p_box, p_tf);      // helper you already have

            for (t_box, t_tf, t_vel) in train_q.iter() {
                let (t_min, t_max) = world_aabb(t_box, t_tf);

                // 2-D overlap check (X/Y only, no Z test)
                if p_min.x <= t_max.x && p_max.x >= t_min.x &&
                p_min.y <= t_max.y && p_max.y >= t_min.y
                {
                    ev_writer.write(PersonExplosionEvent {
                        person_ent,
                        person_ctr: p_tf.translation(),
                        person_vel: p_vel_opt.map_or(Vec3::ZERO, |v| v.0),
                        train_vel : t_vel.0,
                    });
                    return;          // one explosion per frame is enough
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────
    //  Explosion stage 2 – apply forces, sfx, debris
    // ─────────────────────────────────────────────────────────────
    const EXPLOSION_SPEED: f32 = 250.0;
    const EXPLOSION_SPIN_SCALE: f32 = 0.003;
    const PARTICLE_SPEED_FACTOR: f32 = 1.2;
    const PARTICLE_MIN: u32 = 1;
    const PARTICLE_MAX: u32 = 4;
    const PARTICLE_SET: &[char] = &[',', '.', '"'];

    pub fn play_explosion_audio(
        mut ev_reader: EventReader<PersonExplosionEvent>,
        person_q: Query<(
            Entity,
            Option<&TransientAudioPallet<EmotionSounds>>,
        ), With<PersonSprite>>,
        dilation: Res<Dilation>,
        mut commands: Commands,
        mut audio_q: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    ) {
        for event in ev_reader.read() {
            let PersonExplosionEvent {
                person_ent,
                ..
            } = *event;

            if let Ok((root, pallet_opt)) = person_q.get(person_ent) {
                if let Some(pallet) = pallet_opt {
                    TransientAudioPallet::<EmotionSounds>::play_transient_audio(
                        root,
                        &mut commands,
                        pallet,
                        EmotionSounds::Splat,
                        dilation.0,
                        &mut audio_q,
                    );
                    TransientAudioPallet::<EmotionSounds>::play_transient_audio(
                        root,
                        &mut commands,
                        pallet,
                        EmotionSounds::Pop,
                        dilation.0,
                        &mut audio_q,
                    );
                }
            }
        }
    }

    pub fn apply_explosion(
        mut ev_reader: EventReader<PersonExplosionEvent>,
        char_q: Query<(Entity, &GlobalTransform, &Transform, &ChildOf), With<CharacterSprite>>,
        mut rng: ResMut<GlobalRng>,
        mut commands: Commands
    ) {
        for event in ev_reader.read() {
            let PersonExplosionEvent {
                person_ent,
                person_ctr,
                person_vel,
                train_vel,
            } = *event;

            let jitter_n = Normal::new(0.0, 0.12).unwrap();
            let speed_ln = LogNormal::new(0.0, 0.25).unwrap();
            let spin_ln = LogNormal::new(0.0, 0.25).unwrap();

            for (glyph_e, glyph_tf, local_tf, glyph_parent) in char_q.iter() {
                if glyph_parent.parent() != person_ent {
                    continue;
                }

                let mut dir = (glyph_tf.translation() - person_ctr)
                    .try_normalize()
                    .unwrap_or(Vec3::X);
                let jitter: Vec3 = Vec3::from(UnitSphere.sample(&mut *rng)) * jitter_n.sample(&mut *rng);
                dir = (dir + jitter).try_normalize().unwrap_or(dir);

                let speed_scale = speed_ln.sample(&mut *rng);
                let spin_scale = spin_ln.sample(&mut *rng);

                let velocity = dir * Self::EXPLOSION_SPEED * speed_scale + train_vel + person_vel;

                let r = glyph_tf.translation() - person_ctr;
                let mut omega = r.cross(velocity) * Self::EXPLOSION_SPIN_SCALE * spin_scale;
                if omega.length() > 100.0 {
                    omega = omega.clamp_length_max(100.0);
                }

                let world_tf = Transform::from_matrix(glyph_tf.compute_matrix());
                let vz = velocity.z / 500.0;
                let scale_velocity = Vec3::splat(vz);
                let floor_level = Some(-local_tf.translation.y - vz * 1000.0);
                let tint_shade: f32 = rng.random_range(0.0..=3.0);

                commands.entity(glyph_e)
                    .remove::<ChildOf>()
                    .insert((
                        world_tf,
                        TextColor(Color::srgba(3.0, tint_shade, tint_shade, 1.0)),
                        Gravity { floor_level, ..default() },
                        Friction { floor_level, ..default() },
                        ScaleVelocity(scale_velocity),
                        Velocity(velocity),
                        AngularVelocity(omega),
                    ));

                let n_particles = rng.random_range(Self::PARTICLE_MIN..=Self::PARTICLE_MAX);
                for _ in 0..n_particles {
                    let ch = *Self::PARTICLE_SET.choose(&mut *rng).unwrap_or(&'.');
                    let dir: Vec3 = Vec3::from(UnitSphere.sample(&mut *rng)).normalize_or_zero();
                    let velocity = dir
                        * Self::EXPLOSION_SPEED
                        * Self::PARTICLE_SPEED_FACTOR
                        * speed_ln.sample(&mut *rng)
                        + train_vel
                        + person_vel;
                    let vz = velocity.z / 500.0;
                    let scale_velocity = Vec3::splat(vz);
                    let floor_level = Some(-local_tf.translation.y - vz * 400.0);

                    commands.spawn((
                        BloodSprite,
                        TextSprite,
                        TextColor(Color::srgba(2.0, 0.0, 0.0, 1.0)),
                        Text2d::new(ch.to_string()),
                        world_tf,
                        Gravity { floor_level, ..default() },
                        Friction { floor_level, ..default() },
                        ScaleVelocity(scale_velocity),
                        Velocity(velocity),
                    ));
                }
            }

            // clean up empty parent
            commands.entity(person_ent).despawn();
        }
    }

    pub fn bounce_off_train(
        mut commands: Commands, // Added Commands
        mut debris_q: Query<
            (Entity, &mut Velocity, &Aabb, &GlobalTransform), // Added Entity
            (
                With<CharacterSprite>,
                Without<Train>,
                Without<IgnoreTrainCollision>, // Added filter
            ),
        >,
        dilation: Res<Dilation>,
        mut rng: ResMut<GlobalRng>,
        mut audio_q: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
        train_q: Query<(Entity, &Aabb, &GlobalTransform, &Velocity, Option<&TransientAudioPallet::<TrainSounds>>), With<Train>>,
    ) {
        const RESTITUTION: f32 = 0.8;
        const TRAIN_SHARE: f32 = 0.8;
        const PUSH_OUT: f32 = 0.05;

        let mut trains_data: Vec<(Vec3, Vec3, Vec3, Vec3, Option<&TransientAudioPallet::<TrainSounds>>, Entity)> = Vec::new(); // min_aabb, max_aabb, velocity, center
        for (train_entity, box_local, tf, vel, pallet) in train_q.iter() {
            let (min, max) = world_aabb(box_local, tf);
            trains_data.push((min, max, vel.0, tf.translation(), pallet, train_entity));
        }

        if trains_data.is_empty() {
            // No trains to collide with or be ignored by.
            return;
        }

        //  extra punch when hit by the *front* of a moving train
        const FRONT_BONUS : f32 = 1.2;     // 120 % of train speed minimum
        const RANDOM_FAN  : f32 = 1.0;     // radial spray ≤ 30 % train speed

        for (debris_entity, mut vel, d_box, d_tf) in debris_q.iter_mut() {
            let (d_min, d_max) = world_aabb(d_box, d_tf);
            let mut collided_this_frame = false;

            for &(t_min, t_max, train_vel, t_centre, pallet, train_entity) in &trains_data {
                // AABB overlap check (3D)
                if d_min.x > t_max.x || d_max.x < t_min.x ||
                d_min.y > t_max.y || d_max.y < t_min.y ||
                d_min.z > t_max.z || d_max.z < t_min.z {
                    continue; // No overlap with this train
                }

                // Collision occurred
                collided_this_frame = true;

                // Penetration on three axes
                let pen = Vec3::new(
                    (t_max.x - d_min.x).min(d_max.x - t_min.x),
                    (t_max.y - d_min.y).min(d_max.y - t_min.y),
                    (t_max.z - d_min.z).min(d_max.z - t_min.z),
                );

                // Smallest axis = bounce normal
                let debris_centre = d_tf.translation(); // Cache for calculating normal direction
                let (normal, depth) = if pen.x < pen.y && pen.x < pen.z {
                    (Vec3::X * (debris_centre.x - t_centre.x).signum(), pen.x)
                } else if pen.y < pen.z {
                    (Vec3::Y * (debris_centre.y - t_centre.y).signum(), pen.y)
                } else {
                    (Vec3::Z * (debris_centre.z - t_centre.z).signum(), pen.z)
                };

                vel.0 = (vel.0 - 2.0 * vel.0.dot(normal) * normal) * RESTITUTION;
                vel.0 += train_vel * TRAIN_SHARE;
                vel.0 += normal * (depth + PUSH_OUT); // Apply push out

                if train_vel.length_squared() > 0.01 && vel.0.dot(normal) > 0.0 {
                    let extra_needed = train_vel * FRONT_BONUS - vel.0;
                    vel.0 += extra_needed.max(Vec3::ZERO); // ensure at least FRONT_BONUS

                    // random sideways spray
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

                break; // Debris bounces off the first train it hits in a frame
            }

            // After checking collisions with all trains for this debris particle:
            if !collided_this_frame {
                // Only consider tagging if no collision happened this frame.
                // If it collided, its position/velocity changed, so re-evaluate next frame.
                let mut can_be_ignored_for_all_trains = true;
                for &(t_min, t_max, _train_vel, _t_centre, _, _) in &trains_data {
                    let is_outside_width = d_max.z < t_min.z || d_min.z > t_max.z;
                    // "Behind the train" (negative Y): debris is fully "below" the train's min Y.
                    // This assumes Y is the primary axis of train length/movement.
                    // Adjust if your "behind" definition differs (e.g., based on train's velocity vector).
                    let is_behind = d_max.x < t_min.x;

                    if !(is_outside_width || is_behind) {
                        // If for any train, the debris is NOT (outside_width OR behind),
                        // then it cannot be definitively ignored yet.
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
        mut commands: Commands, // Added Commands
        mut debris_q: Query<
            (Entity, &Aabb, &GlobalTransform), // Added Entity
            (
                With<BloodSprite>, Without<IgnoreTrainCollision>,
            ),
        >,
        dilation: Res<Dilation>,
        mut rng: ResMut<GlobalRng>,
        mut audio_q: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
        sprite_q: Query<(Entity, &Aabb, &GlobalTransform, &TextColor), (With<CharacterSprite>, Without<Bloodied>, Without<Gravity>, Without<BloodSprite>)>
    ) {

        let mut part_data: Vec<(Vec3, Vec3, Vec3, Entity, &TextColor)> = Vec::new(); // min_aabb, max_aabb, velocity, center
        for (entity, box_local, tf, color) in sprite_q.iter() {
            let (min, max) = world_aabb(box_local, tf);
            part_data.push((min, max,  tf.translation(), entity, color));
        }

        if part_data.is_empty() {
            return;
        }

        for (debris_entity,  d_box, d_tf) in debris_q.iter_mut() {

            let (d_min, d_max) = world_aabb(d_box, d_tf);

            for &(t_min, t_max, _, entity, color) in &part_data {

                // AABB overlap check (3D)
                if d_min.x > t_max.x || d_max.x < t_min.x ||
                d_min.y > t_max.y || d_max.y < t_min.y {
                //d_min.z > t_max.z || d_max.z < t_min.z {
                    continue; // No overlap with this train
                }

                let tint_shade: f32 = rng.random_range(0.0..=3.0);

                if color.to_linear().to_vec4()[1] > tint_shade {
                    commands.entity(entity).insert((
                        TextColor(Color::srgba(3.0, tint_shade, tint_shade, 1.0)),
                    ));
                    commands.entity(debris_entity).insert(IgnoreTrainCollision);
                }

                if tint_shade < 0.1 {
                    commands.entity(entity).insert(Bloodied);
                }
                break; // Debris bounces off the first train it hits in a frame
            }

            // After checking collisions with all trains for this debris particle:
        }
    }

}

#[derive(Component)]
pub struct IgnoreTrainCollision;

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
