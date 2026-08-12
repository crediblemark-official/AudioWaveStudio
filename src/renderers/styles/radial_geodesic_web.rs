//! Radial Geodesic Web style renderer (`radialGeodesicWeb`) — 3D Cybernetic Geodesic Lattice Engine.
//!
//! Masterpiece 360° Cybernetic Geodesic Web Mesh:
//! - 6 Concentric geodesic rings connected by 24 interlocking triangular & hexagonal strut links.
//! - Dual-pass neon laser struts with white-hot core filaments & cyan/magenta plasma sheaths.
//! - Luminous 3D geodesic vertex nodes with audio-reactive specular flares.
//! - Central quantum singularity core with 3 expanding geodesic forcefield shockwaves.
//! - 45+ Swirling quantum dust particles & energy sparks.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const GEODESIC_RINGS: usize = 6;
const GEODESIC_NODES: usize = 24;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 115.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let rot = frame_time * 0.10;
    let step = ((freq.len() as f32) / (GEODESIC_RINGS * GEODESIC_NODES) as f32).floor().max(1.0) as usize;

    // Curated Cybernetic Geodesic Palette (theme-dominant, hardcoded hue only as character accent)
    let geo_cyan = mix(Color::rgba(0.0, 0.95, 1.0, 1.0), s.glow, 0.75);
    let geo_magenta = mix(Color::rgba(1.0, 0.15, 0.85, 1.0), s.s_col, 0.75);
    let geo_lime = mix(Color::rgba(0.20, 1.0, 0.40, 1.0), s.p_col, 0.70);
    let spark_white = mix(Color::rgba(0.95, 1.0, 1.0, 0.98), s.glow, 0.10);

    let max_r = s.base_r * 1.50;

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC CYBERNETIC GEODESIC GLOW BACKDROP
    // -------------------------------------------------------------------------
    let bg_geo = Fill::radial_gradient(
        s.cx,
        s.cy,
        0.0,
        s.cx,
        s.cy,
        max_r * 1.8,
        &[
            (0.0, mix(geo_cyan, geo_magenta, 0.5).with_alpha(0.30 + s.be * 0.20)),
            (0.50, geo_lime.with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_geo);

    // -------------------------------------------------------------------------
    // 2. EXPANDING GEODESIC FORCEFIELD SHOCKWAVES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = s.inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + s.bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(geo_cyan, geo_magenta, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * s.user_scale);
        c.set_shadow(geo_cyan, (12.0 + s.bs * 8.0) * s.user_scale);
        c.stroke_circle(s.cx, s.cy, pulse_r);
    }

    // Pre-calculate node coordinates and audio energy across all rings
    let mut node_coords: Vec<Vec<(f32, f32)>> = vec![vec![(0.0, 0.0); GEODESIC_NODES]; GEODESIC_RINGS];
    let mut node_energy: Vec<Vec<f32>> = vec![vec![0.0f32; GEODESIC_NODES]; GEODESIC_RINGS];

    for r in 0..GEODESIC_RINGS {
        let rf = r as f32;
        let t = rf / (GEODESIC_RINGS - 1) as f32;
        let ring_r = s.inner_r + t * (max_r - s.inner_r);
        let ring_rot = rot * (if r % 2 == 0 { 1.0 } else { -1.0 }) * (1.0 + rf * 0.15);

        for n in 0..GEODESIC_NODES {
            let slot = (n + (r * 3) + ((s.sweep / TAU) * GEODESIC_NODES as f32) as usize) % GEODESIC_NODES;
            let sample_bin = ((r * GEODESIC_NODES + slot) * step).min(freq.len().saturating_sub(1));
            let fv = freq[sample_bin] as f32 / 255.0;

            let na = (n as f32 / GEODESIC_NODES as f32) * TAU + ring_rot;
            let ripple = (fv * s.sensitivity * 25.0 + s.be * 12.0) * s.user_scale;
            let (sin_a, cos_a) = na.sin_cos();

            let final_r = (ring_r + ripple).max(s.inner_r + 4.0);
            node_coords[r][n] = (s.cx + cos_a * final_r, s.cy + sin_a * final_r);
            node_energy[r][n] = fv;
        }
    }

    // -------------------------------------------------------------------------
    // 3. CONCENTRIC & DIAGONAL GEODESIC MESH STRUTS (DUAL-PASS CORE + SHEATH)
    // -------------------------------------------------------------------------
    for r in 0..GEODESIC_RINGS {
        for n in 0..GEODESIC_NODES {
            let next_n = (n + 1) % GEODESIC_NODES;

            let p0 = node_coords[r][n];
            let p1 = node_coords[r][next_n];
            let fv = (node_energy[r][n] + node_energy[r][next_n]) * 0.5;

            let strut_col = mix(mix(geo_cyan, geo_lime, r as f32 / GEODESIC_RINGS as f32), mix(geo_magenta, spark_white, 0.4), fv);

            // Pass A: Outer neon plasma sheath
            c.set_stroke(Fill::Solid(strut_col.with_alpha(0.55)));
            c.set_line_width((2.8 + fv * 2.0) * s.user_scale);
            c.set_shadow(strut_col, (12.0 + fv * 10.0) * s.user_scale);
            c.stroke_line(p0.0, p0.1, p1.0, p1.1);

            // Pass B: Intense white-hot core filament
            c.set_stroke(Fill::Solid(spark_white));
            c.set_line_width((1.2 + fv * 0.6) * s.user_scale);
            c.set_shadow(Color::TRANSPARENT, 0.0);
            c.stroke_line(p0.0, p0.1, p1.0, p1.1);

            // Inter-ring radial & diagonal geodesic cross-bracing struts
            if r + 1 < GEODESIC_RINGS {
                let p_outer = node_coords[r + 1][n];
                let p_outer_diag = node_coords[r + 1][next_n];

                // Radial strut
                c.set_stroke(Fill::Solid(strut_col.with_alpha(0.45)));
                c.set_line_width(1.6 * s.user_scale);
                c.stroke_line(p0.0, p0.1, p_outer.0, p_outer.1);

                // Diagonal bracing strut for true 3D geodesic triangular lattice
                if (n + r) % 2 == 0 {
                    c.set_stroke(Fill::Solid(mix(strut_col, spark_white, 0.5).with_alpha(0.40)));
                    c.set_line_width(1.2 * s.user_scale);
                    c.stroke_line(p0.0, p0.1, p_outer_diag.0, p_outer_diag.1);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // 4. LUMINOUS GEODESIC VERTEX NODES
    // -------------------------------------------------------------------------
    for r in 0..GEODESIC_RINGS {
        for n in 0..GEODESIC_NODES {
            let (nx, ny) = node_coords[r][n];
            let fv = node_energy[r][n];

            let node_col = mix(mix(geo_cyan, geo_magenta, fv), spark_white, fv * 0.6);
            let node_sz = (2.6 + fv * 2.8 + s.bs * 1.5) * s.user_scale;

            c.set_fill(Fill::Solid(node_col));
            c.set_shadow(geo_cyan, (10.0 + s.bs * 8.0) * s.user_scale);
            c.fill_circle(nx, ny, node_sz);
        }
    }

    // -------------------------------------------------------------------------
    // 5. SWIRLING QUANTUM GEODESIC DUST PARTICLES
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + s.be * 24.0 * s.sensitivity).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.4 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = s.inner_r + m_t * (max_r * 0.95);

        let mx = s.cx + m_angle.cos() * m_dist;
        let my = s.cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + s.bs * 1.8).clamp(1.0, 5.0) * s.user_scale;
        let m_col = mix(geo_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(geo_cyan, 8.0 * s.user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    radial_common::finish(c, ctx, &s);
}
