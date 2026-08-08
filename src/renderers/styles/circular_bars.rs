//! Smooth Circular Bars style renderer (`circularBars`) — Inward Donut Spectrum Ring Engine.
//!
//! Distinct from the `radial` crown: bars here are thick donut ring segments
//! hanging from an outer rim and reaching INWARD toward the center disc, capped
//! by glowing tips, with a rotating dashed track and mirrored under-glow.
//! Features:
//! - 64 inward donut bar segments driven by a mirrored spectrum map
//! - Rotating dashed outer track ring & glowing tip orbs
//! - Bass-reactive rim pulse and soft mirrored under-glow
//! - Center disc (user's Radial Center Image when set)
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const DONUT_BARS: usize = 64;
const TRACK_DASHES: usize = 48;

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
    let rim_r = 150.0 * (reference_size / 500.0);
    let core_r = rim_r * 0.42;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep circular backdrop
    c.set_fill(Fill::Solid(Color::hex("#020308")));
    c.fill_rect(0.0, 0.0, width, height);

    // Ambient halo glow
    let halo_glow = Fill::radial_gradient(
        cx,
        cy,
        core_r,
        cx,
        cy,
        rim_r * 1.6,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.26), 0.5)),
            (0.55, mix(p_col, Color::rgba(0.80, 0.10, 0.60, 0.10), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(halo_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. ROTATING DASHED OUTER TRACK
    // -------------------------------------------------------------------------
    let dash_span = TAU / TRACK_DASHES as f32;
    for d in 0..TRACK_DASHES {
        let a = d as f32 * dash_span + frame_time * 0.30;
        c.set_stroke(Fill::Solid(mix(accent_col, s_col, 0.5).with_alpha(0.55)));
        c.set_line_width(2.0);
        c.stroke_arc(cx, cy, rim_r * 1.14, a, a + dash_span * 0.42);
    }

    // -------------------------------------------------------------------------
    // 2. INWARD DONUT SPECTRUM BARS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);
    let rim_r_pulse = rim_r * (1.0 + be * 0.05);
    let bar_span = TAU / DONUT_BARS as f32;
    let seg_gap = bar_span * 0.30;

    for i in 0..DONUT_BARS {
        let a0 = i as f32 * bar_span + frame_time * 0.08;
        let mirrored = if i < DONUT_BARS / 2 {
            DONUT_BARS / 2 - i - 1
        } else {
            i - DONUT_BARS / 2
        };
        let bin_k = (mirrored * step_f / (DONUT_BARS / 2 / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        // Bars hang from the rim and reach inward.
        let bar_len = 12.0 + fv * (rim_r * 0.55) * sensitivity + be * (rim_r * 0.12);
        let r_outer = rim_r_pulse;
        let r_inner = (r_outer - bar_len).max(core_r * 1.15);

        let bar_col = mix(mix(p_col, glow_col, fv), mix(accent_col, s_col, 0.5), fv);
        c.set_fill(Fill::Solid(bar_col.with_alpha(0.92)));
        c.set_shadow(bar_col.with_alpha(0.6), 8.0);
        c.fill_ring_arc(cx, cy, r_outer, r_inner, a0 + seg_gap * 0.5, a0 + bar_span - seg_gap * 0.5);

        // Glowing tip orb at the inward end of each bar
        let tip_a = a0 + bar_span * 0.5;
        let (sin_a, cos_a) = tip_a.sin_cos();
        let tip_r = (r_inner + r_outer) * 0.5 - bar_len * 0.12;
        let tip_col = if fv > 0.65 {
            Color::WHITE
        } else {
            mix(glow_col, accent_col, 0.4)
        };
        c.set_fill(Fill::Solid(tip_col));
        c.fill_circle(cx + cos_a * tip_r, cy + sin_a * tip_r, 2.2 + fv * 2.2);
    }
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 3. MIRRORED UNDER-GLOW (faint inverted ring below the rim)
    // -------------------------------------------------------------------------
    let mirror_glow = Fill::radial_gradient(
        cx,
        cy + rim_r * 0.35,
        0.0,
        cx,
        cy + rim_r * 0.35,
        rim_r * 0.9,
        &[
            (0.0, s_col.with_alpha(0.10)),
            (0.7, Color::TRANSPARENT),
        ],
    );
    c.set_fill(mirror_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // Core rim ring around the center disc
    c.set_stroke(Fill::Solid(mix(p_col, glow_col, 0.7).with_alpha(0.8)));
    c.set_line_width(2.0);
    c.stroke_circle(cx, cy, core_r * 1.12);

    // Inner Core Disc — user's Radial Center Image when set, themed disc otherwise
    if !draw_radial_center_image(c, ctx, cx, cy, core_r) {
        c.set_fill(Fill::Solid(Color::hex("#050812")));
        c.set_stroke(Fill::Solid(mix(p_col, glow_col, 0.7)));
        c.set_line_width(2.0);
        c.fill_circle(cx, cy, core_r);
        c.stroke_circle(cx, cy, core_r);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
