//! Holographic Vinyl style renderer (`holographicVinyl`) — Synthwave Record Engine.
//!
//! Features:
//! - Rotating holographic vinyl record disc with iridescent rainbow sheen.
//! - Concentric groove lines pulsating with audio bass energy.
//! - 60 radial neon equalizer bars arrayed around the outer rim.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const VINYL_BARS: usize = 60;

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
    let disc_r = 110.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Vaporwave dark purple backdrop
    c.set_fill(Fill::Solid(Color::hex("#0a0412")));
    c.fill_rect(0.0, 0.0, width, height);

    // Ambient Holographic Glow
    let holo_glow = Fill::radial_gradient(
        cx,
        cy,
        disc_r * 0.4,
        cx,
        cy,
        disc_r * 2.8,
        &[
            (0.0, Color::rgba(0.0, 0.90, 1.0, 0.25 + be * 0.15)),
            (0.40, Color::rgba(1.0, 0.20, 0.80, 0.15)),
            (0.80, Color::rgba(0.20, 0.0, 0.40, 0.05)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(holo_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. ROTATING HOLOGRAPHIC VINYL DISC & IRIDESCENT GROOVES
    // -------------------------------------------------------------------------
    let disc_grad = Fill::radial_gradient(
        cx - disc_r * 0.25,
        cy - disc_r * 0.25,
        0.0,
        cx,
        cy,
        disc_r,
        &[
            (0.0, Color::hex("#2a183d")),
            (0.40, Color::hex("#120b1c")),
            (0.75, Color::hex("#06030a")),
            (1.0, Color::hex("#020104")),
        ],
    );
    c.set_fill(disc_grad);
    c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.90, 1.0, 0.70)));
    c.set_line_width(2.0);
    c.set_shadow(Color::rgba(1.0, 0.20, 0.80, 0.80), 16.0);
    c.fill_circle(cx, cy, disc_r);
    c.stroke_circle(cx, cy, disc_r);

    // Iridescent Groove Rings
    c.set_shadow(Color::TRANSPARENT, 0.0);
    for g_i in 1..=5 {
        let gr_r = disc_r * (0.45 + g_i as f32 * 0.10);
        let gr_col = mix(
            Color::rgba(0.0, 0.90, 1.0, 0.35),
            Color::rgba(1.0, 0.30, 0.80, 0.35),
            (g_i % 2) as f32,
        );
        c.set_stroke(Fill::Solid(gr_col));
        c.set_line_width(1.2);
        c.stroke_circle(cx, cy, gr_r);
    }

    // -------------------------------------------------------------------------
    // 2. RADIAL NEON EQUALIZER BARS AROUND OUTER RIM
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for i in 0..VINYL_BARS {
        let angle = (i as f32 / VINYL_BARS as f32) * TAU + frame_time * 0.20;
        let bin_k = (i * step_f / (VINYL_BARS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let bar_h = 10.0 + fv * 120.0 * sensitivity + be * 30.0;
        let r0 = disc_r * (1.02 + be * 0.05);
        let r1 = r0 + bar_h;

        let (sin_a, cos_a) = angle.sin_cos();
        let x0 = cx + cos_a * r0;
        let y0 = cy + sin_a * r0;
        let x1 = cx + cos_a * r1;
        let y1 = cy + sin_a * r1;

        let bar_col = mix(
            Color::rgba(0.0, 0.95, 1.0, 0.90 + bs * 0.10),
            Color::rgba(1.0, 0.20, 0.80, 0.85),
            fv,
        );
        c.set_stroke(Fill::Solid(bar_col));
        c.set_line_width(3.0);
        c.stroke_line(x0, y0, x1, y1);
    }

    // -------------------------------------------------------------------------
    // 3. CENTER HOLOGRAPHIC LABEL & SPINDLE HOLE
    // -------------------------------------------------------------------------
    let label_r = disc_r * 0.38;
    let label_grad = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        label_r,
        &[
            (0.0, Color::rgba(1.0, 0.30, 0.80, 0.95)),
            (0.60, Color::rgba(0.0, 0.85, 1.0, 0.90)),
            (1.0, Color::hex("#120b1c")),
        ],
    );
    c.set_fill(label_grad);
    c.fill_circle(cx, cy, label_r);

    // Spindle Center Hole
    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.fill_circle(cx, cy, label_r * 0.30);

    c.set_global_alpha(1.0);
    c.restore();
}
