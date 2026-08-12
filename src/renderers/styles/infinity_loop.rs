//! Infinity Loop style renderer (`InfinityLoop`) — 3D Quantum Infinity Ribbon Engine.
//!
//! Masterpiece Full-Canvas Quantum Infinity Loop Engine:
//! - Dynamic 3D Lemniscate Infinity Ribbon ($\infty$) weaving seamlessly across the canvas.
//! - 64 Audio-reactive plasma nodes traveling ALONG the infinity ribbon curve.
//! - Dual-Pass Laser Sheaths with white-hot core filaments & cyan/magenta plasma sheaths.
//! - Orbital quantum dust motes bouncing with bass energy across space.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const RIBBON_NODES: usize = 64;

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

    let loop_scale_x = width * 0.35 * user_scale;
    let loop_scale_y = height * 0.32 * user_scale;

    // Palette
    let inf_cyan    = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.75);
    let inf_magenta = mix(accent_col, Color::rgba(1.0, 0.15, 0.85, 1.0), 0.75);
    let inf_yellow  = mix(p_col, Color::rgba(1.0, 0.85, 0.10, 1.0), 0.70);
    let spark_white = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. Ambient Volumetric Space Glow Backdrop
    // -------------------------------------------------------------------------
    let bg_inf = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, width.max(height) * 0.85,
        &[
            (0.00, inf_cyan.with_alpha(0.35 + be * 0.18)),
            (0.50, inf_magenta.with_alpha(0.18 + bs * 0.10)),
            (1.00, Color::rgba(0.02, 0.01, 0.06, 1.0)),
        ],
    );
    c.set_fill(bg_inf);
    c.fill_rect(0.0, 0.0, width, height);

    // Expanding forcefield shockwave ring on drum beats
    if bs > 0.12 {
        let pulse_r = (100.0 + bs * 200.0) * user_scale;
        c.set_stroke(Fill::Solid(inf_cyan.with_alpha((0.60 * (1.0 - bs / 3.0)).max(0.0))));
        c.set_line_width((3.0 + bs * 2.0) * user_scale);
        c.set_shadow(inf_magenta, (16.0 + bs * 10.0) * user_scale);
        c.stroke_circle(cx, cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 2. DYNAMIC 3D LEMNISCATE INFINITY RIBBON PATH & TRAVELLING PLASMA NODES
    // -------------------------------------------------------------------------
    let step = (freq.len() / RIBBON_NODES).max(1);
    let rot = frame_time * 0.30;

    let mut ribbon_pts: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(RIBBON_NODES + 1);

    for i in 0..=RIBBON_NODES {
        let t = i as f32 / RIBBON_NODES as f32;
        let theta = t * TAU + rot;

        // Gerono Lemniscate 3D parameterization: x = a * cos(theta), y = b * sin(2*theta) / 2
        let lx = loop_scale_x * theta.cos();
        let ly = loop_scale_y * (theta * 2.0).sin() * 0.5;

        // Z-depth 3D perspective distortion
        let z_depth = 1.0 + theta.sin() * 0.22;
        let px = cx + lx * z_depth;
        let py = cy + ly * z_depth;

        let fv = super::radial_common::full_random_scattered_bin(freq, step, i % RIBBON_NODES, RIBBON_NODES, ctx.beat_count);
        let val = (fv * sensitivity * 1.1 + be * 0.25).clamp(0.10, 2.2);

        ribbon_pts.push((px, py, z_depth, val));
    }

    // Pass A: Outer Plasma Sheath Glow Ribbon
    for i in 0..RIBBON_NODES {
        let (x0, y0, _z0, val0) = ribbon_pts[i];
        let (x1, y1, _z1, val1) = ribbon_pts[i + 1];
        let val = (val0 + val1) * 0.5;
        let tf = i as f32 / RIBBON_NODES as f32;

        let seg_col = mix(mix(inf_cyan, inf_magenta, tf), mix(inf_yellow, s_col, 0.5), val * 0.5);

        c.set_stroke(Fill::Solid(seg_col.with_alpha(0.75)));
        c.set_line_width((4.5 + val * 3.5) * user_scale);
        c.set_shadow(seg_col, (18.0 + val * 12.0) * user_scale);
        c.stroke_line(x0, y0, x1, y1);

        // White-Hot Core Filament
        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width(1.8 * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_line(x0, y0, x1, y1);
    }

    // Pass B: Traveling Plasma Node Orbs along Infinity Ribbon
    for i in 0..RIBBON_NODES {
        let (px, py, _z, val) = ribbon_pts[i];
        if i % 2 == 0 {
            let tf = i as f32 / RIBBON_NODES as f32;
            let node_col = mix(mix(inf_cyan, inf_magenta, tf), spark_white, val * 0.5);

            c.set_fill(Fill::Solid(node_col));
            c.set_shadow(inf_cyan, (14.0 + val * 10.0) * user_scale);
            c.fill_circle(px, py, (3.2 + val * 2.5) * user_scale);
        }
    }

    // -------------------------------------------------------------------------
    // 3. ORBITING QUANTUM DUST PARTICLES ACROSS CANVAS
    // -------------------------------------------------------------------------
    let mote_count = (28.0 + be * 28.0 * sensitivity).clamp(18.0, 56.0) as usize;
    for m_i in 0..mote_count {
        let mf = m_i as f32;
        let m_t = ((frame_time * 0.4 + mf * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (mf * 31.0).sin() * TAU;
        let m_dist  = m_t * (width * 0.45);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(inf_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(inf_cyan, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    c.set_global_alpha(1.0);
    c.restore();
}


