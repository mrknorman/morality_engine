use bevy::{
    camera::primitives::Aabb,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    window::PrimaryWindow,
};

use rand::{seq::IndexedRandom, Rng};
use rand_distr::{Distribution, LogNormal, Normal, UnitSphere};

use crate::{
    data::rng::GlobalRng,
    entities::{
        person::{BloodSprite, EmotionSounds},
        text::{CharacterSprite, TextSprite},
    },
    systems::{interaction::world_aabb, time::Dilation},
};

use super::audio::{DilatableAudio, TransientAudio, TransientAudioPallet};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSystemsActive {
    #[default]
    False,
    True,
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PhysicsSystemsActive>()
            .add_systems(Update, activate_systems)
            .add_systems(
                Update,
                (
                    Velocity::enact,
                    CameraVelocity::enact,
                    AngularVelocity::enact,
                    ScaleVelocity::enact,
                    Gravity::enact,
                    Friction::enact,
                    Volatile::enact,
                )
                    .run_if(in_state(PhysicsSystemsActive::True)),
            )
            .add_systems(
                PostUpdate,
                DespawnOffscreen::enact.run_if(in_state(PhysicsSystemsActive::True)),
            );
    }
}

fn activate_systems(
    mut physics_state: ResMut<NextState<PhysicsSystemsActive>>,
    physics_query: Query<&Velocity>,
) {
    if !physics_query.is_empty() {
        physics_state.set(PhysicsSystemsActive::True)
    } else {
        physics_state.set(PhysicsSystemsActive::False)
    }
}

#[derive(Component)]
pub struct ExplodedGlyph;

#[derive(Component)]
#[require(Transform, CameraVelocity)]
pub struct Velocity(pub Vec3);

impl Default for Velocity {
    fn default() -> Self {
        Self(Vec3::ZERO)
    }
}

impl Velocity {
    fn enact(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut query: Query<(&mut Transform, &mut Velocity)>,
    ) {
        let duration_seconds = time.delta_secs() * dilation.0;
        for (mut transform, velocity) in query.iter_mut() {
            transform.translation += velocity.0 * duration_seconds;
        }
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct CameraVelocity(pub Vec3);

impl Default for CameraVelocity {
    fn default() -> Self {
        Self(Vec3::ZERO)
    }
}

impl CameraVelocity {
    fn enact(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut query: Query<(&mut Transform, &mut CameraVelocity)>,
    ) {
        let duration_seconds = time.delta_secs() * dilation.0;
        for (mut transform, velocity) in query.iter_mut() {
            transform.translation += velocity.0 * duration_seconds;
        }
    }
}

#[derive(Component, Default)]
#[require(Transform)]
pub struct AngularVelocity(pub Vec3); // Radians per second for rotation around X, Y, Z axes

impl AngularVelocity {
    pub fn enact(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut query: Query<(&mut Transform, &AngularVelocity)>,
    ) {
        let delta_time = time.delta_secs() * dilation.0;

        for (mut transform, angular_velocity) in query.iter_mut() {
            if angular_velocity.0 != Vec3::ZERO {
                let rotation_delta = Quat::from_euler(
                    EulerRot::XYZ,
                    angular_velocity.0.x * delta_time,
                    angular_velocity.0.y * delta_time,
                    angular_velocity.0.z * delta_time,
                );
                transform.rotation = transform.rotation * rotation_delta;
            }
        }
    }
}

#[derive(Component, Default)]
#[require(Transform)]
pub struct ScaleVelocity(pub Vec3); // Radians per second for rotation around X, Y, Z axes

impl ScaleVelocity {
    pub fn enact(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut query: Query<(&mut Transform, &ScaleVelocity)>,
    ) {
        let delta_time = time.delta_secs() * dilation.0;

        for (mut transform, scale_velocity) in query.iter_mut() {
            if scale_velocity.0 != Vec3::ZERO {
                transform.scale += scale_velocity.0 * delta_time;
            }
        }
    }
}

#[derive(Component)]
pub struct Colidable;

#[derive(Component)]
pub struct InertialMass(f32);

#[derive(Component)]
#[require(Transform, Velocity)]
#[component(on_insert = Gravity::on_insert)]
pub struct Gravity {
    pub acceleration: Vec3,
    pub floor_level: Option<f32>,
    pub is_falling: bool,
}

impl Gravity {
    fn enact(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut query: Query<(&mut Gravity, &mut Velocity, &mut Transform)>,
    ) {
        let duration_seconds = time.delta_secs() * dilation.0;
        for (mut gravity, mut velocity, mut transform) in query.iter_mut() {
            if let Some(floor_level) = gravity.floor_level {
                if transform.translation.y > floor_level {
                    gravity.is_falling = true;
                    velocity.0 += gravity.acceleration * duration_seconds;
                } else if gravity.is_falling && transform.translation.y < floor_level {
                    gravity.is_falling = false;
                    transform.translation.y = floor_level;
                    velocity.0.y = 0.0;
                }
            }
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let transform: Option<Transform> = {
            let entity_mut = world.entity(entity);
            entity_mut
                .get::<Transform>()
                .map(|transform: &Transform| transform.clone())
        };

        if let Some(transform) = transform {
            if let Some(mut gravity) = world.entity_mut(entity).get_mut::<Gravity>() {
                if gravity.floor_level.is_none() {
                    gravity.floor_level = Some(transform.translation.y);
                }
            } else {
                eprintln!("Warning: Entity does not contain a Gravity component!");
            }
        }
    }
}

impl Default for Gravity {
    fn default() -> Self {
        Self {
            acceleration: Vec3::new(0.0, -200.0, 1.0),
            floor_level: None,
            is_falling: false,
        }
    }
}

#[derive(Component, Clone)]
#[require(Transform, Velocity)]
#[component(on_insert = Friction::on_insert)]
pub struct Friction {
    pub coefficient: f32,
    pub floor_level: Option<f32>,
    pub is_sliding: bool,
}

impl Friction {
    fn enact(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut query: Query<(
            &mut Friction,
            &mut Velocity,                // linear velocity
            Option<&mut AngularVelocity>, // angular velocity (optional)
            Option<&mut ScaleVelocity>,
            &mut Transform,
        )>,
    ) {
        let dt = time.delta_secs() * dilation.0;

        for (mut friction, mut linear_v, angular_opt, scalar_opt, transform) in query.iter_mut() {
            if let Some(floor) = friction.floor_level {
                let on_floor = transform.translation.y <= floor;

                if on_floor {
                    linear_v.0.y = 0.0;
                }

                /* ---------- LINEAR FRICTION ---------- */
                if on_floor && linear_v.0.length_squared() > 0.0 {
                    friction.is_sliding = true;

                    let v0 = linear_v.0; // copy
                    linear_v.0 -= v0 * friction.coefficient * dt; // mutate

                    if linear_v.0.length_squared() < 0.01 {
                        linear_v.0 = Vec3::ZERO;
                    }
                }

                /* ---------- ANGULAR FRICTION ---------- */
                if let Some(mut angular) = angular_opt {
                    if on_floor {
                        angular.0 = Vec3::ZERO;
                    }
                }

                /* ---------- SCALAR FRICTION ---------- */
                if let Some(mut scalar_opt) = scalar_opt {
                    if on_floor && scalar_opt.0.length_squared() > 0.0 {
                        let w0 = scalar_opt.0; // copy
                        scalar_opt.0 -= w0 * friction.coefficient * dt; // mutate

                        if scalar_opt.0.length_squared() < 0.01 {
                            scalar_opt.0 = Vec3::ZERO;
                        }
                    }
                }

                /* ---------- RESET SLIDING FLAG ---------- */
                if linear_v.0 == Vec3::ZERO {
                    friction.is_sliding = false;
                }
            }
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let transform: Option<Transform> = {
            let entity_mut = world.entity(entity);
            entity_mut
                .get::<Transform>()
                .map(|transform: &Transform| transform.clone())
        };

        if let Some(transform) = transform {
            if let Some(mut friction) = world.entity_mut(entity).get_mut::<Friction>() {
                if friction.floor_level.is_none() {
                    friction.floor_level = Some(transform.translation.y);
                }
            } else {
                eprintln!("Warning: Entity does not contain a friction component!");
            }
        }
    }
}

impl Default for Friction {
    fn default() -> Self {
        Self {
            coefficient: 1.5,
            floor_level: None,
            is_sliding: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Volatile Component - Handles explosion behavior
// ═══════════════════════════════════════════════════════════════════════════
#[derive(Component, Debug, Clone)]
pub struct Volatile {
    pub should_explode: bool,
    pub explosion_speed: f32,
    pub spin_scale: f32,
    pub particle_speed_factor: f32,
    pub particle_count_range: (u32, u32),
    pub particle_chars: &'static [char],
    pub explosion_sounds: Vec<EmotionSounds>,
    pub explosion_center: Vec3,
    pub velocity: Vec3,
    pub impact_velocity: Vec3,
}

impl Default for Volatile {
    fn default() -> Self {
        Self {
            should_explode: false,
            explosion_speed: 250.0,
            spin_scale: 0.003,
            particle_speed_factor: 1.2,
            particle_count_range: (10, 20),
            particle_chars: &[',', '.', '"'],
            explosion_sounds: vec![EmotionSounds::Splat, EmotionSounds::Pop],
            explosion_center: Vec3::ZERO,
            velocity: Vec3::ZERO,
            impact_velocity: Vec3::ZERO,
        }
    }
}

impl Volatile {
    pub fn trigger_explosion(&mut self, center: Vec3, velocity: Vec3, impact_velocity: Vec3) {
        self.should_explode = true;
        self.explosion_center = center;
        self.velocity = velocity;
        self.impact_velocity = impact_velocity;
    }

    pub fn enact(
        mut volatile_q: Query<(
            Entity,
            &mut Volatile,
            Option<&TransientAudioPallet<EmotionSounds>>,
        )>,
        char_q: Query<(Entity, &GlobalTransform, &Transform, &ChildOf), With<CharacterSprite>>,
        mut rng: ResMut<GlobalRng>,
        dilation: Res<Dilation>,
        mut commands: Commands,
        mut audio_q: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    ) {
        for (entity, volatile, pallet_opt) in volatile_q.iter_mut() {
            if !volatile.should_explode {
                continue;
            }

            // Play explosion audio
            if let Some(pallet) = pallet_opt {
                for &sound in &volatile.explosion_sounds {
                    TransientAudioPallet::<EmotionSounds>::play_transient_audio(
                        entity,
                        &mut commands,
                        pallet,
                        sound,
                        dilation.0,
                        &mut audio_q,
                    );
                }
            }

            let jitter_n = Normal::new(0.0, 0.12).unwrap();
            let speed_ln = LogNormal::new(0.0, 0.25).unwrap();
            let spin_ln = LogNormal::new(0.0, 0.25).unwrap();

            // Explode character glyphs
            for (glyph_e, glyph_tf, local_tf, glyph_parent) in char_q.iter() {
                if glyph_parent.parent() != entity {
                    continue;
                }

                let explosion_data = Self::calculate_explosion_forces(
                    glyph_tf.translation(),
                    volatile.explosion_center,
                    volatile.velocity,
                    volatile.impact_velocity,
                    &volatile,
                    &mut rng,
                    &jitter_n,
                    &speed_ln,
                    &spin_ln,
                );

                Self::apply_glyph_explosion(
                    glyph_e,
                    glyph_tf,
                    local_tf,
                    explosion_data,
                    &mut commands,
                );
            }

            // Create particle debris
            Self::create_explosion_particles(
                volatile.explosion_center,
                volatile.velocity,
                volatile.impact_velocity,
                &volatile,
                &mut commands,
                &mut rng,
                &speed_ln,
            );

            // Clean up the exploded entity
            commands.entity(entity).despawn();
        }
    }

    fn calculate_explosion_forces(
        glyph_pos: Vec3,
        explosion_center: Vec3,
        entity_vel: Vec3,
        impact_vel: Vec3,
        volatile: &Volatile,
        rng: &mut GlobalRng,
        jitter_n: &Normal<f32>,
        speed_ln: &LogNormal<f32>,
        spin_ln: &LogNormal<f32>,
    ) -> ExplosionForces {
        let mut dir = (glyph_pos - explosion_center)
            .try_normalize()
            .unwrap_or(Vec3::X);

        let jitter: Vec3 = Vec3::from(UnitSphere.sample(&mut *rng)) * jitter_n.sample(&mut *rng);
        dir = (dir + jitter).try_normalize().unwrap_or(dir);

        let speed_scale = speed_ln.sample(&mut *rng);
        let spin_scale = spin_ln.sample(&mut *rng);

        let velocity = dir * volatile.explosion_speed * speed_scale + impact_vel + entity_vel;

        let r = glyph_pos - explosion_center;
        let mut omega = r.cross(velocity) * volatile.spin_scale * spin_scale;
        if omega.length() > 100.0 {
            omega = omega.clamp_length_max(100.0);
        }

        ExplosionForces {
            velocity,
            angular_velocity: omega,
            tint_shade: rng.random_range(0.0..=3.0),
        }
    }

    fn apply_glyph_explosion(
        glyph_e: Entity,
        glyph_tf: &GlobalTransform,
        local_tf: &Transform,
        forces: ExplosionForces,
        commands: &mut Commands,
    ) {
        let world_tf = Transform::from_matrix(glyph_tf.to_matrix());
        let vz = forces.velocity.z / 500.0;
        let scale_velocity = Vec3::splat(vz);
        let floor_level = Some(-local_tf.translation.y - vz * 1000.0);

        commands.entity(glyph_e).remove::<ChildOf>().insert((
            world_tf,
            ExplodedGlyph,
            TextColor(Color::srgba(3.0, forces.tint_shade, forces.tint_shade, 1.0)),
            Gravity {
                floor_level,
                ..default()
            },
            Friction {
                floor_level,
                ..default()
            },
            ScaleVelocity(scale_velocity),
            Velocity(forces.velocity),
            AngularVelocity(forces.angular_velocity),
            DespawnOffscreen { margin: 500.0 },
        ));
    }

    fn create_explosion_particles(
        explosion_center: Vec3,
        entity_vel: Vec3,
        impact_vel: Vec3,
        volatile: &Volatile,
        commands: &mut Commands,
        rng: &mut GlobalRng,
        speed_ln: &LogNormal<f32>,
    ) {
        let n_particles =
            rng.random_range(volatile.particle_count_range.0..=volatile.particle_count_range.1);

        for _ in 0..n_particles {
            let ch = *volatile.particle_chars.choose(&mut *rng).unwrap_or(&'.');
            let dir: Vec3 = Vec3::from(UnitSphere.sample(&mut *rng)).normalize_or_zero();
            let velocity = dir
                * volatile.explosion_speed
                * volatile.particle_speed_factor
                * speed_ln.sample(&mut *rng)
                + impact_vel
                + entity_vel;

            let vz = velocity.z / 500.0;
            let scale_velocity = Vec3::splat(vz);
            let floor_level = Some(explosion_center.y - vz * 400.0);

            commands.spawn((
                BloodSprite(rng.random_range(1..=3)),
                TextSprite,
                TextColor(Color::srgba(2.0, 0.0, 0.0, 1.0)),
                Text2d::new(ch.to_string()),
                Transform::from_translation(explosion_center),
                Gravity {
                    floor_level,
                    ..default()
                },
                Friction {
                    floor_level,
                    ..default()
                },
                ScaleVelocity(scale_velocity),
                Velocity(velocity),
                DespawnOffscreen { margin: 500.0 },
            ));
        }
    }
}

#[derive(Debug, Clone)]
struct ExplosionForces {
    velocity: Vec3,
    angular_velocity: Vec3,
    tint_shade: f32,
}

#[derive(Component)]
pub struct DespawnOffscreen {
    pub margin: f32,
}

impl Default for DespawnOffscreen {
    fn default() -> Self {
        Self { margin: 0.0 }
    }
}

impl DespawnOffscreen {
    pub fn enact(
        window: Single<&Window, With<PrimaryWindow>>,
        mut commands: Commands,
        query: Query<(Entity, &Aabb, &GlobalTransform, &DespawnOffscreen)>,
    ) {
        let half_width = window.width() / 2.0;
        let half_height = window.height() / 2.0;

        for (entity, aabb, global_tf, marker) in &query {
            let (min, max) = world_aabb(aabb, global_tf);

            let margin = marker.margin;
            let outside = max.x < -half_width - margin
                || min.x > half_width + margin
                || max.y < -half_height - margin
                || min.y > half_height + margin;

            if outside {
                commands.entity(entity).despawn();
            }
        }
    }
}
