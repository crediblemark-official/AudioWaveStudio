//! Quantum Cloud style renderer (`QuantumCloud`).
//!
//! Visual Concept:
//! - 3D Spherical Quantum Particle Cloud forming 3D Lissajous orbits and probability motes.
//! - Audio-reactive orbital expansion and 360° even particle density.
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
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

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

    let num_particles = 96usize;
    let step = (freq.len() / num_particles).max(1);

    // -------------------------------------------------------------------------
    // 1. 3D QUANTUM PARTICLE LISSAJOUS SPHERE SPHERE CLOUD (360° Symmetric)
    // -------------------------------------------------------------------------
    for i in 0..num_particles {
        let pt = i as f32 / num_particles as f32;
        let phi = pt * TAU * 3.0 + frame_time * 0.4;
        let theta = pt * std::f32::consts::PI;

        // Symmetric audio indexing
        let sym_pt = if pt < 0.5 { pt * 2.0 } else { (1.0 - pt) * 2.0 };
        let bin_idx = ((sym_pt * num_particles as f32) as usize * step) % freq.len();
        let audio_v = bin_value(freq, step, bin_idx) * sensitivity;

        let cloud_r = inner_r + (35.0 + audio_v * 75.0 + (phi * 2.0).sin() * 20.0 + be * 25.0) * user_scale;

        // 3D Spherical Projection onto 2D canvas
        let (sin_t, cos_t) = theta.sin_cos();
        let (sin_p, cos_p) = phi.sin_cos();

        let x3d = cloud_r * sin_t * cos_p;
        let y3d = cloud_r * sin_t * sin_p;
        let z3d = cloud_r * cos_t;

        // Depth perspective scale & alpha
        let depth_t = (z3d / (base_r * 2.0) + 0.5).clamp(0.2, 1.0);
        let px = cx + x3d * (0.7 + depth_t * 0.3);
        let py = cy + y3d * (0.7 + depth_t * 0.3);

        // Blend p_col → s_col → accent → glow across the cloud volume
        let hue_t   = pt;
        let near_col = mix(p_col,   s_col,  hue_t);
        let far_col  = mix(accent,  glow,   hue_t);
        let p_color  = mix(near_col, far_col, depth_t);
        let p_size   = (1.8 + depth_t * 2.8 + bs * 1.5) * user_scale;

        c.set_fill(Fill::Solid(p_color.with_alpha(0.38 + depth_t * 0.57)));
        c.set_shadow(p_color, (6.0 + depth_t * 8.0) * user_scale);
        c.fill_circle(px, py, p_size);
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

    let _ = s_col;  // used via near_col mixing
    c.set_global_alpha(1.0);
    c.restore();
}
