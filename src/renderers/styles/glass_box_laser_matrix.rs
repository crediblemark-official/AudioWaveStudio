//! 3D Glass Box: Laser Chamber Matrix (`glassBoxLaserMatrix`).
//!
//! Visual Concept:
//! - A 3D Glass Chamber Cube container with glassmorphism refraction & bevel highlights.
//! - INSIDE the 3D glass box: crisscrossing audio-reactive neon laser beams bouncing off interior glass mirrors!

use std::f32::consts::TAU;

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

    let laser_cyan    = mix(glow, Color::rgba(0.0, 1.0, 0.8, 1.0), 0.90);
    let laser_magenta = mix(accent, Color::rgba(1.0, 0.0, 0.6, 1.0), 0.90);
    let glass_glint   = Color::rgba(1.0, 1.0, 1.0, 0.90);

    // 1. Back Glass Face
    let back_poly = vec![projected[0], projected[1], projected[2], projected[3]];
    c.set_fill(Fill::Solid(mix(p_col, Color::BLACK, 0.8).with_alpha(0.35)));
    c.fill_polygon(&back_poly);

    // 2. INTERNAL CRISSCROSSING NEON LASER BEAMS inside 3D Glass Box
    let laser_count = 8;
    let step = (freq.len() / laser_count).max(1);

    for i in 0..laser_count {
        let t = i as f32 / laser_count as f32;
        let angle = t * TAU + frame_time * 1.5;

        let fv = (freq.get(i * step).copied().unwrap_or(0) as f32 / 255.0) * sensitivity;
        let val = (fv * 1.2 + be * 0.35).clamp(0.1, 2.5);

        let p_start = projected[i % 8];
        let p_end   = projected[(i + 4) % 8];

        let l_col = if i % 2 == 0 { laser_cyan } else { laser_magenta };
        let l_w = (1.8 + val * 2.0 + bs * 1.2) * user_scale;

        c.set_stroke(Fill::Solid(l_col.with_alpha(0.85)));
        c.set_line_width(l_w);
        c.set_shadow(l_col, (16.0 + val * 10.0) * user_scale);
        c.stroke_line(p_start.0, p_start.1, p_end.0, p_end.1);

        // Laser reflection spark dot
        let (ca, sa) = angle.sin_cos();
        let spark_x = cx + ca * box_size * 0.5;
        let spark_y = cy + sa * box_size * 0.5;
        c.set_fill(Fill::Solid(glass_glint));
        c.fill_circle(spark_x, spark_y, (3.5 + val * 2.0) * user_scale);
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

    let _ = s_col;
    c.set_global_alpha(1.0);
    c.restore();
}
