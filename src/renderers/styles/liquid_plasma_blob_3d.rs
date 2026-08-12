//! Liquid 3D Floating Water Spheres style renderer (`liquidPlasmaBlob3D`).
//!
//! Visual Concept:
//! - NASA Zero-Gravity Floating Liquid Water Spheres suspended in 3D space.
//! - 8 Independent 3D floating water orbs orbiting around a central refractive fluid nucleus.
//! - Photorealistic 3D Caustic Glass Highlights, refraction glint caps, & orbital fluid collisions.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const ORBIT_SPHERES: usize = 8;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col      = theme_primary(theme);
    let s_col      = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;

    let be = ctx.bass_energy.clamp(0.0, 3.0);
    let bs = ctx.beat_strength.clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 - pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);
    let max_r   = base_r * 1.70;

    // Palette: Zero-Gravity Glass Water Spheres blended with theme
    let glass_cyan    = mix(glow_col,   Color::rgba(0.0, 0.90, 1.0, 0.85), 0.75);
    let glass_magenta = mix(accent_col, Color::rgba(1.0, 0.20, 0.80, 0.85), 0.75);
    let glass_blue    = mix(p_col,      Color::rgba(0.20, 0.50, 1.0, 0.85), 0.60);
    let spark_white   = mix(s_col,      Color::WHITE, 0.85);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // 1. Ambient Zero-G Space Backdrop Glow — rendered to canvas
    let bg_zero_g = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, max_r * 2.0,
        &[
            (0.0,  mix(glass_cyan, glass_magenta, 0.5).with_alpha(0.32 + be * 0.20)),
            (0.50, glass_blue.with_alpha(0.10)),
            (1.0,  Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_zero_g);
    c.fill_rect(cx - max_r * 2.0, cy - max_r * 2.0, max_r * 4.0, max_r * 4.0);

    // 2. Central refractive fluid nucleus (large pulsing core orb)
    let nucleus_r = (inner_r * 0.85 + be * 14.0 + bs * 8.0) * user_scale.min(1.0).max(1.0);
    let nucleus_r = (nucleus_r).max(inner_r * 0.5).min(inner_r * 1.15);
    let nuc_fill = Fill::radial_gradient(
        cx - nucleus_r * 0.3, cy - nucleus_r * 0.3, nucleus_r * 0.05,
        cx, cy, nucleus_r * 1.4,
        &[
            (0.0,  spark_white),
            (0.25, glass_cyan.with_alpha(0.90)),
            (0.65, mix(glass_cyan, glass_magenta, 0.5).with_alpha(0.70)),
            (1.0,  Color::TRANSPARENT),
        ],
    );
    c.set_fill(nuc_fill);
    c.set_shadow(glass_cyan, (20.0 + bs * 14.0) * user_scale);
    c.fill_circle(cx, cy, nucleus_r * 1.4);

    // 3. 8 Independent 3D Floating Zero-Gravity Water Orbs
    let step = (freq.len() / ORBIT_SPHERES).max(1);

    for s_i in 0..ORBIT_SPHERES {
        let sf = s_i as f32;
        // Two-speed rotation: alternate odd/even orbs orbit opposite directions
        let dir   = if s_i % 2 == 0 { 1.0f32 } else { -1.0 };
        let speed = (0.18 + sf * 0.04) * dir;
        let orb_angle = sf * (TAU / ORBIT_SPHERES as f32) + frame_time * speed;

        let fv  = super::radial_common::full_random_scattered_bin(freq, step, s_i, ORBIT_SPHERES, ctx.beat_count);
        let val = (fv * sensitivity * 1.1 + be * 0.35 + bs * 0.20).clamp(0.10, 2.2);

        // 3D Orbital distance & wobble — each orb has independent radius
        let wobble   = (frame_time * (1.2 + sf * 0.3) + sf).sin() * (8.0 * user_scale);
        let orb_dist = inner_r + (50.0 + val * 55.0 + wobble) * user_scale;
        let (sin_a, cos_a) = orb_angle.sin_cos();

        let ox = cx + cos_a * orb_dist;
        let oy = cy + sin_a * orb_dist * 0.65; // 3D Perspective tilt

        let o_sz  = (14.0 + val * 20.0) * user_scale;
        let o_col = [glass_cyan, glass_magenta, glass_blue, glass_cyan][s_i % 4];
        let o_col = mix(o_col, mix(glass_magenta, glass_blue, sf / ORBIT_SPHERES as f32), 0.35);

        // Thin orbital trail (arc behind the orb)
        c.set_stroke(Fill::Solid(o_col.with_alpha(0.22 + be * 0.08)));
        c.set_line_width((1.0 + val * 0.5) * user_scale);
        c.set_shadow(o_col, (6.0 + be * 4.0) * user_scale);
        let trail_start = orb_angle - dir * 0.55;
        let trail_steps = 16usize;
        let mut trail_pts: Vec<(f32, f32)> = Vec::with_capacity(trail_steps + 1);
        for tk in 0..=trail_steps {
            let ta = trail_start + dir * (tk as f32 / trail_steps as f32) * 0.55;
            let tx = cx + ta.cos() * orb_dist;
            let ty = cy + ta.sin() * orb_dist * 0.65;
            trail_pts.push((tx, ty));
        }
        c.stroke_polyline(&trail_pts);

        // 3D Sphere Glass Gradient Fill
        let orb_fill = Fill::radial_gradient(
            ox - o_sz * 0.32, oy - o_sz * 0.32, o_sz * 0.08,
            ox, oy, o_sz * 1.2,
            &[
                (0.0,  spark_white),
                (0.30, o_col),
                (0.75, mix(o_col, Color::BLACK, 0.45)),
                (1.0,  Color::TRANSPARENT),
            ],
        );
        c.set_fill(orb_fill);
        c.set_shadow(o_col, (18.0 + val * 12.0) * user_scale);
        c.fill_circle(ox, oy, o_sz);

        // 3D Caustic Glint Cap (specular highlight)
        c.set_fill(Fill::Solid(spark_white.with_alpha(0.85)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_circle(ox - o_sz * 0.32, oy - o_sz * 0.32, o_sz * 0.28);

        // Secondary smaller glint
        c.set_fill(Fill::Solid(spark_white.with_alpha(0.40)));
        c.fill_circle(ox + o_sz * 0.22, oy + o_sz * 0.30, o_sz * 0.12);
    }

    // 4. Central Core Disc (on top of nucleus)
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(glass_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(glass_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
