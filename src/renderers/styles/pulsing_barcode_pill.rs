//! Pulsing Barcode Pill style renderer (`pulsingBarcodePill`) — Cyberpunk Holographic Barcode Engine.
//!
//! Masterpiece 360° Cyberpunk Holographic Radial Barcode Array:
//! - 64 Radial Barcode Strips of varying widths (thick bars, thin bars, micro-gap stripes).
//! - Dual-Pass Laser Rendering with white-hot core lines & cyan/magenta plasma sheaths.
//! - Sweeping 360° High-Precision Red/Cyan Laser Scanner Line.
//! - Central Quantum Barcode Core Disc with 3 expanding forcefield shockwave ripples.
//! - 45+ Digital matrix data motes & cyber spark particles.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const BARCODE_COUNT: usize = 64;

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

    // Curated Cyberpunk Barcode Palette
    let bar_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.75);
    let bar_magenta = mix(accent_col, Color::rgba(1.0, 0.15, 0.85, 1.0), 0.75);
    let bar_yellow = mix(p_col, Color::rgba(1.0, 0.85, 0.10, 1.0), 0.70);
    let scan_red = Color::rgba(1.0, 0.10, 0.25, 1.0);
    let spark_white = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC CYBERPUNK BARCODE GLOW BACKDROP
    // -------------------------------------------------------------------------
    let max_r = base_r * 1.55;
    let bg_bar = Fill::radial_gradient(
        cx,
        cy,
        inner_r,
        cx,
        cy,
        max_r * 1.8,
        &[
            (0.0, mix(bar_cyan, bar_magenta, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, mix(bar_yellow, s_col, 0.5).with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_bar);

    // -------------------------------------------------------------------------
    // 2. EXPANDING BARCODE SHOCKWAVE RIPPLES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(bar_cyan, bar_magenta, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * user_scale);
        c.set_shadow(bar_cyan, (12.0 + bs * 8.0) * user_scale);
        c.stroke_circle(cx, cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. 360° RADIAL BARCODE STRIPS (VARIOUS WIDTHS & NEON SHEATHS)
    // -------------------------------------------------------------------------
    let step = (freq.len() / BARCODE_COUNT).max(1);
    let rot = frame_time * 0.08;

    for i in 0..BARCODE_COUNT {
        let angle = (i as f32 / BARCODE_COUNT as f32) * TAU + rot;
        let (sin_a, cos_a) = angle.sin_cos();

        let fv = super::radial_common::opposing_interleaved_bin(freq, step, i, BARCODE_COUNT, ctx.beat_count);

        // Realistic barcode width pattern (pseudo-random deterministic width per bar index)
        let bar_width_pattern = match i % 5 {
            0 => 4.5,
            1 => 1.8,
            2 => 3.2,
            3 => 1.2,
            _ => 2.6,
        };

        let val = (fv * sensitivity * 1.1 + be * 0.30 + bs * 0.20).clamp(0.08, 2.2);
        let bar_len = (35.0 + val * 75.0) * user_scale;

        let x0 = cx + cos_a * inner_r;
        let y0 = cy + sin_a * inner_r;
        let x1 = cx + cos_a * (inner_r + bar_len);
        let y1 = cy + sin_a * (inner_r + bar_len);

        let bar_col = mix(mix(bar_cyan, bar_magenta, i as f32 / BARCODE_COUNT as f32), mix(bar_yellow, spark_white, 0.4), val);

        // Pass A: Outer Plasma Sheath Glow
        c.set_stroke(Fill::Solid(bar_col.with_alpha(0.65)));
        c.set_line_width((bar_width_pattern + val * 1.5) * user_scale);
        c.set_shadow(bar_col, (14.0 + val * 10.0) * user_scale);
        c.stroke_line(x0, y0, x1, y1);

        // Pass B: White-Hot Core Filament
        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width((bar_width_pattern * 0.4).max(1.0) * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_line(x0, y0, x1, y1);

        // Barcode Tip Cap Node
        c.set_fill(Fill::Solid(spark_white));
        c.set_shadow(bar_col, (10.0 + val * 8.0) * user_scale);
        c.fill_circle(x1, y1, (1.8 + val * 1.2) * user_scale);
    }

    // -------------------------------------------------------------------------
    // 4. SWEEPING 360° LASER SCANNER BEAM
    // -------------------------------------------------------------------------
    let scan_angle = frame_time * 1.8 % TAU;
    let (s_sin, s_cos) = scan_angle.sin_cos();

    let scan_x0 = cx + s_cos * inner_r;
    let scan_y0 = cy + s_sin * inner_r;
    let scan_x1 = cx + s_cos * max_r;
    let scan_y1 = cy + s_sin * max_r;

    c.set_stroke(Fill::Solid(scan_red));
    c.set_line_width((3.5 + bs * 2.0) * user_scale);
    c.set_shadow(scan_red, (24.0 + bs * 16.0) * user_scale);
    c.stroke_line(scan_x0, scan_y0, scan_x1, scan_y1);

    c.set_stroke(Fill::Solid(spark_white));
    c.set_line_width(1.6 * user_scale);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.stroke_line(scan_x0, scan_y0, scan_x1, scan_y1);

    // -------------------------------------------------------------------------
    // 5. FLOATING DIGITAL MATRIX MOTES & CYBER SPARKS
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = inner_r + m_t * (max_r * 0.95);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(bar_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(bar_cyan, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // -------------------------------------------------------------------------
    // 6. PUMPING CENTRAL DISC & BARCODE CORE RESERVOIR
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(bar_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(bar_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
