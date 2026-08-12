//! Radial Neon Orbiter style renderer (`radialNeonOrbiter`).
//!
//! Tilted neon orbital rings with orbiting moon particles and a central core.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.05, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let num_rings = 4usize;
    let tilt_angle = -0.35f32;
    let step = ((freq.len() as f32) / num_rings as f32).floor().max(1.0) as usize;

    c.save();
    c.translate(s.cx, s.cy);
    c.rotate(tilt_angle);

    for r in 0..num_rings {
        let rf = r as f32;
        let base_wave = (frame_time * 1.8 + rf).sin() * 0.12 + 0.18;
        let audio_v = radial_common::swept_bin(freq, step, r, num_rings, &s) * s.sensitivity;
        let ring_angle = (rf / num_rings as f32) * TAU;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.15
            + radial_common::beat_bump(&s, ring_angle) * 0.5)
            .clamp(0.15, 1.5);

        let rx0 = s.base_r * (1.1 + rf * 0.25 + val * 0.25);
        let ry0 = rx0 * 0.38;

        let ring_col = mix(s.p_col, s.glow, rf / num_rings as f32);
        c.set_stroke(Fill::Solid(ring_col.with_alpha(0.80)));
        c.set_line_width((2.5 + val * 2.0) * s.user_scale);
        c.set_shadow(ring_col, (12.0 + val * 10.0) * s.user_scale);

        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(65);
        for k in 0..=64 {
            let a = k as f32 / 64.0 * TAU;
            // Per-angle audio + beat bump deform the orbit locally so the ring
            // ripples where the sound hits instead of breathing uniformly.
            let world_a = a + tilt_angle;
            let pt_audio = radial_common::swept_bin(freq, step, k / 2, 33, &s) * s.sensitivity;
            let bump = radial_common::beat_bump(&s, world_a) * 0.45;
            let deform = 1.0 + (pt_audio * 0.30 + bump).clamp(0.0, 0.55) * (1.0 - rf * 0.10);
            pts.push((a.cos() * rx0 * deform, a.sin() * ry0 * deform));
        }
        c.stroke_polyline(&pts);

        // Moon orbit speeds up and brightens with the beat
        let moon_speed = (0.8 + rf * 0.3) * (if r % 2 == 0 { 1.0 } else { -1.0 });
        let moon_a = frame_time * moon_speed * (1.0 + s.bs * 0.4);
        let mx = moon_a.cos() * rx0 * 0.92;
        let my = moon_a.sin() * ry0 * 0.92;

        let moon_col = mix(s.accent, Color::WHITE, 0.6);
        c.set_fill(Fill::Solid(moon_col));
        c.set_shadow(moon_col, (10.0 + s.bs * 8.0) * s.user_scale);
        c.fill_circle(mx, my, (4.5 + val * 2.5 + s.bs * 1.5) * s.user_scale);
    }
    c.restore();

    radial_common::finish(c, ctx, &s);
}
