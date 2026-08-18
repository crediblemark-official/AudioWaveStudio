//! 3D Glass Box: Hologram Reactor Core (`glassBoxHologramCore`).
//!
//! Visual Concept:
//! - A 3D Glass Cube container with glassmorphism refraction & bevel highlights.
//! - INSIDE the 3D glass box: a rotating 3D holographic power reactor core spinning
//!   with audio-reactive energy rings and core pulses!

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
    let frame_time = ctx.frame_time;

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

    let holo_cyan   = mix(glow, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.90);
    let holo_gold   = mix(accent, Color::rgba(1.0, 0.85, 0.0, 1.0), 0.85);
    let glass_glint = Color::rgba(1.0, 1.0, 1.0, 0.90);

    // 1. Back Glass Face
    let back_poly = vec![projected[0], projected[1], projected[2], projected[3]];
    c.set_fill(Fill::Solid(mix(s_col, Color::BLACK, 0.75).with_alpha(0.35)));
    c.fill_polygon(&back_poly);

    // 2. INTERNAL 3D HOLOGRAPHIC REACTOR CORE inside 3D Glass Box
    let ring_count = 3;
    for r_i in 0..ring_count {
        let rf = r_i as f32;
        let r_size = (30.0 + rf * 25.0 + be * 15.0) * user_scale;
        let r_rot  = frame_time * (1.2 + rf * 0.5) * if r_i % 2 == 0 { 1.0 } else { -1.0 };

        let r_col = if r_i % 2 == 0 { holo_cyan } else { holo_gold };

        c.set_stroke(Fill::Solid(r_col.with_alpha(0.80 + be * 0.15)));
        c.set_line_width((2.4 + bs * 1.2) * user_scale);
        c.set_shadow(r_col, (14.0 + bs * 8.0) * user_scale);

        // Draw rotated ellipse ring
        let steps = 36;
        let mut ring_pts: Vec<(f32, f32)> = Vec::with_capacity(steps + 1);
        for s in 0..=steps {
            let t = s as f32 / steps as f32 * TAU;
            let rx_val = t.cos() * r_size;
            let ry_val = t.sin() * r_size * 0.45;

            let (cr, sr) = r_rot.sin_cos();
            let rx_rot = rx_val * cr - ry_val * sr;
            let ry_rot = rx_val * sr + ry_val * cr;

            ring_pts.push((cx + rx_rot, cy + ry_rot));
        }
        c.stroke_polyline(&ring_pts);
    }

    // Glowing Central Hologram Core Disc
    let core_r = (14.0 + be * 10.0 + bs * 8.0) * user_scale;
    c.set_fill(Fill::Solid(glass_glint));
    c.set_shadow(holo_cyan, (20.0 + bs * 12.0) * user_scale);
    c.fill_circle(cx, cy, core_r);

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

    let _ = (p_col, sensitivity);
    c.set_global_alpha(1.0);
    c.restore();
}
