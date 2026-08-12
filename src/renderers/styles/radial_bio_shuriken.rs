//! Radial Bio-Shuriken style renderer (`radialBioShuriken`).
//!
//! Sharp concave 4-point ninja shuriken:
//! - Angular concave blades with a fast spin and ghost motion-blur trails.
//! - Hollow centre ring, tip motes, multi-pass neon glow.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const SHURIKEN_BLADES: usize = 4;

fn blade_polygon(cx: f32, cy: f32, angle: f32, inner_r: f32, outer_r: f32) -> Vec<(f32, f32)> {
    let half_w = 0.78f32;
    let (cos_a, sin_a) = angle.sin_cos();
    let (cos_l, sin_l) = (angle - half_w).sin_cos();
    let (cos_r, sin_r) = (angle + half_w).sin_cos();

    let p_base_l = (cx + cos_l * inner_r, cy + sin_l * inner_r);
    let p_base_r = (cx + cos_r * inner_r, cy + sin_r * inner_r);
    let p_tip = (cx + cos_a * outer_r, cy + sin_a * outer_r);

    // Concave inner cut: pull the blade edges inward at mid radius
    let mid_r = inner_r + (outer_r - inner_r) * 0.45;
    let p_mid_l = (
        cx + (angle - half_w * 0.62).cos() * mid_r,
        cy + (angle - half_w * 0.62).sin() * mid_r,
    );
    let p_mid_r = (
        cx + (angle + half_w * 0.62).cos() * mid_r,
        cy + (angle + half_w * 0.62).sin() * mid_r,
    );

    vec![p_base_l, p_mid_l, p_tip, p_mid_r, p_base_r]
}

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 110.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let step = ((freq.len() as f32) / SHURIKEN_BLADES as f32).floor().max(1.0) as usize;

    // Fast spin — blades sweep through the frame for a slicing feel
    let rot = frame_time * 0.7;

    // -------------------------------------------------------------------------
    // GHOST MOTION-BLUR TRAILS (trailing copies of the blade assembly)
    // -------------------------------------------------------------------------
    for ghost in 1..=3 {
        let ga = rot - ghost as f32 * 0.12;
        let alpha = (0.10 / ghost as f32).min(0.12);
        for b in 0..SHURIKEN_BLADES {
            let angle = (b as f32 / SHURIKEN_BLADES as f32) * TAU + ga;
            let gv = 0.35f32;
            let outer_r = s.inner_r + (55.0 + gv * 90.0) * s.user_scale;
            let poly = blade_polygon(s.cx, s.cy, angle, s.inner_r, outer_r);

            let col = s.accent;
            c.set_fill(Fill::Solid(col.with_alpha(alpha)));
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.fill_polygon(&poly);
        }
    }

    // -------------------------------------------------------------------------
    // MAIN SHURIKEN BLADES (audio-reactive reach)
    // -------------------------------------------------------------------------
    for b in 0..SHURIKEN_BLADES {
        let angle = (b as f32 / SHURIKEN_BLADES as f32) * TAU + rot;

        let base_wave = (frame_time * 2.4 + b as f32).sin() * 0.15 + 0.20;
        let audio_v = radial_common::swept_bin(freq, step, b, SHURIKEN_BLADES, &s) * s.sensitivity;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.35 + s.bs * 0.20
            + radial_common::beat_bump(&s, angle) * 0.5)
            .clamp(0.15, 1.5);

        let outer_r = s.inner_r + (55.0 + val * 130.0) * s.user_scale;
        let poly = blade_polygon(s.cx, s.cy, angle, s.inner_r, outer_r);

        let blade_col = mix(s.p_col, s.glow, b as f32 / SHURIKEN_BLADES as f32);

        // Solid blade fill
        c.set_fill(Fill::Solid(blade_col.with_alpha(0.9)));
        c.set_shadow(blade_col, (14.0 + val * 12.0) * s.user_scale);
        c.fill_polygon(&poly);

        // Bright concave edge contour
        c.set_stroke(Fill::Solid(mix(blade_col, Color::WHITE, 0.55)));
        c.set_line_width((2.0 + val * 1.4) * s.user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&poly);

        // Tip mote
        let p_tip = (s.cx + angle.cos() * outer_r, s.cy + angle.sin() * outer_r);
        c.set_fill(Fill::Solid(mix(blade_col, Color::WHITE, 0.85)));
        c.set_shadow(s.glow, (14.0 + s.bs * 10.0) * s.user_scale);
        c.fill_circle(p_tip.0, p_tip.1, (2.6 + val * 2.2) * s.user_scale);
    }

    radial_common::finish(c, ctx, &s);
}
