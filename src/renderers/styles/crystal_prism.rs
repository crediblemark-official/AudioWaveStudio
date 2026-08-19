//! Crystal Prism style renderer (`CrystalPrism`) — 3D Optical Refractive Crystal Engine.
//!
//! Masterpiece 360° 3D Optical Refractive Crystal Prism:
//! - 24 Faceted 3D Quartz Crystal Shards & Prisms with HSL rainbow spectral dispersion.
//! - Multi-Faceted Crystal Rendering (Refractive Front Facet, Specular Bevel Ridge, Refraction Shadow).
//! - Luminous Specular Caustics & Diamond Tip Glints.
//! - Central Crystal Core Disc with 3 expanding optical dispersion shockwave ripples.
//! - 45+ Floating crystal dust particles & spectral light motes.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const CRYSTAL_SHARDS: usize = 24;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let user_scale = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

    let be = ctx.bass_energy.clamp(0.0, 3.0);
    let bs = ctx.beat_strength.clamp(0.0, 3.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);

    // Curated Optical Prism Palette
    let prism_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.75);
    let prism_magenta = mix(accent_col, Color::rgba(1.0, 0.15, 0.85, 1.0), 0.75);
    let prism_emerald = mix(p_col, Color::rgba(0.10, 1.0, 0.50, 1.0), 0.70);
    let spark_white = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC OPTICAL PRISM BACKDROP GLOW
    // -------------------------------------------------------------------------
    let max_r = base_r * 1.55;
    let bg_prism = Fill::radial_gradient(
        cx,
        cy,
        inner_r,
        cx,
        cy,
        max_r * 1.8,
        &[
            (0.0, mix(prism_cyan, prism_magenta, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, mix(prism_emerald, s_col, 0.5).with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_prism);

    // -------------------------------------------------------------------------
    // 2. EXPANDING OPTICAL DISPERSION SHOCKWAVE RIPPLES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(prism_cyan, prism_magenta, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * user_scale);
        c.set_shadow(prism_cyan, (12.0 + bs * 8.0) * user_scale);
        c.stroke_circle(cx, cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. 24 FACETED 3D QUARTZ CRYSTAL SHARDS & PRISMS
    // -------------------------------------------------------------------------
    let step = (freq.len() / CRYSTAL_SHARDS).max(1);
    let rot = frame_time * 0.06;

    for i in 0..CRYSTAL_SHARDS {
        let t = i as f32 / CRYSTAL_SHARDS as f32;
        let angle1 = t * TAU + rot;
        let angle2 = (t + 1.0 / CRYSTAL_SHARDS as f32) * TAU + rot;
        let angle_mid = (angle1 + angle2) * 0.5;

        let fv = super::radial_common::full_random_scattered_bin(freq, step, i, CRYSTAL_SHARDS, ctx.beat_count);

        let val = (fv * sensitivity * 1.1 + (angle_mid * 3.0).sin() * 0.08 + be * 0.25).clamp(0.10, 2.0);
        let r_mid = inner_r + (45.0 + val * 85.0) * user_scale;
        let r_base_pt = inner_r + 10.0 * user_scale;

        let p_in1 = (cx + angle1.cos() * r_base_pt, cy + angle1.sin() * r_base_pt);
        let p_in2 = (cx + angle2.cos() * r_base_pt, cy + angle2.sin() * r_base_pt);
        let p_tip = (cx + angle_mid.cos() * r_mid, cy + angle_mid.sin() * r_mid);
        let p_core = (cx + angle_mid.cos() * inner_r, cy + angle_mid.sin() * inner_r);

        // HSL Spectral Dispersion Color
        let facet_col = mix(mix(prism_cyan, prism_emerald, t), mix(prism_magenta, spark_white, 0.4), val);

        // Pass A: Left Facet (Refractive Front)
        c.set_fill(Fill::Solid(facet_col.with_alpha(0.70)));
        c.set_shadow(facet_col, (14.0 + val * 10.0) * user_scale);
        c.fill_polygon(&[p_core, p_in1, p_tip]);

        // Pass B: Right Facet (Specular Bevel Shading)
        c.set_fill(Fill::Solid(mix(facet_col, Color::BLACK, 0.30).with_alpha(0.55)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_polygon(&[p_core, p_tip, p_in2]);

        // Pass C: 3D Crystal Ridge Bevel Highlight Line
        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width((1.5 + val * 1.2) * user_scale);
        c.stroke_line(p_core.0, p_core.1, p_tip.0, p_tip.1);

        // Pass D: Outer Crystal Edge Outline
        c.set_stroke(Fill::Solid(mix(facet_col, spark_white, 0.60)));
        c.set_line_width(1.2 * user_scale);
        c.stroke_polyline(&[p_in1, p_tip, p_in2]);

        // Diamond Vertex Glint Mote
        c.set_fill(Fill::Solid(spark_white));
        c.set_shadow(glow_col, (12.0 + bs * 10.0) * user_scale);
        c.fill_circle(p_tip.0, p_tip.1, (2.8 + val * 1.8) * user_scale);
    }

    // -------------------------------------------------------------------------
    // 4. FLOATING CRYSTAL DUST & SPECTRAL MOTES
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = inner_r + m_t * (max_r * 0.95);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(prism_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(prism_cyan, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // -------------------------------------------------------------------------
    // 5. PUMPING CENTRAL DISC & CRYSTAL CORE RESERVOIR
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(prism_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(prism_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
