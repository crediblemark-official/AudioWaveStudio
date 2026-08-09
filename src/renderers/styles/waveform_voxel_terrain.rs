//! Waveform Voxel Terrain style renderer (`waveformVoxelTerrain`).
//!
//! 3D Voxel Cube Waveform Mesh:
//! - Grid of 3D voxel blocks modulating their heights based on time-domain waveform data.
//! - Perspective 3D layout with glowing voxel top caps and shaded walls.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const VOXEL_COLS: usize = 32;
const VOXEL_ROWS: usize = 6;

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
    let cy = height * 0.55 + pos_offset_y;

    let grid_w = width * 0.85 * user_scale;
    let cell_w = grid_w / VOXEL_COLS as f32;
    let start_x = cx - grid_w * 0.5;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // Background
//     c.set_fill(Fill::Solid(Color::hex("#020108")));
//     c.fill_rect(0.0, 0.0, width, height);

    // Horizon bloom
    let bg_glow = Fill::radial_gradient(
        cx, cy, 0.0, cx, cy, width * 0.65,
        &[
            (0.00, mix(glow, Color::rgba(0.0, 0.8, 1.0, 0.3), 0.5).with_alpha(0.25 + be * 0.15)),
            (0.50, mix(p_col, s_col, 0.5).with_alpha(0.08)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_glow);
//     c.fill_rect(0.0, 0.0, width, height);

    let sample_step = (pcm.len() / VOXEL_COLS).max(1);

    // Render back rows first (depth order)
    for row in (0..VOXEL_ROWS).rev() {
        let r_t = row as f32 / VOXEL_ROWS as f32;
        let row_z_scale = 0.55 + (1.0 - r_t) * 0.45; // Far rows are smaller
        let row_y_offset = (r_t - 0.5) * 60.0 * user_scale;

        for col in 0..VOXEL_COLS {
            let col_t = col as f32 / VOXEL_COLS as f32;
            let sample_idx = (col * sample_step).min(pcm.len().saturating_sub(1));
            let val = ((pcm[sample_idx] as f32 / 128.0 - 1.0).abs() * sensitivity * 1.5).clamp(0.05, 1.2);

            let voxel_w = cell_w * 0.82 * row_z_scale;
            let voxel_h = (val * height * 0.25 * row_z_scale + 8.0 * user_scale).clamp(6.0, height * 0.4);

            let vx = start_x + col as f32 * cell_w + (1.0 - row_z_scale) * cell_w * 0.5;
            let vy = cy + row_y_offset - voxel_h;

            let voxel_col = mix(
                mix(p_col, acc_col, col_t),
                mix(acc_col, glow, val),
                val * 0.6 + (1.0 - r_t) * 0.2,
            );

            // Front face fill
            c.set_fill(Fill::Solid(voxel_col.with_alpha(0.85 - r_t * 0.3)));
            c.set_shadow(voxel_col, (4.0 + val * 6.0) * user_scale);
            c.fill_rect(vx, vy, voxel_w, voxel_h);

            // Glowing top cap face
            c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.90)));
            c.set_shadow(voxel_col, 8.0 * user_scale);
            c.fill_rect(vx, vy, voxel_w, (3.0 * user_scale).max(2.0));
        }
    }

    c.set_global_alpha(1.0);
    c.restore();
}
