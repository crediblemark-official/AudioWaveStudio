//! Liquid Neon Cyber Goo style renderer (`liquidNeonCyberGoo`).
//!
//! Visual Concept:
//! - Masterpiece Radial Cyber-Goo & Liquid Metaball Fluid Engine.
//! - Centered glowing cyber-goo core pulsating radially with surface tension waists & organic fluid curves.
//! - Multi-layered neon cyan, magenta, & lime cyber-goo contours driven by audio spectrum.
//! - Audio-reactive dripping cyber-goo droplets, radial shockwaves, and ambient volumetric aura.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

const GOO_SAMPLES:    usize = 72;
const DROPLET_COUNT:  usize = 45;

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

    // Ultra-vibrant Cyber-Goo Palette (saturated, luminous jewel tones)
    let cyber_magenta = mix(p_col, Color::rgba(1.0, 0.05, 0.55, 1.0), 0.85);
    let cyber_cyan    = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.90);
    let cyber_lime    = mix(accent_col, Color::rgba(0.20, 1.0, 0.35, 1.0), 0.85);
    let cyber_violet  = mix(s_col, Color::rgba(0.55, 0.05, 1.0, 1.0), 0.85);
    let wet_glint     = Color::rgba(1.0, 1.0, 1.0, 0.90);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Full-Screen Radial Volumetric Cyber Atmospheric Backdrop
    let bg_goo = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, base_r * 2.8,
        &[
            (0.00, cyber_magenta.with_alpha(0.30 + be * 0.18)),
            (0.40, cyber_violet.with_alpha(0.18 + bs * 0.08)),
            (0.75, cyber_cyan.with_alpha(0.06)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_goo);
    c.fill_rect(cx - base_r * 2.8, cy - base_r * 2.8, base_r * 5.6, base_r * 5.6);

    // Dynamic Cyber Shockwave Ripples on Beats
    if bs > 0.12 {
        let shock_r1 = (base_r * 1.2 + bs * 200.0) * user_scale;
        let shock_r2 = (base_r * 0.8 + bs * 140.0) * user_scale;
        c.set_stroke(Fill::Solid(cyber_cyan.with_alpha((0.50 * (1.0 - bs / 3.0)).max(0.0))));
        c.set_line_width((3.0 + bs * 2.0) * user_scale);
        c.set_shadow(cyber_cyan, (16.0 + bs * 10.0) * user_scale);
        c.stroke_circle(cx, cy, shock_r1);

        c.set_stroke(Fill::Solid(cyber_magenta.with_alpha((0.40 * (1.0 - bs / 3.0)).max(0.0))));
        c.set_line_width((2.0 + bs * 1.5) * user_scale);
        c.stroke_circle(cx, cy, shock_r2);
    }

    // 2. Radial Cyber-Goo Fluid Waves (3 Layered Concentric Waves)
    let step = (freq.len() / GOO_SAMPLES).max(1);

    for layer in 0..3 {
        let lf = layer as f32;
        let rot = frame_time * (0.08 + lf * 0.02) * if layer % 2 == 0 { 1.0 } else { -1.0 };
        let mut raw_pts: Vec<(f32, f32)> = Vec::with_capacity(GOO_SAMPLES + 1);

        for i in 0..GOO_SAMPLES {
            let t = i as f32 / GOO_SAMPLES as f32;
            let angle = t * TAU + rot;

            let fv = smooth_ring_bin(freq, step, i, GOO_SAMPLES);
            let val = (fv * sensitivity * 1.1 + be * 0.35 + bs * 0.25).clamp(0.05, 2.5);

            let wave1 = (angle * 3.0 + frame_time * 2.4 + lf).sin() * (14.0 * user_scale);
            let wave2 = (angle * 6.0 - frame_time * 3.2).cos() * (8.0 * user_scale);

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
            0 => cyber_magenta,
            1 => cyber_cyan,
            _ => cyber_lime,
        };

        let goo_fill = Fill::radial_gradient(
            cx, cy, 0.0,
            cx, cy, inner_r + (80.0 + lf * 40.0) * user_scale,
            &[
                (0.00, mix(l_col, wet_glint, 0.55).with_alpha(0.90)),
                (0.35, l_col.with_alpha(0.78 + be * 0.12)),
                (0.75, cyber_violet.with_alpha(0.45)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(goo_fill);
        c.set_shadow(l_col, (20.0 + bs * 12.0) * user_scale);
        fill_radial_polygon(c, cx, cy, &smooth_pts);

        c.set_stroke(Fill::Solid(mix(l_col, wet_glint, 0.75).with_alpha(0.90)));
        c.set_line_width((2.6 + bs * 1.5) * user_scale);
        c.stroke_polyline(&smooth_pts);
    }

    // 3. Audio-Reactive Orbiting Cyber Droplets & Particle Spray
    for d in 0..DROPLET_COUNT {
        let df = d as f32;
        let drop_r = inner_r + (10.0 + (df * 7.3).sin().abs() * 110.0 + be * 25.0) * user_scale;
        let drop_a = (df * 137.5).to_radians() + frame_time * (0.15 + (df * 3.1).cos() * 0.10);

        let dx = cx + drop_a.cos() * drop_r;
        let dy = cy + drop_a.sin() * drop_r;

        let drop_size  = (2.2 + (df * 7.3).sin().abs() * 4.0 + be * 1.8) * user_scale;
        let drop_alpha = (0.45 + bs * 0.45 + (df * 3.1).sin() * 0.15).clamp(0.15, 0.90);

        let drop_col = match d % 3 {
            0 => cyber_lime,
            1 => cyber_cyan,
            _ => cyber_magenta,
        };

        c.set_fill(Fill::Solid(mix(drop_col, wet_glint, 0.45).with_alpha(drop_alpha)));
        c.set_shadow(drop_col, (10.0 + be * 6.0) * user_scale);
        c.fill_circle(dx, dy, drop_size);
    }

    // 4. Central Glowing Core Disc
    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.85);

    c.set_global_alpha(1.0);
    c.restore();
}




