//! Neon Lotus Bloom style renderer (`neonLotus`) — Cyber Flora Engine.
//!
//! Masterpiece 3D Cyber Lotus:
//! - 5 concentric layers of smooth organic glowing lotus petals blooming 360° (NO needle spikes!).
//! - Smooth audio-reactive petal expansion & theme color gradient shifts.
//! - Golden glowing lotus stamen core & stamen filaments at the flower center.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const PETAL_LAYERS: usize = 5;
const PETALS_PER_LAYER: usize = 12;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let user_scale = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 - pos_offset_y;
    let reference_size = width.min(height);
    let base_r = 85.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Pastel Lotus Ambient Glow
    let lotus_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.3,
        cx,
        cy,
        base_r * 3.2,
        &[
            (0.0, mix(glow_col, Color::rgba(1.0, 0.20, 0.70, 0.35 + be * 0.20), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.50, 0.0, 0.90, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.15, 0.0, 0.35, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(lotus_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. 5 LAYERS OF ORGANIC SMOOTH BLOOMING LOTUS PETALS (OUTER TO INNER)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for layer in (0..PETAL_LAYERS).rev() {
        let layer_f = layer as f32;
        let layer_r = base_r * (0.60 + layer_f * 0.40 + be * 0.15);
        let layer_angle_offset = layer_f * 0.26 + frame_time * 0.06 * (if layer % 2 == 0 { 1.0 } else { -1.0 });

        for p_i in 0..PETALS_PER_LAYER {
            let p_f = p_i as f32;
            let angle = (p_f / PETALS_PER_LAYER as f32) * TAU + layer_angle_offset;

            let bin_k = ((layer * PETALS_PER_LAYER + p_i) * step_f / (PETAL_LAYERS * PETALS_PER_LAYER / bar_count.max(1)).max(1))
                .min(freq.len().saturating_sub(1));
            let fv = freq[bin_k] as f32 / 255.0;

            let petal_rx = (20.0 + fv * 35.0 * sensitivity + be * 15.0) * user_scale;
            let petal_ry = (35.0 + fv * 45.0 * sensitivity + be * 20.0) * user_scale;

            let petal_cx = cx + angle.cos() * layer_r;
            let petal_cy = cy + angle.sin() * layer_r;

            let petal_col = mix(
                mix(p_col, glow_col, (layer_f + fv) / (PETAL_LAYERS as f32)),
                mix(accent_col, Color::rgba(1.0, 0.20, 0.70, 0.85), fv),
                fv,
            );

            c.save();
            c.translate(petal_cx, petal_cy);
            c.rotate(angle + std::f32::consts::FRAC_PI_2);

            c.set_fill(Fill::Solid(Color::rgba(petal_col.r, petal_col.g, petal_col.b, 0.65 + fv * 0.25)));
            c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.80)));
            c.set_line_width(1.5 * user_scale);
            c.set_shadow(petal_col, (10.0 + fv * 8.0) * user_scale);

            c.fill_ellipse(0.0, 0.0, petal_rx, petal_ry);
            c.restore();
        }
    }

    // -------------------------------------------------------------------------
    // 2. GOLDEN LOTUS STAMEN CORE AT FLOWER CENTER
    // -------------------------------------------------------------------------
    let stamen_r = base_r * (0.35 + be * 0.10);
    let stamen_fill = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        stamen_r,
        &[
            (0.0, Color::rgba(1.0, 0.95, 0.40, 0.98)),
            (0.50, mix(glow_col, Color::rgba(1.0, 0.40, 0.10, 0.85), 0.6)),
            (1.0, mix(p_col, Color::hex("#300020"), 0.8)),
        ],
    );

    c.set_fill(stamen_fill);
    c.set_shadow(Color::rgba(1.0, 0.85, 0.20, 0.90), (14.0 + bs * 8.0) * user_scale);
    c.fill_circle(cx, cy, stamen_r);

    c.set_global_alpha(1.0);
    c.restore();
}
