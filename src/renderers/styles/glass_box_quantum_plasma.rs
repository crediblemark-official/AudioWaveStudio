//! 3D Glass Box: Quantum Plasma (`glassBoxQuantumPlasma`).
//!
//! Visual Concept:
//! - A rotating photorealistic 3D Glass Cube container with glassmorphism refraction & bevel highlights.
//! - INSIDE the 3D glass box: a glowing Quantum Plasma energy orb morphs radially to audio spectrum,
//!   emitting electric plasma arcs that bounce off the interior 3D glass walls!

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

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
        // Rotate Y
        let x1 = vx * cos_y + vz * sin_y;
        let z1 = -vx * sin_y + vz * cos_y;
        // Rotate X
        let y2 = vy * cos_x - z1 * sin_x;
        let z2 = vy * sin_x + z1 * cos_x;

        let scale = 400.0 / (400.0 + z2);
        projected.push((cx + x1 * scale, cy + y2 * scale));
    }

    // Colors
    let plasma_cyan   = mix(glow, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.85);
    let plasma_violet = mix(accent, Color::rgba(0.65, 0.05, 1.0, 1.0), 0.85);
    let glass_glint   = Color::rgba(1.0, 1.0, 1.0, 0.90);

    // 1. Back Glass Box Face (Dark Depth Shading)
    let back_poly = vec![projected[0], projected[1], projected[2], projected[3]];
    c.set_fill(Fill::Solid(mix(p_col, Color::BLACK, 0.7).with_alpha(0.35)));
    c.fill_polygon(&back_poly);

    // 2. INTERNAL AUDIO EFFECT: Swirling Quantum Plasma Core inside 3D Glass Box
    let inner_r = box_size * 0.55;
    let step = (freq.len() / 64).max(1);
    let mut plasma_pts: Vec<(f32, f32)> = Vec::with_capacity(65);

    for i in 0..64 {
        let t = i as f32 / 64.0;
        let angle = t * TAU + frame_time * 1.2;
        let fv = smooth_ring_bin(freq, step, i, 64);
        let val = (fv * sensitivity * 1.1 + be * 0.35 + bs * 0.25).clamp(0.05, 2.4);

        let wave = (angle * 4.0 + frame_time * 3.0).sin() * (12.0 * user_scale);
        let r_curr = inner_r + (val * 35.0 * user_scale) + wave;
        let (cos_a, sin_a) = angle.sin_cos();
        plasma_pts.push((cx + cos_a * r_curr, cy + sin_a * r_curr));
    }
    plasma_pts.push(plasma_pts[0]);

    let plasma_fill = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, inner_r * 1.8,
        &[
            (0.00, glass_glint.with_alpha(0.90)),
            (0.40, plasma_cyan.with_alpha(0.80 + be * 0.15)),
            (0.80, plasma_violet.with_alpha(0.45)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(plasma_fill);
    c.set_shadow(plasma_cyan, (22.0 + bs * 14.0) * user_scale);
    fill_radial_polygon(c, cx, cy, &plasma_pts);

    c.set_stroke(Fill::Solid(mix(plasma_cyan, glass_glint, 0.70).with_alpha(0.90)));
    c.set_line_width((2.6 + bs * 1.5) * user_scale);
    c.stroke_polyline(&plasma_pts);

    // Electric Arcs Bouncing Inside 3D Glass Box
    for arc_i in 0..4 {
        let af = arc_i as f32;
        let ang = (af / 4.0) * TAU + frame_time * 2.0;
        let (ca, sa) = ang.sin_cos();
        let target_p = projected[arc_i % 4];
        let mid_p = (cx + ca * inner_r * 1.2, cy + sa * inner_r * 1.2);
        let pts = GpuCanvas::sample_quadratic((cx, cy), mid_p, target_p, 6);

        c.set_stroke(Fill::Solid(plasma_cyan.with_alpha(0.70 + be * 0.25)));
        c.set_line_width((2.0 + bs * 1.2) * user_scale);
        c.set_shadow(plasma_cyan, 12.0 * user_scale);
        c.stroke_polyline(&pts);
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

    // 12 3D Glass Box Edges
    let edges = [
        (0,1), (1,2), (2,3), (3,0), // Back
        (4,5), (5,6), (6,7), (7,4), // Front
        (0,4), (1,5), (2,6), (3,7), // Connecting
    ];

    for &(e1, e2) in &edges {
        let p1 = projected[e1];
        let p2 = projected[e2];
        c.set_stroke(Fill::Solid(mix(glass_tint_color, glass_glint, 0.80).with_alpha(0.90)));
        c.set_line_width((2.4 + bs * 1.2) * user_scale);
        c.set_shadow(glass_tint_color, (12.0 + be * 6.0) * user_scale);
        c.stroke_line(p1.0, p1.1, p2.0, p2.1);
    }

    // Glass Reflection Diagonal Glint Line across front face
    let glint_p1 = projected[4];
    let glint_p2 = projected[6];
    c.set_stroke(Fill::Solid(glass_glint.with_alpha(0.65 + be * 0.20)));
    c.set_line_width((3.2 + bs * 1.5) * user_scale);
    c.stroke_line(glint_p1.0, glint_p1.1, glint_p2.0, glint_p2.1);

    let _ = s_col;
    c.set_global_alpha(1.0);
    c.restore();
}
