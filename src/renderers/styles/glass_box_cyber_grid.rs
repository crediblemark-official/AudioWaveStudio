//! 3D Glass Box: Cyber Grid Waveform (`glassBoxCyberGrid`).
//!
//! Visual Concept:
//! - A 3D Glass Cube container with glassmorphism refraction & bevel highlights.
//! - INSIDE the 3D glass box: a glowing 3D wireframe cyber grid terrain undulating to audio spectrum waves.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col  = theme_primary(theme);
    let s_col  = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let bass_mult    = ctx.config.reactivity.bass_multiplier;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = ctx.config.position_y * height * 0.5;

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let _frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let beat_scale = 1.0 + ctx.beat_strength * 0.35;
    let box_size = 140.0 * (width.min(height) / 500.0) * user_scale * beat_scale;

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 3D Glass Cube Projection (beat-driven pulse, no continuous rotation)
    let rx = 0.35 + ctx.beat_strength * 0.08;
    let ry = 0.45 + ctx.beat_strength * 0.08;

    let (sin_x, cos_x) = rx.sin_cos();
    let (sin_y, cos_y) = ry.sin_cos();

    let vertices_3d = [
        (-box_size, -box_size, -box_size),
        ( box_size, -box_size, -box_size),
        ( box_size,  box_size, -box_size),
        (-box_size,  box_size, -box_size),
        (-box_size, -box_size,  box_size),
        ( box_size, -box_size,  box_size),
        ( box_size,  box_size,  box_size),
        (-box_size,  box_size,  box_size),
    ];

    let mut projected: Vec<(f32, f32)> = Vec::with_capacity(8);
    for &(vx, vy, vz) in &vertices_3d {
        let x1 = vx * cos_y + vz * sin_y;
        let z1 = -vx * sin_y + vz * cos_y;
        let y2 = vy * cos_x - z1 * sin_x;
        let z2 = vy * sin_x + z1 * cos_x;
        let scale = 400.0 / (400.0 + z2);
        projected.push((cx + x1 * scale, cy + y2 * scale));
    }

    let grid_cyan   = mix(glow, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.90);
    let grid_magenta = mix(accent, Color::rgba(1.0, 0.05, 0.65, 1.0), 0.85);
    let glass_glint = Color::rgba(1.0, 1.0, 1.0, 0.90);

    // 1. Back Glass Face
    let back_poly = vec![projected[0], projected[1], projected[2], projected[3]];
    c.set_fill(Fill::Solid(mix(p_col, Color::BLACK, 0.75).with_alpha(0.35)));
    c.fill_polygon(&back_poly);

    // 2. INTERNAL 3D CYBER GRID TERRAIN inside Glass Box
    let rows = 8;
    let cols = 8;
    let step = (freq.len() / (rows * cols)).max(1);

    for r in 0..rows {
        let mut row_pts: Vec<(f32, f32)> = Vec::with_capacity(cols);
        for c_idx in 0..cols {
            let gx = -box_size * 0.7 + (c_idx as f32 / (cols - 1) as f32) * box_size * 1.4;
            let gz = -box_size * 0.7 + (r as f32 / (rows - 1) as f32) * box_size * 1.4;

            let f_val = (freq.get((r * cols + c_idx) * step).copied().unwrap_or(0) as f32 / 255.0) * sensitivity;
            let wave_y = (f_val * 45.0 + be * 12.0) * user_scale;
            let gy = box_size * 0.3 - wave_y;

            let x1 = gx * cos_y + gz * sin_y;
            let z1 = -gx * sin_y + gz * cos_y;
            let y2 = gy * cos_x - z1 * sin_x;
            let z2 = gy * sin_x + z1 * cos_x;

            let scale = 400.0 / (400.0 + z2);
            row_pts.push((cx + x1 * scale, cy + y2 * scale));
        }

        let line_col = mix(grid_cyan, grid_magenta, r as f32 / rows as f32);
        c.set_stroke(Fill::Solid(line_col.with_alpha(0.80)));
        c.set_line_width((1.8 + bs * 1.0) * user_scale);
        c.set_shadow(line_col, (10.0 + bs * 6.0) * user_scale);
        c.stroke_polyline(&row_pts);
    }

    // 3. 3D Glass Box Faces & Refraction (Realistic Glass Tint Color)
    let glass_tint_color = mix(glow, Color::rgba(0.75, 0.92, 1.0, 1.0), 0.70);

    // Top Glass Face
    let top_poly = vec![projected[3], projected[2], projected[6], projected[7]];
    c.set_fill(Fill::Solid(glass_tint_color.with_alpha(0.20 + be * 0.06)));
    c.fill_polygon(&top_poly);

    // Right Glass Face
    let right_poly = vec![projected[1], projected[2], projected[6], projected[5]];
    c.set_fill(Fill::Solid(glass_tint_color.with_alpha(0.15 + be * 0.05)));
    c.fill_polygon(&right_poly);

    // Front Glass Face
    let front_poly = vec![projected[4], projected[5], projected[6], projected[7]];
    c.set_fill(Fill::Solid(glass_tint_color.with_alpha(0.25 + be * 0.08)));
    c.fill_polygon(&front_poly);

    let edges = [
        (0,1), (1,2), (2,3), (3,0),
        (4,5), (5,6), (6,7), (7,4),
        (0,4), (1,5), (2,6), (3,7),
    ];

    for &(e1, e2) in &edges {
        let p1 = projected[e1];
        let p2 = projected[e2];
        c.set_stroke(Fill::Solid(mix(glass_tint_color, glass_glint, 0.80).with_alpha(0.90)));
        c.set_line_width((2.4 + bs * 1.2) * user_scale);
        c.set_shadow(glass_tint_color, (12.0 + be * 6.0) * user_scale);
        c.stroke_line(p1.0, p1.1, p2.0, p2.1);
    }

    // Diagonal Glass Glint Line
    let glint_p1 = projected[4];
    let glint_p2 = projected[6];
    c.set_stroke(Fill::Solid(glass_glint.with_alpha(0.65 + be * 0.20)));
    c.set_line_width((3.2 + bs * 1.5) * user_scale);
    c.stroke_line(glint_p1.0, glint_p1.1, glint_p2.0, glint_p2.1);

    let _ = s_col;
    c.set_global_alpha(1.0);
    c.restore();
}
