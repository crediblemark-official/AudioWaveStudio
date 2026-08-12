//! Liquid Toxic Slime style renderer (`liquidToxicSlime`) — Viscous Biohazard Dripping Acid Slime Engine.
//!
//! Visual Concept:
//! - Masterpiece Radial Biohazard Acid Slime Engine.
//! - Centered glowing radioactive bio-slime pool pulsating radially with fluid surface tension.
//! - Radiating viscous slime tendrils & oozing acid contours driven by audio spectrum.
//! - Popping radioactive acid bubbles, toxic lime foam, & deep biohazard ambient aura.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

const SLIME_SAMPLES: usize = 72;
const BUBBLE_COUNT:  usize = 40;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col      = theme_primary(theme);
    let s_col      = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col   = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let bass_mult   = ctx.config.reactivity.bass_multiplier;
    let user_scale  = ctx.config.scale.clamp(0.1, 5.0);
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

    // Curated Toxic Acid Slime Palette (Fluorescent lime & radioactive yellow)
    let toxic_lime   = mix(p_col, Color::rgba(0.22, 1.0, 0.08, 1.0), 0.90);
    let acid_yellow  = mix(glow_col, Color::rgba(0.88, 1.0, 0.0, 1.0), 0.85);
    let toxic_sludge = mix(s_col, Color::rgba(0.04, 0.25, 0.08, 1.0), 0.85);
    let wet_glint    = Color::rgba(0.95, 1.0, 0.90, 0.92);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Full-Screen Radial Volumetric Toxic Glow Backdrop
    let bg_slime = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, base_r * 2.8,
        &[
            (0.00, mix(toxic_lime, acid_yellow, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.45, mix(toxic_sludge, accent_col, 0.5).with_alpha(0.16 + bs * 0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_slime);
    c.fill_rect(cx - base_r * 2.8, cy - base_r * 2.8, base_r * 5.6, base_r * 5.6);

    // Acid Shockwave Ripple on Beats
    if bs > 0.12 {
        let shock_r = (base_r * 1.2 + bs * 180.0) * user_scale;
        c.set_stroke(Fill::Solid(toxic_lime.with_alpha((0.50 * (1.0 - bs / 3.0)).max(0.0))));
        c.set_line_width((3.0 + bs * 2.0) * user_scale);
        c.set_shadow(toxic_lime, (16.0 + bs * 10.0) * user_scale);
        c.stroke_circle(cx, cy, shock_r);
    }

    // 2. Radial Liquid Acid Slime Pool Rings (3 Layered Concentric Waves)
    let step = (freq.len() / SLIME_SAMPLES).max(1);

    for layer in 0..3 {
        let lf = layer as f32;
        let rot = frame_time * (0.06 + lf * 0.02) * if layer % 2 == 0 { 1.0 } else { -1.0 };
        let mut raw_pts: Vec<(f32, f32)> = Vec::with_capacity(SLIME_SAMPLES + 1);

        for i in 0..SLIME_SAMPLES {
            let t = i as f32 / SLIME_SAMPLES as f32;
            let angle = t * TAU + rot;

            let fv = smooth_ring_bin(freq, step, i, SLIME_SAMPLES);
            let val = (fv * sensitivity * 1.1 + be * 0.35 + bs * 0.25).clamp(0.05, 2.5);

            let wave1 = (angle * 4.0 + frame_time * 2.2 + lf).sin() * (14.0 * user_scale);
            let wave2 = (angle * 8.0 - frame_time * 3.1).cos() * (7.0 * user_scale);

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
            0 => toxic_lime,
            1 => acid_yellow,
            _ => mix(toxic_lime, wet_glint, 0.4),
        };

        let slime_fill = Fill::radial_gradient(
            cx, cy, 0.0,
            cx, cy, inner_r + (80.0 + lf * 40.0) * user_scale,
            &[
                (0.00, mix(l_col, wet_glint, 0.50).with_alpha(0.85)),
                (0.40, l_col.with_alpha(0.70 + be * 0.12)),
                (0.80, toxic_sludge.with_alpha(0.40)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(slime_fill);
        c.set_shadow(l_col, (18.0 + bs * 10.0) * user_scale);
        fill_radial_polygon(c, cx, cy, &smooth_pts);

        c.set_stroke(Fill::Solid(mix(l_col, wet_glint, 0.70).with_alpha(0.88)));
        c.set_line_width((2.4 + bs * 1.4) * user_scale);
        c.stroke_polyline(&smooth_pts);
    }

    // 3. Popping Radial Acid Bubbles & Toxic Spores
    for b_i in 0..BUBBLE_COUNT {
        let b_f = b_i as f32;
        let b_r_dist = inner_r + (10.0 + (b_f * 7.1).sin().abs() * 100.0 + be * 20.0) * user_scale;
        let b_ang = (b_f * 137.5).to_radians() + frame_time * (0.15 + (b_f * 3.3).cos() * 0.08);

        let bx = cx + b_ang.cos() * b_r_dist;
        let by = cy + b_ang.sin() * b_r_dist;
        let b_r = (2.5 + (frame_time * 2.5 + b_f * 5.0).sin().abs() * 4.0 + bs * 1.8) * user_scale;

        let b_col = mix(acid_yellow, toxic_lime, (b_f * 0.3).sin().abs());

        c.set_fill(Fill::Solid(b_col.with_alpha(0.80)));
        c.set_shadow(toxic_lime, 8.0 * user_scale);
        c.fill_circle(bx, by, b_r);

        c.set_fill(Fill::Solid(wet_glint));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_circle(bx - b_r * 0.3, by - b_r * 0.3, b_r * 0.35);
    }

    // 4. Central Core Disc
    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.85);

    c.set_global_alpha(1.0);
    c.restore();
}



