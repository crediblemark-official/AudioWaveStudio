//! Quantum Waveform Ribbon style renderer (`quantumRibbon`) — 3D Continuous Plasma Ribbon Engine.
//!
//! Masterpiece Continuous Plasma Ribbon:
//! - Smooth continuous 3D silk plasma ribbon flowing dynamically across the stage.
//! - Dual glowing neon edge strands & intense white-hot plasma core.
//! - Audio-reactive wave frequency modulation & floating quantum spark embers.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const RIBBON_POINTS: usize = 128;
const PLASMA_EMBERS: usize = 36;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let _s_col = theme_secondary(theme);
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
    let cy = height * 0.5 + pos_offset_y;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Quantum plasma atmospheric glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.70 * user_scale,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.35 + be * 0.15), 0.5)),
            (0.45, mix(p_col, Color::rgba(0.80, 0.0, 0.60, 0.15), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(amb_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CALCULATE 128 CONTINUOUS SMOOTH 3D PLASMA RIBBON POINTS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);
    let ribbon_len = width * 0.88 * user_scale;
    let start_x = cx - ribbon_len * 0.5;

    let mut core_pts: Vec<(f32, f32)> = Vec::with_capacity(RIBBON_POINTS);
    let mut top_pts: Vec<(f32, f32)> = Vec::with_capacity(RIBBON_POINTS);
    let mut bot_pts: Vec<(f32, f32)> = Vec::with_capacity(RIBBON_POINTS);

    for i in 0..RIBBON_POINTS {
        let t = i as f32 / (RIBBON_POINTS - 1) as f32;
        let px = start_x + t * ribbon_len;

        let bin_k = (i * step_f / (RIBBON_POINTS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let wave1 = (t * 5.0 - frame_time * 2.0).sin() * (60.0 + be * 40.0) * user_scale;
        let wave2 = (t * 11.0 + frame_time * 3.0).cos() * (30.0 + fv * 80.0 * sensitivity) * user_scale;
        let py = cy + wave1 + wave2;

        let ribbon_half_w = (12.0 + fv * 24.0 * sensitivity + be * 12.0) * user_scale;

        core_pts.push((px, py));
        top_pts.push((px, py - ribbon_half_w));
        bot_pts.push((px, py + ribbon_half_w));
    }

    // -------------------------------------------------------------------------
    // 2. RENDER CONTINUOUS SMOOTH PLASMA SILK RIBBON
    // -------------------------------------------------------------------------
    let ribbon_col = mix(p_col, glow_col, 0.6);

    // A. Outer Volumetric Plasma Glow (Wide Soft Ribbon Body)
    c.set_line_width((28.0 + be * 16.0) * user_scale);
    c.set_stroke(Fill::Solid(Color::rgba(ribbon_col.r, ribbon_col.g, ribbon_col.b, 0.25 + be * 0.15)));
    c.set_shadow(ribbon_col, (24.0 + bs * 12.0) * user_scale);
    c.stroke_polyline(&core_pts);

    // B. Top & Bottom Glowing Ribbon Edge Strands
    let edge_col = mix(accent_col, Color::rgba(0.0, 0.95, 1.0, 0.90), 0.5);
    c.set_line_width(3.5 * user_scale);
    c.set_stroke(Fill::Solid(edge_col));
    c.set_shadow(edge_col, 10.0 * user_scale);
    c.stroke_polyline(&top_pts);
    c.stroke_polyline(&bot_pts);

    // C. Intense White-Hot Core Plasma Line
    c.set_line_width((3.0 + be * 2.0) * user_scale);
    c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.95)));
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.90), 8.0 * user_scale);
    c.stroke_polyline(&core_pts);

    // -------------------------------------------------------------------------
    // 3. FLOATING QUANTUM PLASMA EMBERS ALONG THE RIBBON WAVE
    // -------------------------------------------------------------------------
    for e in 0..PLASMA_EMBERS {
        let e_f = e as f32;
        let t = (e_f / PLASMA_EMBERS as f32 + frame_time * 0.12) % 1.0;
        let idx = ((t * (RIBBON_POINTS - 1) as f32) as usize).min(RIBBON_POINTS - 1);

        let (bx, by) = core_pts[idx];
        let ember_r = (2.5 + (e % 3) as f32 * 1.5 + be * 2.0) * user_scale;
        let ember_col = mix(Color::rgba(1.0, 1.0, 1.0, 0.95), glow_col, (e % 3) as f32 / 3.0);

        c.set_fill(Fill::Solid(ember_col));
        c.set_shadow(ember_col, (8.0 + be * 6.0) * user_scale);
        c.fill_circle(bx, by + (e_f * 3.0).sin() * (15.0 * user_scale), ember_r);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
