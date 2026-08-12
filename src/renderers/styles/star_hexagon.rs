//! Star Hexagon style renderer (`StarHexagon`).
//!
//! Visual Concept:
//! - Dual Interlocking 6-Pointed Star Hexagram Matrix framing the logo.
//! - Inner triangular lattice weave with 360° even audio vertex expansion.
//! - Zero radial bars.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
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

    let rot = frame_time * 0.05;
    let step = (freq.len() / 6).max(1);

    // -------------------------------------------------------------------------
    // 1. DUAL INTERLOCKING 6-POINTED STAR TRIANGLES (Hexagram Matrix)
    // -------------------------------------------------------------------------
    for layer in 0..2 {
        let layer_rot = rot * (if layer == 0 { 1.0 } else { -1.0 }) + (layer as f32 * std::f32::consts::FRAC_PI_3);
        let mut tri1_pts: Vec<(f32, f32)> = Vec::with_capacity(4);
        let mut tri2_pts: Vec<(f32, f32)> = Vec::with_capacity(4);

        for k in 0..3 {
            let kf = k as f32;
            let a1 = layer_rot + (kf / 3.0) * TAU;
            let a2 = layer_rot + ((kf + 0.5) / 3.0) * TAU;

            let val1 = (bin_value(freq, step, k * 2) * sensitivity + be * 0.20).clamp(0.1, 1.5);
            let val2 = (bin_value(freq, step, k * 2 + 1) * sensitivity + bs * 0.20).clamp(0.1, 1.5);

            let r1 = inner_r + (45.0 + val1 * 65.0 + layer as f32 * 18.0) * user_scale;
            let r2 = inner_r + (45.0 + val2 * 65.0 + layer as f32 * 18.0) * user_scale;

            tri1_pts.push((cx + a1.cos() * r1, cy + a1.sin() * r1));
            tri2_pts.push((cx + a2.cos() * r2, cy + a2.sin() * r2));
        }

        if let Some(&first1) = tri1_pts.first() { tri1_pts.push(first1); }
        if let Some(&first2) = tri2_pts.first() { tri2_pts.push(first2); }

        let star_col = mix(p_col, glow, layer as f32 * 0.5);

        // Triangle 1
        c.set_stroke(Fill::Solid(star_col));
        c.set_line_width((2.8 + bs * 1.5) * user_scale);
        c.set_shadow(star_col, (16.0 + bs * 10.0) * user_scale);
        c.stroke_polyline(&tri1_pts);

        // Triangle 2
        c.set_stroke(Fill::Solid(mix(star_col, Color::WHITE, 0.65)));
        c.set_line_width((2.8 + bs * 1.5) * user_scale);
        c.set_shadow(glow, (16.0 + bs * 10.0) * user_scale);
        c.stroke_polyline(&tri2_pts);

        // Vertex Motes
        for pt in tri1_pts.iter().chain(tri2_pts.iter()).take(6) {
            c.set_fill(Fill::Solid(mix(accent, Color::WHITE, 0.85)));
            c.set_shadow(glow, 10.0 * user_scale);
            c.fill_circle(pt.0, pt.1, (3.2 + bs * 1.5) * user_scale);
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

    let _ = s_col;
    c.set_global_alpha(1.0);
    c.restore();
}
