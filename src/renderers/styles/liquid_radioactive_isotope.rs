//! Liquid Solar Wind Particle Storm style renderer (`liquidRadioactiveIsotope`).
//!
//! Visual Concept:
//! - Roaring 360° Solar Corona & Coronal Mass Ejection (CME) particle storm.
//! - Magnetic arch loops flexing & snapping around the central Solar Core.
//! - Spiral solar wind streams shooting outward with high-velocity plasma particle tails.
//! - Magnetospheric aurora shockwaves radiating on drum beats.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

const PLASMA_STREAMS: usize = 36;
const MAGNETIC_LOOPS: usize = 12;
const SOLAR_MOTES:    usize = 50;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col      = theme_primary(theme);
    let s_col      = theme_secondary(theme);
    let accent_col = theme_accent(theme);
    let glow_col   = theme_glow(theme);

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
    let cy = height * 0.5 - pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.08 + bs * 0.04);

    // Radiant Solar Wind palette
    let solar_crimson = mix(p_col, Color::rgba(1.0, 0.12, 0.0, 1.0), 0.80);
    let solar_amber   = mix(accent_col, Color::rgba(1.0, 0.65, 0.0, 1.0), 0.85);
    let solar_gold    = Color::rgba(1.0, 0.90, 0.20, 1.0);
    let plasma_white  = Color::rgba(1.0, 0.98, 0.92, 0.95);
    let aurora_cyan   = mix(glow_col, Color::rgba(0.0, 0.85, 1.0, 1.0), 0.75);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. Deep Space & Solar Corona Volumetric Glow
    // -------------------------------------------------------------------------
    let bg_corona = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, base_r * 3.5,
        &[
            (0.00, solar_amber.with_alpha(0.40 + be * 0.20)),
            (0.35, solar_crimson.with_alpha(0.22 + bs * 0.12)),
            (0.70, aurora_cyan.with_alpha(0.10)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(bg_corona);
    c.fill_rect(0.0, 0.0, width, height);

    // Coronal Mass Ejection (CME) Beat Shockwave Ring
    if bs > 0.10 {
        let shock_r = inner_r + bs * 160.0 * user_scale;
        c.set_stroke(Fill::Solid(solar_gold.with_alpha((0.65 * (1.0 - bs / 3.0)).max(0.0))));
        c.set_line_width((3.5 + bs * 2.5) * user_scale);
        c.set_shadow(solar_amber, (18.0 + bs * 12.0) * user_scale);
        c.stroke_circle(cx, cy, shock_r);
    }

    // -------------------------------------------------------------------------
    // 2. Magnetic Loop Arches flexing around the Sun Core
    // -------------------------------------------------------------------------
    let step_loop = (freq.len() / MAGNETIC_LOOPS).max(1);
    let rot_loop = frame_time * 0.10;

    for m in 0..MAGNETIC_LOOPS {
        let mf = m as f32;
        let a0 = (mf / MAGNETIC_LOOPS as f32) * TAU + rot_loop;
        let a1 = ((mf + 0.65) / MAGNETIC_LOOPS as f32) * TAU + rot_loop;
        let mid_a = (a0 + a1) * 0.5;

        let fv  = smooth_ring_bin(freq, step_loop, m, MAGNETIC_LOOPS);
        let val = (fv * sensitivity * 1.1 + be * 0.35 + bs * 0.25).clamp(0.0, 2.2);

        let loop_h = inner_r + (25.0 + val * 65.0) * user_scale;

        let (cos0, sin0) = a0.sin_cos();
        let (cos1, sin1) = a1.sin_cos();
        let (cosm, sinm) = mid_a.sin_cos();

        let p0   = (cx + cos0 * inner_r, cy + sin0 * inner_r);
        let pmid = (cx + cosm * loop_h,  cy + sinm * loop_h);
        let p1   = (cx + cos1 * inner_r, cy + sin1 * inner_r);

        let loop_pts = GpuCanvas::sample_quadratic(p0, pmid, p1, 16);

        // Outer glowing arch stroke
        let arch_alpha = (0.50 + val * 0.40).clamp(0.20, 0.90);
        c.set_stroke(Fill::Solid(mix(solar_crimson, solar_amber, mf / MAGNETIC_LOOPS as f32).with_alpha(arch_alpha)));
        c.set_line_width((2.8 + val * 2.0) * user_scale);
        c.set_shadow(solar_amber, (12.0 + val * 8.0) * user_scale);
        c.stroke_polyline(&loop_pts);

        // Core white-hot filament
        c.set_stroke(Fill::Solid(plasma_white.with_alpha(arch_alpha * 0.9)));
        c.set_line_width(1.2 * user_scale);
        c.set_shadow(Color::TRANSPARENT, 0.0);
        c.stroke_polyline(&loop_pts);
    }

    // -------------------------------------------------------------------------
    // 3. 360° Spiral Solar Wind Plasma Streams
    // -------------------------------------------------------------------------
    let step_stream = (freq.len() / PLASMA_STREAMS).max(1);
    let rot_stream  = frame_time * 0.15;

    for i in 0..PLASMA_STREAMS {
        let sf = i as f32;
        let base_angle = (sf / PLASMA_STREAMS as f32) * TAU + rot_stream;

        let fv  = smooth_ring_bin(freq, step_stream, i, PLASMA_STREAMS);
        let val = (fv * sensitivity * 1.2 + be * 0.30 + bs * 0.20).clamp(0.05, 2.5);

        let stream_length = base_r * (0.80 + val * 1.10) * user_scale;

        // Curved spiral trajectory points
        let mut stream_pts: Vec<(f32, f32)> = Vec::with_capacity(12);
        for seg in 0..12 {
            let st = seg as f32 / 11.0;
            let r_cur = inner_r + st * stream_length;
            // Spiral twist angle increases radially
            let twist = (st * 0.45) + (st * frame_time * 0.3).sin() * 0.15;
            let ang   = base_angle + twist;

            let (sin_a, cos_a) = ang.sin_cos();
            stream_pts.push((cx + cos_a * r_cur, cy + sin_a * r_cur));
        }

        // Stream color transition: solar white -> gold -> crimson -> aurora cyan tip
        let stream_col = match i % 3 {
            0 => solar_gold,
            1 => solar_amber,
            _ => aurora_cyan,
        };

        c.set_stroke(Fill::Solid(stream_col.with_alpha((0.45 + val * 0.40).clamp(0.15, 0.85))));
        c.set_line_width((2.4 + val * 1.8) * user_scale);
        c.set_shadow(stream_col, (10.0 + val * 8.0) * user_scale);
        c.stroke_polyline(&stream_pts);

        // Leading plasma head particle
        if let Some(&tip) = stream_pts.last() {
            c.set_fill(Fill::Solid(plasma_white));
            c.set_shadow(solar_gold, (12.0 + val * 8.0) * user_scale);
            c.fill_circle(tip.0, tip.1, (2.2 + val * 2.0) * user_scale);
        }
    }

    // -------------------------------------------------------------------------
    // 4. Solar Prominence Particles & Solar Dust Motes
    // -------------------------------------------------------------------------
    for d in 0..SOLAR_MOTES {
        let df = d as f32;
        let d_angle = (df / SOLAR_MOTES as f32) * TAU + frame_time * (0.06 + (df * 1.3).sin() * 0.02);
        let phase   = ((frame_time * 0.35 + df * 0.27) % 1.0).clamp(0.0, 1.0);

        let dist = inner_r + phase * (base_r * 2.0);
        let (sin_a, cos_a) = d_angle.sin_cos();

        let mote_x = cx + cos_a * dist;
        let mote_y = cy + sin_a * dist;
        let alpha  = (1.0 - phase) * (0.50 + bs * 0.35);

        if alpha > 0.05 {
            let mote_sz = (1.5 + (1.0 - phase) * 3.0 + be * 1.5) * user_scale;
            c.set_fill(Fill::Solid(mix(solar_gold, plasma_white, phase).with_alpha(alpha)));
            c.set_shadow(solar_amber, (6.0 + be * 4.0) * user_scale);
            c.fill_circle(mote_x, mote_y, mote_sz);
        }
    }

    // -------------------------------------------------------------------------
    // 5. Central Solar Core Disc Integration
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(solar_gold));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(solar_gold, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#120502")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = (s_col, accent_col);
    c.set_global_alpha(1.0);
    c.restore();
}

