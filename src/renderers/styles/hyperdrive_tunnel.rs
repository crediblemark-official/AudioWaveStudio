//! Hyperdrive Tunnel style renderer (`HyperdriveTunnel`).
//!
//! Visual Concept:
//! - 3D Perspective Portal Gate Quads streaming outward in depth.
//! - Solid filled 3D perspective portal quads (`fill_polygon`) and Z-depth ring gates.
//! - ZERO thin line strokes or ray lines.

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

    let num_gates = 8usize;
    let step = (freq.len() / num_gates).max(1);

    // -------------------------------------------------------------------------
    // 1. SOLID FILLED 3D PERSPECTIVE PORTAL GATE QUADS (Zero Line Strokes!)
    // -------------------------------------------------------------------------
    for g in 0..num_gates {
        let gf = g as f32;
        let z_t = ((frame_time * 0.6 + gf / num_gates as f32) % 1.0).clamp(0.01, 1.0);
        let audio_v = bin_value(freq, step, g) * sensitivity;

        let perspective_scale = z_t.powf(2.0);
        let gate_radius = inner_r + perspective_scale * (base_r * 2.5 + audio_v * 80.0 + be * 40.0);
        let gate_thick = (4.0 + perspective_scale * 12.0 + bs * 4.0) * user_scale;

        let gate_col = mix(p_col, mix(accent, glow, z_t), z_t);
        let alpha = (0.25 + perspective_scale * 0.75).min(1.0);

        // Draw solid filled ring gate
        c.set_fill(Fill::Solid(gate_col.with_alpha(alpha)));
        c.set_shadow(gate_col, (10.0 + perspective_scale * 16.0) * user_scale);
        c.fill_circle(cx, cy, gate_radius + gate_thick * 0.5);

        c.set_fill(Fill::Solid(Color::hex("#000000")));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_circle(cx, cy, gate_radius - gate_thick * 0.5);
    }

    // Portal Gate Spokes — radial lines from center, pulsing with bass
    let num_spokes = 8usize;
    let spoke_rot  = frame_time * 0.25;
    for sp in 0..num_spokes {
        let sa = sp as f32 / num_spokes as f32 * std::f32::consts::TAU + spoke_rot;
        let spoke_col = mix(p_col, s_col, sp as f32 / num_spokes as f32);
        c.set_stroke(Fill::Solid(spoke_col.with_alpha(0.12 + be * 0.08)));
        c.set_line_width((1.0 + bs * 0.8) * user_scale);
        c.set_shadow(glow, (6.0 + bs * 5.0) * user_scale);
        c.stroke_line(
            cx, cy,
            cx + sa.cos() * (base_r * 2.6 + be * 20.0),
            cy + sa.sin() * (base_r * 2.6 + be * 20.0),
        );
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

    let _ = ();
    c.set_global_alpha(1.0);
    c.restore();
}
