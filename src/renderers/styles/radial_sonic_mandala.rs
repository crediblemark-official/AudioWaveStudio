//! Radial Sonic Mandala style renderer (`radialSonicMandala`).
//!
//! Layered rotating mandala:
//! - Inner ring of broad petals + outer ring of narrow petals spinning in
//!   opposite directions, with glowing interstitial dots — a woven sacred-geometry
//!   bloom clearly distinct from the sharp shuriken style.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const INNER_PETALS: usize = 8;
const OUTER_PETALS: usize = 24;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 110.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let rot_in = frame_time * 0.12;
    let rot_out = frame_time * -0.09;
    let step = ((freq.len() as f32) / OUTER_PETALS as f32).floor().max(1.0) as usize;

    // -------------------------------------------------------------------------
    // 1. OUTER NARROW PETAL RING (counter-clockwise, high count)
    // -------------------------------------------------------------------------
    for p in 0..OUTER_PETALS {
        let angle = (p as f32 / OUTER_PETALS as f32) * TAU + rot_out;

        let base_wave = (frame_time * 1.6 + p as f32).sin() * 0.14 + 0.20;
        let audio_v = radial_common::swept_bin(freq, step, p, OUTER_PETALS, &s) * s.sensitivity;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.25
            + radial_common::beat_bump(&s, angle) * 0.5)
            .clamp(0.15, 1.4);

        let petal_h = (14.0 + val * 55.0) * s.user_scale;
        let p_tip_r = s.inner_r + (46.0 + val * 30.0) * s.user_scale;
        let half_w = 0.16;

        let (cos_a, sin_a) = angle.sin_cos();
        let (cos_l, sin_l) = (angle - half_w).sin_cos();
        let (cos_r, sin_r) = (angle + half_w).sin_cos();

        let p0 = (s.cx + cos_a * (s.inner_r + 30.0 * s.user_scale), s.cy + sin_a * (s.inner_r + 30.0 * s.user_scale));
        let p1 = (s.cx + cos_l * (p_tip_r - petal_h * 0.5), s.cy + sin_l * (p_tip_r - petal_h * 0.5));
        let p2 = (s.cx + cos_a * p_tip_r, s.cy + sin_a * p_tip_r);
        let p3 = (s.cx + cos_r * (p_tip_r - petal_h * 0.5), s.cy + sin_r * (p_tip_r - petal_h * 0.5));

        let petal = vec![p0, p1, p2, p3];
        let m_col = mix(s.accent, s.glow, p as f32 / OUTER_PETALS as f32);

        c.set_fill(Fill::Solid(m_col.with_alpha(0.30 + val * 0.18)));
        c.set_shadow(m_col, (8.0 + val * 8.0) * s.user_scale);
        c.fill_polygon(&petal);

        // Interstitial dots between outer petals
        if p % 3 == 0 {
            c.set_fill(Fill::Solid(mix(s.p_col, Color::WHITE, 0.6).with_alpha(0.8)));
            c.set_shadow(s.glow, (6.0 + s.bs * 5.0) * s.user_scale);
            c.fill_circle((p0.0 + p2.0) * 0.5, (p0.1 + p2.1) * 0.5, 1.6 * s.user_scale);
        }
    }

    // -------------------------------------------------------------------------
    // 2. INNER BROAD PETAL RING (clockwise, low count, deep translucent fills)
    // -------------------------------------------------------------------------
    for p in 0..INNER_PETALS {
        let angle = (p as f32 / INNER_PETALS as f32) * TAU + rot_in;

        let base_wave = (frame_time * 1.8 + p as f32 * 0.7).sin() * 0.15 + 0.22;
        let audio_v = radial_common::swept_bin(freq, step, p, INNER_PETALS, &s) * s.sensitivity;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.30
            + radial_common::beat_bump(&s, angle) * 0.5)
            .clamp(0.15, 1.5);

        let petal_h = (22.0 + val * 80.0) * s.user_scale;
        let p_tip_r = s.inner_r + petal_h;
        let half_w = 0.30;

        let (cos_a, sin_a) = angle.sin_cos();
        let (cos_l, sin_l) = (angle - half_w).sin_cos();
        let (cos_r, sin_r) = (angle + half_w).sin_cos();

        let p0 = (s.cx + cos_a * s.inner_r, s.cy + sin_a * s.inner_r);
        let p1 = (s.cx + cos_l * (s.inner_r + petal_h * 0.6), s.cy + sin_l * (s.inner_r + petal_h * 0.6));
        let p2 = (s.cx + cos_a * p_tip_r, s.cy + sin_a * p_tip_r);
        let p3 = (s.cx + cos_r * (s.inner_r + petal_h * 0.6), s.cy + sin_r * (s.inner_r + petal_h * 0.6));

        let petal = vec![p0, p1, p2, p3];
        let m_col = mix(s.p_col, s.accent, p as f32 / INNER_PETALS as f32);

        c.set_fill(Fill::Solid(m_col.with_alpha(0.40 + val * 0.20)));
        c.set_shadow(m_col, (12.0 + val * 10.0) * s.user_scale);
        c.fill_polygon(&petal);

        c.set_stroke(Fill::Solid(mix(m_col, Color::WHITE, 0.5)));
        c.set_line_width((2.0 + val * 1.5) * s.user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&petal);

        c.set_fill(Fill::Solid(mix(m_col, Color::WHITE, 0.8)));
        c.set_shadow(s.glow, (14.0 + s.bs * 10.0) * s.user_scale);
        c.fill_circle(p2.0, p2.1, (2.2 + val * 2.0) * s.user_scale);
    }

    radial_common::finish(c, ctx, &s);
}
