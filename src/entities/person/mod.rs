use bevy::{
    ecs::{component::HookContext, schedule::SystemSet, world::DeferredWorld},
    prelude::*,
    render::primitives::Aabb,
    sprite::Anchor,
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

use super::train::{Train, TrainCarriage};

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
                            .or(in_state(DilemmaPhase::Consequence)),
                    ),
            )
            .add_systems(
                Update,
                (
                    PersonSprite::bounce_off_train,
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
const CHAR_WIDTH: f32 = 9.0;
const LINE_HEIGHT: f32 = 16.0;

#[derive(Component)]
pub struct CharacterSprite {
    row: usize,
    col: usize,
}

impl CharacterSprite {
    fn offset(&self) -> Vec3 {
        let x = (self.col as f32 - (COLS as f32 - 1.0) * 0.5) * CHAR_WIDTH;
        let y = ((LINES - 1 - self.row) as f32) * LINE_HEIGHT;
        Vec3::new(x, y, 0.0)
    }
}

fn default_person_anchor() -> Anchor {
    Anchor::BottomCenter
}

#[derive(Component)]
pub struct BloodSprite;

// ═══════════════════════════════════════════════════════════════════════════
//  PersonSprite (parent entity)
// ═══════════════════════════════════════════════════════════════════════════
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
    // ─────────────────────────────────────────────────────────────
    //  Spawn hook
    // ─────────────────────────────────────────────────────────────
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        for (row, line) in PERSON.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                world.commands().entity(entity).with_children(|p| {
                    p.spawn((
                        CharacterSprite { row, col },
                        TextSprite,
                        Text2d::new(ch.to_string()),
                        Anchor::BottomCenter,
                        Transform::from_translation(CharacterSprite { row, col }.offset()),
                        GlobalTransform::default(),
                    ));
                });
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

    // ─────────────────────────────────────────────────────────────
    //  Explosion stage 1 – detect collision & emit event
    // ─────────────────────────────────────────────────────────────
    pub fn explode(
        char_q: Query<(&Aabb, &GlobalTransform, &ChildOf), With<CharacterSprite>>,
        person_q: Query<(&GlobalTransform, Option<&Velocity>), With<PersonSprite>>,
        carriage_q: Query<(&Aabb, &GlobalTransform, &ChildOf), With<TrainCarriage>>,
        train_q: Query<&Velocity, With<Train>>,
        mut ev_writer: EventWriter<PersonExplosionEvent>,
    ) {
        for (c_aabb, c_tf, c_parent) in char_q.iter() {
            let c_min = c_tf.transform_point(Vec3::from(c_aabb.center - c_aabb.half_extents));
            let c_max = c_tf.transform_point(Vec3::from(c_aabb.center + c_aabb.half_extents));

            for (t_aabb, t_tf, carriage_parent) in carriage_q.iter() {
                let t_min = t_tf.transform_point(Vec3::from(t_aabb.center - t_aabb.half_extents));
                let t_max = t_tf.transform_point(Vec3::from(t_aabb.center + t_aabb.half_extents));

                if c_min.x <= t_max.x
                    && c_max.x >= t_min.x
                    && c_min.y <= t_max.y
                    && c_max.y >= t_min.y
                {
                    let person_ent = c_parent.parent();
                    let train_ent = carriage_parent.parent();
                    let (person_tf, person_vel_opt) = person_q
                        .get(person_ent)
                        .unwrap_or((&GlobalTransform::IDENTITY, None));
                    let person_vel = person_vel_opt.map_or(Vec3::ZERO, |v| v.0);
                    let train_vel = train_q.get(train_ent).map_or(Vec3::ZERO, |v| v.0);

                    ev_writer.write(PersonExplosionEvent {
                        person_ent,
                        person_ctr: person_tf.translation(),
                        person_vel,
                        train_vel,
                    });
                    return;
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
        mut debris_q : Query<
            (&mut Velocity, &Aabb, &GlobalTransform),
            (Or<(With<CharacterSprite>, With<BloodSprite>)>, Without<Train>)
        >,
        carriage_q  : Query<(&Aabb, &GlobalTransform, &ChildOf), With<TrainCarriage>>,
        train_q     : Query<&Velocity, With<Train>>,
    ) {
        // tuning knobs
        const RESTITUTION : f32 = 0.8;   // bounce energy (1 = perfect)
        const TRAIN_SHARE : f32 = 0.8;   // how much current train velocity to inherit
        const PUSH_OUT    : f32 = 0.05;  // metres to move outward after bounce
        const CAR_WIDTH   : f32 = 10.0;   // ±z half-extent for every carriage

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

        // build all carriage boxes once (saves eight-corner loop per debris)
        let mut car_boxes: Vec<(Vec3, Vec3, Entity, Vec3)> = Vec::with_capacity(carriage_q.iter().len());
        for (c_box, c_tf, child) in carriage_q.iter() {
            let (mut min, mut max) = world_aabb(c_box, c_tf);
            // widen in Z so sprites can hit sides/front
            min.z -= CAR_WIDTH * 0.5;
            max.z += CAR_WIDTH * 0.5;
            car_boxes.push((min, max, child.parent(), (min + max) * 0.5));
        }

        for (mut vel, d_box, d_tf) in debris_q.iter_mut() {
            let (d_min, d_max) = world_aabb(d_box, d_tf);

            for &(c_min, c_max, train_ent, centre) in &car_boxes {
                // axis-aligned overlap with epsilon = 0
                if d_min.x > c_max.x || d_max.x < c_min.x ||
                   d_min.y > c_max.y || d_max.y < c_min.y ||
                   d_min.z > c_max.z || d_max.z < c_min.z
                { continue; }

                // penetration depth on each axis
                let pen_x = (c_max.x - d_min.x).min(d_max.x - c_min.x);
                let pen_y = (c_max.y - d_min.y).min(d_max.y - c_min.y);
                let pen_z = (c_max.z - d_min.z).min(d_max.z - c_min.z);

                // choose axis of smallest penetration
                let (normal, penetration) = {
                    if pen_x < pen_y && pen_x < pen_z {
                        (Vec3::X * (d_tf.translation().x - centre.x).signum(), pen_x)
                    } else if pen_y < pen_z {
                        (Vec3::Y * (d_tf.translation().y - centre.y).signum(), pen_y)
                    } else {
                        (Vec3::Z * (d_tf.translation().z - centre.z).signum(), pen_z)
                    }
                };

                // reflect & damp velocity
                vel.0 = (vel.0 - 2.0 * vel.0.dot(normal) * normal) * RESTITUTION;

                // inherit slice of current train velocity
                if let Ok(train_vel) = train_q.get(train_ent) {
                    vel.0 += train_vel.0 * TRAIN_SHARE;
                }

                // push debris outside plus extra bias so it won’t get stuck
                vel.0 += normal * (penetration + PUSH_OUT);

                break; // one bounce per frame is fine
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
