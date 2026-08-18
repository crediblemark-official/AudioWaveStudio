//! Pulsing Neon Arcade Matrix style renderer (`pulsingNeonArcade`).
//!
//! Segmented digital LED matrix block bars with rounded glowing neon capsules
//! and bright top cap glow motes.

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
    let bar_count    = ctx.config.reactivity.bar_count.clamp(8, 48);

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
    let max_blocks = 9usize;
    let block_gap = 3.5 * user_scale;
    let block_len = 9.0 * user_scale;

    // -------------------------------------------------------------------------
    // 1. SEGMENTED GLOWING NEON LED MATRIX CAPSULES
    // -------------------------------------------------------------------------
    for i in 0..bar_count {
        let t = i as f32 / bar_count as f32;
        let angle = t * TAU + rot;

        let base_wave = (angle * 3.0 + frame_time * 1.5).sin() * 0.12 + 0.18;
        let audio_v = crate::renderers::styles::radial_common::full_random_scattered_bin(
            freq, step, i, bar_count, ctx.beat_count,
        ) * sensitivity;
        let val = (base_wave + audio_v * 0.85 + be * 0.35 + bs * 0.20).clamp(0.15, 2.2);

        let active_blocks = ((val / 2.2) * max_blocks as f32).round() as usize;
        let (cos_a, sin_a) = angle.sin_cos();

        for b in 0..max_blocks {
            let bf = b as f32;
            let block_r0 = inner_r + 8.0 * user_scale + bf * (block_len + block_gap);
            let block_r1 = block_r0 + block_len;

            let mid_r = (block_r0 + block_r1) * 0.5;
            let bx = cx + cos_a * mid_r;
            let by = cy + sin_a * mid_r;

            if b < active_blocks {
                let is_top = b + 1 == active_blocks;
                let block_col = mix(p_col, glow, b as f32 / max_blocks as f32);

                // Rounded capsule via circle or multi-pass line with soft shadow
                c.set_fill(Fill::Solid(block_col));
                c.set_shadow(block_col, (6.0 + bf * 1.2) * user_scale);
                let cap_size = (2.2 + bf * 0.2) * user_scale;
                c.fill_circle(bx, by, cap_size);

                if is_top {
                    // Top peak capsule gets an intense white neon glow
                    c.set_fill(Fill::Solid(mix(block_col, Color::WHITE, 0.75)));
                    c.set_shadow(glow, (14.0 + bs * 10.0) * user_scale);
                    c.fill_circle(bx, by, (3.2 + bf * 0.2) * user_scale);
                }
            } else {
                // Inactive dim background grid dots
                c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.08)));
                c.set_shadow(Color::TRANSPARENT, 0.0);
                c.fill_circle(bx, by, 1.2 * user_scale);
            }
        }
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
