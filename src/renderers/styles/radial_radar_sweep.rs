//! Radial Radar Sweep style renderer (`radialRadarSweep`).
//!
//! A rotating radar sweep with range rings and audio-driven blips that light
//! up as the beam passes over loud parts of the spectrum.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.05, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let rot = frame_time * 0.9;
    let num_blips = 24usize;
    let step = ((freq.len() as f32) / num_blips as f32).floor().max(1.0) as usize;
    let max_r = s.base_r * 1.6;

    // Range rings.
    for ri in 0..3 {
        let rf = ri as f32;
        let rr = s.inner_r + (max_r - s.inner_r) * (rf + 1.0) / 3.0;
        c.set_stroke(Fill::Solid(s.p_col.with_alpha(0.28 - rf * 0.06)));
        c.set_line_width(1.0 * s.user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_circle(s.cx, s.cy, rr);
    }

    // Sweeping beam wedge.
    let beam_a = rot;
    let half = 0.35f32;
    let mut wedge: Vec<(f32, f32)> = Vec::with_capacity(40);
    wedge.push((s.cx, s.cy));
    let segs = 24usize;
    for k in 0..=segs {
        let a = beam_a - half + (k as f32 / segs as f32) * half * 2.0;
        wedge.push((s.cx + a.cos() * max_r, s.cy + a.sin() * max_r));
    }
    c.set_fill(Fill::Solid(s.accent.with_alpha(0.10)));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_polygon(&wedge);

    // Audio blips at fixed angles, lit by the spectrum.
    for i in 0..num_blips {
        let angle = (i as f32 / num_blips as f32) * TAU;
        let audio_v = radial_common::swept_bin(freq, step, i, num_blips, &s) * s.sensitivity;
        let bump = radial_common::beat_bump(&s, angle) * 0.5;
        let val = (audio_v + s.be * 0.3 + bump).clamp(0.0, 1.6);

        let rr = s.inner_r + val * (max_r - s.inner_r) * 0.85;
        let px = s.cx + angle.cos() * rr;
        let py = s.cy + angle.sin() * rr;
        c.set_fill(Fill::Solid(mix(s.accent, Color::WHITE, val * 0.5)));
        c.set_shadow(s.glow, (6.0 + val * 8.0) * s.user_scale);
        c.fill_circle(px, py, (1.5 + val * 2.0) * s.user_scale);
    }

    // Rotating beam line.
    c.set_stroke(Fill::Solid(s.glow.with_alpha(0.6)));
    c.set_line_width(1.8 * s.user_scale);
    c.set_shadow(s.glow, (10.0 + s.bs * 6.0) * s.user_scale);
    c.stroke_line(s.cx, s.cy, s.cx + beam_a.cos() * max_r, s.cy + beam_a.sin() * max_r);

    radial_common::finish(c, ctx, &s);
}
