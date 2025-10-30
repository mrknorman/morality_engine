//! Firework
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
                FireworkLauncher::enact,
                FireworkLauncher::start_rig_shutdown,
                FireworkLauncher::finish_despawn,
                FireworkLauncher::boom_sound,
            ));
    }
}

const FIREWORK_SIZE : f32 = 1.0;

/// Rocket (OnDie) -> child_index 0 used for digits/explosion
fn create_rocket_expl_effect() -> EffectAsset {
    let w = ExprWriter::new();

    let init_pos = SetPositionCircleModifier {
        center: w.lit(Vec3::ZERO).expr(),
        axis: w.lit(Vec3::Y).expr(),
        radius: w.lit(100.).expr(),
        dimension: ShapeDimension::Volume,
    };

    let zero = w.lit(0.);
    let y = w.lit(2400.).uniform(w.lit(2600.));
    let v = zero.clone().vec3(y, zero);
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, v.expr());

    let init_age = SetAttributeModifier::new(Attribute::AGE, w.lit(0.).expr());

    // random color stored for children if needed later
    let rgb = w.rand(VectorType::VEC3F) * w.lit(3.0);
    let color = rgb.vec4_xyz_w(w.lit(1.)).pack4x8unorm();
    let init_trails_color = SetAttributeModifier::new(Attribute::U32_0, color.expr());

    let update_accel = AccelModifier::new((w.lit(Vec3::Y) * w.lit(-16.)).expr());
    let update_drag  = LinearDragModifier::new(w.lit(4.).expr());

    // Only OnDie spawn; child_index = 0 (sole child)
    let spawn_on_die = EmitSpawnEventModifier {
        condition: EventEmitCondition::OnDie,
        count: w.lit(1000u32).expr(),
        child_index: 0,
    };

    let spawner = SpawnerSettings::once(1.0.into());

    let mut module = w.finish();
    let rocket_lt = module.add_property("rocket_lt", 1.0.into());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.prop(rocket_lt));

    EffectAsset::new(32, spawner, module)
        .with_name("rocket_expl")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .init(init_trails_color)
        .update(update_drag)
        .update(update_accel)
        .update(spawn_on_die)
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

/// Rocket (Always) -> child_index 0 used for sparkle trail
fn create_rocket_trail_effect() -> EffectAsset {
    let w = ExprWriter::new();

    let init_pos = SetPositionCircleModifier {
        center: w.lit(Vec3::ZERO).expr(),
        axis: w.lit(Vec3::Y).expr(),
        radius: w.lit(100.).expr(),
        dimension: ShapeDimension::Volume,
    };

    let zero = w.lit(0.);
    let y = w.lit(2400.).uniform(w.lit(2600.));
    let v = zero.clone().vec3(y, zero);
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, v.expr());
    let init_age = SetAttributeModifier::new(Attribute::AGE, w.lit(0.).expr());

    let update_accel = AccelModifier::new((w.lit(Vec3::Y) * w.lit(-16.)).expr());
    let update_drag  = LinearDragModifier::new(w.lit(4.).expr());

    // Only Always spawn; child_index = 0 (sole child)
    let spawn_trail = EmitSpawnEventModifier {
        condition: EventEmitCondition::Always,
        count: w.lit(5u32).expr(),
        child_index: 0,
    };

    let spawner = SpawnerSettings::once(1.0.into());

    let mut module = w.finish();
    let rocket_lt = module.add_property("rocket_lt", 1.0.into());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.prop(rocket_lt));

    EffectAsset::new(32, spawner, module)
        .with_name("rocket_trail")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .update(update_accel)
        .update(spawn_trail)
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

/// Sparkle trail child
fn create_sparkle_trail_effect() -> EffectAsset {
    let w = ExprWriter::new();

    let init_pos = InheritAttributeModifier::new(Attribute::POSITION);

    let vel = w.rand(VectorType::VEC3F);
    let vel = (vel * w.lit(2.) - w.lit(1.)).normalized();
    let vel = (vel * w.lit(1.)).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel);

    let init_age = SetAttributeModifier::new(Attribute::AGE, w.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, w.lit(0.2).expr());
    let update_accel = AccelModifier::new((w.lit(Vec3::Y) * w.lit(-16.)).expr());
    let update_drag  = LinearDragModifier::new(w.lit(4.).expr());

    let spawner = SpawnerSettings::default();

    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient.add_key(0.8, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient.add_key(1.0, Vec4::new(4.0, 4.0, 4.0, 0.0));

    EffectAsset::new(5000, spawner, w.finish())
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

/// Digits (explosion) child
fn create_trails_effect() -> EffectAsset {
    const OPTION_GLOW: f32 = 6.0;
    let mut w = ExprWriter::new();

    let slot0_expr = w.lit(0).expr();

    let init_pos = InheritAttributeModifier::new(Attribute::POSITION);
    let speed    = w.lit(340. * FIREWORK_SIZE).uniform(w.lit(440. * FIREWORK_SIZE));
    let dir      = w.rand(VectorType::VEC3F).mul(w.lit(2.0)).sub(w.lit(1.0)).normalized();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, (dir * speed).expr());
    let init_age = SetAttributeModifier::new(Attribute::AGE, w.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, w.lit(1.2).expr());
    let update_accel  = AccelModifier::new((w.lit(Vec3::Y) * w.lit(-16.)).expr());
    let update_drag   = LinearDragModifier::new(w.lit(2.).expr());

    let idx_f  = (w.rand(ScalarType::Float) * w.lit(2.0)).floor();
    let c0   = w.lit(Vec3::new(0.1015 * OPTION_GLOW, 0.5195 * OPTION_GLOW, 0.9961 * OPTION_GLOW));
    let c1   = w.lit(Vec3::new(0.8314 * OPTION_GLOW, 0.0664 * OPTION_GLOW, 0.3477 * OPTION_GLOW));
    let rgb  = c0.clone() + (c1 - c0) * idx_f.clone();
    let rgba = rgb.vec4_xyz_w(w.lit(1.0)).expr();
    let init_hdr_color = SetAttributeModifier::new(Attribute::HDR_COLOR, rgba);

    let init_alpha = SetAttributeModifier::new(Attribute::ALPHA, w.lit(1.0).expr());
    let orient = OrientModifier::new(OrientMode::ParallelCameraDepthPlane);

    let mut module = w.finish();
    module.add_texture_slot("digits");
    let idx_i = module.cast(idx_f.expr(), ScalarType::Int);
    let init_sprite = SetAttributeModifier::new(Attribute::SPRITE_INDEX, idx_i);

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
        .init(init_hdr_color)
        .init(init_alpha)
        .update(update_drag)
        .update(update_accel)
        .render(ParticleTextureModifier {
            texture_slot: slot0_expr,
            sample_mapping: ImageSampleMapping::Modulate,
        })
        .render(ColorOverLifetimeModifier {
            gradient: fade,
            blend: ColorBlendMode::Modulate,
            mask: ColorBlendMask::RGBA,
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
    pub rocket_expl: Handle<EffectAsset>,
    pub rocket_trail: Handle<EffectAsset>,
    pub sparkle: Handle<EffectAsset>,
    pub trails: Handle<EffectAsset>,
}
impl FromWorld for FireworkFx {
    fn from_world(world: &mut World) -> Self {
        let mut effects = world.resource_mut::<Assets<EffectAsset>>();
        Self {
            rocket_expl: effects.add(create_rocket_expl_effect()),
            rocket_trail: effects.add(create_rocket_trail_effect()),
            sparkle: effects.add(create_sparkle_trail_effect()),
            trails: effects.add(create_trails_effect()),
        }
    }
}

#[derive(Component)]
struct BoomTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct DespawnAfterFrames(u32);

use bevy::ecs::entity_disabling::Disabled;

#[derive(Component, Clone)]
#[component(on_remove = FireworkLauncher::on_remove)]
#[component(on_insert = FireworkLauncher::on_insert)]
pub struct FireworkLauncher {
    pub radius: f32,
    pub min_delay: f32,
    pub max_delay: f32,
    timer: Timer,
}

#[derive(Component)]
struct RigShutdown;

#[derive(Component, Copy, Clone)]
struct RigRoot(pub Entity);

#[derive(Component)]
struct RocketExplEnt(pub Entity);

#[derive(Component)]
struct RocketTrailEnt(pub Entity);

impl FireworkLauncher {
    pub fn new(radius: f32, min_delay: f32, max_delay: f32) -> Self {
        let (min_d, max_d) = if min_delay <= max_delay { (min_delay, max_delay) } else { (max_delay, min_delay) };
        let initial = rand_delay(min_d, max_d);
        Self {
            radius,
            min_delay: min_d,
            max_delay: max_d,
            timer: Timer::from_seconds(initial, TimerMode::Once),
        }
    }

    pub fn on_remove(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let root = world.entity(entity).get::<RigRoot>().map(|rr| rr.0);
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
            let mut stack = vec![root];
            while let Some(e) = stack.pop() {
                if let Ok(mut spawner) = q_spawner.get_mut(e) {
                    spawner.active = false;
                }
                commands.entity(e).insert(Disabled);
                if let Ok(children) = q_children.get(e) {
                    for c in children.iter() {
                        stack.push(c);
                    }
                }
            }
            commands.entity(root).insert(DespawnAfterFrames(2));
        }
    }

    fn finish_despawn(
        mut commands: Commands,
        mut q: Query<(Entity, &mut DespawnAfterFrames), With<Disabled>>,
    ) {
        for (root, mut frames) in &mut q {
            if **frames > 0 { **frames -= 1; continue; }
            commands.entity(root).despawn();
        }
    }

    pub fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        // Prefetch
        let (rocket_expl_h, rocket_trail_h, sparkle_h, trails_h, digits_img, xform) = {
            let fx = world.get_resource::<FireworkFx>().expect("FireworkFx not initialized");
            let digits_sheet = world.get_resource::<DigitSheet>().expect("DigitSheet not loaded");
            let xform = world.entity(entity).get::<Transform>().cloned().unwrap_or_default();
            (
                fx.rocket_expl.clone(),
                fx.rocket_trail.clone(),
                fx.sparkle.clone(),
                fx.trails.clone(),
                digits_sheet.0.clone(),
                xform,
            )
        };

        // Build rig
        let rig_root = world.commands().spawn((Name::new("firework_rig_root"), xform)).id();

        // Rocket A: explosion-only (OnDie), child_index 0 -> digits
        let rocket_expl_e = world.commands().spawn((
            Name::new("rocket_expl_emitter"),
            ParticleEffect::new(rocket_expl_h),
            EffectProperties::default(),
            EffectSpawner::new(&SpawnerSettings::once(1.0.into()).with_emit_on_start(false)),
            Transform::default(),
            GlobalTransform::default(),
        )).id();

        let digits_e = world.commands().spawn((
            Name::new("digit_trails"),
            ParticleEffect::new(trails_h),
            EffectParent::new(rocket_expl_e), // maps to child_index 0
            EffectMaterial { images: vec![digits_img], ..Default::default() },
            Transform::default(),
            GlobalTransform::default(),
        )).id();

        world.commands().entity(rocket_expl_e).add_child(digits_e);

        // Rocket B: trail-only (Always), child_index 0 -> sparkle
        let rocket_trail_e = world.commands().spawn((
            Name::new("rocket_trail_emitter"),
            ParticleEffect::new(rocket_trail_h),
            EffectProperties::default(),
            EffectSpawner::new(&SpawnerSettings::once(1.0.into()).with_emit_on_start(false)),
            Transform::default(),
            GlobalTransform::default(),
        )).id();

        let sparkle_e = world.commands().spawn((
            Name::new("sparkle_trail"),
            ParticleEffect::new(sparkle_h),
            EffectParent::new(rocket_trail_e), // maps to child_index 0
            Transform::default(),
            GlobalTransform::default(),
        )).id();

        world.commands().entity(rocket_trail_e).add_child(sparkle_e);

        // Parent both rockets under the rig root
        world.commands().entity(rig_root).add_children(&[rocket_expl_e, rocket_trail_e]);

        // Store refs on the launcher
        world.commands().entity(entity).insert((
            RigRoot(rig_root),
            RocketExplEnt(rocket_expl_e),
            RocketTrailEnt(rocket_trail_e),
        ));
    }

    fn enact(
        time: Res<Time>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut q: Query<(&GlobalTransform, &mut FireworkLauncher, Option<&RigRoot>, Option<&RocketExplEnt>, Option<&RocketTrailEnt>)>,
        mut q_root_xform: Query<&mut Transform>,
        mut q_props: Query<&mut EffectProperties>,
        mut q_spawner: Query<&mut bevy_hanabi::EffectSpawner>,
    ) {
        for (gt, mut launcher, rig_opt, expl_opt, trail_opt) in &mut q {
            launcher.timer.tick(time.delta());
            if !launcher.timer.is_finished() { continue; }

            let theta = 2.0 * std::f32::consts::PI * rand::random::<f32>();
            let r = launcher.radius * rand::random::<f32>().sqrt();
            let offset = Vec3::new(r * theta.cos(), 0.0, r * theta.sin());
            let spawn_pos = gt.translation() + offset;

            if let Some(RigRoot(root)) = rig_opt {
                if let Ok(mut t) = q_root_xform.get_mut(*root) {
                    t.translation = spawn_pos;
                }
            }

            let lt: f32 = 0.8 + 0.4 * rand::random::<f32>();

            // Arm both rockets identically
            if let Some(RocketExplEnt(e)) = expl_opt {
                if let Ok(mut props) = q_props.get_mut(*e) {
                    *props = EffectProperties::default().with_properties([("rocket_lt".to_string(), lt.into())]);
                }
                if let Ok(mut spawner) = q_spawner.get_mut(*e) {
                    spawner.active = true;
                    spawner.reset();
                }
            }
            if let Some(RocketTrailEnt(e)) = trail_opt {
                if let Ok(mut props) = q_props.get_mut(*e) {
                    *props = EffectProperties::default().with_properties([("rocket_lt".to_string(), lt.into())]);
                }
                if let Ok(mut spawner) = q_spawner.get_mut(*e) {
                    spawner.active = true;
                    spawner.reset();
                }
            }

            // whistle + delayed boom
            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/effects/firework_whistle.ogg")),
                PlaybackSettings { volume: Volume::Linear(0.02), ..PlaybackSettings::DESPAWN },
            ));
            commands.spawn(BoomTimer(Timer::from_seconds(lt + 0.35, TimerMode::Once)));

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
            if !boom.0.just_finished() { continue; }
            commands.spawn((
                AudioPlayer::new(asset_server.load("audio/effects/firework.ogg")),
                PlaybackSettings { volume: Volume::Linear(0.1), ..PlaybackSettings::DESPAWN },
            ));
            commands.entity(e).despawn();
        }
    }
}

fn rand_delay(min_d: f32, max_d: f32) -> f32 {
    min_d + (max_d - min_d) * rand::random::<f32>()
}
