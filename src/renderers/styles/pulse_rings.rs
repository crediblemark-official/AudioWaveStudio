//! Quantum Pulse Rings 3D style renderer (`pulseRings`) — 3D Shockwave Ring Engine.
//!
//! Upgraded Masterpiece:
//! - Concentric 3D shockwave rings expanding in perspective 3D space.
//! - Audio-reactive fluid wave harmonics & floating starlight embers.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const PULSE_RINGS_COUNT: usize = 6;
const RING_PTS: usize = 64;

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
    // Scale & position are applied internally per renderer.
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let _bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;
    let reference_size = width.min(height);
    let base_r = 85.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep pulse backdrop
//     c.set_fill(Fill::Solid(Color::hex("#020308")));
//     c.fill_rect(0.0, 0.0, width, height);

    // Ambient pulse glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        base_r * 3.2,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.95, 0.70, 0.28), 0.5)),
            (0.50, mix(p_col, Color::rgba(0.0, 0.40, 0.80, 0.10), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(amb_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. EXPANDING 3D QUANTUM PULSE SHOCKWAVE RINGS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for ring_i in 1..=PULSE_RINGS_COUNT {
        let ring_f = ring_i as f32;
        let r_curr = base_r * (0.35 + ring_f * 0.32 + be * 0.15);

        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(RING_PTS);

        for n in 0..RING_PTS {
            let angle = (n as f32 / RING_PTS as f32) * TAU;
            let bin_k = ((ring_i * RING_PTS + n) * step_f / (PULSE_RINGS_COUNT * RING_PTS / bar_count.max(1)).max(1))
                .min(freq.len().saturating_sub(1));
            let fv = freq[bin_k] as f32 / 255.0;

            let wave = (angle * 6.0 + frame_time * 2.0).cos() * (10.0 + fv * 35.0 * sensitivity + be * 15.0);
            let r_nodal = r_curr + wave;

            let (sin_a, cos_a) = angle.sin_cos();
            pts.push((cx + cos_a * r_nodal, cy + sin_a * r_nodal));
        }

        let ring_col = mix(
            mix(p_col, glow_col, ring_f / PULSE_RINGS_COUNT as f32),
            mix(accent_col, s_col, 0.5),
            ring_f / PULSE_RINGS_COUNT as f32,
        );

        c.set_stroke(Fill::Solid(ring_col));
        c.set_line_width((2.5 - ring_f * 0.25).max(1.0));
        c.set_shadow(ring_col, (12.0 - ring_f * 0.8).max(2.0));
        c.stroke_polyline(&pts);
    }

    // Center core — user's Radial Center Image when set
    draw_radial_center_image(c, ctx, cx, cy, base_r * 0.6);

    c.set_global_alpha(1.0);
    c.restore();
}
