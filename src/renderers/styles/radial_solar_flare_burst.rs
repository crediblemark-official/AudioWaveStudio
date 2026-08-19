//! Radial Solar Flare Burst style renderer (`radialSolarFlareBurst`) — Solar Helios Crown Engine.
//!
//! Masterpiece Solar Helios Crown & Sun Corona:
//! - 48 Blazing solar prominence arches & plasma flame tongues arcing over the solar surface.
//! - Dual-pass solar flare rendering with white-hot core filaments & solar plasma sheaths.
//! - Luminous golden Sun core sphere with 3 expanding solar energy pulse shockwaves.
//! - 45+ Floating solar wind embers & prominence sparks flying outward into space.
//! - Full UI Theme colors and settings integration (Scale, Position X & Y, Sensitivity, Bar Count).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const NUM_FLAMES: usize = 48;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 110.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let rot = frame_time * 0.08;
    let step = ((freq.len() as f32) / NUM_FLAMES as f32).floor().max(1.0) as usize;

    // Curated Solar Helios Palette (theme-dominant, hardcoded hue only as character accent)
    let solar_gold = mix(Color::rgba(1.0, 0.82, 0.10, 1.0), s.accent, 0.75);
    let solar_orange = mix(Color::rgba(1.0, 0.40, 0.02, 1.0), s.accent, 0.75);
    let solar_crimson = mix(Color::rgba(0.90, 0.10, 0.02, 1.0), s.s_col, 0.70);
    let spark_white = mix(Color::rgba(1.0, 0.98, 0.90, 0.98), s.glow, 0.15);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC BLAZING SOLAR CORONA GLOW BACKDROP
    // -------------------------------------------------------------------------
    let bg_solar = Fill::radial_gradient(
        s.cx,
        s.cy,
        0.0,
        s.cx,
        s.cy,
        s.base_r * 2.6,
        &[
            (0.0, mix(solar_gold, solar_orange, 0.5).with_alpha(0.32 + s.be * 0.20)),
            (0.40, solar_crimson.with_alpha(0.15)),
            (0.75, s.p_col.with_alpha(0.05)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_solar);

    // -------------------------------------------------------------------------
    // 2. EXPANDING SOLAR ENERGY PULSE SHOCKWAVES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = s.inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + s.bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(solar_gold, solar_crimson, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * s.user_scale);
        c.set_shadow(solar_orange, (14.0 + s.bs * 10.0) * s.user_scale);
        c.stroke_circle(s.cx, s.cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. 48 BLAZING SOLAR PROMINENCE FLAME TONGUES & PLASMA ARCHES
    // -------------------------------------------------------------------------
    for f in 0..NUM_FLAMES {
        let t = f as f32 / NUM_FLAMES as f32;
        let angle = t * TAU + rot;

        let base_wave = (angle * 4.0 + frame_time * 1.8).sin() * 0.12 + 0.18;
        let audio_v = radial_common::swept_bin(freq, step, f, NUM_FLAMES, &s) * s.sensitivity;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.35 + s.bs * 0.20
            + radial_common::beat_bump(&s, angle) * 0.5)
            .clamp(0.15, 2.2);

        let flame_h = (30.0 + val * 115.0) * s.user_scale;
        let tip_r = s.inner_r + flame_h;
        let half_w = (TAU / NUM_FLAMES as f32) * 0.70;

        let (cos_a, sin_a) = angle.sin_cos();
        let (cos_l, sin_l) = (angle - half_w).sin_cos();
        let (cos_r, sin_r) = (angle + half_w).sin_cos();
        let (cos_ml, sin_ml) = (angle - half_w * 0.55).sin_cos();
        let (cos_mr, sin_mr) = (angle + half_w * 0.55).sin_cos();

        let p_base_l = (s.cx + cos_l * s.inner_r, s.cy + sin_l * s.inner_r);
        let p_base_r = (s.cx + cos_r * s.inner_r, s.cy + sin_r * s.inner_r);
        let p_mid_l = (s.cx + cos_ml * (s.inner_r + flame_h * 0.52), s.cy + sin_ml * (s.inner_r + flame_h * 0.52));
        let p_mid_r = (s.cx + cos_mr * (s.inner_r + flame_h * 0.52), s.cy + sin_mr * (s.inner_r + flame_h * 0.52));
        let p_tip = (s.cx + cos_a * tip_r, s.cy + sin_a * tip_r);

        let flame_poly = vec![p_base_l, p_mid_l, p_tip, p_mid_r, p_base_r];
        let flare_col = mix(mix(solar_gold, solar_orange, t), mix(solar_crimson, spark_white, 0.4), val * 0.5);

        // Pass A: Outer soft solar corona flame fill
        c.set_fill(Fill::Solid(flare_col.with_alpha(0.55)));
        c.set_shadow(flare_col, (18.0 + val * 14.0) * s.user_scale);
        c.fill_polygon(&flame_poly);

        // Pass B: Inner white-hot core flame (narrower, intense heat)
        let core_poly = vec![
            p_base_l,
            (s.cx + cos_ml * (s.inner_r + flame_h * 0.40), s.cy + sin_ml * (s.inner_r + flame_h * 0.40)),
            p_tip,
            (s.cx + cos_mr * (s.inner_r + flame_h * 0.40), s.cy + sin_mr * (s.inner_r + flame_h * 0.40)),
            p_base_r,
        ];
        c.set_fill(Fill::Solid(mix(flare_col, spark_white, 0.70).with_alpha(0.95)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_polygon(&core_poly);

        // Corona peak spark at flame tip
        c.set_fill(Fill::Solid(mix(flare_col, spark_white, 0.85)));
        c.set_shadow(solar_gold, (14.0 + s.bs * 10.0) * s.user_scale);
        c.fill_circle(p_tip.0, p_tip.1, (3.2 + val * 2.2) * s.user_scale);
    }

    // -------------------------------------------------------------------------
    // 4. FLOATING SOLAR WIND EMBERS & PROMINENCE SPARKS
    // -------------------------------------------------------------------------
    let mote_count = (20.0 + s.be * 24.0).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = s.inner_r + (m_i as f32 * 17.0).cos().abs() * (s.base_r * 1.3) + m_t * 30.0;

        let mx = s.cx + m_angle.cos() * m_dist;
        let my = s.cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + s.bs * 1.8).clamp(1.0, 5.0) * s.user_scale;
        let m_col = mix(solar_gold, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.1, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(solar_orange, 8.0 * s.user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    radial_common::finish(c, ctx, &s);
}
