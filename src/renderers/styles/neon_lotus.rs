//! Neon Lotus Bloom style renderer (`neonLotus`) — Cyber Flora Engine.
//!
//! Features:
//! - Multi-layered neon lotus petals blooming radially 360°.
//! - Audio-reactive petal expansion & synthwave pastel color shifts.
//! - Golden glowing lotus stamen core reacting to bass energy.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const PETAL_LAYERS: usize = 4;
const PETALS_PER_LAYER: usize = 16;

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
    let base_r = 75.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep synthwave violet backdrop
    c.set_fill(Fill::Solid(Color::hex("#080210")));
    c.fill_rect(0.0, 0.0, width, height);

    // Pastel Lotus Glow
    let lotus_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.4,
        cx,
        cy,
        base_r * 3.5,
        &[
            (0.0, Color::rgba(1.0, 0.20, 0.70, 0.30 + be * 0.20)),
            (0.45, Color::rgba(0.50, 0.0, 0.90, 0.15)),
            (0.80, Color::rgba(0.15, 0.0, 0.35, 0.04)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(lotus_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. MULTI-LAYERED BLOOMING PETALS (OUTER TO INNER)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for layer in (0..PETAL_LAYERS).rev() {
        let layer_f = layer as f32;
        let layer_r = base_r * (0.80 + layer_f * 0.45 + be * 0.15);
        let layer_angle_offset = layer_f * 0.20 + frame_time * 0.08 * (if layer % 2 == 0 { 1.0 } else { -1.0 });

        for p_i in 0..PETALS_PER_LAYER {
            let p_f = p_i as f32;
            let angle = (p_f / PETALS_PER_LAYER as f32) * TAU + layer_angle_offset;

            let bin_k = ((layer * PETALS_PER_LAYER + p_i) * step_f / (PETAL_LAYERS * PETALS_PER_LAYER / bar_count.max(1)).max(1))
                .min(freq.len().saturating_sub(1));
            let fv = freq[bin_k] as f32 / 255.0;

            let petal_len = 20.0 + fv * 90.0 * sensitivity + be * 25.0;
            let r0 = layer_r;
            let r1 = r0 + petal_len;

            let (sin_a, cos_a) = angle.sin_cos();
            let x0 = cx + cos_a * r0;
            let y0 = cy + sin_a * r0;
            let x1 = cx + cos_a * r1;
            let y1 = cy + sin_a * r1;

            let petal_w = 14.0 + fv * 12.0;
            let px = -sin_a * (petal_w * 0.5);
            let py = cos_a * (petal_w * 0.5);

            let pts = vec![
                (x0, y0),
                (x0 - px, y0 - py),
                (x1, y1),
                (x0 + px, y0 + py),
            ];

            let petal_col = mix(
                Color::rgba(1.0, 0.20, 0.70, 0.85),
                Color::rgba(0.0, 0.85, 1.0, 0.75 + bs * 0.15),
                (layer_f + fv) / (PETAL_LAYERS as f32),
            );

            c.set_fill(Fill::Solid(petal_col));
            c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.90, 0.95, 0.60)));
            c.set_line_width(1.2);
            c.fill_polygon(&pts);
            c.stroke_polyline(&pts);
        }
    }

    // -------------------------------------------------------------------------
    // 2. GOLDEN LOTUS STAMEN CORE
    // -------------------------------------------------------------------------
    let stamen_r = base_r * (0.65 + be * 0.15);
    let stamen_grad = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        stamen_r,
        &[
            (0.0, Color::rgba(1.0, 0.95, 0.60, 1.0)),
            (0.40, Color::rgba(1.0, 0.60, 0.10, 0.90)),
            (0.75, Color::rgba(0.80, 0.10, 0.50, 0.75)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(stamen_grad);
    c.set_shadow(Color::rgba(1.0, 0.80, 0.20, 0.95), 18.0);
    c.fill_circle(cx, cy, stamen_r);

    c.set_global_alpha(1.0);
    c.restore();
}
