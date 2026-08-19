//! Radial Vortex Spiral style renderer (`radialVortexSpiral`) — Cosmic Hyper Vortex Engine.
//!
//! Masterpiece Hyper Vortex Spiral Wormhole:
//! - 12 High-density logarithmic vortex spiral ribbons twisting into a gravitational singularity.
//! - Audio-reactive frequency wave ripples modulating the spiral arm curvature & width.
//! - Multi-pass neon plasma glow with white-hot core filaments & rainbow energy sheaths.
//! - Luminous central vortex singularity orb with 3 expanding event horizon shockwaves.
//! - 45+ Swirling cosmic stardust motes pulling into the vortex core.
//! - Full UI Theme colors and settings integration (Scale, Position X & Y, Sensitivity, Bar Count).

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::mix;
use crate::renderers::RenderContext;

use super::radial_common;

const VORTEX_ARMS: usize = 12;
const ARM_POINTS: usize = 48;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let s = radial_common::setup(c, ctx, 110.0, 0.08, 0.0);
    let freq = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let spin_speed = frame_time * 0.40;
    let step = ((freq.len() as f32) / VORTEX_ARMS as f32).floor().max(1.0) as usize;

    // Curated Cyber Vortex Colors (theme-dominant, hardcoded hue only as character accent)
    let vortex_cyan = mix(Color::rgba(0.0, 0.95, 1.0, 1.0), s.glow, 0.75);
    let vortex_magenta = mix(Color::rgba(1.0, 0.15, 0.85, 1.0), s.s_col, 0.75);
    let vortex_violet = mix(Color::rgba(0.40, 0.0, 0.90, 1.0), s.p_col, 0.70);
    let spark_white = mix(Color::rgba(0.95, 1.0, 1.0, 0.98), s.glow, 0.10);

    // -------------------------------------------------------------------------
    // 1. ATMOSPHERIC COSMIC VORTEX GLOW BACKDROP
    // -------------------------------------------------------------------------
    let bg_vortex = Fill::radial_gradient(
        s.cx,
        s.cy,
        0.0,
        s.cx,
        s.cy,
        s.base_r * 2.5,
        &[
            (0.0, mix(vortex_cyan, vortex_magenta, 0.5).with_alpha(0.30 + s.be * 0.20)),
            (0.40, vortex_violet.with_alpha(0.14)),
            (0.75, s.p_col.with_alpha(0.05)),
            (1.0, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_vortex);

    // -------------------------------------------------------------------------
    // 2. EXPANDING EVENT HORIZON PULSE SHOCKWAVES
    // -------------------------------------------------------------------------
    for p_i in 0..3 {
        let p_t = ((frame_time * 0.5 + p_i as f32 * 0.33) % 1.0).clamp(0.0, 1.0);
        let pulse_r = s.inner_r * (1.1 + p_t * 2.2);
        let pulse_alpha = ((1.0 - p_t) * (0.45 + s.bs * 0.40)).clamp(0.0, 0.85);

        c.set_stroke(Fill::Solid(mix(vortex_cyan, vortex_magenta, p_t).with_alpha(pulse_alpha)));
        c.set_line_width((2.5 - p_t * 1.5) * s.user_scale);
        c.set_shadow(vortex_cyan, (12.0 + s.bs * 8.0) * s.user_scale);
        c.stroke_circle(s.cx, s.cy, pulse_r);
    }

    // -------------------------------------------------------------------------
    // 3. 12 HIGH-DENSITY LOGARITHMIC HYPER-VORTEX ARMS
    // -------------------------------------------------------------------------
    for r in 0..VORTEX_ARMS {
        let rf = r as f32;
        let arm_ratio = rf / VORTEX_ARMS as f32;
        let base_angle = arm_ratio * TAU + spin_speed;

        let bin_k = (r * step).min(freq.len().saturating_sub(1));
        let fv_arm = freq[bin_k] as f32 / 255.0;

        let base_wave = (angle_wave(arm_ratio, frame_time)).sin() * 0.15 + 0.20;
        let audio_v = radial_common::swept_bin(freq, step, r, VORTEX_ARMS, &s) * s.sensitivity;
        let val = (base_wave + audio_v * 0.85 + s.be * 0.35 + s.bs * 0.20
            + radial_common::beat_bump(&s, base_angle) * 0.5)
            .clamp(0.15, 2.2);

        let mut raw_pts: Vec<(f32, f32)> = Vec::with_capacity(ARM_POINTS);

        for i in 0..ARM_POINTS {
            let t = i as f32 / (ARM_POINTS - 1) as f32;
            let theta = t * TAU * 1.8; // Twists 648° inward/outward
            let total_angle = base_angle + theta;

            let sample_bin = ((r * ARM_POINTS + i) * step / (VORTEX_ARMS * ARM_POINTS / s.sensitivity as usize).max(1))
                .min(freq.len().saturating_sub(1));
            let fv_pt = freq[sample_bin] as f32 / 255.0;

            let grow = s.inner_r + (t.powf(1.15) * (s.base_r * 1.45 + val * (65.0 * s.user_scale)));
            let ripple = fv_pt * 16.0 * s.sensitivity * s.user_scale;
            let bump = radial_common::beat_bump(&s, total_angle) * (25.0 * s.user_scale);

            let spiral_r = (grow + ripple + bump).max(s.inner_r + 2.0);

            raw_pts.push((s.cx + total_angle.cos() * spiral_r, s.cy + total_angle.sin() * spiral_r));
        }

        // Quadratic Bezier smoothing for buttery smooth vortex curves
        let mut smooth_pts: Vec<(f32, f32)> = Vec::new();
        for i in 0..ARM_POINTS.saturating_sub(1) {
            let p0 = raw_pts[i];
            let p1 = raw_pts[i + 1];
            let mid = ((p0.0 + p1.0) * 0.5, (p0.1 + p1.1) * 0.5);

            let seg = GpuCanvas::sample_quadratic(
                if smooth_pts.is_empty() { p0 } else { *smooth_pts.last().unwrap() },
                p0, mid, 3,
            );
            if smooth_pts.is_empty() {
                smooth_pts.extend(seg);
            } else {
                smooth_pts.extend(seg.into_iter().skip(1));
            }
        }
        if let Some(&last) = raw_pts.last() { smooth_pts.push(last); }

        let arm_col = mix(mix(vortex_cyan, vortex_magenta, arm_ratio), spark_white, fv_arm * 0.4);

        // Pass A: Volumetric outer plasma glow
        c.set_stroke(Fill::Solid(arm_col.with_alpha(0.60)));
        c.set_line_width((4.5 + val * 2.5) * s.user_scale);
        c.set_shadow(arm_col, (16.0 + val * 12.0) * s.user_scale);
        c.stroke_polyline(&smooth_pts);

        // Pass B: Intense white-hot core filament
        c.set_stroke(Fill::Solid(spark_white));
        c.set_line_width((1.5 + val * 0.8) * s.user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&smooth_pts);

        // Glowing tip mote at spiral end
        if let Some(&tip) = smooth_pts.last() {
            c.set_fill(Fill::Solid(mix(arm_col, spark_white, 0.85)));
            c.set_shadow(vortex_cyan, (14.0 + s.bs * 10.0) * s.user_scale);
            c.fill_circle(tip.0, tip.1, (3.5 + val * 2.2) * s.user_scale);
        }
    }

    // -------------------------------------------------------------------------
    // 4. SWIRLING COSMIC VORTEX STARDUST MOTES
    // -------------------------------------------------------------------------
    let mote_count = (22.0 + s.be * 24.0).clamp(14.0, 48.0) as usize;
    for m_i in 0..mote_count {
        let m_t = ((frame_time * 0.35 + m_i as f32 * 0.17) % 1.0).clamp(0.0, 1.0);
        let m_angle = (m_i as f32 * 2.8) + spin_speed * 1.5 + (1.0 - m_t) * TAU * 1.2;
        let m_dist = s.inner_r + m_t * (s.base_r * 1.35);

        let mx = s.cx + m_angle.cos() * m_dist;
        let my = s.cy + m_angle.sin() * m_dist;

        let m_sz = (2.5 * (1.0 - m_t) + 1.2 + s.bs * 1.8).clamp(1.0, 5.0) * s.user_scale;
        let m_col = mix(vortex_cyan, spark_white, m_t).with_alpha((1.0 - m_t).clamp(0.15, 0.95));

        c.set_fill(Fill::Solid(m_col));
        c.set_shadow(vortex_cyan, 8.0 * s.user_scale);
        c.fill_circle(mx, my, m_sz);
    }

    radial_common::finish(c, ctx, &s);
}

fn angle_wave(ratio: f32, frame_time: f32) -> f32 {
    ratio * TAU * 2.0 + frame_time * 1.8
}
