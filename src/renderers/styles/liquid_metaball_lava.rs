//! Liquid Metaball Lava style renderer (`liquidMetaballLava`) — 3D Floating Lava Lamp Wax Orbs.
//!
//! Masterpiece Full-Canvas Psychedelic Lava Lamp Engine:
//! - 18 Individual 3D Floating Organic Lava Wax Globules & Metaballs morphing & floating across the screen.
//! - 3D Specular Glass Highlights (white glint cap) on every lava orb for photorealistic depth.
//! - Gooey Metaball Necks connecting adjacent melting lava globules.
//! - Floating thermal convection micro-bubbles & lava plasma motes across the canvas.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const LAVA_ORBS: usize = 18;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col      = theme_primary(theme);
    let s_col      = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col   = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let bass_mult   = ctx.config.reactivity.bass_multiplier;
    let user_scale  = ctx.config.scale.clamp(0.1, 5.0);

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5;
    let cy = height * 0.5;

    // Curated 70s Lava Lamp Palette
    let lava_magenta = mix(accent_col, Color::rgba(1.0, 0.0, 0.50, 1.0), 0.85); // Hot Magenta
    let lava_orange  = mix(glow_col, Color::rgba(1.0, 0.40, 0.0, 1.0), 0.80);   // Glowing Orange
    let lava_gold    = Color::rgba(1.0, 0.75, 0.0, 1.0);                          // Liquid Gold
    let lava_purple  = mix(s_col, Color::rgba(0.60, 0.0, 0.95, 1.0), 0.75);     // Deep Purple
    let spark_white  = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. Full-Screen Atmospheric Psychedelic Lava Glow Backdrop
    // -------------------------------------------------------------------------
    let bg_lava = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, width.max(height) * 0.85,
        &[
            (0.00, mix(lava_magenta, lava_orange, 0.5).with_alpha(0.38 + be * 0.20)),
            (0.50, mix(lava_purple, p_col, 0.5).with_alpha(0.18 + bs * 0.10)),
            (1.00, Color::rgba(0.03, 0.01, 0.08, 1.0)),
        ],
    );
    c.set_fill(bg_lava);
    c.fill_rect(0.0, 0.0, width, height);

    // -------------------------------------------------------------------------
    // 2. 18 Floating 3D Organic Lava Wax Globules & Metaballs Across Canvas
    // -------------------------------------------------------------------------
    let step = (freq.len() / LAVA_ORBS).max(1);
    let mut orb_data: Vec<(f32, f32, f32, Color)> = Vec::with_capacity(LAVA_ORBS);

    for i in 0..LAVA_ORBS {
        let i_f = i as f32;
        let t   = i_f / LAVA_ORBS as f32;

        let fv  = super::radial_common::full_random_scattered_bin(freq, step, i, LAVA_ORBS, ctx.beat_count);
        let val = (fv * sensitivity * 1.1 + be * 0.35 + bs * 0.25).clamp(0.05, 2.4);

        // Vertical thermal convection loop (rising & falling lava orbs across full height)
        let speed  = 0.15 + (i_f * 3.7).sin().abs() * 0.20;
        let cycle_y = ((frame_time * speed + i_f * 0.413) % 1.2) - 0.1; // -0.1 to 1.1 height range
        let oy      = height * (1.0 - cycle_y);

        // Horizontal sinusoidal drift
        let drift_x = (i_f * 137.5).sin() * 0.45 * width;
        let wobble_x = (frame_time * 1.2 + i_f * 2.3).sin() * (35.0 * user_scale);
        let ox      = cx + drift_x + wobble_x;

        let orb_r = (24.0 + val * 38.0 + be * 10.0) * user_scale;
        let orb_col = mix(mix(lava_magenta, lava_orange, t), mix(lava_gold, lava_purple, fv), (i % 4) as f32 * 0.25);

        orb_data.push((ox, oy, orb_r, orb_col));
    }

    // Pass A: Gooey Metaball Necks between close floating lava blobs
    for i in 0..LAVA_ORBS {
        for j in (i + 1)..LAVA_ORBS {
            let (x1, y1, r1, c1) = orb_data[i];
            let (x2, y2, r2, c2) = orb_data[j];

            let dx = x2 - x1;
            let dy = y2 - y1;
            let dist = (dx * dx + dy * dy).sqrt();
            let max_connect_dist = (r1 + r2) * 1.55;

            if dist < max_connect_dist && dist > 10.0 {
                let neck_col = mix(c1, c2, 0.5);
                let alpha_val = ((1.0 - dist / max_connect_dist) * 0.75 + be * 0.15).clamp(0.15, 0.85);

                let ang = dy.atan2(dx);
                let mid_x = (x1 + x2) * 0.5;
                let mid_y = (y1 + y2) * 0.5;

                let t_ratio = dist / max_connect_dist;
                let spread = (std::f32::consts::TAU * 0.12) * (1.0 - t_ratio * 0.6);

                let p1_a = (x1 + r1 * 0.80 * (ang + spread).cos(), y1 + r1 * 0.80 * (ang + spread).sin());
                let p1_b = (x1 + r1 * 0.80 * (ang - spread).cos(), y1 + r1 * 0.80 * (ang - spread).sin());

                let p2_a = (x2 + r2 * 0.80 * (ang + std::f32::consts::TAU * 0.5 - spread).cos(), y2 + r2 * 0.80 * (ang + std::f32::consts::TAU * 0.5 - spread).sin());
                let p2_b = (x2 + r2 * 0.80 * (ang + std::f32::consts::TAU * 0.5 + spread).cos(), y2 + r2 * 0.80 * (ang + std::f32::consts::TAU * 0.5 + spread).sin());

                let pinch_factor = 0.40 * t_ratio + 0.10;
                let ctrl_a = (mid_x + (p1_a.0 + p2_a.0 - 2.0 * mid_x) * pinch_factor, mid_y + (p1_a.1 + p2_a.1 - 2.0 * mid_y) * pinch_factor);
                let ctrl_b = (mid_x + (p1_b.0 + p2_b.0 - 2.0 * mid_x) * pinch_factor, mid_y + (p1_b.1 + p2_b.1 - 2.0 * mid_y) * pinch_factor);

                let seg_a = GpuCanvas::sample_quadratic(p1_a, ctrl_a, p2_a, 6);
                let seg_b = GpuCanvas::sample_quadratic(p2_b, ctrl_b, p1_b, 6);

                let mut bridge_pts = Vec::with_capacity(seg_a.len() + seg_b.len() + 2);
                bridge_pts.extend(seg_a);
                bridge_pts.extend(seg_b);
                if let Some(&first) = bridge_pts.first() { bridge_pts.push(first); }

                let bridge_fill = Fill::radial_gradient(
                    mid_x, mid_y, 0.0,
                    mid_x, mid_y, dist * 0.85,
                    &[
                        (0.00, mix(neck_col, Color::WHITE, 0.35).with_alpha(alpha_val)),
                        (0.60, neck_col.with_alpha(alpha_val * 0.80)),
                        (1.00, Color::TRANSPARENT),
                    ],
                );
                c.set_fill(bridge_fill);
                c.set_shadow(neck_col, (14.0 + bs * 8.0) * user_scale);
                fill_radial_polygon(c, mid_x, mid_y, &bridge_pts);
            }
        }
    }

    // Pass B: 3D Volumetric Floating Lava Wax Spheres
    for &(ox, oy, orb_r, orb_col) in &orb_data {
        let orb_fill = Fill::radial_gradient(
            ox - orb_r * 0.30,
            oy - orb_r * 0.30,
            0.0,
            ox,
            oy,
            orb_r * 1.8,
            &[
                (0.00, mix(orb_col, spark_white, 0.45).with_alpha(0.95)),
                (0.55, orb_col.with_alpha(0.85)),
                (1.00, Color::TRANSPARENT),
            ],
        );

        c.set_fill(orb_fill);
        c.set_shadow(orb_col, (16.0 + bs * 10.0) * user_scale);
        c.fill_circle(ox, oy, orb_r);

        // 3D Glass Specular Highlight Cap
        c.set_fill(Fill::Solid(spark_white.with_alpha(0.85)));
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.fill_circle(ox - orb_r * 0.32, oy - orb_r * 0.32, (orb_r * 0.28).max(1.5 * user_scale));
    }

    // -------------------------------------------------------------------------
    // 3. Floating Thermal Convection Micro-Bubbles
    // -------------------------------------------------------------------------
    let mote_count = (30.0 + be * 30.0).clamp(18.0, 60.0) as usize;
    for m_i in 0..mote_count {
        let mf = m_i as f32;
        let m_t = ((frame_time * 0.35 + mf * 0.17) % 1.0).clamp(0.0, 1.0);

        let mx = (mf * 137.5).sin() * 0.48 * width + cx;
        let my = height * (1.0 - m_t);

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(lava_magenta, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(lava_magenta, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    c.set_global_alpha(1.0);
    c.restore();
}

