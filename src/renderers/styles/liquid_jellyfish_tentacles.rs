//! Liquid Jellyfish Tentacles style renderer (`liquidJellyfishTentacles`).
//!
//! Visual Concept:
//! - Masterpiece Radial Bioluminescent Jellyfish Engine.
//! - Translucent glowing jellyfish bell centered on canvas with smooth pulsating rim.
//! - 32 Wavy bioluminescent fluid tentacles drifting radially outward with fluid sine-wave motion.
//! - Sparkling bioluminescent spore motes & deep-sea volumetric ambient aura.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

const TENTACLE_COUNT: usize = 32;
const SPORE_COUNT:    usize = 45;

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
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 125.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.38 + be * 0.10);

    // Deep-sea Bioluminescent Palette
    let bio_cyan    = mix(glow, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.85);
    let bio_magenta = mix(accent, Color::rgba(1.0, 0.10, 0.70, 1.0), 0.85);
    let bio_indigo  = mix(p_col, Color::rgba(0.12, 0.05, 0.45, 1.0), 0.75);
    let spark_white = Color::rgba(1.0, 1.0, 1.0, 0.95);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Deep-Sea Ambient Radial Aura
    let bg_sea = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, base_r * 2.8,
        &[
            (0.00, mix(bio_cyan, bio_magenta, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, bio_indigo.with_alpha(0.15 + bs * 0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_sea);
    c.fill_rect(cx - base_r * 2.8, cy - base_r * 2.8, base_r * 5.6, base_r * 5.6);

    // 2. Pulsating Jellyfish Bell Dome
    let bell_r = inner_r * (1.0 + 0.08 * bs);
    let bell_fill = Fill::radial_gradient(
        cx - bell_r * 0.2, cy - bell_r * 0.2, 0.0,
        cx, cy, bell_r * 1.5,
        &[
            (0.00, spark_white.with_alpha(0.85)),
            (0.40, bio_cyan.with_alpha(0.60 + be * 0.15)),
            (0.80, bio_magenta.with_alpha(0.35)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bell_fill);
    c.set_shadow(bio_cyan, (20.0 + bs * 14.0) * user_scale);
    c.fill_circle(cx, cy, bell_r * 1.15);

    c.set_stroke(Fill::Solid(mix(bio_cyan, spark_white, 0.60).with_alpha(0.85)));
    c.set_line_width((2.4 + be * 1.4) * user_scale);
    c.stroke_circle(cx, cy, bell_r * 1.15);

    // 3. 32 Wavy Radial Bio-Tentacles
    let step = (freq.len() / TENTACLE_COUNT).max(1);
    let rot = frame_time * 0.05;

    for i in 0..TENTACLE_COUNT {
        let t = i as f32 / TENTACLE_COUNT as f32;
        let angle = t * TAU + rot;

        let fv = smooth_ring_bin(freq, step, i, TENTACLE_COUNT);
        let val = (fv * sensitivity * 1.2 + be * 0.30 + bs * 0.20).clamp(0.05, 2.4);

        let t_len = (30.0 + val * 120.0) * user_scale;
        let (cos_a, sin_a) = angle.sin_cos();

        // Slow fluid sway along the length of each tentacle
        let sway = (frame_time * 1.8 + t * 7.0).sin() * 0.22 + (frame_time * 0.9 + t * 3.0).cos() * 0.12;
        let mid_a = angle + sway;
        let tip_a = mid_a + sway * 0.6;

        let (cos_m, sin_m) = mid_a.sin_cos();
        let (cos_t, sin_t) = tip_a.sin_cos();

        let p0    = (cx + cos_a * bell_r, cy + sin_a * bell_r);
        let p_mid = (cx + cos_m * (bell_r + t_len * 0.5), cy + sin_m * (bell_r + t_len * 0.5));
        let p_tip = (cx + cos_t * (bell_r + t_len), cy + sin_t * (bell_r + t_len));

        let tentacle_pts = GpuCanvas::sample_quadratic(p0, p_mid, p_tip, 8);
        let tentacle_col = match i % 3 {
            0 => bio_cyan,
            1 => bio_magenta,
            _ => mix(glow, spark_white, 0.5),
        };

        c.set_stroke(Fill::Solid(tentacle_col.with_alpha(0.65 + val * 0.25)));
        c.set_line_width((2.2 + val * 1.4) * user_scale);
        c.set_shadow(tentacle_col, (12.0 + val * 10.0) * user_scale);
        c.stroke_polyline(&tentacle_pts);

        // Glowing spore mote at tentacle tip
        c.set_fill(Fill::Solid(spark_white.with_alpha(0.85)));
        c.set_shadow(tentacle_col, (10.0 + bs * 8.0) * user_scale);
        c.fill_circle(p_tip.0, p_tip.1, (2.6 + val * 1.8) * user_scale);
    }

    // 4. Floating Ambient Bio-Spores
    for d in 0..SPORE_COUNT {
        let df = d as f32;
        let spore_r = inner_r + (10.0 + (df * 7.3).sin().abs() * 110.0 + be * 25.0) * user_scale;
        let spore_a = (df * 137.5).to_radians() + frame_time * (0.12 + (df * 3.1).cos() * 0.08);

        let sx = cx + spore_a.cos() * spore_r;
        let sy = cy + spore_a.sin() * spore_r;
        let s_sz = (1.5 + (df * 5.3).sin().abs() * 3.0 + be * 1.2) * user_scale;
        let s_col = mix(bio_cyan, bio_magenta, (df * 0.25).sin().abs());

        c.set_fill(Fill::Solid(mix(s_col, spark_white, 0.50).with_alpha(0.75)));
        c.set_shadow(s_col, (8.0 + be * 6.0) * user_scale);
        c.fill_circle(sx, sy, s_sz);
    }

    // 5. Central Glowing Core Disc
    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.85);

    let _ = s_col;
    c.set_global_alpha(1.0);
    c.restore();
}

