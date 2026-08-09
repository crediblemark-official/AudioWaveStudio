//! Waveform Stepped Arcade style renderer (`waveformSteppedArcade`).
//!
//! 8-Bit Pixelated Stepped Arcade Waveform:
//! - Quantized step blocks creating a retro 8-bit arcade aesthetic.
//! - Flashing peak pixel blocks and arcade glow effects.

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
    let bs = ctx.beat_strength.clamp(0.0, 1.0);
    let pcm = ctx.time_data;

    let cx = width  * 0.5 + pos_offset_x;
    let cy = height * 0.50 + pos_offset_y;

    let span_w = width * 0.88 * user_scale;
    let start_x = cx - span_w * 0.5;
    let max_amp = height * 0.38 * user_scale;

    let step_cols = 48usize;
    let col_w = span_w / step_cols as f32;
    let pixel_h = (10.0 * user_scale).max(4.0);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#05020c")));
//     c.fill_rect(0.0, 0.0, width, height);

    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, width * 0.65,
        &[
            (0.00, mix(glow, Color::rgba(0.0, 1.0, 0.5, 0.3), 0.5).with_alpha(0.22 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    let sample_step = (pcm.len() / step_cols).max(1);

    for i in 0..step_cols {
        let t = i as f32 / step_cols as f32;
        let x = start_x + i as f32 * col_w;

        let sample_idx = (i * sample_step).min(pcm.len().saturating_sub(1));
        let val = (pcm[sample_idx] as f32 / 128.0 - 1.0) * sensitivity;

        // 8-bit Quantized height
        let raw_h = val * max_amp;
        let quant_steps = (raw_h / pixel_h).round();
        let quant_y = cy + quant_steps * pixel_h;

        let px_w = col_w * 0.88;

        let col_color = mix(
            mix(p_col, Color::rgba(0.0, 0.95, 0.45, 1.0), t),
            mix(acc_col, Color::rgba(1.0, 0.85, 0.1, 1.0), val.abs()),
            val.abs(),
        );

        // Pixel Step Block
        let block_top = cy.min(quant_y);
        let block_bot = cy.max(quant_y);
        let block_h   = (block_bot - block_top).max(pixel_h);

        c.set_fill(Fill::Solid(col_color.with_alpha(0.85)));
        c.set_shadow(col_color, (6.0 + val.abs() * 6.0) * user_scale);
        c.fill_rect(x, block_top, px_w, block_h);

        // Flashing Peak Pixel Block
        let peak_y = if quant_y < cy { block_top } else { block_bot - pixel_h };
        let flash_col = mix(col_color, Color::rgba(1.0, 1.0, 1.0, 0.98), bs * 0.6 + 0.3);
        c.set_fill(Fill::Solid(flash_col));
        c.set_shadow(flash_col, 10.0 * user_scale);
        c.fill_rect(x, peak_y, px_w, pixel_h);
    }

    let _ = (s_col, acc_col);

    c.set_global_alpha(1.0);
    c.restore();
}
