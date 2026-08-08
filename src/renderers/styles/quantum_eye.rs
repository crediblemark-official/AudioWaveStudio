//! Quantum Eye Portal style renderer (`quantumEye`) — Cybernetic Aperture Engine.
//!
//! Features:
//! - Quantum pupil opening & closing dynamically driven by volume & bass energy.
//! - Concentric iris aperture blades with neon cyan & magenta glowing edges.
//! - 64 radial fiber-optic laser tendrils pulsating to audio frequencies.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const BLADE_COUNT: usize = 12;
const LASER_COUNT: usize = 64;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let _p = theme_primary(theme);
    let _s = theme_secondary(theme);
    let _accent = theme_accent(theme);
    let _glow = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5;
    let cy = height * 0.5;
    let reference_size = width.min(height);
    let base_r = 90.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep cyan cybernetic backdrop
    c.set_fill(Fill::Solid(Color::hex("#020810")));
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. RADIAL FIBER-OPTIC LASER TENDRILS (OUTER IRIS)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..LASER_COUNT {
        let angle = (i as f32 / LASER_COUNT as f32) * TAU + frame_time * 0.15;
        let bin_k = (i * step_f / (LASER_COUNT / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let r0 = base_r * (1.05 + be * 0.10);
        let laser_len = 15.0 + fv * 150.0 * sensitivity + be * 35.0;
        let r1 = r0 + laser_len;

        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let laser_col = mix(
            Color::rgba(0.0, 0.95, 1.0, 0.85 + bs * 0.15),
            Color::rgba(1.0, 0.0, 0.70, 0.65),
            fv,
        );

        c.set_stroke(Fill::Solid(laser_col));
        c.set_line_width(2.0 + fv * 3.0);
        c.stroke_line(x0, y0, x1, y1);
    }

    // -------------------------------------------------------------------------
    // 2. APERTURE IRIS BLADES (12 MECHANICAL SEGMENTS)
    // -------------------------------------------------------------------------
    let iris_r = base_r * (1.0 + be * 0.15);
    let pupil_r = base_r * (0.35 + be * 0.25 + bs * 0.10);

    for b in 0..BLADE_COUNT {
        let a0 = (b as f32 / BLADE_COUNT as f32) * TAU + frame_time * 0.2;
        let a1 = ((b + 1) as f32 / BLADE_COUNT as f32) * TAU + frame_time * 0.2;

        let p0 = (cx + a0.cos() * pupil_r, cy + a0.sin() * pupil_r);
        let p1 = (cx + a1.cos() * pupil_r, cy + a1.sin() * pupil_r);
        let o1 = (cx + (a1 + 0.3).cos() * iris_r, cy + (a1 + 0.3).sin() * iris_r);
        let o0 = (cx + (a0 + 0.3).cos() * iris_r, cy + (a0 + 0.3).sin() * iris_r);

        let blade_pts = vec![p0, p1, o1, o0];
        let blade_col = mix(
            Color::hex("#0a1526"),
            Color::hex("#1a3054"),
            (b % 2) as f32,
        );

        c.set_fill(Fill::Solid(blade_col));
        c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.90, 1.0, 0.80)));
        c.set_line_width(1.5);
        c.fill_polygon(&blade_pts);
        c.stroke_polyline(&blade_pts);
    }

    // -------------------------------------------------------------------------
    // 3. QUANTUM PUPIL CORE & SINGULARITY DOT
    // -------------------------------------------------------------------------
    // Deep Pupil Black Hole
    c.set_fill(Fill::Solid(Color::hex("#00050d")));
    c.set_shadow(Color::rgba(0.0, 0.90, 1.0, 0.90), 16.0);
    c.fill_circle(cx, cy, pupil_r);

    // Glowing Singularity Center
    let sing_r = pupil_r * 0.35;
    let sing_grad = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        sing_r,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 1.0)),
            (0.50, Color::rgba(0.0, 0.95, 1.0, 0.90)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(sing_grad);
    c.fill_circle(cx, cy, sing_r);

    c.set_global_alpha(1.0);
    c.restore();
}
