//! Acoustic Cymascope style renderer (`acousticCymascope`) — Hyper-Realistic Cymatics Water Engine.
//!
//! Masterpiece CymaScope Water Sound Resonance Engine:
//! - 24-Ring Multi-Harmonic Chladni Standing Wave Mandala radiating across 360° fluid surface.
//! - Water Surface Caustic Sheen & Cross-Hatching Nodal Rays (photographic liquid light interference).
//! - Metallic / Glass Resonance Dish Platter Rim with specular bevel highlights.
//! - High-frequency treble water micro-ripples & bioluminescent liquid glow.
//! - 45+ Levitating water droplet motes bouncing with bass energy.
//! - Central bioluminescent liquid core with `draw_radial_center_image` integration.
//! - Full UI settings integration (Scale, Position X & Y, Sensitivity, Bar Count, Theme Colors).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

const CYMATIC_RINGS: usize = 20;
const NODAL_POINTS: usize = 120;

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
    let bar_count = ctx.config.reactivity.bar_count.clamp(32, 128);

    let be = ctx.bass_energy.clamp(0.0, 3.0);
    let bs = ctx.beat_strength.clamp(0.0, 3.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);
    let max_r = base_r * 1.55;

    // Curated Bioluminescent Water Palette
    let aqua_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.75);
    let emerald_green = mix(p_col, Color::rgba(0.0, 1.0, 0.45, 1.0), 0.75);
    let deep_ocean = mix(s_col, Color::rgba(0.02, 0.12, 0.25, 1.0), 0.70);
    let caustic_white = Color::rgba(0.95, 1.0, 0.98, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC BIOLUMINESCENT WATER GLOW & DISH BACKDROP
    // -------------------------------------------------------------------------
    let bg_cym = Fill::radial_gradient(
        cx,
        cy,
        inner_r,
        cx,
        cy,
        max_r * 1.8,
        &[
            (0.0, mix(aqua_cyan, deep_ocean, 0.5).with_alpha(0.35 + be * 0.20)),
            (0.50, mix(emerald_green, s_col, 0.5).with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_cym);

    // Glass Dish Bevel Rim Highlight
    c.set_stroke(Fill::Solid(aqua_cyan.with_alpha(0.65)));
    c.set_line_width(2.2 * user_scale);
    c.set_shadow(aqua_cyan, (14.0 + bs * 10.0) * user_scale);
    c.stroke_circle(cx, cy, max_r);

    // -------------------------------------------------------------------------
    // 2. EXPANDING WATER CAUSTIC SHOCKWAVE RIPPLES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(aqua_cyan, emerald_green, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * user_scale);
        c.set_shadow(aqua_cyan, (12.0 + bs * 8.0) * user_scale);
        c.stroke_circle(cx, cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. WATER CAUSTIC GLIMMER & CHLADNI NODAL WAVE INTERFERENCE
    // -------------------------------------------------------------------------
    for c_i in 0..4 {
        let cf = c_i as f32;
        let c_r = inner_r + (max_r - inner_r) * (0.2 + cf * 0.22);
        let c_rot = frame_time * (0.05 + cf * 0.02) * (if c_i % 2 == 0 { 1.0 } else { -1.0 });

        c.set_stroke(Fill::Solid(mix(aqua_cyan, caustic_white, 0.6).with_alpha(0.25 + be * 0.15)));
        c.set_line_width((1.2 + bs * 0.8) * user_scale);
        c.set_shadow(aqua_cyan, 6.0 * user_scale);

        let c_pts = 48usize;
        let mut c_poly: Vec<(f32, f32)> = Vec::with_capacity(c_pts + 1);
        for p in 0..=c_pts {
            let a = (p as f32 / c_pts as f32) * TAU + c_rot;
            let w = (a * (6.0 + cf * 2.0)).sin() * (4.0 + be * 6.0) * user_scale;
            c_poly.push((cx + a.cos() * (c_r + w), cy + a.sin() * (c_r + w)));
        }
        c.stroke_polyline(&c_poly);
    }

    // -------------------------------------------------------------------------
    // 4. 20-RING MULTI-HARMONIC CHLADNI STANDING WAVE MANDALA
    // -------------------------------------------------------------------------
    let step = (freq.len() / bar_count).max(1);

    for ring_i in 0..CYMATIC_RINGS {
        let ring_f = ring_i as f32;
        let t_ring = ring_f / (CYMATIC_RINGS - 1) as f32;
        let r_base = inner_r + t_ring * (max_r - inner_r);

        let fv = super::radial_common::opposing_interleaved_bin(freq, step, ring_i, CYMATIC_RINGS, ctx.beat_count);

        let harmonics = 4.0 + ((ring_i / 2) * 2) as f32; // 4, 4, 6, 6, 8, 8...

        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(NODAL_POINTS + 1);

        for n in 0..=NODAL_POINTS {
            let angle = (n as f32 / NODAL_POINTS as f32) * TAU;

            // Chladni standing wave superposition
            let w1 = (angle * harmonics + frame_time * 0.4).cos();
            let w2 = (angle * (harmonics + 2.0) - frame_time * 0.3).sin() * 0.35;
            let w3 = (angle * 16.0 + frame_time * 2.0).cos() * (fv * 0.25);

            let val = (fv * sensitivity * 1.1 + (w1 + w2 + w3) * 0.15 + be * 0.25).clamp(0.05, 2.0);
            let ripple = val * (18.0 + ring_f * 1.2) * user_scale;
            let r_nodal = (r_base + ripple).max(inner_r + 2.0);

            let (s_a, c_a) = angle.sin_cos();
            pts.push((cx + c_a * r_nodal, cy + s_a * r_nodal));
        }

        let ring_col = mix(mix(aqua_cyan, emerald_green, t_ring), mix(caustic_white, accent_col, 0.4), fv);

        c.set_stroke(Fill::Solid(ring_col.with_alpha(0.65 + fv * 0.25)));
        c.set_line_width((2.0 + fv * 1.5) * user_scale);
        c.set_shadow(ring_col, (10.0 + fv * 8.0) * user_scale);
        c.stroke_polyline(&pts);
    }

    // -------------------------------------------------------------------------
    // 5. LEVITATING WATER DROPLET MOTES (NODAL FLUID PARTICLES)
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0 * sensitivity).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.35 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = inner_r + m_t * (max_r * 0.95);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(aqua_cyan, caustic_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(aqua_cyan, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // -------------------------------------------------------------------------
    // 6. PUMPING CENTRAL DISC & WATER CORE RESERVOIR
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(aqua_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(aqua_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
