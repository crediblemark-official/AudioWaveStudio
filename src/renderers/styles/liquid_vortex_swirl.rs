//! Liquid Vortex Swirl style renderer (`liquidVortexSwirl`).
//!
//! Visual Concept:
//! - Masterpiece Radial Liquid Whirlpool & Fluid Vortex Engine.
//! - 6 Smooth logarithmic liquid vortex spiral arms winding radially outward with fluid surface tension fills.
//! - Audio-reactive winding velocity, glowing fluid caustics, & orbiting whirlpool motes.
//! - Deep volumetric atmospheric backdrop.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

const VORTEX_ARMS: usize = 6;
const ARM_STEPS:   usize = 64;
const MOTE_COUNT:  usize = 45;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col  = theme_primary(theme);
    let s_col  = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let bass_mult    = ctx.config.reactivity.bass_multiplier;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = ctx.config.position_y * height * 0.5;

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 130.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.38 + be * 0.10);

    // Liquid Vortex Palette
    let vortex_cyan   = mix(glow, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.85);
    let vortex_violet = mix(accent, Color::rgba(0.60, 0.05, 1.0, 1.0), 0.85);
    let vortex_indigo = mix(p_col, Color::rgba(0.08, 0.04, 0.35, 1.0), 0.75);
    let spark_white   = Color::rgba(1.0, 1.0, 1.0, 0.95);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Deep Volumetric Radial Vortex Backdrop
    let bg_vortex = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, base_r * 2.8,
        &[
            (0.00, mix(vortex_cyan, vortex_violet, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, vortex_indigo.with_alpha(0.16 + bs * 0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_vortex);
    c.fill_rect(cx - base_r * 2.8, cy - base_r * 2.8, base_r * 5.6, base_r * 5.6);

    // 2. 6 Logarithmic Radial Liquid Vortex Spiral Arms
    let step = (freq.len() / VORTEX_ARMS).max(1);
    let rot = frame_time * 0.22;

    for arm in 0..VORTEX_ARMS {
        let af = arm as f32;
        let arm_angle_base = (af / VORTEX_ARMS as f32) * TAU + rot;

        let fv = smooth_ring_bin(freq, step, arm, VORTEX_ARMS);
        let val = (fv * sensitivity * 1.2 + be * 0.30 + bs * 0.20).clamp(0.05, 2.4);

        let wind = (120.0 + val * 100.0) * user_scale;

        let mut top_pts: Vec<(f32, f32)> = Vec::with_capacity(ARM_STEPS + 1);
        let mut bot_pts: Vec<(f32, f32)> = Vec::with_capacity(ARM_STEPS + 1);

        for si in 0..=ARM_STEPS {
            let t = si as f32 / ARM_STEPS as f32;
            let r = inner_r + t.powf(1.25) * wind;
            let angle = arm_angle_base + t * 2.2 * TAU;

            let arm_w = (12.0 + (1.0 - t) * 20.0 + val * 8.0) * user_scale;

            let (cos_a, sin_a) = angle.sin_cos();
            let (perp_cos, perp_sin) = (angle + std::f32::consts::FRAC_PI_2).sin_cos();

            top_pts.push((cx + cos_a * r + perp_cos * (arm_w * 0.5), cy + sin_a * r + perp_sin * (arm_w * 0.5)));
            bot_pts.push((cx + cos_a * r - perp_cos * (arm_w * 0.5), cy + sin_a * r - perp_sin * (arm_w * 0.5)));
        }

        let mut arm_polygon = Vec::with_capacity(top_pts.len() + bot_pts.len() + 1);
        arm_polygon.extend(top_pts.iter().copied());
        arm_polygon.extend(bot_pts.iter().rev().copied());
        if let Some(&first) = arm_polygon.first() { arm_polygon.push(first); }

        let arm_col = match arm % 3 {
            0 => vortex_cyan,
            1 => vortex_violet,
            _ => mix(glow, spark_white, 0.5),
        };

        let arm_fill = Fill::radial_gradient(
            cx, cy, 0.0,
            cx, cy, inner_r + wind,
            &[
                (0.00, mix(arm_col, spark_white, 0.55).with_alpha(0.85)),
                (0.40, arm_col.with_alpha(0.70 + be * 0.12)),
                (0.80, vortex_indigo.with_alpha(0.35)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(arm_fill);
        c.set_shadow(arm_col, (16.0 + bs * 10.0) * user_scale);
        fill_radial_polygon(c, cx, cy, &arm_polygon);

        c.set_stroke(Fill::Solid(mix(arm_col, spark_white, 0.70).with_alpha(0.88)));
        c.set_line_width((2.2 + bs * 1.2) * user_scale);
        c.stroke_polyline(&top_pts);
    }

    // 3. Orbiting Whirlpool Motes
    for d in 0..MOTE_COUNT {
        let df = d as f32;
        let mote_r = inner_r + (10.0 + (df * 7.3).sin().abs() * 110.0 + be * 25.0) * user_scale;
        let mote_a = (df * 137.5).to_radians() + frame_time * (0.25 + (df * 3.1).cos() * 0.10);

        let mx = cx + mote_a.cos() * mote_r;
        let my = cy + mote_a.sin() * mote_r;
        let m_sz = (1.5 + (df * 5.3).sin().abs() * 3.0 + be * 1.2) * user_scale;
        let m_col = mix(vortex_cyan, vortex_violet, (df * 0.25).sin().abs());

        c.set_fill(Fill::Solid(mix(m_col, spark_white, 0.50).with_alpha(0.75)));
        c.set_shadow(m_col, (8.0 + be * 6.0) * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // 4. Central Glowing Core Disc
    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.85);

    let _ = s_col;
    c.set_global_alpha(1.0);
    c.restore();
}
