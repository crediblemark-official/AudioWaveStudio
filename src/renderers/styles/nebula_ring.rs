//! Nebula Ring style renderer (`NebulaRing`).
//!
//! Visual Concept:
//! - 3 Counter-Rotating Silky Luminous Wave Ribbons forming an ethereal cosmic Moire pattern.
//! - Smooth harmonic wave expansion driven by audio frequency spectrum.
//! - Orbiting cosmic dust particles, starlight motes, & glowing volumetric gas backdrop.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

const DUST_PARTICLES: usize = 40;

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
    let inner_r = base_r * (0.35 + be * 0.10);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Ambient Volumetric Cosmic Nebula Backdrop
    let bg_nebula = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, base_r * 2.8,
        &[
            (0.00, mix(glow, accent, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.40, mix(p_col, s_col, 0.5).with_alpha(0.16 + bs * 0.08)),
            (0.80, Color::rgba(0.05, 0.02, 0.12, 0.20)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_nebula);
    c.fill_rect(cx - base_r * 2.8, cy - base_r * 2.8, base_r * 5.6, base_r * 5.6);

    let num_samples = 96usize;
    let step = (freq.len() / num_samples).max(1);

    // -------------------------------------------------------------------------
    // 2. 3 COUNTER-ROTATING SILKY COSMIC WAVE RIBBONS
    // -------------------------------------------------------------------------
    let colors = [p_col, s_col, accent];

    for layer in 0..3 {
        let lf = layer as f32;
        let dir = if layer % 2 == 0 { 1.0 } else { -1.0 };
        let rot = frame_time * (0.07 + lf * 0.03) * dir;
        let wave_n = 4.0 + lf * 2.0;

        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(num_samples + 1);

        for i in 0..num_samples {
            let t = i as f32 / num_samples as f32;
            let theta = t * TAU;
            let angle = theta + rot;

            let base_wave = (theta * wave_n + frame_time * 1.8).sin() * 12.0 * user_scale;
            let fv = smooth_ring_bin(freq, step, i, num_samples);
            let val = (fv * sensitivity * 1.1 + be * 0.30 + bs * 0.20).clamp(0.05, 2.2);

            let r_curr = inner_r + (20.0 + val * (50.0 + lf * 12.0)) * user_scale + base_wave;
            let (cos_a, sin_a) = angle.sin_cos();

            pts.push((cx + cos_a * r_curr, cy + sin_a * r_curr));
        }
        pts.push(pts[0]); // close smooth loop

        let l_col = colors[layer];

        // Smooth ribbon polygon fill
        c.set_fill(Fill::Solid(mix(l_col, glow, 0.40).with_alpha(0.16 + be * 0.08)));
        c.set_shadow(l_col, (12.0 + bs * 8.0) * user_scale);
        fill_radial_polygon(c, cx, cy, &pts);

        // Bright Glowing Ribbon Outline
        c.set_stroke(Fill::Solid(mix(l_col, Color::WHITE, 0.65).with_alpha(0.88)));
        c.set_line_width((2.2 + bs * 1.4) * user_scale);
        c.set_shadow(l_col, (16.0 + bs * 10.0) * user_scale);
        c.stroke_polyline(&pts);
    }

    // -------------------------------------------------------------------------
    // 3. ORBITING COSMIC DUST & STARLIGHT MOTES
    // -------------------------------------------------------------------------
    for d in 0..DUST_PARTICLES {
        let df = d as f32;
        let orbit_r = inner_r + (10.0 + (df * 7.1).sin().abs() * 90.0 + be * 20.0) * user_scale;
        let orbit_a = (df * 137.5).to_radians() + frame_time * (0.15 + (df * 3.3).cos() * 0.10);

        let px = cx + orbit_a.cos() * orbit_r;
        let py = cy + orbit_a.sin() * orbit_r;
        let p_sz = (1.5 + (df * 5.1).sin().abs() * 3.0 + be * 1.2) * user_scale;
        let p_col = mix(glow, accent, (df * 0.2).sin().abs());

        c.set_fill(Fill::Solid(mix(p_col, Color::WHITE, 0.50).with_alpha(0.75)));
        c.set_shadow(p_col, (8.0 + be * 6.0) * user_scale);
        c.fill_circle(px, py, p_sz);
    }

    // -------------------------------------------------------------------------
    // 4. PUMPING GLOWING NEBULA CORE RING & DISC
    // -------------------------------------------------------------------------
    let core_glow_fill = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, inner_r * 1.2,
        &[
            (0.00, Color::WHITE.with_alpha(0.90)),
            (0.40, glow.with_alpha(0.75)),
            (0.85, accent.with_alpha(0.40)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(core_glow_fill);
    c.set_shadow(glow, (22.0 + bs * 14.0) * user_scale);
    c.fill_circle(cx, cy, inner_r * 0.85);

    c.set_stroke(Fill::Solid(mix(glow, Color::WHITE, 0.70)));
    c.set_line_width((3.0 + be * 2.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.75);

    c.set_global_alpha(1.0);
    c.restore();
}

