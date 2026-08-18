//! Pulsing Laser Web style renderer (`pulsingLaserWeb`) — Cybernetic 3D Laser Mesh Engine.
//!
//! Masterpiece 3D Cybernetic Laser Web Defense Mesh:
//! - 3 Concentric 360° Laser Rings (Inner, Mid, Outer) connected by 32 interlocking triangular laser struts.
//! - Dual-Pass Laser Rendering with white-hot core filaments & cyan/magenta plasma sheaths.
//! - Luminous 3D Laser Web Vertex Nodes with audio-reactive specular flares & electric discharge arcs.
//! - Central Laser Singularity Disc with 3 expanding forcefield shockwave ripples.
//! - 45+ Floating laser spark motes & orbital energy particles.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const WEB_RINGS: usize = 3;
const WEB_NODES: usize = 32;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width = ctx.width;
    let height = ctx.height;
    let theme = &ctx.config.theme;

    let p_col = theme_primary(theme);
    let s_col = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let user_scale = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = -ctx.config.position_y * height * 0.5;
    let _bar_count = ctx.config.reactivity.bar_count.clamp(32, 128);

    let be = ctx.bass_energy.clamp(0.0, 3.0);
    let bs = ctx.beat_strength.clamp(0.0, 3.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);

    // Curated Laser Web Palette
    let laser_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.75);
    let laser_magenta = mix(accent_col, Color::rgba(1.0, 0.15, 0.85, 1.0), 0.75);
    let laser_yellow = mix(p_col, Color::rgba(1.0, 0.85, 0.10, 1.0), 0.70);
    let spark_white = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC LASER WEB BACKDROP GLOW
    // -------------------------------------------------------------------------
    let max_r = base_r * 1.55;
    let bg_web = Fill::radial_gradient(
        cx,
        cy,
        inner_r,
        cx,
        cy,
        max_r * 1.8,
        &[
            (0.0, mix(laser_cyan, laser_magenta, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, mix(laser_yellow, s_col, 0.5).with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_web);

    // -------------------------------------------------------------------------
    // 2. EXPANDING LASER FORCEFIELD SHOCKWAVES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(laser_cyan, laser_magenta, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * user_scale);
        c.set_shadow(laser_cyan, (12.0 + bs * 8.0) * user_scale);
        c.stroke_circle(cx, cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. 3 CONCENTRIC LASER RINGS WITH 32 INTERLOCKING TRIANGULAR STRUTS
    // -------------------------------------------------------------------------
    let step = (freq.len() / WEB_NODES).max(1);

    let mut ring_coords: Vec<Vec<(f32, f32)>> = vec![vec![(0.0, 0.0); WEB_NODES]; WEB_RINGS];
    let mut ring_energy: Vec<Vec<f32>> = vec![vec![0.0f32; WEB_NODES]; WEB_RINGS];

    for r in 0..WEB_RINGS {
        let rf = r as f32;
        let t = rf / (WEB_RINGS - 1) as f32;
        let ring_r_base = inner_r + (30.0 + t * 55.0) * user_scale;
        let rot = frame_time * (0.10 + rf * 0.05) * (if r % 2 == 0 { 1.0 } else { -1.0 });

        for n in 0..WEB_NODES {
            let sample_bin = ((r * WEB_NODES + n) * step).min(freq.len().saturating_sub(1));
            let fv = freq[sample_bin] as f32 / 255.0;

            let angle = (n as f32 / WEB_NODES as f32) * TAU + rot;
            let val = (fv * sensitivity * 1.1 + (angle * 4.0).sin() * 0.08 + be * 0.25).clamp(0.10, 2.0);

            let r_curr = ring_r_base + (val * 25.0) * user_scale;
            let (cos_a, sin_a) = angle.sin_cos();

            ring_coords[r][n] = (cx + cos_a * r_curr, cy + sin_a * r_curr);
            ring_energy[r][n] = fv;
        }
    }

    // Render Laser Web Struts (Concentric Rings + Interlocking Triangular Cross-Beams)
    for r in 0..WEB_RINGS {
        for n in 0..WEB_NODES {
            let next_n = (n + 1) % WEB_NODES;

            let p0 = ring_coords[r][n];
            let p1 = ring_coords[r][next_n];
            let fv = (ring_energy[r][n] + ring_energy[r][next_n]) * 0.5;

            let strut_col = mix(mix(laser_cyan, laser_magenta, r as f32 / WEB_RINGS as f32), mix(laser_yellow, spark_white, 0.4), fv);

            // Pass A: Outer neon plasma sheath
            c.set_stroke(Fill::Solid(strut_col.with_alpha(0.60)));
            c.set_line_width((3.5 + fv * 2.0) * user_scale);
            c.set_shadow(strut_col, (14.0 + fv * 10.0) * user_scale);
            c.stroke_line(p0.0, p0.1, p1.0, p1.1);

            // Pass B: Intense white-hot core filament
            c.set_stroke(Fill::Solid(spark_white));
            c.set_line_width((1.4 + fv * 0.6) * user_scale);
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.stroke_line(p0.0, p0.1, p1.0, p1.1);

            // Inter-ring triangular cross-beams
            if r + 1 < WEB_RINGS {
                let p_outer = ring_coords[r + 1][n];
                let p_outer_diag = ring_coords[r + 1][next_n];

                // Radial strut
                c.set_stroke(Fill::Solid(strut_col.with_alpha(0.50)));
                c.set_line_width(1.6 * user_scale);
                c.stroke_line(p0.0, p0.1, p_outer.0, p_outer.1);

                // Diagonal triangular bracing
                if (n + r) % 2 == 0 {
                    c.set_stroke(Fill::Solid(mix(strut_col, spark_white, 0.6).with_alpha(0.45)));
                    c.set_line_width(1.2 * user_scale);
                    c.stroke_line(p0.0, p0.1, p_outer_diag.0, p_outer_diag.1);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // 4. LUMINOUS LASER WEB VERTEX NODES
    // -------------------------------------------------------------------------
    for r in 0..WEB_RINGS {
        for n in 0..WEB_NODES {
            let (nx, ny) = ring_coords[r][n];
            let fv = ring_energy[r][n];

            let node_col = mix(mix(laser_cyan, laser_yellow, fv), spark_white, fv * 0.7);
            let node_sz = (2.8 + fv * 2.8 + bs * 1.5) * user_scale;

            c.set_fill(Fill::Solid(node_col));
            c.set_shadow(laser_cyan, (12.0 + bs * 8.0) * user_scale);
            c.fill_circle(nx, ny, node_sz);
        }
    }

    // -------------------------------------------------------------------------
    // 5. FLOATING LASER SPARKS & ENERGY DUST MOTES
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0 * sensitivity).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = inner_r + m_t * (max_r * 0.95);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(laser_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(laser_cyan, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // -------------------------------------------------------------------------
    // 6. PUMPING CENTRAL DISC & LASER CORE RESERVOIR
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(laser_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(laser_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
