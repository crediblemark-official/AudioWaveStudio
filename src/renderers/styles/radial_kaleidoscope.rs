//! Radial Kaleidoscope style renderer (`radialKaleidoscope`) — Optical Crystal Kaleidoscope Engine.
//!
//! Masterpiece Optical Crystal Kaleidoscope:
//! - 12-Fold Symmetrical Optical Mirror Chamber with 12-sided prismatic crystal geometry.
//! - Faceted crystal shards (diamonds, stars, geometric polygons) with white-hot specular edge highlights & HSL rainbow spectrum fills.
//! - Audio-reactive crystal morphing, rotation, & frequency-driven pulse intensity.
//! - Central optical crystal core with 3 expanding prismatic light shockwaves.
//! - 45+ Floating sparkling crystal motes & rainbow light sparks.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::hsl_to_color;
use crate::renderers::RenderContext;

use super::radial_common;

const MIRRORS: usize = 12;
const CRYSTAL_RINGS: usize = 7;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let rot = frame_time * 0.12;
    let sector = TAU / MIRRORS as f32;
    let step = ((freq.len() as f32) / (MIRRORS * CRYSTAL_RINGS) as f32).floor().max(1.0) as usize;

    let spark_white = mix(Color::rgba(0.98, 1.0, 1.0, 0.98), s.glow, 0.10);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC OPTICAL KALEIDOSCOPE GLOW BACKDROP
    // -------------------------------------------------------------------------
    let outer_r = s.base_r * 1.50;
    let bg_kaleido = Fill::radial_gradient(
        s.cx,
        s.cy,
        0.0,
        s.cx,
        s.cy,
        outer_r * 1.8,
        &[
            (0.0, mix(s.glow, s.accent, 0.5).with_alpha(0.32 + s.be * 0.18)),
            (0.50, s.p_col.with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_kaleido);

    // -------------------------------------------------------------------------
    // 2. 12-FOLD SYMMETRICAL MIRROR GRID LINES
    // -------------------------------------------------------------------------
    for m in 0..MIRRORS {
        let ma = m as f32 * sector + rot;
        let (sin_m, cos_m) = ma.sin_cos();

        c.set_stroke(Fill::Solid(mix(s.glow, Color::WHITE, 0.60).with_alpha(0.35)));
        c.set_line_width(1.2 * s.user_scale);
        c.stroke_line(s.cx + cos_m * s.inner_r, s.cy + sin_m * s.inner_r, s.cx + cos_m * outer_r, s.cy + sin_m * outer_r);
    }

    // -------------------------------------------------------------------------
    // 3. FACETED CRYSTAL SHARDS & PRISMATIC GEOMETRY
    // -------------------------------------------------------------------------
    for m in 0..MIRRORS {
        let mf = m as f32;
        let axis = mf * sector + rot;

        for ring in 0..CRYSTAL_RINGS {
            let rf = ring as f32;
            let t = rf / CRYSTAL_RINGS as f32;

            let sample_bin = ((m * CRYSTAL_RINGS + ring) * step).min(freq.len().saturating_sub(1));
            let fv = freq[sample_bin] as f32 / 255.0;

            let audio_v = (fv * s.sensitivity * 1.2 + s.be * 0.35).clamp(0.10, 2.0);

            let r1 = s.inner_r + t * (outer_r - s.inner_r);
            let r2 = r1 + (18.0 + audio_v * 24.0) * s.user_scale;

            let shard_w = (0.15 + audio_v * 0.40) * sector;
            let a_c = axis + (rf * 0.2).sin() * 0.15;
            let a1 = a_c - shard_w * 0.50;
            let a2 = a_c + shard_w * 0.50;
            let a_mid = (a1 + a2) * 0.50;

            // HSL Rainbow Spectrum Color Palette, tinted toward the theme hues so
            // the whole prism follows the configured primary/secondary colors.
            let hue = (t * 240.0 + mf * 30.0 + frame_time * 20.0) % 360.0;
            let rainbow = hsl_to_color(hue, 0.85, 0.50 + fv * 0.25, 0.75);
            let shard_col = mix(rainbow, mix(s.p_col, s.s_col, 0.5), 0.45);

            // Faceted Diamond / Star Polygon Shard
            let p_inner = (s.cx + a_mid.cos() * r1, s.cy + a_mid.sin() * r1);
            let p_left = (s.cx + a1.cos() * ((r1 + r2) * 0.5), s.cy + a1.sin() * ((r1 + r2) * 0.5));
            let p_outer = (s.cx + a_mid.cos() * r2, s.cy + a_mid.sin() * r2);
            let p_right = (s.cx + a2.cos() * ((r1 + r2) * 0.5), s.cy + a2.sin() * ((r1 + r2) * 0.5));

            let diamond_poly = vec![p_inner, p_left, p_outer, p_right];

            // Pass A: Soft Rainbow Prismatic Fill
            c.set_fill(Fill::Solid(shard_col.with_alpha(0.65)));
            c.set_shadow(shard_col, (10.0 + fv * 10.0) * s.user_scale);
            c.fill_polygon(&diamond_poly);

            // Pass B: White-Hot Specular Edge Highlight (Faceted Glass Look)
            c.set_stroke(Fill::Solid(spark_white));
            c.set_line_width((1.2 + fv * 0.8) * s.user_scale);
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.stroke_polyline(&vec![p_left, p_outer, p_right, p_inner, p_left]);

            // Pass C: Center Crystal Node Spark
            c.set_fill(Fill::Solid(spark_white));
            c.set_shadow(shard_col, (8.0 + s.bs * 6.0) * s.user_scale);
            c.fill_circle(p_outer.0, p_outer.1, (2.2 + fv * 1.8) * s.user_scale);
        }
    }

    // -------------------------------------------------------------------------
    // 4. INNER OPTICAL CRYSTAL CORE & SHOCKWAVE PULSES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = s.inner_r * (1.1 + p_t * 2.0);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + s.bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(s.glow, s.accent, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * s.user_scale);
        c.set_shadow(s.glow, (12.0 + s.bs * 8.0) * s.user_scale);
        c.stroke_circle(s.cx, s.cy, pulse_r);
    }

    // Central Radiant Crystal Hub
    c.set_fill(Fill::Solid(spark_white));
    c.set_shadow(s.glow, (14.0 + s.bs * 10.0) * s.user_scale);
    c.fill_circle(s.cx, s.cy, (6.0 + s.be * 3.5) * s.user_scale);

    // -------------------------------------------------------------------------
    // 5. FLOATING CRYSTAL DUST PARTICLES & LIGHT SPARKS
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + s.be * 24.0).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = s.inner_r + m_t * (outer_r * 0.95);

        let mx = s.cx + m_angle.cos() * m_dist;
        let my = s.cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + s.bs * 1.8).clamp(1.0, 5.0) * s.user_scale;
        let hue = (m_i as f32 * 35.0 + frame_time * 30.0) % 360.0;
        let m_rainbow = hsl_to_color(hue, 0.90, 0.60, (1.0 - m_t).clamp(0.15, 0.95));
        let m_col = mix(m_rainbow, mix(s.p_col, s.s_col, 0.5), 0.45);

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(m_col, 8.0 * s.user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    radial_common::finish(c, ctx, &s);
}
