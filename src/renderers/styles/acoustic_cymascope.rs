//! Acoustic Cymascope Ripple style renderer (`acousticCymascope`) — Cymatics Fluid Engine.
//!
//! Features:
//! - Cymatics liquid ripple simulation forming symmetrical geometric mandala patterns.
//! - Audio-reactive concentric wave nodes with smooth fluid wave interference.
//! - Bioluminescent electric blue & emerald water glow.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const MANDALA_PETALS: usize = 8;
const MANDALA_RING_NODES: usize = 64;

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
    let base_r = 95.0 * (reference_size / 500.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep bioluminescent water backdrop
    c.set_fill(Fill::Solid(Color::hex("#01090c")));
    c.fill_rect(0.0, 0.0, width, height);

    // Cymascope Ripple Ambient Glow
    let cym_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.3,
        cx,
        cy,
        base_r * 3.5,
        &[
            (0.0, Color::rgba(0.0, 0.95, 0.70, 0.28 + be * 0.18)),
            (0.40, Color::rgba(0.0, 0.60, 0.90, 0.12)),
            (0.80, Color::rgba(0.0, 0.20, 0.40, 0.04)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(cym_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CYMATICS CONCENTRIC WATER RIPPLE MANDALA RINGS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for ring_i in 1..=6 {
        let ring_f = ring_i as f32;
        let r_base = base_r * (0.30 + ring_f * 0.28 + be * 0.12);

        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(MANDALA_RING_NODES);

        for n in 0..MANDALA_RING_NODES {
            let angle = (n as f32 / MANDALA_RING_NODES as f32) * TAU;
            let bin_k = ((ring_i * MANDALA_RING_NODES + n) * step_f / (6 * MANDALA_RING_NODES / bar_count.max(1)).max(1))
                .min(freq.len().saturating_sub(1));
            let fv = freq[bin_k] as f32 / 255.0;

            // Cymatics 8-fold harmonic standing wave formula
            let wave_harmonic = (angle * MANDALA_PETALS as f32 + frame_time * 0.6).cos();
            let r_nodal = r_base + wave_harmonic * (12.0 + fv * 45.0 * sensitivity + be * 20.0);

            let (sin_a, cos_a) = angle.sin_cos();
            pts.push((cx + cos_a * r_nodal, cy + sin_a * r_nodal));
        }

        let ripple_col = mix(
            Color::rgba(0.0, 0.95, 0.70, 0.85 + bs * 0.12),
            Color::rgba(0.0, 0.60, 0.95, 0.65),
            ring_f / 6.0,
        );

        c.set_stroke(Fill::Solid(ripple_col));
        c.set_line_width((2.2 - ring_f * 0.25).max(1.0));
        c.set_shadow(ripple_col, 10.0);
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
            (0.0, Color::rgba(1.0, 1.0, 1.0, 1.0)),
            (0.40, Color::rgba(0.0, 0.95, 0.70, 0.95)),
            (0.75, Color::rgba(0.0, 0.50, 0.90, 0.85)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(core_grad);
    c.set_shadow(Color::rgba(0.0, 0.95, 0.70, 1.0), 20.0);
    c.fill_circle(cx, cy, core_r);

    c.set_global_alpha(1.0);
    c.restore();
}
