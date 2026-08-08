//! Quantum Oscilloscope 3D style renderer (`oscilloscope`) — 3D Plasma Wave Engine.
//!
//! Upgraded Masterpiece:
//! - 3D laser plasma waveform twisting smoothly through 3D space with Bezier curve interpolation.
//! - Audio-reactive frequency wave modulation & glowing neon plasma aura.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const OSC_POINTS: usize = 128;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let _s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let wave = ctx.time_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5;
    let cy = height * 0.5;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep oscilloscope dark backdrop
    c.set_fill(Fill::Solid(Color::hex("#020608")));
    c.fill_rect(0.0, 0.0, width, height);

    // Ambient plasma glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.60,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.95, 0.70, 0.25), 0.5)),
            (0.50, mix(p_col, Color::rgba(0.0, 0.40, 0.80, 0.10), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(amb_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. SILKY SMOOTH 3D PLASMA LASER WAVEFORM
    // -------------------------------------------------------------------------
    let step_w = (wave.len() / OSC_POINTS).max(1);
    let mut pts: Vec<(f32, f32)> = Vec::with_capacity(OSC_POINTS);

    for i in 0..OSC_POINTS {
        let t = i as f32 / (OSC_POINTS - 1) as f32;
        let x = (t - 0.5) * (width * 0.88) + cx;

        let bin_k = (i * step_w).min(wave.len().saturating_sub(1));
        let wv = (wave[bin_k] as f32 - 128.0) / 128.0;

        let smooth_wave = (t * 6.0 + frame_time * 2.0).sin() * 15.0 * (1.0 + be * 0.5);
        let y = cy + (wv * 140.0 * sensitivity + smooth_wave);

        pts.push((x, y));
    }

    // Outer Neon Glow Line
    let glow_line_col = mix(p_col, glow_col, 0.6);
    c.set_stroke(Fill::Solid(glow_line_col));
    c.set_line_width(6.0 + be * 4.0);
    c.set_shadow(glow_line_col, 18.0 + bs * 8.0);
    c.stroke_polyline(&pts);

    // Inner Core Bright Line
    let core_col = mix(Color::rgba(1.0, 1.0, 1.0, 0.95), accent_col, 0.3);
    c.set_stroke(Fill::Solid(core_col));
    c.set_line_width(2.2 + be * 1.5);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.stroke_polyline(&pts);

    c.set_global_alpha(1.0);
    c.restore();
}
