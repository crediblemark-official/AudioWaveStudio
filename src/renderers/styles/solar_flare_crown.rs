//! Solar Flare Crown style renderer (`solarFlareCrown`) — 360° Sun Corona Engine.
//!
//! Features:
//! - Blinding golden solar core with pulsing bass excursion.
//! - 80+ organic fluid solar prominence ribbons sweeping 360°.
//! - S-curve Bezier turbulence & solar prominence sparks flying outward.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const FLARE_STRANDS: usize = 80;
const SEGMENTS: usize = 16;
const PROMINENCE_SPARKS: usize = 48;

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

    let cx = width * 0.5 ;
    let cy = height * 0.5 ;
    let reference_size = width.min(height);
    let base_r = 80.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep warm cosmic space backdrop
    c.set_fill(Fill::Solid(Color::hex("#080201")));
    c.fill_rect(0.0, 0.0, width, height);

    let corona_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.5,
        cx,
        cy,
        base_r * 3.6,
        &[
            (0.0, Color::rgba(1.0, 0.55, 0.05, 0.35 + be * 0.20)),
            (0.45, Color::rgba(0.90, 0.20, 0.02, 0.15)),
            (0.80, Color::rgba(0.30, 0.03, 0.01, 0.04)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(corona_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. 360° FLUID SOLAR PROMINENCE FLAME STRANDS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    c.set_blend_additive();

    for strand_i in 0..FLARE_STRANDS {
        let angle = (strand_i as f32 / FLARE_STRANDS as f32) * TAU + frame_time * 0.10;
        let bin_k = (strand_i * step_f / (FLARE_STRANDS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let strand_h = 20.0 + fv * 160.0 * sensitivity + be * 45.0;
        let r_start = base_r * (0.95 + be * 0.10);

        let (sin_a, cos_a) = angle.sin_cos();
        let perp_x = -sin_a;
        let perp_y = cos_a;

        let mut left_edge: Vec<(f32, f32)> = Vec::with_capacity(SEGMENTS + 1);
        let mut right_edge: Vec<(f32, f32)> = Vec::with_capacity(SEGMENTS + 1);

        for seg in 0..=SEGMENTS {
            let t = seg as f32 / SEGMENTS as f32;
            let r_curr = r_start + t * strand_h;

            let sway = (t * 5.0 - frame_time * 2.2 + strand_i as f32 * 0.4).sin() * (8.0 + t * 18.0);
            let px = cx + cos_a * r_curr + perp_x * sway;
            let py = cy + sin_a * r_curr + perp_y * sway;

            let taper = (std::f32::consts::PI * t).sin() * (1.0 - t * 0.5);
            let half_w = ((5.0 + fv * 10.0) * taper).clamp(0.5, 22.0);

            left_edge.push((px - perp_x * half_w, py - perp_y * half_w));
            right_edge.push((px + perp_x * half_w, py + perp_y * half_w));
        }

        let mut poly_pts = left_edge;
        right_edge.reverse();
        poly_pts.extend(right_edge);

        let strand_col = mix(
            Color::rgba(1.0, 0.95, 0.50, 0.90 + bs * 0.10),
            Color::rgba(0.95, 0.25, 0.02, 0.65),
            strand_i as f32 / FLARE_STRANDS as f32,
        );
        c.set_fill(Fill::Solid(strand_col));
        c.fill_polygon(&poly_pts);
    }

    // -------------------------------------------------------------------------
    // 2. GOLDEN SOLAR CORE
    // -------------------------------------------------------------------------
    let sun_r = base_r * (0.85 + be * 0.15);
    let sun_grad = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        sun_r,
        &[
            (0.0, Color::rgba(1.0, 0.98, 0.85, 1.0)),
            (0.35, Color::rgba(1.0, 0.70, 0.10, 0.95)),
            (0.75, Color::rgba(0.92, 0.30, 0.02, 0.85)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(sun_grad);
    c.set_shadow(Color::rgba(1.0, 0.60, 0.0, 0.95), 20.0);
    c.fill_circle(cx, cy, sun_r);

    // -------------------------------------------------------------------------
    // 3. SOLAR PROMINENCE SPARKS
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    for s_i in 0..PROMINENCE_SPARKS {
        let s_t = ((frame_time * 0.40 + s_i as f32 * 0.08) % 1.0).clamp(0.0, 1.0);
        let s_r = base_r + s_t * (width * 0.40);
        let s_ang = s_i as f32 * 0.75 + frame_time * 0.15;

        let sx = cx + s_ang.cos() * s_r;
        let sy = cy + s_ang.sin() * s_r;

        let s_sz = (4.0 * (1.0 - s_t) + 1.0).clamp(1.0, 5.5);
        let s_col = Color::rgba(1.0, 0.85, 0.30, (1.0 - s_t) * 0.90);

        c.set_fill(Fill::Solid(s_col));
        c.fill_ellipse(sx, sy, s_sz, s_sz);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
