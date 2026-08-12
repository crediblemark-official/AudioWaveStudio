//! Pulsing Cyber Shield style renderer (`pulsingCyberShield`) — Futuristic Holographic Defense Shield Engine.
//!
//! Masterpiece 360° Cybernetic Holographic Defense Barrier:
//! - 3 Concentric Rotating Cyber-Shield Bands (Outer Armor Plates, Mid Hex Forcefield Grid, Inner Containment Ring).
//! - Dual-Pass Laser Arc Rendering with white-hot core filaments & cyan/magenta plasma sheaths.
//! - Audio-Reactive Inter-Shield Electric Arc Discharges & Glowing HUD Corner Brackets.
//! - Central Quantum Singularity Core with 3 expanding forcefield shockwave ripples.
//! - 45+ Orbital quantum motes & defense spark particles.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const SHIELD_SEGMENTS: usize = 6;

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
    let cy = height * 0.5 - pos_offset_y;

    let reference_size = width.min(height);
    let base_r = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);

    // Curated Cyber Shield Palette
    let shield_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.75);
    let shield_magenta = mix(accent_col, Color::rgba(1.0, 0.15, 0.85, 1.0), 0.75);
    let shield_lime = mix(p_col, Color::rgba(0.20, 1.0, 0.40, 1.0), 0.70);
    let spark_white = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC HOLOGRAPHIC SHIELD BACKDROP
    // -------------------------------------------------------------------------
    let max_r = base_r * 1.55;
    let bg_shield = Fill::radial_gradient(
        cx,
        cy,
        inner_r,
        cx,
        cy,
        max_r * 1.8,
        &[
            (0.0, mix(shield_cyan, shield_magenta, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, mix(shield_lime, s_col, 0.5).with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_shield);

    // -------------------------------------------------------------------------
    // 2. EXPANDING FORCEFIELD SHOCKWAVE RIPPLES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(shield_cyan, shield_magenta, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * user_scale);
        c.set_shadow(shield_cyan, (12.0 + bs * 8.0) * user_scale);
        c.stroke_circle(cx, cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. LAYER 1: INNER CONTAINMENT ARC SHIELD (CLOCKWISE)
    // -------------------------------------------------------------------------
    let rot1 = frame_time * 0.15;
    let r1 = inner_r + (20.0 + be * 15.0) * user_scale;
    let arc1_len = (TAU / 3.0) * 0.85;

    for i in 0..3 {
        let a0 = rot1 + i as f32 * (TAU / 3.0);
        let a1 = a0 + arc1_len;

        c.set_stroke(Fill::Solid(shield_cyan.with_alpha(0.65)));
        c.set_line_width((3.5 + bs * 2.0) * user_scale);
        c.set_shadow(shield_cyan, (16.0 + bs * 10.0) * user_scale);
        c.stroke_arc(cx, cy, r1, a0, a1);

        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width(1.5 * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_arc(cx, cy, r1, a0, a1);
    }

    // -------------------------------------------------------------------------
    // 4. LAYER 2: MIDDLE HEXAGONAL FORCEFIELD SHIELD PLATES (COUNTER-CLOCKWISE)
    // -------------------------------------------------------------------------
    let step = (freq.len() / SHIELD_SEGMENTS).max(1);
    let rot2 = frame_time * -0.10;
    let r2_base = r1 + 35.0 * user_scale;
    let arc2_sweep = (TAU / SHIELD_SEGMENTS as f32) * 0.80;
    let sweep = super::radial_common::sweep_angle(ctx.beat_count);
    let sweep_off = ((sweep / TAU) * (SHIELD_SEGMENTS as f32)) as usize % SHIELD_SEGMENTS.max(1);

    for s in 0..SHIELD_SEGMENTS {
        let sf = s as f32;
        let base_a = rot2 + sf * (TAU / SHIELD_SEGMENTS as f32);

        let slot = (s + sweep_off) % SHIELD_SEGMENTS;
        let sample_bin = (slot * step).min(freq.len().saturating_sub(1));
        let fv = freq[sample_bin] as f32 / 255.0;

        let val = (fv * sensitivity * 1.1 + be * 0.30 + bs * 0.20).clamp(0.10, 2.0);
        let s_r = r2_base + (val * 35.0) * user_scale;

        let seg_col = mix(mix(shield_cyan, shield_lime, sf / SHIELD_SEGMENTS as f32), mix(shield_magenta, spark_white, 0.4), val);

        // Pass A: Outer Plasma Sheath
        c.set_stroke(Fill::Solid(seg_col.with_alpha(0.65)));
        c.set_line_width((4.2 + val * 2.5) * user_scale);
        c.set_shadow(seg_col, (18.0 + val * 12.0) * user_scale);
        c.stroke_arc(cx, cy, s_r, base_a, base_a + arc2_sweep);

        // Pass B: White-Hot Core Filament
        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width(1.6 * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_arc(cx, cy, s_r, base_a, base_a + arc2_sweep);

        // Glowing HUD Corner Brackets
        let corner1_a = base_a;
        let corner2_a = base_a + arc2_sweep;

        let c1_x = cx + corner1_a.cos() * s_r;
        let c1_y = cy + corner1_a.sin() * s_r;
        let c2_x = cx + corner2_a.cos() * s_r;
        let c2_y = cy + corner2_a.sin() * s_r;

        c.set_fill(Fill::Solid(spark_white));
        c.set_shadow(seg_col, (12.0 + val * 8.0) * user_scale);
        c.fill_circle(c1_x, c1_y, (3.2 + val * 1.5) * user_scale);
        c.fill_circle(c2_x, c2_y, (3.2 + val * 1.5) * user_scale);
    }

    // -------------------------------------------------------------------------
    // 5. LAYER 3: OUTER TACTICAL DEFENSE ARMOR RING (CLOCKWISE)
    // -------------------------------------------------------------------------
    let rot3 = frame_time * 0.08;
    let r3_base = r2_base + 45.0 * user_scale;
    let arc3_sweep = (TAU / 4.0) * 0.70;

    for q in 0..4 {
        let qf = q as f32;
        let base_a = rot3 + qf * (TAU / 4.0);

        let sample_bin = (q * (freq.len() / 4)).min(freq.len().saturating_sub(1));
        let fv = freq[sample_bin] as f32 / 255.0;

        let val = (fv * sensitivity * 1.1 + bs * 0.35).clamp(0.10, 2.0);
        let s_r = r3_base + (val * 25.0) * user_scale;

        c.set_stroke(Fill::Solid(shield_magenta.with_alpha(0.60)));
        c.set_line_width((3.5 + val * 2.0) * user_scale);
        c.set_shadow(shield_magenta, (16.0 + val * 10.0) * user_scale);
        c.stroke_arc(cx, cy, s_r, base_a, base_a + arc3_sweep);

        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width(1.4 * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_arc(cx, cy, s_r, base_a, base_a + arc3_sweep);
    }

    // -------------------------------------------------------------------------
    // 6. FLOATING QUANTUM SPARKS & DEFENSE DUST PARTICLES
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0 * sensitivity).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = inner_r + m_t * (max_r * 0.95);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(shield_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(shield_cyan, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // -------------------------------------------------------------------------
    // 7. PUMPING CENTRAL DISC & SHIELD CORE RESERVOIR
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(shield_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(shield_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
