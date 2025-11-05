// src/systems/cascade_gpu/mod.rs

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_hanabi::prelude::*;
use std::{cell::RefCell, rc::Rc};

use crate::startup::cursor::CustomCursor;
use crate::startup::textures::DigitSheet;
use crate::systems::colors::{OPTION_1_COLOR, OPTION_2_COLOR};

/// Convert Bevy Color (likely sRGB) to linear RGB for HDR_COLOR.
fn lin_rgb(c: Color) -> (f32, f32, f32) {
    let l = c.to_linear();
    (l.red, l.green, l.blue)
}

pub struct CascadeGpuPlugin;
impl Plugin for CascadeGpuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_cursor_hover_props);
    }
}

#[derive(Component, Clone)]
#[require(Transform, Visibility)]
#[component(on_insert = CascadeGPU::on_insert)]
pub struct CascadeGPU {
    pub digit_size_px: f32,
    pub spacing: f32,
    pub speed_px_s: f32,
}
impl Default for CascadeGPU {
    fn default() -> Self {
        Self {
            digit_size_px: 14.0,
            spacing: 1.20,
            speed_px_s: 110.0,
        }
    }
}

#[derive(Component)]
struct CascadeColumn;

/// Grid/phys parameters derived from config + window.
struct Grid {
    cols: u32,
    start_x: f32,
    emit_y: f32,
    rows_per_sec: f32,
    lifetime: f32,
    capacity: u32,
    glyph_px: f32,
    step: f32,
    speed: f32,
}

/// Load sprite sheet handle and window size.
fn load_digits_and_window(world: &mut DeferredWorld) -> (Handle<Image>, Vec2) {
    let digits = world
        .get_resource::<DigitSheet>()
        .expect("DigitSheet not loaded");
    let img = digits.0.clone();

    let size = world
        .try_query::<&Window>()
        .and_then(|mut q| q.iter(world).next())
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::new(1280.0, 720.0));

    (img, size)
}

/// Compute grid/physics numbers from config and window.
fn grid_from(cfg: &CascadeGPU, win_size: Vec2) -> Grid {
    let half_w = win_size.x * 0.5;
    let half_h = win_size.y * 0.5;

    let glyph_px = cfg.digit_size_px.max(1.0);
    let step = (glyph_px * cfg.spacing).max(1.0);

    let cols = ((2.0 * half_w) / step).floor().max(1.0) as u32;
    let start_x = -half_w + step * 0.5;
    let emit_y = half_h + step * 0.5;

    let speed = cfg.speed_px_s.max(1.0);
    let rows_per_sec = speed / step;

    let lifetime = ((2.0 * half_h) + step) / speed + 0.5;
    let rows_visible = ((2.0 * half_h) / step).ceil().max(1.0) as u32 + 2;
    let capacity = rows_visible.saturating_mul(4);

    Grid {
        cols,
        start_x,
        emit_y,
        rows_per_sec,
        lifetime,
        capacity,
        glyph_px,
        step,
        speed,
    }
}

/// Build the Hanabi effect and return its handle.
fn build_effect(effects: &mut Assets<EffectAsset>, grid: &Grid) -> Handle<EffectAsset> {
    // -------- Init module (writer #1) ----------
    let w = ExprWriter::new();
    let slot0_expr = w.lit(0).expr();

    // Init: POSITION / VELOCITY / LIFETIME
    let zero = w.lit(0.0);
    let pos = zero.clone().vec3(zero.clone(), zero.clone());
    let init_pos = SetAttributeModifier::new(Attribute::POSITION, pos.expr());

    let ny = w.lit(-grid.speed);
    let vel = zero.clone().vec3(ny, zero.clone());
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel.expr());

    let init_lifetime =
        SetAttributeModifier::new(Attribute::LIFETIME, w.lit(grid.lifetime).expr());

    // Per-particle seed + row index at spawn (anchors pattern to letters)
    let seed0 = w.rand(ScalarType::Float);
    let init_row_seed = SetAttributeModifier::new(Attribute::F32_0, seed0.expr());
    let spawn_row = (w.time() * w.lit(grid.rows_per_sec)).floor();
    let init_row_index = SetAttributeModifier::new(Attribute::F32_1, spawn_row.expr());

    // Close writer #1
    let module = w.finish();
    let module_rc = Rc::new(RefCell::new(module));

    {
        let mut m = module_rc.borrow_mut();
        m.add_texture_slot("digits");
    }

    // -------- Properties ----------
    let col_seed = {
        let mut m = module_rc.borrow_mut();
        m.add_property("col_seed", 0.0.into())
    };
    let col_id = {
        let mut m = module_rc.borrow_mut();
        m.add_property("col_id", 0.0.into())
    };

    // Visibility / noise props
    let evolution_rate = {
        let mut m = module_rc.borrow_mut();
        m.add_property("evolution_rate", 0.05_f32.into())
    };
    let noise_scale_x = {
        let mut m = module_rc.borrow_mut();
        m.add_property("noise_scale_x", 0.005_f32.into())
    };
    let noise_scale_y = {
        let mut m = module_rc.borrow_mut();
        m.add_property("noise_scale_y", 0.005_f32.into())
    };
    let x_jitter_scale = {
        let mut m = module_rc.borrow_mut();
        m.add_property("x_jitter_scale", 0.30_f32.into())
    };
    let y_jitter_scale = {
        let mut m = module_rc.borrow_mut();
        m.add_property("y_jitter_scale", 0.30_f32.into())
    };
    let cross_mix = {
        let mut m = module_rc.borrow_mut();
        m.add_property("cross_mix", 0.12_f32.into())
    };
    let rotate_deg = {
        let mut m = module_rc.borrow_mut();
        m.add_property("rotate_deg", 15.0_f32.into())
    };
    let flow_x = {
        let mut m = module_rc.borrow_mut();
        m.add_property("flow_x", 0.85_f32.into())
    };
    let flow_y = {
        let mut m = module_rc.borrow_mut();
        m.add_property("flow_y", (-0.65_f32).into())
    };
    let on_threshold = {
        let mut m = module_rc.borrow_mut();
        m.add_property("on_threshold", 0.50_f32.into())
    };
    let threshold_jitter = {
        let mut m = module_rc.borrow_mut();
        m.add_property("threshold_jitter", 0.04_f32.into())
    };

    // Palette (linear) + flip chance
    let base_r = {
        let mut m = module_rc.borrow_mut();
        m.add_property("base_r", 1.0_f32.into())
    };
    let base_g = {
        let mut m = module_rc.borrow_mut();
        m.add_property("base_g", 1.0_f32.into())
    };
    let base_b = {
        let mut m = module_rc.borrow_mut();
        m.add_property("base_b", 1.0_f32.into())
    };
    let opt1_r = {
        let mut m = module_rc.borrow_mut();
        m.add_property("opt1_r", 0.20_f32.into())
    };
    let opt1_g = {
        let mut m = module_rc.borrow_mut();
        m.add_property("opt1_g", 1.00_f32.into())
    };
    let opt1_b = {
        let mut m = module_rc.borrow_mut();
        m.add_property("opt1_b", 0.60_f32.into())
    };
    let opt2_r = {
        let mut m = module_rc.borrow_mut();
        m.add_property("opt2_r", 1.00_f32.into())
    };
    let opt2_g = {
        let mut m = module_rc.borrow_mut();
        m.add_property("opt2_g", 0.20_f32.into())
    };
    let opt2_b = {
        let mut m = module_rc.borrow_mut();
        m.add_property("opt2_b", 0.60_f32.into())
    };
    let color_flip_chance = {
        let mut m = module_rc.borrow_mut();
        m.add_property("color_flip_chance", 0.10_f32.into())
    };

    // Cursor-hover properties
    let cur_dx = {
        let mut m = module_rc.borrow_mut();
        m.add_property("cur_dx", 1.0e9_f32.into())
    };
    let cur_dy = {
        let mut m = module_rc.borrow_mut();
        m.add_property("cur_dy", 1.0e9_f32.into())
    };
    let hover_radius = {
        let mut m = module_rc.borrow_mut();
        m.add_property("hover_radius", 150.0_f32.into())
    };
    let hover_min_dist = {
        let mut m = module_rc.borrow_mut();
        m.add_property("hover_min_dist", 10.0_f32.into())
    };
    let hover_max_scale = {
        let mut m = module_rc.borrow_mut();
        m.add_property("hover_max_scale", 2.0_f32.into())
    };

    // -------- Writer #2 (update shader) ----------
    let mut w2 = ExprWriter::from_module(module_rc.clone());

    // 0/1 digit flip desynced per particle (phase/period from spatial hash)
    let age = w2.attr(Attribute::AGE);
    let col = w2.prop(col_id);
    let row_idx = w2.attr(Attribute::F32_1);
    let seed = w2.prop(col_seed);
    let hash = |x: WriterExpr| {
        let s = x.clone().sin() * w2.lit(43758.5453123);
        s.clone() - s.floor()
    };
    let phase_src =
        col.clone() * w2.lit(12.9898) + row_idx.clone() * w2.lit(78.233) + seed.clone() * w2.lit(37.719);
    let phase = hash(phase_src);
    let per_src = col.clone() * w2.lit(4.898) + row_idx.clone() * w2.lit(7.23) + seed.clone() * w2.lit(1.23);
    let per_h = hash(per_src);
    let period = w2.lit(0.5) + per_h * w2.lit(9.5);
    let cycles = age.clone() / period.clone() + phase.clone();
    let frac = cycles.clone() - cycles.floor();
    let idx_f = (frac * w2.lit(2.0)).floor();
    let idx_i = {
        let mut m = module_rc.borrow_mut();
        m.cast(idx_f.clone().expr(), ScalarType::Int)
    };
    let init_sprite = SetAttributeModifier::new(Attribute::SPRITE_INDEX, idx_i);

    // Visibility mask (same as before)
    let col_jit = (w2.prop(col_seed) - w2.lit(0.5)) * w2.prop(x_jitter_scale);
    let phi = w2.lit(1.61803398875);
    let s = w2.attr(Attribute::F32_0) * phi.clone();
    let row_jit_src = s.clone() - s.floor();
    let row_jit = (row_jit_src - w2.lit(0.5)) * w2.prop(y_jitter_scale);

    let col_f = w2.prop(col_id) + col_jit;
    let row_f = w2.attr(Attribute::F32_1) + row_jit;
    let gx0 = col_f.clone() * w2.prop(noise_scale_x) + row_f.clone() * w2.prop(cross_mix);
    let gy0 = row_f.clone() * w2.prop(noise_scale_y) + col_f.clone() * w2.prop(cross_mix);

    let pi = w2.lit(3.14159265358979323846);
    let ang = w2.prop(rotate_deg) * pi / w2.lit(180.0);
    let ca = ang.clone().cos();
    let sa = ang.clone().sin();
    let gx = ca.clone() * gx0.clone() - sa.clone() * gy0.clone();
    let gy = sa * gx0 + ca * gy0;

    let phase_adv = w2.time() * w2.prop(evolution_rate);
    let x = gx.clone() + phase_adv.clone() * w2.prop(flow_x);
    let y = gy.clone() + phase_adv.clone() * w2.prop(flow_y);

    let ix = x.clone().floor();
    let iy = y.clone().floor();
    let fx = x.clone() - ix.clone();
    let fy = y.clone() - iy.clone();

    let qfade = |t: WriterExpr| {
        let t2 = t.clone() * t.clone();
        let t3 = t2.clone() * t.clone();
        let t4 = t3.clone() * t.clone();
        let t5 = t4.clone() * t.clone();
        t5 * w2.lit(6.0) - t4 * w2.lit(15.0) + t3 * w2.lit(10.0)
    };
    let u = qfade(fx.clone());
    let v = qfade(fy.clone());

    let hash2 = |ix_e: WriterExpr, iy_e: WriterExpr| {
        let s = (ix_e.clone() * w2.lit(127.1)
            + iy_e.clone() * w2.lit(311.7)
            + w2.prop(col_seed) * w2.lit(17.3))
            .sin()
            * w2.lit(43758.5453123);
        s.clone() - s.floor()
    };

    let ix0 = ix.clone();
    let iy0 = iy.clone();
    let ix1 = ix.clone() + w2.lit(1.0);
    let iy1 = iy.clone() + w2.lit(1.0);
    let h00 = hash2(ix0.clone(), iy0.clone());
    let h10 = hash2(ix1.clone(), iy0.clone());
    let h01 = hash2(ix0.clone(), iy1.clone());
    let h11 = hash2(ix1.clone(), iy1.clone());
    let tau = w2.lit(6.28318530717958647692);

    let a00 = h00.clone() * tau.clone();
    let a10 = h10.clone() * tau.clone();
    let a01 = h01.clone() * tau.clone();
    let a11 = h11.clone() * tau.clone();
    let g00x = a00.clone().cos();
    let g00y = a00.clone().sin();
    let g10x = a10.clone().cos();
    let g10y = a10.clone().sin();
    let g01x = a01.clone().cos();
    let g01y = a01.clone().sin();
    let g11x = a11.clone().cos();
    let g11y = a11.clone().sin();

    let dx00 = fx.clone() - w2.lit(0.0);
    let dy00 = fy.clone() - w2.lit(0.0);
    let dx10 = fx.clone() - w2.lit(1.0);
    let dy10 = fy.clone() - w2.lit(0.0);
    let dx01 = fx.clone() - w2.lit(0.0);
    let dy01 = fy.clone() - w2.lit(1.0);
    let dx11 = fx.clone() - w2.lit(1.0);
    let dy11 = fy.clone() - w2.lit(1.0);

    let n00 = g00x.clone() * dx00.clone() + g00y.clone() * dy00.clone();
    let n10 = g10x.clone() * dx10.clone() + g10y.clone() * dy10.clone();
    let n01 = g01x.clone() * dx01.clone() + g01y.clone() * dy01.clone();
    let n11 = g11x.clone() * dx11.clone() + g11y.clone() * dy11.clone();

    let lerp = |a: WriterExpr, b: WriterExpr, texpr: WriterExpr| a.clone() + texpr.clone() * (b.clone() - a.clone());
    let nx0 = lerp(n00.clone(), n10.clone(), u.clone());
    let nx1 = lerp(n01.clone(), n11.clone(), u.clone());
    let n = lerp(nx0, nx1, v.clone());

    let noise01 = n.clone() * w2.lit(0.5) + w2.lit(0.5);

    // Binary visibility (no hysteresis)
    let thr_j = (w2.attr(Attribute::F32_0) - w2.lit(0.5)) * w2.prop(threshold_jitter);
    let thr = w2.prop(on_threshold) + thr_j;
    let step_src = (noise01.clone() - thr.clone()) * w2.lit(1000.0);
    let mask = step_src.max(w2.lit(0.0)).min(w2.lit(1.0)).ceil();

    // Occasional color flips on digit flip cycles
    let cyc = (age.clone() / period.clone() + phase.clone()).floor();
    let chance_src = cyc.clone() * w2.lit(19.19)
        + col.clone() * w2.lit(7.37)
        + row_idx.clone() * w2.lit(5.11)
        + seed.clone() * w2.lit(13.13);
    let chance = {
        let s = chance_src.clone().sin() * w2.lit(43758.5453123);
        s.clone() - s.floor()
    };
    let thresh = w2.lit(1.0) - w2.prop(color_flip_chance);
    let rare = ((chance - thresh) * w2.lit(1000.0))
        .max(w2.lit(0.0))
        .min(w2.lit(1.0))
        .ceil();

    let one = w2.lit(1.0);
    let idx_inv = one.clone() - idx_f.clone();
    let opt_r = w2.prop(opt2_r) * idx_inv.clone() + w2.prop(opt1_r) * idx_f.clone();
    let opt_g = w2.prop(opt2_g) * idx_inv.clone() + w2.prop(opt1_g) * idx_f.clone();
    let opt_b = w2.prop(opt2_b) * idx_inv.clone() + w2.prop(opt1_b) * idx_f.clone();

    let inv_rare = one.clone() - rare.clone();
    let chosen_r = w2.prop(base_r) * inv_rare.clone() + opt_r * rare.clone();
    let chosen_g = w2.prop(base_g) * inv_rare.clone() + opt_g * rare.clone();
    let chosen_b = w2.prop(base_b) * inv_rare.clone() + opt_b * rare.clone();

    let rgb = chosen_r.clone().vec3(chosen_g.clone(), chosen_b.clone()) * mask.clone();
    let rgba = rgb.vec4_xyz_w(mask.clone());
    let set_hdr = SetAttributeModifier::new(Attribute::HDR_COLOR, rgba.expr());
    let set_alpha = SetAttributeModifier::new(Attribute::ALPHA, mask.clone().expr());

    // Per-frame sprite update (int flip)
    let set_sprite = SetAttributeModifier::new(
        Attribute::SPRITE_INDEX,
        {
            let frac_dyn =
                ((w2.attr(Attribute::AGE) / period.clone()) + phase.clone()).clone()
                    - ((w2.attr(Attribute::AGE) / period) + phase).floor();
            let idx_f2 = (frac_dyn * w2.lit(2.0)).floor();
            let idx_i2 = {
                let mut m = module_rc.borrow_mut();
                m.cast(idx_f2.expr(), ScalarType::Int)
            };
            idx_i2
        },
    );

    // >>> Cursor-proximity enlarge (multiply existing size; no vecN construction) <<<
    let p = w2.attr(Attribute::POSITION);
    let dx = w2.prop(cur_dx) - p.clone().x();
    let dy = w2.prop(cur_dy) - p.y();
    let dist = dx.clone().vec2(dy.clone()).length();

    let rad = w2.prop(hover_radius);
    let min_d = w2.prop(hover_min_dist);
    let max_s = w2.prop(hover_max_scale);

    // t = clamp((rad - dist) / (rad - min_d), 0, 1)
    let t_raw = (rad.clone() - dist.clone()) / (rad.clone() - min_d.clone());
    let t = t_raw.max(w2.lit(0.0)).min(w2.lit(1.0));
    let scale = w2.lit(1.0) + (max_s - w2.lit(1.0)) * t;

    // base_px ~ your glyph pixel size (use the same value you intended before)
    let base_px = w2.lit(grid.glyph_px * 0.96);
    let final_px = base_px * scale;

    // Write final pixel size directly; keep it vec3 to match the render pipeline.
    let size_vec3 = final_px.clone().vec3(final_px.clone(), final_px.clone());
    let set_size = SetAttributeModifier::new(Attribute::SIZE, size_vec3.expr());
    
    // -------- Build effect ----------
    let spawner = SpawnerSettings::rate(CpuValue::Single(grid.rows_per_sec));
    let module = w2.finish();
    let effect = EffectAsset::new(grid.capacity, spawner, module)
        .with_name("cascade_gpu_column")
        .init(init_pos)
        .init(init_vel)
        .init(init_sprite)
        .init(init_lifetime)
        .init(init_row_seed)
        .init(init_row_index)
        .update(set_sprite)
        .update(set_alpha)
        .update(set_hdr)
        .update(set_size)
        .render(FlipbookModifier {
            sprite_grid_size: UVec2::new(5, 2),
        })
        .render(ParticleTextureModifier {
            texture_slot: slot0_expr,                 // <- uses slot 0
            sample_mapping: ImageSampleMapping::Modulate, // tint with HDR_COLOR, keep alpha
        })
        // Base size in pixels; we then multiply it in the update by `scale`.
        .render(SizeOverLifetimeModifier {
            gradient: bevy_hanabi::Gradient::constant(Vec3::splat(1.0)),
            screen_space_size: true,
        })
        .render(OrientModifier::new(OrientMode::ParallelCameraDepthPlane))
        .with_alpha_mode(bevy_hanabi::AlphaMode::Blend);

    effects.add(effect)
}

/// Spawn the root and per-column particle entities. Returns the root entity.
fn spawn_columns(
    cmd: &mut Commands,
    effect_handle: Handle<EffectAsset>,
    images: Vec<Handle<Image>>,
    grid: &Grid,
    parent_xform: Transform,
    opt1_lin: (f32, f32, f32),
    opt2_lin: (f32, f32, f32),
) -> Entity {
    let parent = cmd
        .spawn((
            Name::new("cascade_gpu_root"),
            Transform::from_translation(parent_xform.translation),
            GlobalTransform::default(),
            Visibility::Visible,
        ))
        .id();

    let (o1r, o1g, o1b) = opt1_lin;
    let (o2r, o2g, o2b) = opt2_lin;

    for c in 0..grid.cols {
        let x = grid.start_x + (c as f32) * grid.step;
        let seed_value: f32 = rand::random::<f32>();

        let col = cmd
            .spawn((
                Name::new(format!("cascade_col_{c}")),
                CascadeColumn,
                ParticleEffect::new(effect_handle.clone()),
                EffectMaterial {
                    images: images.clone(),
                    ..Default::default()
                },
                EffectProperties::default().with_properties([
                    ("col_seed".to_string(), seed_value.into()),
                    ("col_id".to_string(), (c as f32).into()),
                    // Evolution / spatial scales
                    ("evolution_rate".to_string(), 0.05_f32.into()),
                    ("noise_scale_x".to_string(), 0.005_f32.into()),
                    ("noise_scale_y".to_string(), 0.005_f32.into()),
                    // Jitter / coupling / rotation
                    ("x_jitter_scale".to_string(), 0.30_f32.into()),
                    ("y_jitter_scale".to_string(), 0.30_f32.into()),
                    ("cross_mix".to_string(), 0.12_f32.into()),
                    ("rotate_deg".to_string(), 15.0_f32.into()),
                    // Drift
                    ("flow_x".to_string(), 0.85_f32.into()),
                    ("flow_y".to_string(), (-0.65_f32).into()),
                    // Visibility threshold
                    ("on_threshold".to_string(), 0.50_f32.into()),
                    ("threshold_jitter".to_string(), 0.04_f32.into()),
                    // Palette (linear) + flip chance
                    ("base_r".to_string(), 1.0_f32.into()),
                    ("base_g".to_string(), 1.0_f32.into()),
                    ("base_b".to_string(), 1.0_f32.into()),
                    ("opt1_r".to_string(), o1r.into()),
                    ("opt1_g".to_string(), o1g.into()),
                    ("opt1_b".to_string(), o1b.into()),
                    ("opt2_r".to_string(), o2r.into()),
                    ("opt2_g".to_string(), o2g.into()),
                    ("opt2_b".to_string(), o2b.into()),
                    ("color_flip_chance".to_string(), 0.10_f32.into()),
                    // Hover scale defaults
                    ("cur_dx".to_string(), (1.0e9_f32).into()),
                    ("cur_dy".to_string(), (1.0e9_f32).into()),
                    ("hover_radius".to_string(), 150.0_f32.into()),
                    ("hover_min_dist".to_string(), 10.0_f32.into()),
                    ("hover_max_scale".to_string(), 2.0_f32.into()),
                ]),
                Transform::from_translation(Vec3::new(x, grid.emit_y, 0.0)),
                GlobalTransform::default(),
                Visibility::Visible,
            ))
            .id();

        cmd.entity(parent).add_child(col);
    }

    parent
}

impl CascadeGPU {
    pub fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        // Config + parent transform
        let (cfg, parent_xform) = {
            let e = world.entity(entity);
            (
                e.get::<CascadeGPU>().cloned().unwrap_or_default(),
                e.get::<Transform>().cloned().unwrap_or_default(),
            )
        };

        // Resources + grid
        let (digits_image, win_size) = load_digits_and_window(&mut world);
        let grid = grid_from(&cfg, win_size);

        // Build effect
        let effect_handle = {
            let mut effects = world.resource_mut::<Assets<EffectAsset>>();
            build_effect(&mut effects, &grid)
        };

        // Spawn columns
        let mut cmd = world.commands();
        let parent = {
            let images = vec![digits_image];
            let opt1_lin = lin_rgb(OPTION_1_COLOR);
            let opt2_lin = lin_rgb(OPTION_2_COLOR);
            spawn_columns(
                &mut cmd,
                effect_handle,
                images,
                &grid,
                parent_xform,
                opt1_lin,
                opt2_lin,
            )
        };

        // Attach to the CascadeGPU entity
        cmd.entity(entity).add_child(parent);
    }
}

/// Per-frame: push cursor delta (cursor_world - column_world) into each column’s EffectProperties.
fn update_cursor_hover_props(
    cursor: Res<CustomCursor>,
    mut q: Query<(&GlobalTransform, &mut EffectProperties), With<CascadeColumn>>,
) {
    let Some(cursor_pos) = cursor.position else { return };

    for (gt, mut props) in &mut q {
        let col_world = gt.translation();
        let dx = cursor_pos.x - col_world.x;
        let dy = cursor_pos.y - col_world.y;
        props.set("cur_dx", dx.into());
        props.set("cur_dy", dy.into());
    }
}
