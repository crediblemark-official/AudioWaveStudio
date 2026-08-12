//! Biomorphic Bloom style renderer (`BiomorphicBloom`).
//!
//! Visual Concept:
//! - Fibonacci Golden Ratio Spiral (Phyllotaxis Sunflower / Pinecone pattern).
//! - Golden angle $\theta = n \times 137.5077^\circ$ dot matrix with 360° even audio expansion.
//! - Zero standard mandala petals.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const GOLDEN_ANGLE: f32 = 2.399_963_2; // 137.5077 degrees in radians

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
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.10 + bs * 0.05);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Ambient bloom backdrop
    let bg_bloom = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, base_r * 2.0,
        &[
            (0.0,  mix(p_col, s_col, 0.5).with_alpha(0.12 + be * 0.08)),
            (0.65, glow.with_alpha(0.04)),
            (1.0,  Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_bloom);
    c.fill_rect(cx - base_r * 2.0, cy - base_r * 2.0, base_r * 4.0, base_r * 4.0);

    let num_seeds = 120usize;
    let step = (freq.len() / num_seeds).max(1);
    let rot = frame_time * 0.06;

    // -------------------------------------------------------------------------
    // 1. FIBONACCI GOLDEN RATIO SPIRAL DOT MATRIX (Phyllotaxis)
    // -------------------------------------------------------------------------
    for n in 0..num_seeds {
        let nf = n as f32;
        let theta = nf * GOLDEN_ANGLE + rot;

        let audio_v = crate::renderers::styles::radial_common::full_random_scattered_bin(
            freq, step, n, num_seeds, ctx.beat_count,
        ) * sensitivity;

        // Fibonacci sqrt radius formula: r = c * sqrt(n)
        let max_seed_sqrt = (num_seeds as f32).sqrt();
        let spiral_r = inner_r + (nf.sqrt() / max_seed_sqrt) * (base_r * 1.35 + audio_v * 75.0 + be * 30.0);

        let px = cx + theta.cos() * spiral_r;
        let py = cy + theta.sin() * spiral_r;

        let seed_col = mix(p_col, mix(accent, Color::WHITE, 0.65), nf / num_seeds as f32);
        let seed_size = (2.0 + (nf / num_seeds as f32) * 2.5 + bs * 1.5) * user_scale;

        c.set_fill(Fill::Solid(seed_col));
        c.set_shadow(glow, (8.0 + bs * 8.0) * user_scale);
        c.fill_circle(px, py, seed_size);
    }

    // Bloom web: connect each seed to its nearby golden-angle neighbours
    // (6 steps away in the sequence ≈ nearest in the spiral pattern).
    let connect_offsets = [1usize, 6, 13];
    for n in 0..num_seeds {
        let nf    = n as f32;
        let theta = nf * GOLDEN_ANGLE + rot;
        let max_sq = (num_seeds as f32).sqrt();
        let r0 = inner_r + (nf.sqrt() / max_sq) * (base_r * 1.35);
        let px0 = cx + theta.cos() * r0;
        let py0 = cy + theta.sin() * r0;

        for &off in &connect_offsets {
            let m = n + off;
            if m >= num_seeds { continue; }

            let mf    = m as f32;
            let theta2 = mf * GOLDEN_ANGLE + rot;
            let r1 = inner_r + (mf.sqrt() / max_sq) * (base_r * 1.35);
            let px1 = cx + theta2.cos() * r1;
            let py1 = cy + theta2.sin() * r1;

            // Fade the connection by distance in spiral space
            let dist_t = off as f32 / *connect_offsets.last().unwrap() as f32;
            let line_col = mix(mix(p_col, s_col, nf / num_seeds as f32), accent, dist_t * 0.5);

            c.set_stroke(Fill::Solid(line_col.with_alpha(0.18 - dist_t * 0.10)));
            c.set_line_width((0.6 + bs * 0.4) * user_scale);
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.stroke_line(px0, py0, px1, py1);
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

    let _ = (s_col, TAU);
    c.set_global_alpha(1.0);
    c.restore();
}
