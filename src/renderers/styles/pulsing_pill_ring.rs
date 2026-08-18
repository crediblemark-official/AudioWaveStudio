//! Pulsing Pill Ring style renderer (`pulsingPillRing`).
//!
//! Concentric Stadium Capsule Ring:
//! - Concentric radial ring of horizontal stadium pill capsules (`fill_rounded_rect`).
//! - Each pill capsule expands and glows with audio amplitude in 360° even symmetry.
//! - Zero flower petals or simple line bars.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

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
    let bar_count    = ctx.config.reactivity.bar_count.clamp(16, 64);

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.10 + bs * 0.05);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    let step = ((freq.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
    let rot = frame_time * 0.06;

    // -------------------------------------------------------------------------
    // 1. CONCENTRIC STADIUM PILL CAPSULE RING (360° Symmetrical Distribution)
    // -------------------------------------------------------------------------
    for i in 0..bar_count {
        let t = i as f32 / bar_count as f32;
        let angle = t * TAU + rot;

        let base_wave = (angle * 3.0 + frame_time * 1.5).sin() * 0.12 + 0.18;
        let audio_v = crate::renderers::styles::radial_common::full_random_scattered_bin(
            freq, step, i, bar_count, ctx.beat_count,
        ) * sensitivity;
        let val = (base_wave + audio_v * 0.85 + be * 0.35 + bs * 0.20).clamp(0.15, 2.2);

        let pill_dist = inner_r + (25.0 + val * 85.0) * user_scale;
        let (cos_a, sin_a) = angle.sin_cos();

        let px = cx + cos_a * pill_dist;
        let py = cy + sin_a * pill_dist;

        let pill_w = (14.0 + val * 12.0) * user_scale;
        let pill_h = (6.0 + val * 4.0) * user_scale;

        let pill_col = mix(p_col, glow, t);

        c.save();
        c.translate(px, py);
        c.rotate(angle);

        c.set_fill(Fill::Solid(pill_col));
        c.set_shadow(pill_col, (10.0 + val * 10.0) * user_scale);
        c.fill_rounded_rect(-pill_w * 0.5, -pill_h * 0.5, pill_w, pill_h, pill_h * 0.5);

        // Core white highlight
        c.set_fill(Fill::Solid(mix(pill_col, Color::WHITE, 0.70)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_rounded_rect(-pill_w * 0.3, -pill_h * 0.25, pill_w * 0.6, pill_h * 0.5, pill_h * 0.25);

        c.restore();
    }

    // -------------------------------------------------------------------------
    // 2. PUMPING CENTRAL DISC & NEON RING
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(glow));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(glow, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = (s_col, accent);
    c.set_global_alpha(1.0);
    c.restore();
}
