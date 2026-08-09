//! Acoustic Cymascope Ripple style renderer (`acousticCymascope`) — Cymatics Fluid Engine.
//!
//! Masterpiece Chladni Fluid Resonance Mandala:
//! - 8-fold Chladni resonance standing wave rings forming a sacred geometry liquid mandala (NO needle spikes!).
//! - Audio-reactive concentric wave nodes with smooth fluid wave interference & bioluminescent water glow.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const MANDALA_PETALS: usize = 8;
const MANDALA_RING_NODES: usize = 96;

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
    let base_r = 100.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Cymascope Ripple Ambient Glow
    let cym_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.3,
        cx,
        cy,
        base_r * 3.5,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.95, 0.70, 0.35 + be * 0.18), 0.5)),
            (0.40, mix(p_col, Color::rgba(0.0, 0.60, 0.90, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.0, 0.20, 0.40, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(cym_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CYMATICS CONCENTRIC WATER RIPPLE MANDALA RINGS (8-FOLD CHLADNI STANDING WAVES)
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for ring_i in 1..=8 {
        let ring_f = ring_i as f32;
        let r_base = base_r * (0.25 + ring_f * 0.22 + be * 0.12);

        let bin_ring = (ring_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv_ring = freq[bin_ring] as f32 / 255.0;

        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(MANDALA_RING_NODES + 1);

        for n in 0..=MANDALA_RING_NODES {
            let angle = (n as f32 / MANDALA_RING_NODES as f32) * TAU;
            let bin_k = ((ring_i * MANDALA_RING_NODES + n) * step_f / (8 * MANDALA_RING_NODES / bar_count.max(1)).max(1))
                .min(freq.len().saturating_sub(1));
            let fv = freq[bin_k] as f32 / 255.0;

            // Cymatics 8-fold harmonic standing wave formula
            let wave_harmonic = (angle * MANDALA_PETALS as f32 + frame_time * 0.8).cos();
            let r_nodal = r_base + wave_harmonic * (10.0 + fv * 50.0 * sensitivity + be * 20.0) * user_scale;

            let (sin_a, cos_a) = angle.sin_cos();
            pts.push((cx + cos_a * r_nodal, cy + sin_a * r_nodal));
        }

        let ripple_col = mix(
            mix(glow_col, Color::rgba(0.0, 0.95, 0.70, 0.90 + bs * 0.10), ring_f / 8.0),
            mix(p_col, accent_col, fv_ring),
            ring_f / 8.0,
        );

        c.set_stroke(Fill::Solid(ripple_col));
        c.set_line_width((2.5 - ring_f * 0.18).max(1.2) * user_scale);
        c.set_shadow(ripple_col, (10.0 + fv_ring * 6.0) * user_scale);
        c.stroke_polyline(&pts);
    }

    // -------------------------------------------------------------------------
    // 2. CENTRAL BIOLUMINESCENT FLUID NODE CORE
    // -------------------------------------------------------------------------
    let core_r = base_r * (0.35 + be * 0.15);
    let core_grad = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 0.98)),
            (0.40, mix(glow_col, Color::rgba(0.0, 0.90, 0.60, 0.85), 0.6)),
            (1.0, mix(p_col, Color::hex("#01090c"), 0.8)),
        ],
    );

    c.set_fill(core_grad);
    c.set_shadow(glow_col, (14.0 + bs * 8.0) * user_scale);
    c.fill_circle(cx, cy, core_r);

    c.set_global_alpha(1.0);
    c.restore();
}
