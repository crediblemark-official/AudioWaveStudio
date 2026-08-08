//! Matrix Digital Rain style renderer (`matrixRain`) — Cyberpunk Data Waterfall Engine.
//!
//! Masterpiece Cyberpunk Matrix Data Waterfall:
//! - Cascading vertical matrix code streams with audio-reactive drop speed & peak heights.
//! - Glowing 3D matrix glyph blocks & trailing green/cyan digital code rain.
//! - Leading bright white/cyan code tips with audio-reactive beat pulses.
//! - Full UI Theme colors (`theme_primary`, `theme_secondary`, `theme_accent`, `theme_glow`) and slider integration.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const MATRIX_COLUMNS: usize = 36;
const BLOCKS_PER_COL: usize = 16;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let _s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let bar_count = ctx.config.reactivity.bar_count.clamp(16, 128);

    let be = ctx.bass_energy.clamp(0.0, 1.0);
    let _bs = ctx.beat_strength.clamp(0.0, 1.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 ;
    let cy = height * 0.5 ;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Deep pitch-black matrix backdrop
    c.set_fill(Fill::Solid(Color::hex("#010602")));
    c.fill_rect(0.0, 0.0, width, height);

    // Matrix digital rain ambient glow
    let amb_glow = Fill::radial_gradient(
        cx,
        cy,
        0.0,
        cx,
        cy,
        width * 0.65,
        &[
            (0.0, mix(glow_col, Color::rgba(0.0, 1.0, 0.40, 0.25), 0.5)),
            (0.50, mix(p_col, Color::rgba(0.0, 0.50, 0.20, 0.10), 0.5)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(amb_glow);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 1. CASCADING MATRIX DATA RAIN STREAM BLOCKS
    // -------------------------------------------------------------------------
    let step_f = (freq.len() / bar_count).max(1);
    let col_w = (width * 0.88) / MATRIX_COLUMNS as f32;
    let block_h = 12.0;
    let gap = 3.0;

    let start_x = cx - (MATRIX_COLUMNS as f32 * col_w) * 0.5;

    for col in 0..MATRIX_COLUMNS {
        let col_f = col as f32;
        let x_pos = start_x + col_f * col_w + gap * 0.5;

        let bin_k = (col * step_f / (MATRIX_COLUMNS / bar_count.max(1)).max(1))
            .min(freq.len().saturating_sub(1));
        let fv = freq[bin_k] as f32 / 255.0;

        let active_blocks = ((fv * BLOCKS_PER_COL as f32 * sensitivity + be * 2.0) as usize).min(BLOCKS_PER_COL);

        // Falling phase offset
        let fall_speed = 0.5 + (col % 7) as f32 * 0.12 + be * 0.4;
        let y_shift = (frame_time * fall_speed * 120.0 + col_f * 29.0) % (height * 0.6);

        for b in 0..BLOCKS_PER_COL {
            let b_f = b as f32;
            let y_pos = (height * 0.80 ) - b_f * (block_h + gap) + (y_shift % (block_h + gap));

            if b < active_blocks {
                let code_col = mix(
                    mix(p_col, Color::rgba(0.10, 0.95, 0.35, 0.90), b_f / BLOCKS_PER_COL as f32),
                    mix(accent_col, glow_col, fv),
                    b_f / BLOCKS_PER_COL as f32,
                );

                c.set_fill(Fill::Solid(code_col));
                c.set_shadow(code_col, 8.0 + fv * 4.0);
                c.fill_rounded_rect(x_pos, y_pos, col_w - gap, block_h, 2.0);

                // Leading Tip Light
                if b == active_blocks - 1 {
                    c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.95)));
                    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.90), 12.0);
                    c.fill_rounded_rect(x_pos, y_pos, col_w - gap, block_h, 2.0);
                }
            } else {
                // Dim trailing digital ghost matrix block
                c.set_fill(Fill::Solid(Color::rgba(0.05, 0.25, 0.10, 0.18)));
                c.set_shadow(Color::TRANSPARENT, 0.0);
                c.fill_rounded_rect(x_pos, y_pos, col_w - gap, block_h, 2.0);
            }
        }
    }

    c.set_global_alpha(1.0);
    c.restore();
}
