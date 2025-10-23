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

use bevy::{
    core_pipeline::tonemapping::Tonemapping, post_process::bloom::Bloom, prelude::*,
    render::view::Hdr,
};
use bevy_hanabi::prelude::*;

use crate::startup::textures::DigitSheet;


const FIREWORK_SIZE : f32 = 1.0;

/// Create the effect for the rocket itself, which spawns infrequently and
/// rapidly raises until it explodes (dies).
fn create_rocket_effect() -> EffectAsset {
    let writer = ExprWriter::new();

    // Always start from the same launch point
    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(400.).expr(),
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

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.8).uniform(writer.lit(1.2)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

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


    let spawner = SpawnerSettings::rate((1., 3.).into());

    EffectAsset::new(32, spawner, writer.finish())
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

    EffectAsset::new(1000, spawner, writer.finish())
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
    let mut w = ExprWriter::new();

    // We'll bind our spritesheet to slot 0 on the entity. Make the "0" Expr now.
    let slot0_expr = w.lit(0).expr();

    // --- your usual expressions ---
    let init_pos = InheritAttributeModifier::new(Attribute::POSITION);

    let speed   = w.lit(340. * FIREWORK_SIZE).uniform(w.lit(440. * FIREWORK_SIZE));
    let dir     = w.rand(VectorType::VEC3F).mul(w.lit(2.0)).sub(w.lit(1.0)).normalized();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, (dir * speed).expr());

    let init_age      = SetAttributeModifier::new(Attribute::AGE, w.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, w.lit(1.2).expr());
    let update_accel  = AccelModifier::new(w.lit(Vec3::Y * -16.).expr());
    let update_drag   = LinearDragModifier::new(w.lit(2.).expr());

    // Flipbook frame 0..9 as floored float (0.17 accepts float for SPRITE_INDEX)
    let rand_f  = w.rand(ScalarType::Float);           // [0,1)
    let idx_f   = (rand_f * w.lit(2.0)).floor();   

    // Face camera so digits read well
    let orient = OrientModifier::new(OrientMode::ParallelCameraDepthPlane);

    // --- finish writer, then declare the texture slot on the Module ---
    let mut module = w.finish();
    // This line is the key: declare at least 1 texture slot in the module.
    // Use either of these depending on what your 0.17 patch exposes:
    module.add_texture_slot("digits"); // preferred on 0.17
    // (If your exact patch lacks the named overload, use: module.add_texture_slot();)

    
    // CAST float -> int here (SPRITE_INDEX requires i32)
    let idx_f_h = idx_f.expr();                            // <-- turn into ExprHandle
    let idx_i = module.cast(idx_f_h, ScalarType::Int);

    // Now we can build the init for SPRITE_INDEX using the i32 handle
    let init_sprite = SetAttributeModifier::new(Attribute::SPRITE_INDEX, idx_i);

    EffectAsset::new(10000, SpawnerSettings::default(), module)
        .with_name("trail_digits")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .init(init_sprite)
        .update(update_drag)
        .update(update_accel)
        // Keep color simple in RENDER stage (avoid packing u32 in INIT)
        .render(ColorOverLifetimeModifier {
            gradient: bevy_hanabi::Gradient::constant(Vec4::ONE),
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        // Tell Hanabi to sample from slot 0; we'll bind the image at runtime
        .render(ParticleTextureModifier {
            texture_slot: slot0_expr,
            sample_mapping: ImageSampleMapping::Modulate,
        })
        // 5x2 grid (0..9)
        .render(FlipbookModifier { sprite_grid_size: UVec2::new(5, 2) })
        // Make digits obvious first; tweak later
        .render(SizeOverLifetimeModifier {
            gradient: bevy_hanabi::Gradient::constant(Vec3::splat(10.0)),
            screen_space_size: true,
        })
        .render(orient)
        .with_alpha_mode(bevy_hanabi::AlphaMode::Blend)
}

pub fn add_fireworks(
    mut commands: Commands, 
    effects: ResMut<Assets<EffectAsset>>,
    digits: Res<DigitSheet>
    ) {
        
    create_effect(commands, effects, &digits.0);
}

fn create_effect(
    mut commands: Commands, 
    mut effects: ResMut<Assets<EffectAsset>>,
    digits: &Handle<Image>,
) {
    // Rocket
    let rocket_effect = effects.add(create_rocket_effect());
    let rocket_entity = commands
        .spawn((Name::new("rocket"), ParticleEffect::new(rocket_effect), Transform::from_xyz(-0.0, -400.0, 100.0)))
        .id();

    // Sparkle trail
    let sparkle_trail_effect = effects.add(create_sparkle_trail_effect());
    commands.spawn((
        Name::new("sparkle_trail"),
        ParticleEffect::new(sparkle_trail_effect),
        // Set the rocket effect as parent. This gives access to the rocket effect's particles,
        // which in turns allows inheriting their position (and other attributes if needed).
        EffectParent::new(rocket_entity),
    ));

    // Trails
    let trails_effect = effects.add(create_trails_effect());
    commands.spawn((
        Name::new("trails"),
        ParticleEffect::new(trails_effect),
        // Set the rocket effect as parent. This gives access to the rocket effect's particles,
        // which in turns allows inheriting their position (and other attributes if needed).
        EffectParent::new(rocket_entity),

        EffectMaterial {
            images: vec![digits.clone()],    
            ..Default::default()
        }
    ));
}