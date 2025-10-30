//! Firework
//!
//! This example demonstrates the use of the [`LinearDragModifier`] to slow down
//! particles over time. Combined with an HDR camera with Bloom, the example
//! renders a firework explosion.
//!
//! The firework effect is composed of several key elements:
//! - An HDR camera with [`Bloom`] to ensure the particles "glow".
//! - Use of a [`ColorOverLifetimeModifier`] with a [`Gradient`] made of colors
//!   outside the \[0:1\] range, to ensure bloom has an effect.
//! - [`SetVelocitySphereModifier`] with a reasonably large initial speed for
//!   particles, and [`LinearDragModifier`] to quickly slow them down. This is
//!   the core of the "explosion" effect.
//! - An [`AccelModifier`] to pull particles down once they slow down, for
//!   increased realism. This is a subtle effect, but of importance.
//!
//! The particles also have a trail. The trail particles are stitched together
//! to form an arc using [`EffectAsset::with_ribbons`].

use bevy::{audio::Volume, ecs::{lifecycle::HookContext, world::DeferredWorld}, prelude::*};
use bevy_hanabi::prelude::*;
use std::time::Duration;

use crate::startup::textures::DigitSheet;

pub struct ParticlePlugin;
impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {	
		app
        .init_resource::<FireworkFx>()
        .add_systems(Update, (
            //FireworkLauncher::setup_rig_on_add, // NEW
            FireworkLauncher::enact,            // reuses the rig now
            FireworkLauncher::start_rig_shutdown,
            FireworkLauncher::finish_despawn,
            FireworkLauncher::boom_sound
        ));
    }

}

const FIREWORK_SIZE : f32 = 1.0;

/// Create the effect for the rocket itself, which spawns infrequently and
/// rapidly raises until it explodes (dies).
fn create_rocket_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    // Always start from the same launch point
    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(100.).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed and direction
    let zero = writer.lit(0.);
    let y = writer.lit(2400.).uniform(writer.lit(2600.));
    let v = zero.clone().vec3(y, zero);
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, v.expr());

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Store a random color per particle, which will be inherited by the spark ones
    // on explosion. We don't store it in Attribute::COLOR otherwise it's going to
    // affect the color of the rocket particle itself.
    let rgb = writer.rand(VectorType::VEC3F) * writer.lit(3.0);
    let color = rgb.vec4_xyz_w(writer.lit(1.)).pack4x8unorm();
    let init_trails_color = SetAttributeModifier::new(Attribute::U32_0, color.expr());

    // Add constant downward acceleration to simulate gravity
    let accel = writer.lit(Vec3::Y * -16.).expr();
    let update_accel = AccelModifier::new(accel);

    // Add drag to make particles slow down as they ascend
    let drag = writer.lit(4.).expr();
    let update_drag = LinearDragModifier::new(drag);

    // explosion should be index 0
    let update_spawn_on_die = EmitSpawnEventModifier {
        condition: EventEmitCondition::OnDie,
        count: writer.lit(1000u32).expr(),
        child_index: 0,
    };

    // trail should be index 1
    let update_spawn_trail = EmitSpawnEventModifier {
        condition: EventEmitCondition::Always,
        count: writer.lit(5u32).expr(),
        child_index: 1,
    };

    let spawner = SpawnerSettings::once(1.0.into());

    let mut module = writer.finish();
    // Give a bit of variation by randomizing the lifetime per particle
    let rocket_lt = module.add_property("rocket_lt", 1.0.into());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.prop(rocket_lt));

    EffectAsset::new(32, spawner, module)
        .with_name("rocket")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .init(init_trails_color)
        .update(update_drag)
        .update(update_accel)
        .update(update_spawn_trail)
        .update(update_spawn_on_die)
        .render(ColorOverLifetimeModifier {
            gradient: bevy_hanabi::Gradient::constant(Vec4::ONE),
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: bevy_hanabi::Gradient::constant(Vec3::ONE * FIREWORK_SIZE),
            screen_space_size: false,
        })
}

/// Create the effect for the sparkle trail coming out of the rocket as it
/// raises in the air.
fn create_sparkle_trail_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    // Inherit the start position from the parent effect (the rocket particle)
    let init_pos = InheritAttributeModifier::new(Attribute::POSITION);

    // The velocity is random in any direction
    let vel = writer.rand(VectorType::VEC3F);
    let vel = vel * writer.lit(2.) - writer.lit(1.); // remap [0:1] to [-1:1]
    let vel = vel.normalized();
    let speed = writer.lit(1.); //.uniform(writer.lit(4.));
    let vel = (vel * speed).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel);

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.2).expr(); //.uniform(writer.lit(0.4)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add constant downward acceleration to simulate gravity
    let accel = writer.lit(Vec3::Y * -16.).expr();
    let update_accel = AccelModifier::new(accel);

    // Add drag to make particles slow down as they ascend
    let drag = writer.lit(4.).expr();
    let update_drag = LinearDragModifier::new(drag);

    // The (CPU) spawner is unused
    let spawner = SpawnerSettings::default();

    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient.add_key(0.8, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient.add_key(1.0, Vec4::new(4.0, 4.0, 4.0, 0.0));

    EffectAsset::new(5000, spawner, writer.finish())
        .with_name("sparkle_trail")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Modulate,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: bevy_hanabi::Gradient::constant(Vec3::ONE * FIREWORK_SIZE),
            screen_space_size: false,
        })
}

fn create_trails_effect() -> EffectAsset {
    const OPTION_GLOW: f32 = 6.0;

    let mut w = ExprWriter::new();

    // Texture slot 0
    let slot0_expr = w.lit(0).expr();

    // Position / motion
    let init_pos = InheritAttributeModifier::new(Attribute::POSITION);
    let speed    = w.lit(340. * FIREWORK_SIZE).uniform(w.lit(440. * FIREWORK_SIZE));
    let dir      = w.rand(VectorType::VEC3F).mul(w.lit(2.0)).sub(w.lit(1.0)).normalized();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, (dir * speed).expr());
    let init_age = SetAttributeModifier::new(Attribute::AGE, w.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, w.lit(1.2).expr());
    let update_accel  = AccelModifier::new(w.lit(Vec3::Y * -16.).expr());
    let update_drag   = LinearDragModifier::new(w.lit(2.).expr());

    // Random 0/1 index for digit AND color
    let rand_f = w.rand(ScalarType::Float);
    let idx_f  = (rand_f * w.lit(2.0)).floor();

    // Choose color by 0/1, write to HDR_COLOR
    let c0   = w.lit(Vec3::new(0.1015 * OPTION_GLOW, 0.5195 * OPTION_GLOW, 0.9961 * OPTION_GLOW));
    let c1   = w.lit(Vec3::new(0.8314 * OPTION_GLOW, 0.0664 * OPTION_GLOW, 0.3477 * OPTION_GLOW));
    let diff = c1.clone() - c0.clone();
    let rgb  = c0 + diff * idx_f.clone();
    let rgba = rgb.vec4_xyz_w(w.lit(1.0)).expr();
    let init_hdr_color = SetAttributeModifier::new(Attribute::HDR_COLOR, rgba);

    // Start fully opaque
    let init_alpha = SetAttributeModifier::new(Attribute::ALPHA, w.lit(1.0).expr());

    let orient = OrientModifier::new(OrientMode::ParallelCameraDepthPlane);

    // Finish writer and build module
    let mut module = w.finish();
    module.add_texture_slot("digits");

    // Flipbook needs SPRITE_INDEX (i32)
    let idx_i = module.cast(idx_f.expr(), ScalarType::Int);
    let init_sprite = SetAttributeModifier::new(Attribute::SPRITE_INDEX, idx_i);

    // Alpha fade: multiply alpha by this over lifetime (RGB are 1, so color unchanged)
    let mut fade = bevy_hanabi::Gradient::new();
    fade.add_key(0.0,  Vec4::new(1.0, 1.0, 1.0, 1.0));
    fade.add_key(1.0,  Vec4::new(1.0, 1.0, 1.0, 0.0));

    EffectAsset::new(5000, SpawnerSettings::default(), module)
        .with_name("trail_digits")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .init(init_sprite)
        .init(init_hdr_color)     // HDR tint
        .init(init_alpha)         // start alpha = 1
        .update(update_drag)
        .update(update_accel)
        // 1) Apply sprite (multiplies RGB and A by texture)
        .render(ParticleTextureModifier {
            texture_slot: slot0_expr,
            sample_mapping: ImageSampleMapping::Modulate,
        })
        // 2) Then fade alpha multiplicatively (RGB=1, so only A changes)
        .render(ColorOverLifetimeModifier {
            gradient: fade,
            blend: ColorBlendMode::Modulate,
            mask: ColorBlendMask::RGBA, // use RGBA for 0.17; RGB are 1 so only alpha effectively changes
        })
        .render(FlipbookModifier { sprite_grid_size: UVec2::new(5, 2) })
        .render(SizeOverLifetimeModifier {
            gradient: bevy_hanabi::Gradient::constant(Vec3::splat(10.0)),
            screen_space_size: true,
        })
        .render(orient)
        .with_alpha_mode(bevy_hanabi::AlphaMode::Blend)
}

#[derive(Resource)]
pub struct FireworkFx {
    pub rocket: Handle<EffectAsset>,
    pub sparkle: Handle<EffectAsset>,
    pub trails: Handle<EffectAsset>,
}

impl FromWorld for FireworkFx {
    fn from_world(world: &mut World) -> Self {
        let mut effects = world.resource_mut::<Assets<EffectAsset>>();

        Self {
            rocket: effects.add(create_rocket_effect()),
            sparkle: effects.add(create_sparkle_trail_effect()),
            trails: effects.add(create_trails_effect()),
        }
    }
}

#[derive(Component)]
struct BoomTimer(Timer);
/// Countdown to actually delete the subtree.
#[derive(Component, Deref, DerefMut)]
struct DespawnAfterFrames(u32);

use bevy::ecs::entity_disabling::Disabled;
#[derive(Component, Clone)]
#[component(on_remove = FireworkLauncher::on_remove)]
#[component(on_insert = FireworkLauncher::on_insert)]
pub struct FireworkLauncher {
    /// Launch circle radius (XZ plane) around the entity's Transform.
    pub radius: f32,
    /// Minimum time between launches (seconds).
    pub min_delay: f32,
    /// Maximum time between launches (seconds).
    pub max_delay: f32,
    /// Internal timer for next launch.
    timer: Timer
}

#[derive(Component)]
struct RigShutdown;


#[derive(Component, Copy, Clone)]
struct RigRoot(pub Entity);

#[derive(Component)]
struct RocketEnt(pub Entity);

impl FireworkLauncher {
    
    pub fn on_remove(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        // End the immutable borrow before calling `commands()`
        let root = {
            world.entity(entity).get::<RigRoot>().map(|rr| rr.0)
        };

        if let Some(root) = root {
            world.commands().entity(root).insert(RigShutdown);
        }
    }

    fn start_rig_shutdown(
        mut commands: Commands,
        q_added: Query<Entity, Added<RigShutdown>>,
        q_children: Query<&Children>,
        mut q_spawner: Query<&mut bevy_hanabi::EffectSpawner>,
    ) {
        for root in &q_added {
            // DFS the rig subtree
            let mut stack = vec![root];
            while let Some(e) = stack.pop() {
                if let Ok(mut spawner) = q_spawner.get_mut(e) {
                    spawner.active = false; // stop emission immediately
                }
                commands.entity(e).insert(Disabled);

                if let Ok(children) = q_children.get(e) {
                    // <-- Use a simple loop; no copied()/cloned() needed.
                    for c in children.iter() {
                        stack.push(c);
                    }
                }
            }
            // Give the renderer a couple of frames to stop referencing buffers
            commands.entity(root).insert(DespawnAfterFrames(2));
        }
    }

    fn finish_despawn(
        mut commands: Commands,
        mut q: Query<(Entity, &mut DespawnAfterFrames), With<Disabled>>,
    ) {
        for (root, mut frames) in &mut q {
            if **frames > 0 { **frames -= 1; continue; }
            // Despawn the whole rig subtree after a short delay
            commands.entity(root).despawn();
        }
    }

    pub fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        // --- 1) Prefetch owned data in a short scope (ends all &World borrows) ---
        let (rocket_h, sparkle_h, trails_h, digits_img, xform) = {
            let fx = world
                .get_resource::<FireworkFx>()
                .expect("FireworkFx not initialized");
            let digits_sheet = world
                .get_resource::<DigitSheet>()
                .expect("DigitSheet not loaded");

            let rocket_h  = fx.rocket.clone();
            let sparkle_h = fx.sparkle.clone();
            let trails_h  = fx.trails.clone();
            let digits_img = digits_sheet.0.clone();
            let xform = world
                .entity(entity)
                .get::<Transform>()
                .cloned()
                .unwrap_or_default();

            (rocket_h, sparkle_h, trails_h, digits_img, xform)
        }; // <-- immutable borrows dropped here

        // --- 2) Now it’s safe to use world.commands() (mutable) ---
        let rig_root = world
            .commands()
            .spawn((Name::new("firework_rig_root"), xform))
            .id();

        let rocket_e = world
            .commands()
            .spawn((
                Name::new("rocket_emitter"),
                ParticleEffect::new(rocket_h),
                EffectProperties::default(),
                EffectSpawner::new(
                    &SpawnerSettings::once(1.0.into()).with_emit_on_start(false),
                ),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        // child_index 0 → explosion digits (spawn-on-die)
        let digits_e = world
            .commands()
            .spawn((
                Name::new("digit_trails"),
                ParticleEffect::new(trails_h),
                EffectParent::new(rocket_e),
                EffectMaterial {
                    images: vec![digits_img],
                    ..Default::default()
                },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        // child_index 1 → sparkle trail (always)
        let sparkle_e = world
            .commands()
            .spawn((
                Name::new("sparkle_trail"),
                ParticleEffect::new(sparkle_h),
                EffectParent::new(rocket_e),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        // lock sibling order on the ROCKET (0 then 1)
        world
            .commands()
            .entity(rocket_e)
            .add_children(&[digits_e, sparkle_e]);

        // rocket under rig root
        world.commands().entity(rig_root).add_child(rocket_e);

        // store IDs on the launcher entity via small helper components
        world
            .commands()
            .entity(entity)
            .insert((RigRoot(rig_root), RocketEnt(rocket_e)));
    }

    /// Create a launcher that spawns fireworks every random N seconds in [min_delay, max_delay],
    /// positioned uniformly inside a circle of given radius.
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
            timer: Timer::from_seconds(initial, TimerMode::Once)
        }
    }

    fn enact(
        time: Res<Time>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut q: Query<(&GlobalTransform, &mut FireworkLauncher, Option<&RigRoot>, Option<&RocketEnt>)>,
        mut q_root_xform: Query<&mut Transform>,
        mut q_props: Query<&mut EffectProperties>,
        mut q_spawner: Query<&mut bevy_hanabi::EffectSpawner>,
    ) {
        for (gt, mut launcher, rig_opt, rocket_opt) in &mut q {
            launcher.timer.tick(time.delta());
            if !launcher.timer.is_finished() { continue; }

            // Random point in disc
            let theta = 2.0 * std::f32::consts::PI * rand::random::<f32>();
            let r = launcher.radius * rand::random::<f32>().sqrt();
            let offset = Vec3::new(r * theta.cos(), 0.0, r * theta.sin());
            let spawn_pos = gt.translation() + offset;

            // Move rig root in WORLD space (rig is detached)
            if let Some(RigRoot(root)) = rig_opt {
                if let Ok(mut t) = q_root_xform.get_mut(*root) {
                    t.translation = spawn_pos;
                }
            }

            // Randomize per-launch lifetime and fire one rocket
            let lt: f32 = 0.8 + 0.4 * rand::random::<f32>();
            if let Some(RocketEnt(rocket)) = rocket_opt {
                if let Ok(mut props) = q_props.get_mut(*rocket) {
                    *props = EffectProperties::default()
                        .with_properties([("rocket_lt".to_string(), lt.into())]);
                }


                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/effects/firework_whistle.ogg")),
                    PlaybackSettings {
                        volume: Volume::Linear(0.02),
                        ..PlaybackSettings::DESPAWN
                    },
                ));

                // separate delayed boom timer
                commands.spawn(BoomTimer(Timer::from_seconds(lt + 0.35, TimerMode::Once)));
                if let Ok(mut spawner) = q_spawner.get_mut(*rocket) {
                    spawner.active = true;
                    spawner.reset();
                }
            }

            // next shot
            let next = rand_delay(launcher.min_delay, launcher.max_delay);
            launcher.timer.set_duration(Duration::from_secs_f32(next));
            launcher.timer.reset();
        }
    }

    fn boom_sound(
        time: Res<Time>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
        mut q: Query<(Entity, &mut BoomTimer)>,
    ) {
        for (e, mut boom) in &mut q {
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
            commands.entity(e).despawn(); // one-shot timer
        }
    }
}

fn rand_delay(min_d: f32, max_d: f32) -> f32 {
    min_d + (max_d - min_d) * rand::random::<f32>()
}