//! Pulsing Dual Ring style renderer (`pulsingDualRing`) — Cybernetic Concentric Dual-Ring Engine.
//!
//! Masterpiece Cybernetic Concentric Dual-Ring Resonance System:
//! - Dual Concentric 360° Neon Plasma Rings (Outer Frequency Ring & Inner Harmonic Ring).
//! - Dual-Pass Laser Ring Rendering with white-hot core filaments & cyan/magenta plasma sheaths.
//! - Inter-Ring Electric Laser Discharge Arcs bridging adjacent ring nodes.
//! - Central Quantum Singularity Disc with 3 expanding energy pulse shockwaves.
//! - 45+ Orbital plasma motes & energy spark particles circling in opposite directions.
//! - Full UI Theme colors and settings integration.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

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

    // Curated Cybernetic Dual Ring Palette
    let ring_cyan = mix(glow_col, Color::rgba(0.0, 0.95, 1.0, 1.0), 0.75);
    let ring_magenta = mix(accent_col, Color::rgba(1.0, 0.15, 0.85, 1.0), 0.75);
    let ring_gold = mix(p_col, Color::rgba(1.0, 0.80, 0.10, 1.0), 0.70);
    let spark_white = Color::rgba(0.98, 1.0, 1.0, 0.98);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC CONCENTRIC BACKDROP GLOW
    // -------------------------------------------------------------------------
    let bg_ring = Fill::radial_gradient(
        cx,
        cy,
        inner_r,
        cx,
        cy,
        base_r * 2.5,
        &[
            (0.0, mix(ring_cyan, ring_magenta, 0.5).with_alpha(0.32 + be * 0.18)),
            (0.50, mix(ring_gold, s_col, 0.5).with_alpha(0.12)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_ring);

    // -------------------------------------------------------------------------
    // 2. EXPANDING DUAL RING PULSE SHOCKWAVES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(ring_cyan, ring_magenta, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * user_scale);
        c.set_shadow(ring_cyan, (12.0 + bs * 8.0) * user_scale);
        c.stroke_circle(cx, cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. INNER NEON HARMONIC RING (CLOCKWISE ROTATION)
    // -------------------------------------------------------------------------
    let step = (freq.len() / bar_count).max(1);
    let rot1 = frame_time * 0.15;
    let r1_base = inner_r + (25.0 + be * 15.0) * user_scale;

    let mut ring1_pts: Vec<(f32, f32)> = Vec::with_capacity(bar_count + 1);
    let mut ring1_radii: Vec<f32> = Vec::with_capacity(bar_count);

    for i in 0..bar_count {
        let t = i as f32 / bar_count as f32;
        let angle = t * TAU + rot1;

        let sample_bin = (i * step).min(freq.len().saturating_sub(1));
        let fv = freq[sample_bin] as f32 / 255.0;

        let base_wave = (angle * 3.0 + frame_time * 1.5).sin() * 0.08 + 0.12;
        let val = (fv * sensitivity * 1.1 + base_wave + be * 0.25).clamp(0.10, 2.0);

        let ripple_r = r1_base + (val * 35.0) * user_scale;
        ring1_radii.push(ripple_r);

        let (cos_a, sin_a) = angle.sin_cos();
        ring1_pts.push((cx + cos_a * ripple_r, cy + sin_a * ripple_r));
    }
    if let Some(&first) = ring1_pts.first() {
        ring1_pts.push(first);
    }

    let mut smooth_ring1: Vec<(f32, f32)> = Vec::new();
    let num_pts1 = ring1_pts.len();
    for i in 0..num_pts1.saturating_sub(1) {
        let p0 = ring1_pts[i];
        let p1 = ring1_pts[i + 1];
        let mid = ((p0.0 + p1.0) * 0.5, (p0.1 + p1.1) * 0.5);

        let seg = GpuCanvas::sample_quadratic(
            if smooth_ring1.is_empty() { p0 } else { *smooth_ring1.last().unwrap() },
            p0,
            mid,
            4,
        );
        if smooth_ring1.is_empty() {
            smooth_ring1.extend(seg);
        } else {
            smooth_ring1.extend(seg.into_iter().skip(1));
        }
    }

    // Pass A: Ring 1 Outer Plasma Sheath
    c.set_stroke(Fill::Solid(ring_cyan.with_alpha(0.65)));
    c.set_line_width((3.8 + bs * 2.0) * user_scale);
    c.set_shadow(ring_cyan, (18.0 + bs * 12.0) * user_scale);
    c.stroke_polyline(&smooth_ring1);

    // Pass B: Ring 1 White-Hot Core Filament
    c.set_stroke(Fill::Solid(spark_white));
    c.set_line_width(1.6 * user_scale);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.stroke_polyline(&smooth_ring1);

    // -------------------------------------------------------------------------
    // 4. OUTER NEON FREQUENCY RING (COUNTER-CLOCKWISE ROTATION)
    // -------------------------------------------------------------------------
    let rot2 = frame_time * -0.12;
    let r2_base = r1_base + 45.0 * user_scale;

    let mut ring2_pts: Vec<(f32, f32)> = Vec::with_capacity(bar_count + 1);
    let mut ring2_radii: Vec<f32> = Vec::with_capacity(bar_count);

    for i in 0..bar_count {
        let t = i as f32 / bar_count as f32;
        let angle = t * TAU + rot2;

        let sample_bin = ((bar_count - 1 - i) * step).min(freq.len().saturating_sub(1));
        let fv = freq[sample_bin] as f32 / 255.0;

        let base_wave = (angle * 4.0 - frame_time * 1.8).sin() * 0.08 + 0.12;
        let val = (fv * sensitivity * 1.1 + base_wave + bs * 0.30).clamp(0.10, 2.0);

        let ripple_r = r2_base + (val * 45.0) * user_scale;
        ring2_radii.push(ripple_r);

        let (cos_a, sin_a) = angle.sin_cos();
        ring2_pts.push((cx + cos_a * ripple_r, cy + sin_a * ripple_r));
    }
    if let Some(&first) = ring2_pts.first() {
        ring2_pts.push(first);
    }

    let mut smooth_ring2: Vec<(f32, f32)> = Vec::new();
    let num_pts2 = ring2_pts.len();
    for i in 0..num_pts2.saturating_sub(1) {
        let p0 = ring2_pts[i];
        let p1 = ring2_pts[i + 1];
        let mid = ((p0.0 + p1.0) * 0.5, (p0.1 + p1.1) * 0.5);

        let seg = GpuCanvas::sample_quadratic(
            if smooth_ring2.is_empty() { p0 } else { *smooth_ring2.last().unwrap() },
            p0,
            mid,
            4,
        );
        if smooth_ring2.is_empty() {
            smooth_ring2.extend(seg);
        } else {
            smooth_ring2.extend(seg.into_iter().skip(1));
        }
    }

    // Pass A: Ring 2 Outer Plasma Sheath
    c.set_stroke(Fill::Solid(ring_magenta.with_alpha(0.65)));
    c.set_line_width((3.8 + bs * 2.0) * user_scale);
    c.set_shadow(ring_magenta, (18.0 + bs * 12.0) * user_scale);
    c.stroke_polyline(&smooth_ring2);

    // Pass B: Ring 2 White-Hot Core Filament
    c.set_stroke(Fill::Solid(spark_white));
    c.set_line_width(1.6 * user_scale);
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.stroke_polyline(&smooth_ring2);

    // -------------------------------------------------------------------------
    // 5. INTER-RING ELECTRIC LASER DISCHARGE ARCS (BRIDGING RINGS)
    // -------------------------------------------------------------------------
    let arc_bridges = 16usize;
    for a_i in 0..arc_bridges {
        let a_t = a_i as f32 / arc_bridges as f32;
        let angle1 = a_t * TAU + rot1;
        let angle2 = a_t * TAU + rot2;

        let idx = (a_i * (bar_count / arc_bridges)).min(bar_count - 1);
        let r1 = ring1_radii[idx];
        let r2 = ring2_radii[idx];

        let p1 = (cx + angle1.cos() * r1, cy + angle1.sin() * r1);
        let p2 = (cx + angle2.cos() * r2, cy + angle2.sin() * r2);

        let bridge_col = mix(ring_cyan, ring_gold, a_t);
        c.set_stroke(Fill::Solid(spark_white.with_alpha(0.70)));
        c.set_line_width(1.4 * user_scale);
        c.set_shadow(bridge_col, 10.0 * user_scale);
        c.stroke_line(p1.0, p1.1, p2.0, p2.1);
    }

    // -------------------------------------------------------------------------
    // 6. ORBITING PLASMA MOTES & ENERGY SPARKS
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + be * 24.0 * sensitivity).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.35 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 31.0).sin() * TAU;
        let m_dist = inner_r + m_t * (r2_base * 1.10);

        let mx = cx + m_angle.cos() * m_dist;
        let my = cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + bs * 1.8).clamp(1.0, 5.0) * user_scale;
        let m_col = mix(ring_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(ring_cyan, 8.0 * user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    // -------------------------------------------------------------------------
    // 7. PUMPING CENTRAL DISC & LIQUID CORE RESERVOIR
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(ring_cyan));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(ring_cyan, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    c.set_global_alpha(1.0);
    c.restore();
}
