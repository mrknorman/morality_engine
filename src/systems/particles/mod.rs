//! Firework
use bevy::{
    audio::Volume,
    image::{TextureAtlas, TextureAtlasLayout},
    prelude::*,
};
use std::time::Duration;

use crate::startup::textures::DigitSheet;

const ROCKET_MIN_SPEED: f32 = 2400.0;
const ROCKET_MAX_SPEED: f32 = 2600.0;
const ROCKET_DRAG: f32 = 4.0;
const ROCKET_GRAVITY: f32 = -16.0;
const ROCKET_SIZE: f32 = 6.0;
const ROCKET_MIN_LIFETIME: f32 = 0.8;
const ROCKET_MAX_LIFETIME: f32 = 1.2;

const SPARKLE_SPAWN_INTERVAL: f32 = 1.0 / 90.0;
const SPARKLE_SPAWNS_PER_TICK: usize = 2;
const SPARKLE_SPEED: f32 = 120.0;
const SPARKLE_LIFETIME: f32 = 0.2;
const SPARKLE_DRAG: f32 = 4.0;
const SPARKLE_GRAVITY: f32 = -16.0;
const SPARKLE_SIZE_START: f32 = 6.0;
const SPARKLE_SIZE_END: f32 = 0.5;

const EXPLOSION_PARTICLE_COUNT: usize = 320;
const EXPLOSION_SPEED_MAX: f32 = 460.0;
const EXPLOSION_LIFETIME: f32 = 1.2;
const EXPLOSION_DRAG: f32 = 2.0;
const EXPLOSION_GRAVITY: f32 = -16.0;
const EXPLOSION_SIZE_START: f32 = 10.0;
const EXPLOSION_SIZE_END: f32 = 10.0;
const EXPLOSION_SLOW_SIZE_BONUS_START: f32 = 1.2;
const EXPLOSION_SLOW_SIZE_BONUS_END: f32 = 2.2;

const DIGITS_COLUMNS: u32 = 5;
const DIGITS_ROWS: u32 = 2;

pub struct ParticlePlugin;
impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FireworkFx>().add_systems(
            Update,
            (
                FireworkLauncher::enact,
                FireworkLauncher::simulate_rockets,
                FireworkLauncher::simulate_particles,
                FireworkLauncher::boom_sound,
                FireworkLauncher::cleanup_orphans,
            )
                .chain(),
        );
    }
}

#[derive(Resource)]
struct FireworkFx {
    digits_image: Handle<Image>,
    digits_atlas_layout: Handle<TextureAtlasLayout>,
}

impl FromWorld for FireworkFx {
    fn from_world(world: &mut World) -> Self {
        let digits_image = world
            .get_resource::<DigitSheet>()
            .expect("DigitSheet not loaded")
            .0
            .clone();

        let tile_size = {
            let images = world.resource::<Assets<Image>>();
            let image = images
                .get(&digits_image)
                .expect("DigitSheet image missing from Assets<Image>");
            UVec2::new(
                (image.width() / DIGITS_COLUMNS).max(1),
                (image.height() / DIGITS_ROWS).max(1),
            )
        };

        let digits_atlas_layout = {
            let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
            layouts.add(TextureAtlasLayout::from_grid(
                tile_size,
                DIGITS_COLUMNS,
                DIGITS_ROWS,
                None,
                None,
            ))
        };

        Self {
            digits_image,
            digits_atlas_layout,
        }
    }
}

#[derive(Component, Copy, Clone)]
struct OwnedBy(Entity);

#[derive(Component)]
struct FireworkRocket {
    velocity: Vec3,
    age: f32,
    lifetime: f32,
    trail_timer: f32,
}

#[derive(Component)]
struct FireworkParticle {
    velocity: Vec3,
    age: f32,
    lifetime: f32,
    drag: f32,
    gravity: f32,
    start_size: f32,
    end_size: f32,
    start_color: Vec4,
    end_color: Vec4,
}

#[derive(Component)]
struct BoomTimer(Timer);

#[derive(Component, Clone)]
#[require(Transform, GlobalTransform)]
pub struct FireworkLauncher {
    pub radius: f32,
    pub min_delay: f32,
    pub max_delay: f32,
    timer: Timer,
}

impl FireworkLauncher {
    pub fn new(radius: f32, min_delay: f32, max_delay: f32) -> Self {
        let (min_d, max_d) = if min_delay <= max_delay {
            (min_delay, max_delay)
        } else {
            (max_delay, min_delay)
        };
        let initial = rand_delay(min_d, max_d);
        Self {
            radius,
            min_delay: min_d,
            max_delay: max_d,
            timer: Timer::from_seconds(initial, TimerMode::Once),
        }
    }

    fn enact(
        time: Res<Time>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut q: Query<(Entity, &GlobalTransform, &mut FireworkLauncher)>,
    ) {
        for (launcher_entity, gt, mut launcher) in &mut q {
            launcher.timer.tick(time.delta());
            if !launcher.timer.just_finished() {
                continue;
            }

            let theta = std::f32::consts::TAU * rand::random::<f32>();
            let r = launcher.radius * rand::random::<f32>().sqrt();
            let offset = Vec3::new(r * theta.cos(), 0.0, r * theta.sin());
            let spawn_pos = gt.translation() + offset;

            let lifetime = rand_delay(ROCKET_MIN_LIFETIME, ROCKET_MAX_LIFETIME);
            let velocity = Vec3::new(0.0, rand_delay(ROCKET_MIN_SPEED, ROCKET_MAX_SPEED), 0.0);

            commands.spawn((
                Name::new("firework_rocket"),
                OwnedBy(launcher_entity),
                FireworkRocket {
                    velocity,
                    age: 0.0,
                    lifetime,
                    trail_timer: 0.0,
                },
                Sprite::from_color(Color::linear_rgb(2.0, 2.0, 2.0), Vec2::splat(ROCKET_SIZE)),
                Transform::from_translation(spawn_pos),
            ));

            commands.spawn((
                OwnedBy(launcher_entity),
                BoomTimer(Timer::from_seconds(lifetime + 0.35, TimerMode::Once)),
            ));

            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/effects/firework_whistle.ogg")),
                PlaybackSettings {
                    volume: Volume::Linear(0.02),
                    ..PlaybackSettings::DESPAWN
                },
            ));

            let next = rand_delay(launcher.min_delay, launcher.max_delay);
            launcher.timer.set_duration(Duration::from_secs_f32(next));
            launcher.timer.reset();
        }
    }

    fn simulate_rockets(
        time: Res<Time>,
        mut commands: Commands,
        fx: Res<FireworkFx>,
        mut q: Query<(Entity, &OwnedBy, &mut FireworkRocket, &mut Transform)>,
    ) {
        let dt = time.delta_secs();
        for (rocket_entity, owner, mut rocket, mut transform) in &mut q {
            rocket.age += dt;
            rocket.velocity.y += ROCKET_GRAVITY * dt;
            let rocket_drag = rocket.velocity * ROCKET_DRAG * dt;
            rocket.velocity -= rocket_drag;
            transform.translation += rocket.velocity * dt;

            rocket.trail_timer += dt;
            while rocket.trail_timer >= SPARKLE_SPAWN_INTERVAL {
                rocket.trail_timer -= SPARKLE_SPAWN_INTERVAL;
                for _ in 0..SPARKLE_SPAWNS_PER_TICK {
                    Self::spawn_sparkle_particle(
                        &mut commands,
                        owner.0,
                        transform.translation,
                        transform.translation.z + 0.1,
                    );
                }
            }

            if rocket.age < rocket.lifetime {
                continue;
            }

            Self::spawn_explosion_digits(&mut commands, &fx, owner.0, transform.translation);
            commands.entity(rocket_entity).despawn();
        }
    }

    fn spawn_explosion_digits(
        commands: &mut Commands,
        fx: &FireworkFx,
        owner: Entity,
        origin: Vec3,
    ) {
        let color_a = Vec4::new(0.609, 3.117, 5.977, 1.0);
        let color_b = Vec4::new(4.988, 0.398, 2.086, 1.0);

        for _ in 0..EXPLOSION_PARTICLE_COUNT {
            let theta = std::f32::consts::TAU * rand::random::<f32>();
            let dir = Vec2::new(theta.cos(), theta.sin());
            // Wide 0..max spread to fill the burst disk instead of a ring.
            let speed_t = rand::random::<f32>();
            let speed = EXPLOSION_SPEED_MAX * speed_t;
            let velocity = Vec3::new(dir.x * speed, dir.y * speed, 0.0);
            let slow_factor = 1.0 - speed_t;
            let start_size = EXPLOSION_SIZE_START + EXPLOSION_SLOW_SIZE_BONUS_START * slow_factor;
            let end_size = EXPLOSION_SIZE_END + EXPLOSION_SLOW_SIZE_BONUS_END * slow_factor;

            let start_color = if rand::random::<f32>() < 0.5 {
                color_a
            } else {
                color_b
            };

            let atlas_index = if rand::random::<f32>() < 0.5 { 0 } else { 1 };
            let mut sprite = Sprite::from_atlas_image(
                fx.digits_image.clone(),
                TextureAtlas {
                    layout: fx.digits_atlas_layout.clone(),
                    index: atlas_index,
                },
            );
            sprite.custom_size = Some(Vec2::splat(start_size));
            sprite.color = Color::linear_rgba(
                start_color.x,
                start_color.y,
                start_color.z,
                start_color.w,
            );

            commands.spawn((
                Name::new("firework_digit_particle"),
                OwnedBy(owner),
                FireworkParticle {
                    velocity,
                    age: 0.0,
                    lifetime: EXPLOSION_LIFETIME,
                    drag: EXPLOSION_DRAG,
                    gravity: EXPLOSION_GRAVITY,
                    start_size,
                    end_size,
                    start_color,
                    end_color: Vec4::new(start_color.x, start_color.y, start_color.z, 0.0),
                },
                sprite,
                Transform::from_translation(origin),
            ));
        }
    }

    fn spawn_sparkle_particle(commands: &mut Commands, owner: Entity, origin: Vec3, z: f32) {
        let theta = std::f32::consts::TAU * rand::random::<f32>();
        let speed = SPARKLE_SPEED * rand::random::<f32>();
        let velocity = Vec3::new(theta.cos() * speed, theta.sin() * speed, 0.0);

        let start_color = Vec4::new(4.0, 4.0, 4.0, 1.0);
        let end_color = Vec4::new(4.0, 4.0, 4.0, 0.0);

        commands.spawn((
            Name::new("firework_sparkle_particle"),
            OwnedBy(owner),
            FireworkParticle {
                velocity,
                age: 0.0,
                lifetime: SPARKLE_LIFETIME,
                drag: SPARKLE_DRAG,
                gravity: SPARKLE_GRAVITY,
                start_size: SPARKLE_SIZE_START,
                end_size: SPARKLE_SIZE_END,
                start_color,
                end_color,
            },
            Sprite::from_color(
                Color::linear_rgba(
                    start_color.x,
                    start_color.y,
                    start_color.z,
                    start_color.w,
                ),
                Vec2::splat(SPARKLE_SIZE_START),
            ),
            Transform::from_translation(Vec3::new(origin.x, origin.y, z)),
        ));
    }

    fn simulate_particles(
        time: Res<Time>,
        mut commands: Commands,
        mut q: Query<(Entity, &mut FireworkParticle, &mut Transform, &mut Sprite)>,
    ) {
        let dt = time.delta_secs();
        for (entity, mut particle, mut transform, mut sprite) in &mut q {
            particle.age += dt;
            if particle.age >= particle.lifetime {
                commands.entity(entity).despawn();
                continue;
            }

            particle.velocity.y += particle.gravity * dt;
            let particle_drag = particle.velocity * particle.drag * dt;
            particle.velocity -= particle_drag;
            transform.translation += particle.velocity * dt;

            let t = (particle.age / particle.lifetime).clamp(0.0, 1.0);
            let size = particle.start_size + (particle.end_size - particle.start_size) * t;
            sprite.custom_size = Some(Vec2::splat(size));

            let color = particle.start_color + (particle.end_color - particle.start_color) * t;
            sprite.color = Color::linear_rgba(color.x, color.y, color.z, color.w);
        }
    }

    fn boom_sound(
        time: Res<Time>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
        q_launchers: Query<(), With<FireworkLauncher>>,
        mut q: Query<(Entity, &OwnedBy, &mut BoomTimer)>,
    ) {
        for (entity, owner, mut boom) in &mut q {
            if q_launchers.get(owner.0).is_err() {
                commands.entity(entity).despawn();
                continue;
            }

            boom.0.tick(time.delta());
            if !boom.0.just_finished() {
                continue;
            }
            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/effects/firework.ogg")),
                PlaybackSettings {
                    volume: Volume::Linear(0.1),
                    ..PlaybackSettings::DESPAWN
                },
            ));
            commands.entity(entity).despawn();
        }
    }

    fn cleanup_orphans(
        mut commands: Commands,
        q_launchers: Query<(), With<FireworkLauncher>>,
        q_owned: Query<
            (Entity, &OwnedBy),
            Or<(With<FireworkRocket>, With<FireworkParticle>, With<BoomTimer>)>,
        >,
    ) {
        for (entity, owner) in &q_owned {
            if q_launchers.get(owner.0).is_err() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn rand_delay(min_d: f32, max_d: f32) -> f32 {
    min_d + (max_d - min_d) * rand::random::<f32>()
}
