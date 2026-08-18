//! Liquid Chromatic Viscosity style renderer (`liquidChromaticViscosity`).
//!
//! Visual Concept:
//! - High-viscosity Iridescent Chromatic Fluid & Prism Light Dispersion engine.
//! - Overlapping viscous liquid ribbons flowing organically with RGB chromatic aberration.
//! - Dynamic fluid surface tension with wet specular highlights & prismatic sheen.
//! - Iridescent fluid droplets & chromatic dispersion shockwaves expanding on drum beats.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

const WAVE_LAYERS: usize = 3;
const SAMPLES_PER_LAYER: usize = 80;
const CHROMATIC_MOTES: usize = 40;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col      = theme_primary(theme);
    let s_col      = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let bass_mult    = ctx.config.reactivity.bass_multiplier;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);

    // Iridescent Chromatic Dispersion Palette
    let chrom_red    = Color::rgba(1.0, 0.05, 0.30, 0.85);
    let chrom_cyan   = Color::rgba(0.0, 0.95, 1.0, 0.85);
    let chrom_violet = mix(accent_col, Color::rgba(0.65, 0.10, 1.0, 1.0), 0.80);
    let chrom_gold   = mix(glow_col, Color::rgba(1.0, 0.85, 0.10, 1.0), 0.80);
    let wet_white    = Color::rgba(1.0, 1.0, 1.0, 0.95);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. Ambient Iridescent Backdrop Glow
    // -------------------------------------------------------------------------
    let bg_chrom = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, base_r * 3.0,
        &[
            (0.00, chrom_violet.with_alpha(0.35 + be * 0.18)),
            (0.45, chrom_cyan.with_alpha(0.18 + bs * 0.10)),
            (0.80, chrom_gold.with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_chrom);
    c.fill_rect(0.0, 0.0, width, height);

    // Chromatic Beat Dispersion Ring
    if bs > 0.12 {
        let shock_r = inner_r + bs * 150.0 * user_scale;
        c.set_stroke(Fill::Solid(chrom_cyan.with_alpha((0.60 * (1.0 - bs / 3.0)).max(0.0))));
        c.set_line_width((3.0 + bs * 2.0) * user_scale);
        c.set_shadow(chrom_violet, (16.0 + bs * 10.0) * user_scale);
        c.stroke_circle(cx, cy, shock_r);
    }

    // -------------------------------------------------------------------------
    // 2. Multi-Layer Viscous Chromatic Fluid Waves (RGB Split Layers)
    // -------------------------------------------------------------------------
    let step = (freq.len() / SAMPLES_PER_LAYER).max(1);

    for layer in 0..WAVE_LAYERS {
        let l_f = layer as f32;
        let phase_offset = l_f * 0.35;
        let speed_dir    = if layer % 2 == 0 { 1.0 } else { -0.7 };
        let rot          = frame_time * 0.10 * speed_dir + phase_offset;

        let layer_col = match layer {
            0 => chrom_cyan,
            1 => chrom_red,
            _ => chrom_gold,
        };

        let mut raw_pts: Vec<(f32, f32)> = Vec::with_capacity(SAMPLES_PER_LAYER + 1);

        for i in 0..SAMPLES_PER_LAYER {
            let t     = i as f32 / SAMPLES_PER_LAYER as f32;
            let angle = t * TAU + rot;

            let fv  = smooth_ring_bin(freq, step, i, SAMPLES_PER_LAYER);
            let val = (fv * sensitivity * 1.1 + be * 0.32 + bs * 0.20).clamp(0.05, 2.4);

            // Viscous liquid wave harmonics (multiple frequencies blending)
            let w1 = (angle * 3.0 + frame_time * 1.8 + l_f).sin() * (12.0 * user_scale);
            let w2 = (angle * 7.0 - frame_time * 2.4).cos() * (6.0 * user_scale);
            let w3 = (angle * 12.0 + frame_time * 3.2).sin() * (3.0 * user_scale);

            let r_curr = inner_r + (val * (55.0 + l_f * 15.0) * user_scale) + w1 + w2 + w3;

            let (cos_a, sin_a) = angle.sin_cos();
            raw_pts.push((cx + cos_a * r_curr, cy + sin_a * r_curr));
        }
        raw_pts.push(raw_pts[0]); // close loop

        // Smooth contour into fluid spline
        let mut smooth_pts: Vec<(f32, f32)> = Vec::with_capacity(raw_pts.len() * 3);
        let n_pts = raw_pts.len() - 1;
        for i in 0..n_pts {
            let p0  = raw_pts[i];
            let p1  = raw_pts[(i + 1) % n_pts];
            let mid = ((p0.0 + p1.0) * 0.5, (p0.1 + p1.1) * 0.5);
            let seg = GpuCanvas::sample_quadratic(
                if smooth_pts.is_empty() { p0 } else { *smooth_pts.last().unwrap() },
                p0, mid, 3,
            );
            if smooth_pts.is_empty() {
                smooth_pts.extend(seg);
            } else {
                smooth_pts.extend(seg.into_iter().skip(1));
            }
        }
        if let Some(&first) = smooth_pts.first() { smooth_pts.push(first); }

        // Render Viscous Liquid Fill
        let wave_fill = Fill::radial_gradient(
            cx, cy, inner_r,
            cx, cy, base_r * 2.2,
            &[
                (0.00, layer_col.with_alpha(0.55 + be * 0.15)),
                (0.60, mix(layer_col, chrom_violet, 0.4).with_alpha(0.35)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(wave_fill);
        c.set_shadow(layer_col, (16.0 + bs * 10.0) * user_scale);
        fill_radial_polygon(c, cx, cy, &smooth_pts);

        // Specular Wet Contour Edge
        c.set_stroke(Fill::Solid(mix(layer_col, wet_white, 0.65).with_alpha(0.80)));
        c.set_line_width((2.0 + bs * 1.2) * user_scale);
        c.set_shadow(layer_col, (10.0 + be * 6.0) * user_scale);
        c.stroke_polyline(&smooth_pts);
    }

    // -------------------------------------------------------------------------
    // 3. Floating Chromatic Iridescent Droplets
    // -------------------------------------------------------------------------
    for d in 0..CHROMATIC_MOTES {
        let df = d as f32;
        let d_angle = (df / CHROMATIC_MOTES as f32) * TAU + frame_time * (0.07 + (df * 1.5).sin() * 0.02);
        let phase   = ((frame_time * 0.38 + df * 0.31) % 1.0).clamp(0.0, 1.0);

        let dist = inner_r + phase * (base_r * 2.1);
        let (sin_a, cos_a) = d_angle.sin_cos();

        let drop_x = cx + cos_a * dist;
        let drop_y = cy + sin_a * dist;
        let alpha  = (1.0 - phase) * (0.50 + bs * 0.40);

        if alpha > 0.05 {
            let drop_sz  = (2.0 + (1.0 - phase) * 4.0 + be * 1.5) * user_scale;
            let drop_col = match d % 4 {
                0 => chrom_cyan,
                1 => chrom_red,
                2 => chrom_gold,
                _ => chrom_violet,
            };

            c.set_fill(Fill::Solid(mix(drop_col, wet_white, 0.40).with_alpha(alpha)));
            c.set_shadow(drop_col, (8.0 + be * 5.0) * user_scale);
            c.fill_circle(drop_x, drop_y, drop_sz);
        }
    }

    // -------------------------------------------------------------------------
    // 4. Central Core Disc Integration
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(chrom_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(chrom_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#06030e")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = (p_col, s_col, accent_col, glow_col);
    c.set_global_alpha(1.0);
    c.restore();
}

