//! Waveform Barcode Pulse style renderer (`waveformBarcodePulse`).
//!
//! High-Density Barcode Pulse Waveform:
//! - Hundreds of vertical barcode lines whose thickness & opacity pulse with audio amplitude peaks.

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

    let span_w = width * 0.90 * user_scale;
    let start_x = cx - span_w * 0.5;
    let max_amp = height * 0.38 * user_scale;

    let bar_lines = 120usize;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#020108")));
//     c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, width * 0.65,
        &[
            (0.00, mix(glow, Color::rgba(0.0, 0.9, 0.7, 0.3), 0.5).with_alpha(0.22 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
    c.fill_rect(0.0, 0.0, width, height);

    let sample_step = (pcm.len() / bar_lines).max(1);

    for i in 0..bar_lines {
        let t = i as f32 / bar_lines as f32;
        let x = start_x + t * span_w;

        let sample_idx = (i * sample_step).min(pcm.len().saturating_sub(1));
        let val = ((pcm[sample_idx] as f32 / 128.0 - 1.0) * sensitivity).abs();

        let line_h = (val * max_amp + 12.0 * user_scale).clamp(8.0, max_amp);
        let top_y = cy - line_h;
        let bot_y = cy + line_h;

        let line_w = (1.2 + val * 4.0) * user_scale;
        let alpha  = (0.35 + val * 0.60).clamp(0.15, 0.95);

        let barcode_col = mix(
            mix(p_col, acc_col, t),
            mix(glow, Color::rgba(1.0, 1.0, 1.0, 1.0), val),
            val * 0.7,
        );

        c.set_stroke(Fill::Solid(barcode_col.with_alpha(alpha)));
        c.set_line_width(line_w);
        c.set_shadow(barcode_col, (4.0 + val * 8.0) * user_scale);
        c.stroke_line(x, top_y, x, bot_y);
    }

    c.set_global_alpha(1.0);
    c.restore();
}
