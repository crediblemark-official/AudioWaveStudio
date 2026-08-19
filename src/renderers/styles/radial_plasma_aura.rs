//! Radial Plasma Aura style renderer (`radialPlasmaAura`) — Ionized Plasma Energy Engine.
//!
//! Masterpiece Plasma Energy Aura Corona:
//! - 64 high-density fractal plasma lightning tendrils arcing outward from an electric core.
//! - Cross-discharge lightning arcs jumping between adjacent plasma tendrils (Tesla-coil plasma field).
//! - 180-segment smooth plasma wave aura corona enclosing the electric discharges.
//! - Luminous white-hot plasma orb core with 3 expanding electric energy pulse shockwaves.
//! - Floating ionized plasma spark motes drifting in atmospheric electric haze.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const TENDRILS: usize = 64;
const ARC_SEGS: usize = 14;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let step = ((freq.len() as f32) / TENDRILS as f32).floor().max(1.0) as usize;
    let rot = frame_time * 0.15;

    // Curated Plasma Energy Palette (theme-dominant, hardcoded hue only as character accent)
    let plasma_cyan = mix(Color::rgba(0.0, 0.95, 1.0, 1.0), s.glow, 0.75);
    let plasma_magenta = mix(Color::rgba(1.0, 0.15, 0.85, 1.0), s.s_col, 0.75);
    let electric_violet = mix(Color::rgba(0.50, 0.0, 1.0, 1.0), s.p_col, 0.70);
    let spark_white = mix(Color::rgba(0.95, 1.0, 1.0, 0.98), s.glow, 0.10);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC ELECTRIC PLASMA HAZE BACKDROP
    // -------------------------------------------------------------------------
    let bg_plasma = Fill::radial_gradient(
        s.cx,
        s.cy,
        0.0,
        s.cx,
        s.cy,
        s.base_r * 2.5,
        &[
            (0.0, mix(plasma_cyan, plasma_magenta, 0.5).with_alpha(0.30 + s.be * 0.20)),
            (0.40, electric_violet.with_alpha(0.14)),
            (0.75, s.p_col.with_alpha(0.05)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_plasma);

    // -------------------------------------------------------------------------
    // 2. EXPANDING ELECTRIC PULSE SHOCKWAVES (PLASMA ORB CORONA)
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = s.inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + s.bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(plasma_cyan, plasma_magenta, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * s.user_scale);
        c.set_shadow(plasma_cyan, (12.0 + s.bs * 8.0) * s.user_scale);
        c.stroke_circle(s.cx, s.cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. 64 FRACTAL PLASMA LIGHTNING TENDRILS & DISCHARGE ARCS
    // -------------------------------------------------------------------------
    let mut tip_points: Vec<(f32, f32)> = Vec::with_capacity(TENDRILS);

    for i in 0..TENDRILS {
        let t = i as f32 / TENDRILS as f32;
        let angle = t * TAU + rot;

        let base_wave = (angle * 5.0 + frame_time * 2.2).sin() * 0.12 + 0.18;
        let audio_v = radial_common::swept_bin(freq, step, i, TENDRILS, &s) * s.sensitivity;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.35 + s.bs * 0.20
            + radial_common::beat_bump(&s, angle) * 0.5)
            .clamp(0.15, 2.2);

        let max_r = s.inner_r + (35.0 + val * 115.0) * s.user_scale;

        // Build jittered plasma arc path from inner_r to max_r
        let mut arc_pts: Vec<(f32, f32)> = Vec::with_capacity(ARC_SEGS + 1);

        for seg in 0..=ARC_SEGS {
            let seg_t = seg as f32 / ARC_SEGS as f32;
            let r_curr = s.inner_r + seg_t * (max_r - s.inner_r);

            // Multi-frequency fractal lightning jitter
            let jitter_phase = frame_time * 12.0 + i as f32 * 1.9 + seg_t * 7.0;
            let jitter_fast = (seg_t * 18.0 + jitter_phase).sin();
            let jitter_slow = (seg_t * 6.0 - jitter_phase * 0.5).cos() * 0.5;

            let jitter = (seg_t * (1.0 - seg_t * 0.2))
                * (jitter_fast + jitter_slow)
                * (14.0 * s.user_scale * val.min(1.4));

            let (cos_a, sin_a) = angle.sin_cos();
            let (perp_cos, perp_sin) = (angle + std::f32::consts::FRAC_PI_2).sin_cos();

            let px = s.cx + cos_a * r_curr + perp_cos * jitter;
            let py = s.cy + sin_a * r_curr + perp_sin * jitter;

            arc_pts.push((px, py));
        }

        if let Some(&tip) = arc_pts.last() {
            tip_points.push(tip);
        }

        let plasma_col = mix(plasma_cyan, mix(plasma_magenta, spark_white, 0.4), t);

        // Pass A: Outer glowing plasma halo
        c.set_stroke(Fill::Solid(plasma_col.with_alpha(0.55)));
        c.set_line_width((4.5 + val * 2.2) * s.user_scale);
        c.set_shadow(plasma_col, (16.0 + val * 14.0) * s.user_scale);
        c.stroke_polyline(&arc_pts);

        // Pass B: Intense core lightning filament
        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width((1.5 + val * 0.8) * s.user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&arc_pts);

        // Cross-discharge electric arc jumping between adjacent tendrils
        if i % 3 == 0 && i + 1 < TENDRILS {
            let next_i = i + 1;
            let t_next = next_i as f32 / TENDRILS as f32;
            let angle_next = t_next * TAU + rot;
            let r_mid = s.inner_r + (max_r - s.inner_r) * 0.60;

            let x_a = s.cx + angle.cos() * r_mid;
            let y_a = s.cy + angle.sin() * r_mid;
            let x_b = s.cx + angle_next.cos() * r_mid;
            let y_b = s.cy + angle_next.sin() * r_mid;

            let mid_x = (x_a + x_b) * 0.5 + (frame_time * 20.0 + i as f32).sin() * 8.0 * s.user_scale;
            let mid_y = (y_a + y_b) * 0.5 + (frame_time * 20.0 + i as f32).cos() * 8.0 * s.user_scale;

            c.set_stroke(Fill::Solid(spark_white.with_alpha(0.85)));
            c.set_line_width(1.2 * s.user_scale);
            c.set_shadow(plasma_cyan, 10.0 * s.user_scale);
            c.stroke_polyline(&[(x_a, y_a), (mid_x, mid_y), (x_b, y_b)]);
        }
    }

    // -------------------------------------------------------------------------
    // 4. OUTER PLASMA AURA BOUNDARY RING (CONNECTING TENDRIL TIPS)
    // -------------------------------------------------------------------------
    if !tip_points.is_empty() {
        let mut closed_tips = tip_points.clone();
        if let Some(&first) = tip_points.first() {
            closed_tips.push(first);
        }
        c.set_stroke(Fill::Solid(mix(plasma_cyan, spark_white, s.bs * 0.5).with_alpha(0.75)));
        c.set_line_width(2.0 * s.user_scale);
        c.set_shadow(plasma_magenta, (12.0 + s.bs * 10.0) * s.user_scale);
        c.stroke_polyline(&closed_tips);
    }

    // -------------------------------------------------------------------------
    // 5. FLOATING IONIZED PLASMA SPARK MOTES
    // -------------------------------------------------------------------------
    let mote_count = (20.0 + s.be * 24.0).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = s.inner_r + (m_i as f32 * 17.0).cos().abs() * (s.base_r * 1.3) + m_t * 30.0;

        let mx = s.cx + m_angle.cos() * m_dist;
        let my = s.cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + s.bs * 1.8).clamp(1.0, 5.0) * s.user_scale;
        let m_col = mix(plasma_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.1, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(plasma_cyan, 8.0 * s.user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    radial_common::finish(c, ctx, &s);
}
