//! 3D Glass Box: Neon Spectrum Matrix (`glassBoxNeonSpectrum`).
//!
//! Visual Concept:
//! - A 3D Glass Cube container with glassmorphism refraction & bevel highlights.
//! - INSIDE the 3D glass box: a 4x4 3D isometric matrix of glowing neon equalizer spectrum pillars
//!   pulsating up and down to audio frequencies with glass floor reflections!

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
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

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
    let rx = 0.30 + ctx.beat_strength * 0.08;
    let ry = 0.40 + ctx.beat_strength * 0.08;

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

    let neon_magenta = mix(p_col, Color::rgba(1.0, 0.05, 0.55, 1.0), 0.85);
    let neon_cyan    = mix(glow, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.90);
    let glass_glint  = Color::rgba(1.0, 1.0, 1.0, 0.90);

    // 1. Back Glass Face
    let back_poly = vec![projected[0], projected[1], projected[2], projected[3]];
    c.set_fill(Fill::Solid(mix(s_col, Color::BLACK, 0.7).with_alpha(0.35)));
    c.fill_polygon(&back_poly);

    // 2. INTERNAL 4x4 3D NEON SPECTRUM MATRIX PILLARS inside Glass Box
    let grid_cols = 4;
    let grid_rows = 4;
    let cell_w = box_size * 1.2 / grid_cols as f32;
    let step = (freq.len() / (grid_cols * grid_rows)).max(1);

    for r in 0..grid_rows {
        for col in 0..grid_cols {
            let idx = r * grid_cols + col;
            let f_val = (freq.get(idx * step).copied().unwrap_or(0) as f32 / 255.0) * sensitivity;
            let height_val = (15.0 + f_val * 70.0 + be * 15.0) * user_scale;

            let bx = -box_size * 0.6 + col as f32 * cell_w;
            let bz = -box_size * 0.6 + r as f32 * cell_w;

            // Pillar 3D Vertices
            let px1 = bx * cos_y + bz * sin_y;
            let pz1 = -bx * sin_y + bz * cos_y;
            let py2 = (box_size * 0.7) * cos_x - pz1 * sin_x;
            let pz2 = (box_size * 0.7) * sin_x + pz1 * cos_x;
            let scale1 = 400.0 / (400.0 + pz2);

            let top_y2 = (box_size * 0.7 - height_val) * cos_x - pz1 * sin_x;
            let scale2 = 400.0 / (400.0 + pz2);

            let base_pt = (cx + px1 * scale1, cy + py2 * scale1);
            let top_pt  = (cx + px1 * scale2, cy + top_y2 * scale2);

            let bar_col = if (r + col) % 2 == 0 { neon_cyan } else { neon_magenta };
            let pillar_w = 8.0 * user_scale;

            // Render Pillar Line & Glowing Top Disc
            c.set_stroke(Fill::Solid(bar_col.with_alpha(0.85)));
            c.set_line_width(pillar_w);
            c.set_shadow(bar_col, (12.0 + bs * 8.0) * user_scale);
            c.stroke_line(base_pt.0, base_pt.1, top_pt.0, top_pt.1);

            c.set_fill(Fill::Solid(glass_glint));
            c.fill_circle(top_pt.0, top_pt.1, (4.0 + bs * 2.0) * user_scale);
        }
    }

    // 3. 3D Glass Box Faces (Smooth Specular Light Glare Gradient)
    let glass_tint_color = mix(glow, Color::rgba(0.75, 0.92, 1.0, 1.0), 0.70);
    let (p4, p5, p6, p7) = (projected[4], projected[5], projected[6], projected[7]);

    // Top Glass Face
    let top_poly = vec![projected[3], projected[2], projected[6], projected[7]];
    c.set_fill(Fill::linear_gradient(
        projected[3].0, projected[3].1,
        projected[6].0, projected[6].1,
        &[
            (0.00, Color::rgba(1.0, 1.0, 1.0, 0.35 + be * 0.10)),
            (0.60, glass_tint_color.with_alpha(0.18)),
            (1.00, glass_tint_color.with_alpha(0.08)),
        ],
    ));
    c.fill_polygon(&top_poly);

    // Right Glass Face
    let right_poly = vec![projected[1], projected[2], projected[6], projected[5]];
    c.set_fill(Fill::linear_gradient(
        projected[1].0, projected[1].1,
        projected[6].0, projected[6].1,
        &[
            (0.00, glass_tint_color.with_alpha(0.12)),
            (1.00, Color::rgba(1.0, 1.0, 1.0, 0.22 + be * 0.08)),
        ],
    ));
    c.fill_polygon(&right_poly);

    // Front Glass Face (Smooth 3D Glass Light Glare Gradient)
    let front_poly = vec![p4, p5, p6, p7];
    c.set_fill(Fill::linear_gradient(
        p4.0, p4.1,
        p6.0, p6.1,
        &[
            (0.00, Color::rgba(1.0, 1.0, 1.0, 0.38 + be * 0.12)),
            (0.35, Color::rgba(1.0, 1.0, 1.0, 0.16 + be * 0.05)),
            (0.70, glass_tint_color.with_alpha(0.12)),
            (1.00, glass_tint_color.with_alpha(0.22 + be * 0.06)),
        ],
    ));
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

    let _ = accent;
    c.set_global_alpha(1.0);
    c.restore();
}
