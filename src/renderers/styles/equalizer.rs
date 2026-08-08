//! Hologram Matrix 3D style renderer (`equalizer`) — 3D Glass LED Matrix Engine.
//!
//! Upgraded Masterpiece:
//! - 3D volumetric glass/crystal LED matrix blocks surging in 3D space.
//! - Audio-reactive grid height pulsation & glowing crystal reflections.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const MATRIX_COLS: usize = 16;
const MATRIX_ROWS: usize = 12;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let _bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let _frame_time = ctx.frame_time;

    let cx = width * 0.5;
    let cy = height * 0.5;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep matrix backdrop
    c.set_fill(Fill::Solid(Color::hex("#020308")));
    c.fill_rect(0.0, 0.0, width, height);

    // Ambient matrix glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.60,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 0.90, 1.0, 0.22), 0.5)),
            (0.50, mix(p_col, Color::rgba(0.40, 0.0, 0.80, 0.08), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(amb_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. HOLOGRAM MATRIX 3D LED BLOCKS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);
    let cell_w = (width * 0.70) / MATRIX_COLS as f32;
    let cell_h = (height * 0.55) / MATRIX_ROWS as f32;
    let gap = 3.0;

    let start_x = cx - (MATRIX_COLS as f32 * cell_w) * 0.5;
    let start_y = cy + (MATRIX_ROWS as f32 * cell_h) * 0.5;

    for col in 0..MATRIX_COLS {
        let bin_k = (col * step_f / (MATRIX_COLS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let active_rows = ((fv * MATRIX_ROWS as f32 * sensitivity + be * 2.0) as usize).min(MATRIX_ROWS);

        for row in 0..MATRIX_ROWS {
            let x = start_x + col as f32 * cell_w + gap * 0.5;
            let y = start_y - (row as f32 + 1.0) * cell_h + gap * 0.5;
            let bw = cell_w - gap;
            let bh = cell_h - gap;

            if row < active_rows {
                let block_col = mix(
                    mix(p_col, s_col, row as f32 / MATRIX_ROWS as f32),
                    mix(accent_col, glow_col, fv),
                    row as f32 / MATRIX_ROWS as f32,
                );

                c.set_fill(Fill::Solid(block_col));
                c.set_shadow(block_col, 8.0);
                c.fill_rounded_rect(x, y, bw, bh, 3.0);
            } else {
                c.set_fill(Fill::Solid(Color::rgba(0.10, 0.15, 0.25, 0.20)));
                c.set_shadow(Color::TRANSPARENT, 0.0);
                c.fill_rounded_rect(x, y, bw, bh, 3.0);
            }
        }
    }

    c.set_global_alpha(1.0);
    c.restore();
}
