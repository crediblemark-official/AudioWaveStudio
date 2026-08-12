//! Liquid Molten Mercury style renderer (`liquidMoltenMercury`).
//!
//! Visual Concept:
//! - Masterpiece Flowing Liquid Chrome & Molten Metal Stream Engine.
//! - 5 Dynamic Horizontal/Diagonal Liquid Mercury Currents waving smoothly across the screen.
//! - High-gloss metallic specular highlights, deep steel blue shading, & white-hot liquid metal sheens.
//! - Audio-reactive swell amplitude, metallic ripple waves, & floating liquid mercury droplets.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{fill_radial_polygon, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};
use crate::renderers::styles::radial_common::smooth_ring_bin;

const MERCURY_STREAMS: usize = 5;
const STREAM_SAMPLES:  usize = 80;
const MERCURY_DROPLETS: usize = 40;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col  = theme_primary(theme);
    let s_col  = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow   = theme_glow(theme);

    let sensitivity = ctx.config.reactivity.sensitivity;
    let bass_mult   = ctx.config.reactivity.bass_multiplier;
    let user_scale  = ctx.config.scale.clamp(0.1, 5.0);

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5;
    let cy = height * 0.5;

    // Liquid Metal Chrome Palette
    let chrome_silver = mix(glow, Color::rgba(0.88, 0.94, 1.0, 1.0), 0.85);  // High-gloss liquid silver
    let hot_amber     = mix(accent, Color::rgba(1.0, 0.70, 0.20, 1.0), 0.75); // White-hot molten core
    let steel_dark    = mix(s_col, Color::rgba(0.10, 0.12, 0.22, 1.0), 0.85);  // Deep metallic shadow
    let chrome_glint   = Color::rgba(1.0, 1.0, 1.0, 0.95);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. Volumetric Metallic Atmosphere & Backdrop Glow
    // -------------------------------------------------------------------------
    let bg_mercury = Fill::radial_gradient(
        cx, cy, 0.0,
        cx, cy, width.max(height) * 0.85,
        &[
            (0.00, mix(chrome_silver, hot_amber, 0.4).with_alpha(0.30 + be * 0.15)),
            (0.45, steel_dark.with_alpha(0.20 + bs * 0.10)),
            (1.00, Color::rgba(0.02, 0.02, 0.06, 1.0)),
        ],
    );
    c.set_fill(bg_mercury);
    c.fill_rect(0.0, 0.0, width, height);

    // Metallic Ripple Shockwaves on Beat
    if bs > 0.12 {
        let shock_r = (90.0 + bs * 220.0) * user_scale;
        c.set_stroke(Fill::Solid(chrome_silver.with_alpha((0.50 * (1.0 - bs / 3.0)).max(0.0))));
        c.set_line_width((3.0 + bs * 2.0) * user_scale);
        c.set_shadow(chrome_silver, (16.0 + bs * 10.0) * user_scale);
        c.stroke_circle(cx, cy, shock_r);
    }

    // -------------------------------------------------------------------------
    // 2. Flowing Liquid Chrome Ribbon Currents Across the Canvas
    // -------------------------------------------------------------------------
    let step = (freq.len() / (MERCURY_STREAMS * STREAM_SAMPLES)).max(1);

    for stream_idx in 0..MERCURY_STREAMS {
        let sf = stream_idx as f32;
        let base_y = height * (0.18 + sf * 0.16);
        let stream_h = (45.0 + (sf * 1.3).sin().abs() * 25.0) * user_scale;

        let mut top_pts: Vec<(f32, f32)> = Vec::with_capacity(STREAM_SAMPLES + 1);
        let mut bot_pts: Vec<(f32, f32)> = Vec::with_capacity(STREAM_SAMPLES + 1);

        for i in 0..=STREAM_SAMPLES {
            let t = i as f32 / STREAM_SAMPLES as f32;
            let px = t * (width + 100.0) - 50.0;

            let bin_idx = (stream_idx * STREAM_SAMPLES + i) % (freq.len() / step.max(1)).max(1);
            let fv = smooth_ring_bin(freq, step, bin_idx, STREAM_SAMPLES);
            let val = (fv * sensitivity * 1.1 + be * 0.35 + bs * 0.20).clamp(0.05, 2.5);

            // Sine wave liquid current motion
            let wave1 = (t * TAU * 2.5 + frame_time * 2.2 + sf * 1.4).sin() * (24.0 * user_scale);
            let wave2 = (t * TAU * 5.0 - frame_time * 3.1).cos() * (12.0 * user_scale);
            let audio_swell = val * 55.0 * user_scale;

            let curr_y = base_y + wave1 + wave2;
            let half_h = (stream_h + audio_swell) * 0.5;

            top_pts.push((px, curr_y - half_h));
            bot_pts.push((px, curr_y + half_h));
        }

        // Construct closed liquid ribbon polygon
        let mut ribbon_polygon = Vec::with_capacity(top_pts.len() + bot_pts.len() + 1);
        ribbon_polygon.extend(top_pts.iter().copied());
        ribbon_polygon.extend(bot_pts.iter().rev().copied());
        if let Some(&first) = ribbon_polygon.first() { ribbon_polygon.push(first); }

        let stream_col = match stream_idx % 3 {
            0 => chrome_silver,
            1 => hot_amber,
            _ => mix(p_col, chrome_silver, 0.6),
        };

        // Metallic Gloss Radial Gradient Fill
        let ribbon_fill = Fill::radial_gradient(
            cx, base_y, 0.0,
            cx, base_y, width * 0.6,
            &[
                (0.00, mix(stream_col, chrome_glint, 0.60).with_alpha(0.88)),
                (0.40, stream_col.with_alpha(0.75 + be * 0.12)),
                (0.80, steel_dark.with_alpha(0.50)),
                (1.00, Color::TRANSPARENT),
            ],
        );
        c.set_fill(ribbon_fill);
        c.set_shadow(stream_col, (18.0 + bs * 10.0) * user_scale);
        fill_radial_polygon(c, cx, base_y, &ribbon_polygon);

        // White-Hot Metallic Sheen Highlight Along Top Edge
        c.set_stroke(Fill::Solid(mix(stream_col, chrome_glint, 0.75).with_alpha(0.90)));
        c.set_line_width((2.6 + bs * 1.4) * user_scale);
        c.set_shadow(chrome_glint, (12.0 + be * 8.0) * user_scale);
        c.stroke_polyline(&top_pts);
    }

    // -------------------------------------------------------------------------
    // 4. Detached Molten Mercury Droplets Floating Across Canvas
    // -------------------------------------------------------------------------
    for d in 0..MERCURY_DROPLETS {
        let df = d as f32;
        let dx = (df * 137.5).sin() * 0.48 * width + cx;
        let dy = ((df * 29.3 + frame_time * 22.0) % (height + 40.0)) - 20.0;

        let drop_size  = (2.2 + (df * 7.3).sin().abs() * 4.2 + be * 2.0) * user_scale;
        let drop_alpha = (0.50 + bs * 0.40 + (df * 3.1).sin() * 0.15).clamp(0.15, 0.95);

        let drop_col = match d % 3 {
            0 => hot_amber,
            1 => chrome_silver,
            _ => mix(glow, chrome_silver, 0.7),
        };

        c.set_fill(Fill::Solid(mix(drop_col, Color::WHITE, 0.50).with_alpha(drop_alpha)));
        c.set_shadow(drop_col, (10.0 + be * 6.0) * user_scale);
        c.fill_circle(dx, dy, drop_size);
    }

    let _ = s_col;
    c.set_global_alpha(1.0);
    c.restore();
}

