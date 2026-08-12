//! Saturn Halo style renderer (`SaturnHalo`).
//!
//! Visual Concept:
//! - 3D Tilted Saturn Planetary Ring with orbiting stardust particle belt.
//! - Concentric planetary rings with 360° even audio ripple.
//! - Zero radial bars/rays.

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
    let pos_offset_y = ctx.config.position_y * height * 0.5;
    let bar_count    = ctx.config.reactivity.bar_count.clamp(16, 96);

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
    let num_samples = 128usize;

    // -------------------------------------------------------------------------
    // 1. TILTED SATURN PLANETARY RING BELT (3D Perspective)
    // -------------------------------------------------------------------------
    let rot = frame_time * 0.08;
    let tilt_y = 0.42; // 3D perspective compression
    let ring_r_inner = base_r * (1.10 + be * 0.10);
    let ring_r_outer = base_r * (1.45 + be * 0.20);

    // Inner & Outer Saturn Belt Polylines
    let mut inner_pts: Vec<(f32, f32)> = Vec::with_capacity(num_samples + 1);
    let mut outer_pts: Vec<(f32, f32)> = Vec::with_capacity(num_samples + 1);

    for i in 0..=num_samples {
        let t = i as f32 / num_samples as f32;
        let angle = t * TAU + rot;

        let base_wave = (angle * 3.0 + frame_time * 1.5).sin() * 0.10 + 0.15;
        let audio_v = crate::renderers::styles::radial_common::full_random_scattered_bin(
            freq, step, i, num_samples, ctx.beat_count,
        ) * sensitivity;
        let val = (base_wave + audio_v * 0.85 + be * 0.25).clamp(0.10, 1.8);

        let r_in = ring_r_inner + (val * 15.0) * user_scale;
        let r_out = ring_r_outer + (val * 35.0) * user_scale;

        let (cos_a, sin_a) = angle.sin_cos();

        inner_pts.push((cx + cos_a * r_in, cy + sin_a * (r_in * tilt_y)));
        outer_pts.push((cx + cos_a * r_out, cy + sin_a * (r_out * tilt_y)));
    }

    // Saturn Ring Surface Fill
    let mut belt_poly = inner_pts.clone();
    let mut outer_rev = outer_pts.clone();
    outer_rev.reverse();
    belt_poly.extend(outer_rev);

    let belt_grad = Fill::radial_gradient(
        cx, cy, ring_r_inner,
        cx, cy, ring_r_outer * 1.2,
        &[
            (0.00, p_col.with_alpha(0.70)),
            (0.50, s_col.with_alpha(0.45)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(belt_grad);
    c.set_shadow(p_col, (14.0 + bs * 10.0) * user_scale);
    c.fill_polygon(&belt_poly);

    // Outer & Inner Saturn Ring Edges
    c.set_stroke(Fill::Solid(mix(glow, Color::WHITE, 0.70)));
    c.set_line_width((2.0 + bs * 1.2) * user_scale);
    c.set_shadow(glow, (16.0 + bs * 12.0) * user_scale);
    c.stroke_polyline(&outer_pts);
    c.stroke_polyline(&inner_pts);

    // -------------------------------------------------------------------------
    // 2. ORBITING STARDUST PARTICLES ALONG PLANETARY RING
    // -------------------------------------------------------------------------
    let num_motes = 20usize;
    for m in 0..num_motes {
        let mf = m as f32;
        let mote_a = rot * 1.5 + (mf / num_motes as f32) * TAU;
        let mote_r = ring_r_inner + (m % 3) as f32 * 12.0 * user_scale;

        let mx = cx + mote_a.cos() * mote_r;
        let my = cy + mote_a.sin() * (mote_r * tilt_y);

        let mote_col = mix(s_col, Color::WHITE, (m % 2) as f32);
        c.set_fill(Fill::Solid(mote_col));
        c.set_shadow(glow, (10.0 + bs * 8.0) * user_scale);
        c.fill_circle(mx, my, (2.5 + bs * 1.5) * user_scale);
    }

    // -------------------------------------------------------------------------
    // 3. PUMPING CENTRAL PLANET DISC & NEON RING
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

    let _ = accent;
    c.set_global_alpha(1.0);
    c.restore();
}
