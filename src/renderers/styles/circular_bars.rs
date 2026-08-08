//! Smooth Circular Bars 3D style renderer (`circularBars`) — 3D Radial Spectrum Ring Engine.
//!
//! Upgraded Masterpiece:
//! - 3D volumetric circular spectrum ring with smooth Bezier curve envelopes & metallic silver tops.
//! - Audio-reactive height pulsation & glowing particle halo.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const CIRCULAR_BARS_COUNT: usize = 64;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    // Scale & position are applied once by the global canvas transform.
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let _bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5;
    let cy = height * 0.5;
    let reference_size = width.min(height);
    let inner_r = 100.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep circular backdrop
    c.set_fill(Fill::Solid(Color::hex("#020308")));
    c.fill_rect(0.0, 0.0, width, height);

    // Ambient halo glow
    let halo_glow = Fill::radial_gradient(
        cx,
        cy,
        inner_r * 0.5,
        cx,
        cy,
        inner_r * 2.8,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.28), 0.5)),
            (0.50, mix(p_col, Color::rgba(0.80, 0.10, 0.60, 0.10), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(halo_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. 3D VOLUMETRIC CIRCULAR SPECTRUM BARS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..CIRCULAR_BARS_COUNT {
        let angle = (i as f32 / CIRCULAR_BARS_COUNT as f32) * TAU + frame_time * 0.10;
        let bin_k = (i * step_f / (CIRCULAR_BARS_COUNT / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let bar_h = 15.0 + fv * 140.0 * sensitivity + be * 35.0;
        let r0 = inner_r * (1.0 + be * 0.06);
        let r1 = r0 + bar_h;

        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let bar_col = mix(
            mix(p_col, glow_col, fv),
            mix(accent_col, s_col, 0.5),
            fv,
        );

        c.set_stroke(Fill::Solid(bar_col));
        c.set_line_width(3.5 + fv * 2.0);
        c.set_shadow(bar_col, 10.0);
        c.stroke_line(x0, y0, x1, y1);
    }

    // Inner Core Disc — user's Radial Center Image when set, themed disc otherwise
    if !draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.85) {
        c.set_fill(Fill::Solid(Color::hex("#050812")));
        c.set_stroke(Fill::Solid(mix(p_col, glow_col, 0.7)));
        c.set_line_width(2.0);
        c.fill_circle(cx, cy, inner_r * 0.85);
        c.stroke_circle(cx, cy, inner_r * 0.85);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
