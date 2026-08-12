//! Radial Cyber Rune style renderer (`radialCyberRune`).
//!
//! Sci-fi cybernetic HUD compass rune with glowing tick motes, smooth arcs,
//! and glowing diamond corner targets.

use std::f32::consts::TAU;

use crate::gpu2d::{Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 110.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let num_ticks = 48usize;
    let compass_r = s.inner_r + 20.0 * s.user_scale;
    let tick_rot = frame_time * 0.15;
    let tick_step = ((freq.len() as f32) / num_ticks as f32).floor().max(1.0) as usize;
    let step = ((freq.len() as f32) / 4.0).floor().max(1.0) as usize;

    // -------------------------------------------------------------------------
    // 1. GLOWING HUD COMPASS TICKS (audio-reactive motes and tapered tick markers)
    // -------------------------------------------------------------------------
    for i in 0..num_ticks {
        let angle = (i as f32 / num_ticks as f32) * TAU + tick_rot;
        let is_major = i % 6 == 0;

        // Per-tick audio + beat bump drives tick length and glow
        let tick_v = radial_common::swept_bin(freq, tick_step, i, num_ticks, &s) * s.sensitivity;
        let bump = radial_common::beat_bump(&s, angle);
        let react = (tick_v * 0.9 + s.be * 0.25 + bump * 0.9).clamp(0.0, 1.3);
        let tick_len = (if is_major { 14.0 } else { 7.0 } + react * 12.0) * s.user_scale;

        let (cos_a, sin_a) = angle.sin_cos();
        let x0 = s.cx + cos_a * compass_r;
        let y0 = s.cy + sin_a * compass_r;
        let x1 = s.cx + cos_a * (compass_r + tick_len);
        let y1 = s.cy + sin_a * (compass_r + tick_len);

        let tick_col = if is_major { s.glow } else { s.p_col.with_alpha(0.65) };

        // Tapered tick marker line
        c.set_stroke(Fill::Solid(tick_col));
        c.set_line_width((if is_major { 2.2 } else { 1.2 }) * s.user_scale);
        c.set_shadow(tick_col, (6.0 + react * 8.0) * s.user_scale);
        c.stroke_line(x0, y0, x1, y1);

        // Soft glowing dot cap at tick tip
        let dot_r = (if is_major { 2.4 } else { 1.2 }) * (1.0 + react * 0.8) * s.user_scale;
        c.set_fill(Fill::Solid(tick_col));
        c.set_shadow(tick_col, (6.0 + s.bs * 6.0) * s.user_scale);
        c.fill_circle(x1, y1, dot_r);
    }

    // -------------------------------------------------------------------------
    // 2. CONCENTRIC AUDIO ARC GAUGES
    // -------------------------------------------------------------------------
    let gauge_r = compass_r + 28.0 * s.user_scale;

    for g in 0..4 {
        let gf = g as f32;
        let start_a = (gf / 4.0) * TAU + frame_time * -0.20;

        let base_sweep = (frame_time * 1.5 + gf).sin() * 0.15 + 0.35;
        let audio_v = radial_common::swept_bin(freq, step, g, 4, &s) * s.sensitivity;
        let val = (base_sweep + audio_v * 0.65 + s.be * 0.20
            + radial_common::beat_bump(&s, start_a) * 0.5)
            .clamp(0.20, 1.0);

        let sweep_a = (TAU / 4.0 * 0.85) * val;

        let gauge_col = mix(s.p_col, s.accent, gf / 4.0);
        c.set_stroke(Fill::Solid(gauge_col));
        c.set_line_width((3.5 + s.bs * 1.5) * s.user_scale);
        c.set_shadow(gauge_col, (14.0 + s.bs * 10.0) * s.user_scale);
        c.stroke_arc(s.cx, s.cy, gauge_r + gf * 12.0 * s.user_scale, start_a, start_a + sweep_a);
    }

    // -------------------------------------------------------------------------
    // 3. CORNER TARGET BRACKETS (glowing diamond nodes)
    // -------------------------------------------------------------------------
    let bracket_r = s.base_r * (1.35 + s.be * 0.15);
    for b in 0..4 {
        let b_angle = (b as f32 / 4.0) * TAU + std::f32::consts::FRAC_PI_4;
        let bx = s.cx + b_angle.cos() * bracket_r;
        let by = s.cy + b_angle.sin() * bracket_r;

        c.set_fill(Fill::Solid(s.glow));
        c.set_shadow(s.glow, 12.0 * s.user_scale);
        c.fill_circle(bx, by, 3.5 * s.user_scale);
    }

    radial_common::finish(c, ctx, &s);
}
