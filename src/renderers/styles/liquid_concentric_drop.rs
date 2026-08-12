//! Liquid Concentric Drop style renderer (`liquidConcentricDrop`).
//!
//! Visual Concept:
//! - Water-drop ripples that continuously radiate outward from the center on a
//!   decaying audio envelope, each ring brightening on bass hits.
//! - ZERO AMPLITUDE STEALTH HIDING: When music is quiet (`audio_v == 0`), rings
//!   collapse to the central logo disc (`radius == inner_r`).

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

    let num_ripples = 6usize;
    let step = (freq.len() / num_ripples).max(1);
    let audio_v = bin_value(freq, step, 0) * sensitivity;
    let env = (audio_v * 0.85 + be * 0.25).clamp(0.0, 1.8);

    // Ripples march outward on a time-based phase, restarted by each beat so a
    // fresh wave launches on kick hits. When silent (env == 0) every ring rests
    // exactly at inner_r.
    let base_speed = 0.22 + bs * 0.18;
    for r in 0..num_ripples {
        let rf    = r as f32;
        let age   = (frame_time * base_speed + rf / num_ripples as f32) % 1.0;
        let env_boost = env * (1.0 - age * 0.55);
        let ripple_dist = inner_r + env_boost * (30.0 + rf * 34.0) * user_scale * (0.35 + age);

        let fade = (1.0 - age).max(0.0);
        if fade > 0.02 && env_boost > 0.01 {
            let ripple_col = mix(p_col, mix(accent, glow, age), rf / num_ripples as f32);

            // Elliptical deformation: each ring has a slightly different aspect ratio
            // so the concentric ripples look like real water drops on a surface.
            let tilt_seed = ((ctx.beat_count.wrapping_add(r as u64 * 3)) as f32) * 0.618_034;
            let ell_ratio = 0.72 + (tilt_seed.fract() * 0.26); // 0.72..0.98
            let n_pts = 56usize;
            let mut ring_pts: Vec<(f32, f32)> = Vec::with_capacity(n_pts + 1);
            for k in 0..=n_pts {
                let a = k as f32 / n_pts as f32 * TAU;
                // Small audio-driven wobble on each ring
                let wobble = (a * 3.0 + frame_time * 2.0 + rf).sin() * (env_boost * 4.0 * user_scale);
                let rx = ripple_dist + wobble;
                let ry = ripple_dist * ell_ratio + wobble * 0.5;
                ring_pts.push((cx + a.cos() * rx, cy + a.sin() * ry));
            }

            c.set_stroke(Fill::Solid(ripple_col.with_alpha(fade * (0.9 - rf * 0.08).max(0.2))));
            c.set_line_width((2.6 - rf * 0.3 + bs * 1.5).max(0.8) * user_scale);
            c.set_shadow(ripple_col, (14.0 + bs * 10.0) * user_scale * fade);
            c.stroke_polyline(&ring_pts);
        }
    }

    // Central droplet flash on beats.
    if bs > 0.4 {
        c.set_fill(Fill::Solid(mix(glow, Color::WHITE, 0.7).with_alpha(0.5)));
        c.set_shadow(glow, (22.0 + bs * 14.0) * user_scale);
        c.fill_circle(cx, cy, (5.0 + bs * 5.0) * user_scale);
    }

    // Center Logo Disc
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
