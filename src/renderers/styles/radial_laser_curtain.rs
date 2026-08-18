//! Radial Laser Curtain style renderer (`radialLaserCurtain`) — 360° Volumetric Laser Stage Engine.
//!
//! Masterpiece 360° Radial Laser Light Curtain Stage:
//! - Central 3D rotating laser diode scanner core with optic aperture lens.
//! - 64 High-density 360° volumetric laser light beams with white-hot core filaments & neon plasma sheaths.
//! - Concentric outer laser curtain boundary ring with laser impact reflection flares.
//! - Interlocking laser scanner cross-discharge rays bridging adjacent laser beams.
//! - Stage fog haze light scattering & floating laser dust motes.
//! - Full UI Theme colors and settings integration (Scale, Position X & Y, Sensitivity, Bar Count).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;
    let bar_count = ctx.config.reactivity.bar_count.clamp(8, 128);

    let step = ((freq.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
    let rot = frame_time * 0.12;

    // Curated Laser Stage Palette (theme-dominant, hardcoded hue only as character accent)
    let laser_cyan = mix(Color::rgba(0.0, 0.95, 1.0, 1.0), s.glow, 0.75);
    let laser_magenta = mix(Color::rgba(1.0, 0.15, 0.85, 1.0), s.s_col, 0.75);
    let laser_green = mix(Color::rgba(0.0, 1.0, 0.40, 1.0), s.p_col, 0.70);
    let spark_white = mix(Color::rgba(0.95, 1.0, 1.0, 0.98), s.glow, 0.10);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC 360° LASER HAZE BACKDROP GLOW
    // -------------------------------------------------------------------------
    let bg_laser = Fill::radial_gradient(
        s.cx,
        s.cy,
        0.0,
        s.cx,
        s.cy,
        s.base_r * 2.5,
        &[
            (0.0, mix(laser_cyan, laser_magenta, 0.5).with_alpha(0.30 + s.be * 0.20)),
            (0.40, laser_green.with_alpha(0.14)),
            (0.75, s.p_col.with_alpha(0.05)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_laser);

    // -------------------------------------------------------------------------
    // 2. EXPANDING LASER PULSE SHOCKWAVES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = s.inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + s.bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(laser_cyan, laser_magenta, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * s.size_scale);
        c.set_shadow(laser_cyan, (12.0 + s.bs * 8.0) * s.size_scale);
        c.stroke_circle(s.cx, s.cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. 360° VOLUMETRIC LASER LIGHT BEAMS
    // -------------------------------------------------------------------------
    let mut tip_points: Vec<(f32, f32)> = Vec::with_capacity(bar_count);

    for i in 0..bar_count {
        let t = i as f32 / bar_count as f32;
        let angle = t * TAU + rot;
        let (cos_a, sin_a) = angle.sin_cos();

        let base_wave = (angle * 4.0 + frame_time * 2.0).sin() * 0.12 + 0.18;
        let audio_v = radial_common::swept_bin(freq, step, i, bar_count, &s) * s.sensitivity;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.35 + s.bs * 0.20
            + radial_common::beat_bump(&s, angle) * 0.5)
            .clamp(0.15, 2.2);

        let max_r = s.inner_r + (35.0 + val * 115.0) * s.size_scale;
        let x0 = s.cx + cos_a * s.inner_r;
        let y0 = s.cy + sin_a * s.inner_r;
        let x1 = s.cx + cos_a * max_r;
        let y1 = s.cy + sin_a * max_r;

        tip_points.push((x1, y1));

        let ray_col = mix(mix(laser_cyan, laser_green, t), mix(laser_magenta, spark_white, 0.4), fv_from_bin(freq, step, i, bar_count));

        // Pass A: Outer volumetric neon plasma laser sheath
        c.set_stroke(Fill::Solid(ray_col.with_alpha(0.60)));
        c.set_line_width((4.2 + val * 2.2) * s.size_scale);
        c.set_shadow(ray_col, (16.0 + val * 14.0) * s.size_scale);
        c.stroke_line(x0, y0, x1, y1);

        // Pass B: Intense white-hot core laser filament
        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width((1.4 + val * 0.8) * s.size_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_line(x0, y0, x1, y1);

        // Pass C: Inner Laser Diode Optic Emitter Lens
        c.set_fill(Fill::Solid(spark_white));
        c.set_shadow(ray_col, (10.0 + val * 8.0) * s.size_scale);
        c.fill_circle(x0, y0, (2.6 + val * 1.5) * s.size_scale);

        // Pass D: Outer Laser Curtain Boundary Impact Flare
        c.set_fill(Fill::Solid(mix(ray_col, spark_white, 0.80)));
        c.set_shadow(ray_col, (14.0 + s.bs * 10.0) * s.size_scale);
        c.fill_circle(x1, y1, (3.5 + val * 2.2) * s.size_scale);

        // Interlocking laser scanner cross-beams bridging adjacent laser rays
        if i % 2 == 0 && i + 1 < bar_count {
            let next_i = i + 1;
            let t_next = next_i as f32 / bar_count as f32;
            let angle_next = t_next * TAU + rot;

            let r_mid = s.inner_r + (max_r - s.inner_r) * 0.65;
            let xa = s.cx + angle.cos() * r_mid;
            let ya = s.cy + angle.sin() * r_mid;
            let xb = s.cx + angle_next.cos() * r_mid;
            let yb = s.cy + angle_next.sin() * r_mid;

            c.set_stroke(Fill::Solid(spark_white.with_alpha(0.70)));
            c.set_line_width(1.2 * s.size_scale);
            c.set_shadow(laser_cyan, 8.0 * s.size_scale);
            c.stroke_line(xa, ya, xb, yb);
        }
    }

    // -------------------------------------------------------------------------
    // 4. OUTER 360° LASER CURTAIN BOUNDARY RING
    // -------------------------------------------------------------------------
    if !tip_points.is_empty() {
        let mut closed_tips = tip_points.clone();
        if let Some(&first) = tip_points.first() {
            closed_tips.push(first);
        }
        c.set_stroke(Fill::Solid(mix(laser_cyan, spark_white, s.bs * 0.5).with_alpha(0.75)));
        c.set_line_width(2.0 * s.user_scale);
        c.set_shadow(laser_magenta, (12.0 + s.bs * 10.0) * s.user_scale);
        c.stroke_polyline(&closed_tips);
    }

    // -------------------------------------------------------------------------
    // 5. FLOATING STAGE DUST & LASER SPARK PARTICLES
    // -------------------------------------------------------------------------
    let mote_count = (20.0 + s.be * 24.0 * s.sensitivity).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = s.inner_r + (m_i as f32 * 17.0).cos().abs() * (s.base_r * 1.3) + m_t * 30.0;

        let mx = s.cx + m_angle.cos() * m_dist;
        let my = s.cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + s.bs * 1.8).clamp(1.0, 5.0) * s.user_scale;
        let m_col = mix(laser_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.1, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(laser_cyan, 8.0 * s.user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    radial_common::finish(c, ctx, &s);
}

fn fv_from_bin(freq: &[u8], step: usize, slot: usize, _slots: usize) -> f32 {
    let idx = (slot * step).min(freq.len().saturating_sub(1));
    freq[idx] as f32 / 255.0
}
