//! Waveform Topographic Ribbon style renderer (`waveformTopographicRibbon`).
//!
//! Dual Contour Topographic Waveform:
//! - Dual upper & lower waveform contour lines connected by dense vertical hatch shading lines,
//!   resembling a 3D topographic island / elevation profile map.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col   = theme_primary(theme);
    let s_col   = theme_secondary(theme);
    let acc_col = theme_accent(theme);
    let glow    = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width  * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let pcm = ctx.time_data;

    let cx = width  * 0.5 + pos_offset_x;
    let cy = height * 0.50 + pos_offset_y;

    let span_w = width * 0.88 * user_scale;
    let start_x = cx - span_w * 0.5;
    let max_amp = height * 0.38 * user_scale;

    let samples = pcm.len().min(160);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#020108")));
//     c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, width * 0.65,
        &[
            (0.00, mix(glow, Color::rgba(0.0, 0.8, 1.0, 0.3), 0.5).with_alpha(0.22 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    let mut upper_pts: Vec<(f32, f32)> = Vec::with_capacity(samples);
    let mut lower_pts: Vec<(f32, f32)> = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / (samples - 1) as f32;
        let x = start_x + t * span_w;

        let val = (pcm[i] as f32 / 128.0 - 1.0) * sensitivity;
        let amp_h = (val.abs() * max_amp + 4.0 * user_scale).clamp(4.0, max_amp);

        upper_pts.push((x, cy - amp_h));
        lower_pts.push((x, cy + amp_h));
    }

    if upper_pts.len() >= 2 {
        // 1. Vertical Topographic Hatching Lines
        c.set_shadow(Color::TRANSPARENT, 0.0);
        for i in (0..samples).step_by(2) {
            let (ux, uy) = upper_pts[i];
            let (_, ly)  = lower_pts[i];
            let t = i as f32 / samples as f32;

            let hatch_col = mix(p_col, acc_col, t).with_alpha(0.35);
            c.set_stroke(Fill::Solid(hatch_col));
            c.set_line_width(1.0 * user_scale);
            c.stroke_line(ux, uy, ux, ly);
        }

        // 2. Upper Contour Line (Primary Color)
        c.set_stroke(Fill::Solid(p_col));
        c.set_line_width(2.2 * user_scale);
        c.set_shadow(p_col, 10.0 * user_scale);
        c.stroke_polyline(&upper_pts);

        // 3. Lower Contour Line (Accent Color)
        c.set_stroke(Fill::Solid(acc_col));
        c.set_line_width(2.2 * user_scale);
        c.set_shadow(acc_col, 10.0 * user_scale);
        c.stroke_polyline(&lower_pts);
    }

    let _ = (s_col, be);

    c.set_global_alpha(1.0);
    c.restore();
}
