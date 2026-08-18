//! 3D Glass Box: Bio Jellyfish Chamber (`glassBoxBioluminescentJellyfish`).
//!
//! Visual Concept:
//! - A 3D Glass Aquarium Cube container with glassmorphism refraction & bevel highlights.
//! - INSIDE the 3D glass box: a glowing bioluminescent jellyfish with waving fluid bio-tentacles
//!   drifting inside the glass volume to audio frequencies!

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
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
    let rx = 0.25 + ctx.beat_strength * 0.08;
    let ry = 0.35 + ctx.beat_strength * 0.08;

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

    let bio_cyan    = mix(glow, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.90);
    let bio_magenta = mix(accent, Color::rgba(1.0, 0.10, 0.70, 1.0), 0.85);
    let glass_glint = Color::rgba(1.0, 1.0, 1.0, 0.90);

    // 1. Back Glass Face (Deep Sea Water Shading)
    let back_poly = vec![projected[0], projected[1], projected[2], projected[3]];
    c.set_fill(Fill::Solid(mix(s_col, Color::hex("#020815"), 0.8).with_alpha(0.38)));
    c.fill_polygon(&back_poly);

    // 2. INTERNAL BIOLUMINESCENT JELLYFISH inside 3D Glass Aquarium
    let bell_r = box_size * 0.40 * (1.0 + bs * 0.08);
    let bell_fill = Fill::radial_gradient(
        cx - bell_r * 0.2, cy - bell_r * 0.2, 0.0,
        cx, cy, bell_r * 1.5,
        &[
            (0.00, glass_glint.with_alpha(0.85)),
            (0.40, bio_cyan.with_alpha(0.60 + be * 0.15)),
            (0.80, bio_magenta.with_alpha(0.35)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bell_fill);
    c.set_shadow(bio_cyan, (18.0 + bs * 10.0) * user_scale);
    c.fill_circle(cx, cy, bell_r * 1.10);

    // 24 Wavy Bio-Tentacles inside Glass Container
    let tentacle_count = 24;
    let step = (freq.len() / tentacle_count).max(1);

    for i in 0..tentacle_count {
        let t = i as f32 / tentacle_count as f32;
        let angle = t * TAU + frame_time * 0.10;

        let fv = smooth_ring_bin(freq, step, i, tentacle_count);
        let val = (fv * sensitivity * 1.1 + be * 0.30 + bs * 0.20).clamp(0.05, 2.2);

        let t_len = (20.0 + val * 65.0) * user_scale;
        let (cos_a, sin_a) = angle.sin_cos();

        let sway = (frame_time * 2.0 + t * 6.0).sin() * 0.25;
        let (cos_m, sin_m) = (angle + sway).sin_cos();
        let (cos_t, sin_t) = (angle + sway * 1.5).sin_cos();

        let p0    = (cx + cos_a * bell_r, cy + sin_a * bell_r);
        let p_mid = (cx + cos_m * (bell_r + t_len * 0.5), cy + sin_m * (bell_r + t_len * 0.5));
        let p_tip = (cx + cos_t * (bell_r + t_len), cy + sin_t * (bell_r + t_len));

        let tentacle_pts = GpuCanvas::sample_quadratic(p0, p_mid, p_tip, 6);
        let t_col = if i % 2 == 0 { bio_cyan } else { bio_magenta };

        c.set_stroke(Fill::Solid(t_col.with_alpha(0.70 + val * 0.20)));
        c.set_line_width((2.0 + val * 1.2) * user_scale);
        c.set_shadow(t_col, 10.0 * user_scale);
        c.stroke_polyline(&tentacle_pts);
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

    let _ = p_col;
    c.set_global_alpha(1.0);
    c.restore();
}
