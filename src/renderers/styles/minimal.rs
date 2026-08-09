//! Aurora Horizon 3D style renderer (`minimal`) — Multi-Layered Floating Wave Engine.
//!
//! Upgraded Masterpiece:
//! - Multi-layered 3D floating aurora wave ribbons with translucent fill & smooth wave turbulence.
//! - Audio-reactive amplitude expansion & ambient horizon glow.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const WAVE_SEGS: usize = 96;

pub fn effective_bar_count(cfg: &crate::config::VisualizerConfig) -> usize {
    cfg.reactivity.bar_count.clamp(16, 128) as usize
}

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

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let _bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep aurora backdrop
//     c.set_fill(Fill::Solid(Color::hex("#020409")));
//     c.fill_rect(0.0, 0.0, width, height);

    // Ambient horizon glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy + height * 0.15,
        0.0,
        cx,
        cy + height * 0.15,
        width * 0.65,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.85, 1.0, 0.22), 0.5)),
            (0.50, mix(p_col, Color::rgba(0.50, 0.0, 0.85, 0.08), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(amb_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. 3 MULTI-LAYERED AURORA WAVE RIBBONS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / WAVE_SEGS).max(1);

    for layer in 0..3 {
        let layer_f = layer as f32;
        let mut top_pts: Vec<(f32, f32)> = Vec::with_capacity(WAVE_SEGS + 1);

        for seg in 0..=WAVE_SEGS {
            let t = seg as f32 / WAVE_SEGS as f32;
            let x = (t - 0.5) * (width * 0.90) + cx;

            let bin_k = (seg * step_f).min(freq.len().saturating_sub(1));
            let fv = freq[bin_k] as f32 / 255.0;

            let wave1 = (t * 5.0 + frame_time * (1.2 + layer_f * 0.4)).sin() * (30.0 + layer_f * 15.0) * user_scale;
            let wave2 = (t * 9.0 - frame_time * 1.5).cos() * (15.0 + fv * 60.0 * sensitivity) * user_scale;
            let y = cy + (wave1 + wave2 + (layer_f - 1.0) * 35.0 + be * 20.0);

            top_pts.push((x, y));
        }

        let mut fill_pts = top_pts.clone();
        fill_pts.push((cx + width * 0.45, height));
        fill_pts.push((cx - width * 0.45, height));

        let wave_col = mix(
            mix(p_col, s_col, layer_f / 2.0),
            mix(accent_col, glow_col, 0.5),
            0.4 + layer_f * 0.2,
        );

        c.set_fill(Fill::Solid(Color::rgba(wave_col.r, wave_col.g, wave_col.b, 0.25 - layer_f * 0.06)));
        c.fill_polygon(&fill_pts);

        c.set_stroke(Fill::Solid(wave_col));
        c.set_line_width((2.5 - layer_f * 0.4).max(1.0));
        c.set_shadow(wave_col, 12.0);
        c.stroke_polyline(&top_pts);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
