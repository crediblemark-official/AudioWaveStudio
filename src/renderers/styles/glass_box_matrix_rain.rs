//! 3D Glass Box: Cyber Matrix Rain (`glassBoxMatrixRain`).
//!
//! Visual Concept:
//! - A 3D Glass Cube container with glassmorphism refraction & bevel highlights.
//! - INSIDE the 3D glass box: cascading 3D neon digital rain & cyber code particles falling down
//!   through the glass cube volume driven by audio frequencies!

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
    let frame_time = ctx.frame_time;

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

    let matrix_green = Color::rgba(0.20, 1.0, 0.35, 1.0);
    let matrix_cyan  = mix(glow, Color::rgba(0.0, 0.95, 0.8, 1.0), 0.85);
    let glass_glint  = Color::rgba(1.0, 1.0, 1.0, 0.90);

    // 1. Back Glass Face
    let back_poly = vec![projected[0], projected[1], projected[2], projected[3]];
    c.set_fill(Fill::Solid(mix(p_col, Color::hex("#011005"), 0.85).with_alpha(0.35)));
    c.fill_polygon(&back_poly);

    // 2. INTERNAL CASCADING NEON MATRIX DIGITAL RAIN STREAMS inside 3D Glass Box
    let stream_count = 12;
    let step = (freq.len() / stream_count).max(1);

    for s_i in 0..stream_count {
        let sf = s_i as f32;
        let sx = -box_size * 0.7 + (sf / stream_count as f32) * box_size * 1.4;

        let fv = (freq.get(s_i * step).copied().unwrap_or(0) as f32 / 255.0) * sensitivity;
        let speed = (80.0 + fv * 120.0 + be * 30.0) * user_scale;

        let sy_start = ((frame_time * speed + sf * 45.0) % (box_size * 1.4)) - box_size * 0.7;
        let sy_end   = sy_start + (25.0 + fv * 35.0) * user_scale;

        // Project 3D stream segment
        let sz = (sf * 17.3).sin() * box_size * 0.5;

        let x1 = sx * cos_y + sz * sin_y;
        let z1 = -sx * sin_y + sz * cos_y;

        let y1_top = sy_start * cos_x - z1 * sin_x;
        let y2_bot = sy_end * cos_x - z1 * sin_x;

        let scale = 400.0 / (400.0 + (sy_start * sin_x + z1 * cos_x));

        let p_top = (cx + x1 * scale, cy + y1_top * scale);
        let p_bot = (cx + x1 * scale, cy + y2_bot * scale);

        let stream_col = if s_i % 2 == 0 { matrix_green } else { matrix_cyan };

        c.set_stroke(Fill::Solid(stream_col.with_alpha(0.85)));
        c.set_line_width((2.2 + bs * 1.2) * user_scale);
        c.set_shadow(stream_col, (12.0 + bs * 8.0) * user_scale);
        c.stroke_line(p_top.0, p_top.1, p_bot.0, p_bot.1);

        // Leading Matrix Code Spark
        c.set_fill(Fill::Solid(glass_glint));
        c.fill_circle(p_bot.0, p_bot.1, (3.2 + bs * 1.5) * user_scale);
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

    let _ = (s_col, accent);
    c.set_global_alpha(1.0);
    c.restore();
}
