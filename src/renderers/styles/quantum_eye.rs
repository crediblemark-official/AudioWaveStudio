//! Quantum Eye Portal style renderer (`quantumEye`) — Cybernetic Aperture Engine.
//!
//! Masterpiece Cybernetic Iris Aperture:
//! - 16 mechanical cybernetic iris aperture blades opening & closing dynamically (NO needle spikes!).
//! - 6 concentric cybernetic iris rings pulsating with audio frequencies.
//! - Quantum Singularity Pupil at center expanding dynamically driven by volume & bass energy.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const BLADE_COUNT: usize = 16;
const IRIS_RINGS: usize = 6;

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
    let base_r = 110.0 * (reference_size / 500.0) * user_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Quantum Iris Ambient Glow

    // Quantum Iris Ambient Glow
    let eye_glow = Fill::radial_gradient(
        cx,
        cy,
        base_r * 0.3,
        cx,
        cy,
        base_r * 3.2,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 0.35 + be * 0.20), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.80, 0.0, 0.60, 0.15), 0.5)),
            (0.80, mix(s_col, Color::rgba(0.05, 0.10, 0.25, 0.04), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(eye_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. 6 CONCENTRIC CYBERNETIC IRIS RINGS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);

    for r_i in 1..=IRIS_RINGS {
        let r_f = r_i as f32;
        let bin_k = (r_i * 2 * step_f).min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let ring_r = base_r * (0.45 + r_f * 0.16 + fv * 0.06 * sensitivity);
        let ring_col = mix(
            mix(p_col, glow_col, r_f / IRIS_RINGS as f32),
            mix(accent_col, Color::rgba(0.0, 0.95, 1.0, 0.90), fv),
            fv,
        );

        c.set_stroke(Fill::Solid(ring_col));
        c.set_line_width((2.5 + fv * 4.0) * user_scale);
        c.set_shadow(ring_col, (10.0 + fv * 8.0) * user_scale);
        c.stroke_circle(cx, cy, ring_r);
    }

    // -------------------------------------------------------------------------
    // 2. 16 MECHANICAL CYBERNETIC IRIS APERTURE BLADES
    // -------------------------------------------------------------------------
    let iris_outer_r = base_r * (1.10 + be * 0.10);
    let pupil_r = base_r * (0.35 + be * 0.25 + bs * 0.10);

    for b in 0..BLADE_COUNT {
        let a0 = (b as f32 / BLADE_COUNT as f32) * TAU + frame_time * 0.18;
        let a1 = ((b + 1) as f32 / BLADE_COUNT as f32) * TAU + frame_time * 0.18;

        let p0 = (cx + a0.cos() * pupil_r, cy + a0.sin() * pupil_r);
        let p1 = (cx + a1.cos() * pupil_r, cy + a1.sin() * pupil_r);
        let o1 = (cx + (a1 + 0.25).cos() * iris_outer_r, cy + (a1 + 0.25).sin() * iris_outer_r);
        let o0 = (cx + (a0 + 0.25).cos() * iris_outer_r, cy + (a0 + 0.25).sin() * iris_outer_r);

        let blade_pts = vec![p0, p1, o1, o0];
        let blade_col = mix(Color::hex("#0a1526"), Color::hex("#1a3054"), (b % 2) as f32);

        c.set_fill(Fill::Solid(blade_col));
        c.set_stroke(Fill::Solid(glow_col));
        c.set_line_width(1.5 * user_scale);
        c.fill_polygon(&blade_pts);
        c.stroke_polyline(&blade_pts);
    }

    // -------------------------------------------------------------------------
    // 3. QUANTUM SINGULARITY PUPIL AT CENTER
    // -------------------------------------------------------------------------
    let core_r = pupil_r * 0.85;
    let pupil_grad = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        core_r,
        &[
            (0.0, Color::rgba(1.0, 1.0, 1.0, 0.98)),
            (0.40, mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 0.85), 0.6)),
            (1.0, mix(p_col, Color::hex("#020810"), 0.85)),
        ],
    );

    c.set_fill(pupil_grad);
    c.set_shadow(glow_col, (18.0 + bs * 10.0) * user_scale);
    c.fill_circle(cx, cy, core_r);

    c.set_global_alpha(1.0);
    c.restore();
}
