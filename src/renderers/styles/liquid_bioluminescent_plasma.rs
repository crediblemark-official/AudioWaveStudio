//! Liquid Bio Cell Mitosis style renderer (`liquidBioluminescentPlasma`).
//!
//! Visual Concept:
//! - Masterpiece Radial Bioluminescent Plasma Cell Engine.
//! - Centered glowing bio-plasma core pulsating radially with fluid lipid membrane waves.
//! - Multi-layered cyan & magenta plasma cell contours driven by audio spectrum.
//! - Floating bioluminescent spores & deep-sea volumetric ambient aura.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

const PLASMA_SAMPLES: usize = 72;
const SPORE_COUNT:    usize = 45;

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
    let pos_offset_y = ctx.config.position_y * height * 0.5;

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 130.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.38 + be * 0.10);

    // Palette: Deep-Sea Bioluminescent Plasma
    let plasma_cyan    = mix(glow_col,   Color::rgba(0.0, 0.95, 1.0, 1.0),  0.85);
    let plasma_magenta = mix(accent_col, Color::rgba(1.0, 0.10, 0.70, 1.0), 0.85);
    let plasma_indigo  = mix(p_col,      Color::rgba(0.10, 0.05, 0.40, 1.0), 0.75);
    let spark_white    = Color::rgba(1.0, 1.0, 1.0, 0.95);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Full-Screen Radial Volumetric Deep-Sea Bio Plasma Ambient Backdrop
    let bg_plasma = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, base_r * 2.8,
        &[
            (0.00, mix(plasma_cyan, plasma_magenta, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, plasma_indigo.with_alpha(0.15 + bs * 0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_plasma);
    c.fill_rect(cx - base_r * 2.8, cy - base_r * 2.8, base_r * 5.6, base_r * 5.6);

    // Dynamic Plasma Shockwave Ripples
    if bs > 0.12 {
        let shock_r = (base_r * 1.2 + bs * 180.0) * user_scale;
        c.set_stroke(Fill::Solid(plasma_cyan.with_alpha((0.50 * (1.0 - bs / 3.0)).max(0.0))));
        c.set_line_width((3.0 + bs * 2.0) * user_scale);
        c.set_shadow(plasma_cyan, (16.0 + bs * 10.0) * user_scale);
        c.stroke_circle(cx, cy, shock_r);
    }

    // 2. Radial Bioluminescent Plasma Cell Membrane Waves (3 Layered Concentric Waves)
    let step = (freq.len() / PLASMA_SAMPLES).max(1);

    for layer in 0..3 {
        let lf = layer as f32;
        let rot = frame_time * (0.07 + lf * 0.02) * if layer % 2 == 0 { 1.0 } else { -1.0 };
        let mut raw_pts: Vec<(f32, f32)> = Vec::with_capacity(PLASMA_SAMPLES + 1);

        for i in 0..PLASMA_SAMPLES {
            let t = i as f32 / PLASMA_SAMPLES as f32;
            let angle = t * TAU + rot;

            let fv = smooth_ring_bin(freq, step, i, PLASMA_SAMPLES);
            let val = (fv * sensitivity * 1.1 + be * 0.35 + bs * 0.25).clamp(0.05, 2.5);

            let wave1 = (angle * 3.0 + frame_time * 2.2 + lf).sin() * (15.0 * user_scale);
            let wave2 = (angle * 6.0 - frame_time * 3.1).cos() * (8.0 * user_scale);

            let r_curr = inner_r + (20.0 + lf * 35.0 + val * (45.0 + lf * 15.0)) * user_scale + wave1 + wave2;
            let (cos_a, sin_a) = angle.sin_cos();

            raw_pts.push((cx + cos_a * r_curr, cy + sin_a * r_curr));
        }
        raw_pts.push(raw_pts[0]);

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

        let l_col = match layer {
            0 => plasma_cyan,
            1 => plasma_magenta,
            _ => mix(plasma_cyan, spark_white, 0.4),
        };

        let cell_fill = Fill::radial_gradient(
            cx, cy, 0.0,
            cx, cy, inner_r + (80.0 + lf * 40.0) * user_scale,
            &[
                (0.00, spark_white.with_alpha(0.85)),
                (0.35, l_col.with_alpha(0.65 + be * 0.15)),
                (0.75, plasma_indigo.with_alpha(0.35)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(cell_fill);
        c.set_shadow(l_col, (18.0 + bs * 10.0) * user_scale);
        fill_radial_polygon(c, cx, cy, &smooth_pts);

        c.set_stroke(Fill::Solid(mix(l_col, spark_white, 0.70).with_alpha(0.88)));
        c.set_line_width((2.4 + bs * 1.4) * user_scale);
        c.stroke_polyline(&smooth_pts);
    }

    // 3. Floating Radial Bioluminescent Spores
    for d in 0..SPORE_COUNT {
        let df = d as f32;
        let spore_r = inner_r + (10.0 + (df * 7.3).sin().abs() * 110.0 + be * 25.0) * user_scale;
        let spore_a = (df * 137.5).to_radians() + frame_time * (0.12 + (df * 3.1).cos() * 0.08);

        let sx = cx + spore_a.cos() * spore_r;
        let sy = cy + spore_a.sin() * spore_r;
        let s_sz = (1.5 + (df * 5.3).sin().abs() * 3.0 + be * 1.2) * user_scale;
        let s_col = mix(plasma_cyan, plasma_magenta, (df * 0.25).sin().abs());

        c.set_fill(Fill::Solid(mix(s_col, spark_white, 0.50).with_alpha(0.75)));
        c.set_shadow(s_col, (8.0 + be * 6.0) * user_scale);
        c.fill_circle(sx, sy, s_sz);
    }

    // 4. Central Glowing Core Disc
    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.85);

    let _ = s_col;
    c.set_global_alpha(1.0);
    c.restore();
}


