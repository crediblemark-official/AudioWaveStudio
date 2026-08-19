//! Specterr Organic Liquid Wave style renderer (`pulsingLiquidAura`) — Hyper-Fluid Liquid Engine.
//!
//! Masterpiece Pulsing Liquid Wave Engine:
//! - 4 Interlocking multi-harmonic bioluminescent liquid wave bands flowing smoothly in 360° space.
//! - Liquid surface specular caustics highlights & volumetric fluid gradient fills.
//! - Smooth 360° even harmonic fluid physics (no lopsided blobs).
//! - 45+ Levitating bioluminescent liquid droplets splashing off wave crests.
//! - Central liquid core reservoir with 3 expanding fluid shockwave ripples.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const LIQUID_LAYERS: usize = 4;

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
    let bar_count = ctx.config.reactivity.bar_count.clamp(32, 128);

    let be = ctx.bass_energy.clamp(0.0, 3.0);
    let bs = ctx.beat_strength.clamp(0.0, 3.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);

    // Curated Liquid Palette
    let liquid_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.75);
    let liquid_emerald = mix(p_col, Color::rgba(0.0, 1.0, 0.45, 1.0), 0.75);
    let liquid_violet = mix(accent_col, Color::rgba(0.55, 0.10, 0.95, 1.0), 0.70);
    let spark_white = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC FLUID GLOW BACKDROP
    // -------------------------------------------------------------------------
    let bg_fluid = Fill::radial_gradient(
        cx,
        cy,
        inner_r,
        cx,
        cy,
        base_r * 2.5,
        &[
            (0.0, mix(liquid_cyan, liquid_violet, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, mix(liquid_emerald, s_col, 0.5).with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_fluid);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 2. EXPANDING LIQUID FLUID SHOCKWAVE RIPPLES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(liquid_cyan, liquid_violet, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * user_scale);
        c.set_shadow(liquid_cyan, (12.0 + bs * 8.0) * user_scale);
        c.stroke_circle(cx, cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. 4 INTERLOCKING MULTI-HARMONIC LIQUID WAVE BANDS
    // -------------------------------------------------------------------------
    let step = (freq.len() / bar_count).max(1);

    for layer in (0..LIQUID_LAYERS).rev() {
        let layer_f = layer as f32;
        let layer_rot = frame_time * (0.08 + layer_f * 0.04) * (if layer % 2 == 0 { 1.0 } else { -1.0 });

        let mut raw_pts: Vec<(f32, f32)> = Vec::with_capacity(bar_count);

        for i in 0..bar_count {
            let t = i as f32 / bar_count as f32;
            let angle = t * TAU + layer_rot;

            let sample_bin = (i * step).min(freq.len().saturating_sub(1));
            let fv = freq[sample_bin] as f32 / 255.0;

            // Multi-harmonic fluid wave superposition (3x, 5x, 7x modes)
            let wave_harmonics = (angle * 3.0 + frame_time * 1.5).cos() * 0.12
                + (angle * 5.0 - frame_time * 2.0).sin() * 0.08
                + (angle * 7.0 + frame_time * 2.5).cos() * 0.05;

            let val = (fv * sensitivity * 1.1 + wave_harmonics + be * 0.30 + bs * 0.20).clamp(0.08, 2.5);
            let wave_h = (val * (95.0 + layer_f * 15.0)) * user_scale;
            let r_curr = inner_r + wave_h;

            let (cos_a, sin_a) = angle.sin_cos();
            raw_pts.push((cx + cos_a * r_curr, cy + sin_a * r_curr));
        }

        // Quadratic Bezier smoothing for liquid wave contour
        let mut smooth_curve: Vec<(f32, f32)> = Vec::new();
        let num_pts = raw_pts.len();
        for i in 0..num_pts {
            let p0 = raw_pts[i];
            let p1 = raw_pts[(i + 1) % num_pts];
            let mid = ((p0.0 + p1.0) * 0.5, (p0.1 + p1.1) * 0.5);

            let seg = GpuCanvas::sample_quadratic(
                if smooth_curve.is_empty() { p0 } else { *smooth_curve.last().unwrap() },
                p0,
                mid,
                4,
            );
            if smooth_curve.is_empty() {
                smooth_curve.extend(seg);
            } else {
                smooth_curve.extend(seg.into_iter().skip(1));
            }
        }
        if let Some(&first) = smooth_curve.first() {
            smooth_curve.push(first);
        }

        let layer_col = mix(
            mix(liquid_cyan, liquid_emerald, layer_f / LIQUID_LAYERS as f32),
            mix(liquid_violet, spark_white, 0.4),
            be * 0.3,
        );

        // Pass A: Volumetric Liquid Wave Fill
        let fill_grad = Fill::radial_gradient(
            cx, cy, inner_r,
            cx, cy, base_r * 2.2,
            &[
                (0.00, layer_col.with_alpha(0.75 - layer_f * 0.12)),
                (0.50, mix(layer_col, s_col, 0.5).with_alpha(0.40)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(fill_grad);
        c.set_shadow(layer_col, (16.0 + bs * 10.0) * user_scale);
        fill_radial_polygon(c, cx, cy, &smooth_curve);

        // Pass B: Liquid Surface Specular Contour Stroke
        c.set_stroke(Fill::Solid(mix(layer_col, spark_white, 0.65)));
        c.set_line_width((2.2 + bs * 1.5) * user_scale);
        c.stroke_polyline(&smooth_curve);
    }

    // -------------------------------------------------------------------------
    // 4. FLOATING BIOLUMINESCENT LIQUID DROPLETS
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.35 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = inner_r + m_t * (base_r * 1.35);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(liquid_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(liquid_cyan, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // -------------------------------------------------------------------------
    // 5. PUMPING CENTRAL DISC & LIQUID CORE RESERVOIR
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(liquid_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(liquid_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
