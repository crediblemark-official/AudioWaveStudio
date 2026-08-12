//! Pulsing Polar Rose style renderer (`pulsingShockwave`).
//!
//! Replaces the generic "spectrum bars + shockwave rings" with a completely
//! different concept: Audio Polar Rose Waveform.
//!
//! Visual concept:
//! - The full spectrum is mapped to r(theta) = base_r + amplitude * sin(n*theta),
//!   producing a continuous smooth petal-rose curve that reshapes based on sound.
//! - Number of petals controlled by Bar Count slider.
//! - Ring glows and breathes with bass. Petal tips glow on peaks.
//! - Pumping central logo disc scaling on bass beats.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, fill_radial_polygon, mix};
use crate::renderers::{
    bin_value, theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
};

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
    let width  = ctx.width;
    let height = ctx.height;
    let theme  = &ctx.config.theme;

    let p_col  = theme_primary(theme);
    let s_col  = theme_secondary(theme);
    let accent = theme_accent(theme);
    let glow   = theme_glow(theme);

    let sensitivity  = ctx.config.reactivity.sensitivity;
    let bass_mult    = ctx.config.reactivity.bass_multiplier;
    let user_scale   = ctx.config.scale.clamp(0.1, 5.0);
    let pos_offset_x = ctx.config.position_x * width * 0.5;
    let pos_offset_y = ctx.config.position_y * height * 0.5;
    let bar_count    = ctx.config.reactivity.bar_count.clamp(3, 16) as f32;

    let be = (ctx.bass_energy * bass_mult).clamp(0.0, 3.0);
    let bs = (ctx.beat_strength * bass_mult).clamp(0.0, 3.0);
    let freq       = ctx.freq_data;
    let frame_time = ctx.frame_time;

    let cx = width * 0.5 + pos_offset_x;
    let cy = height * 0.5 + pos_offset_y;

    let reference_size = width.min(height);
    let base_r  = 115.0 * (reference_size / 500.0) * user_scale;
    let inner_r = base_r * (0.35 + be * 0.10 + bs * 0.05);

    c.save();
    c.set_shadow(Color::TRANSPARENT, 0.0);

    // -------------------------------------------------------------------------
    // 1. POLAR ROSE WAVEFORM — r(θ) = base + Σfreq * sin(n*θ + phase)
    // -------------------------------------------------------------------------
    let num_sample = 256usize;
    let rot = frame_time * 0.12;

    // Build smooth amplitude modulation from full spectrum
    let step = ((freq.len() as f32) / bar_count).floor().max(1.0) as usize;
    let mut amplitudes: Vec<f32> = Vec::with_capacity(bar_count as usize);
    for k in 0..(bar_count as usize) {
        let audio_v = bin_value(freq, step, k) * sensitivity;
        let amp = (audio_v * 0.85 + be * 0.25).clamp(0.0, 1.8);
        amplitudes.push(amp);
    }

    let mut rose_pts: Vec<(f32, f32)> = Vec::with_capacity(num_sample + 1);
    for i in 0..=num_sample {
        let theta = (i as f32 / num_sample as f32) * TAU + rot;

        // Sum harmonic petals weighted by frequency amplitude
        let mut r = inner_r;
        for (k, &amp) in amplitudes.iter().enumerate() {
            let n = (k + 1) as f32;
            r += amp * (45.0 * user_scale) * (n * theta).sin().abs();
        }

        rose_pts.push((cx + theta.cos() * r, cy + theta.sin() * r));
    }

    // Filled rose with gradient
    let rose_fill = Fill::radial_gradient(
        cx, cy, inner_r,
        cx, cy, base_r * 2.8,
        &[
            (0.00, p_col.with_alpha(0.80)),
            (0.55, accent.with_alpha(0.50)),
            (1.00, Color::TRANSPARENT),
        ],
    );
    c.set_fill(rose_fill);
    c.set_shadow(glow, (18.0 + bs * 14.0) * user_scale);
    fill_radial_polygon(c, cx, cy, &rose_pts);

    // Rose outline stroke
    c.set_stroke(Fill::Solid(mix(glow, Color::WHITE, 0.55)));
    c.set_line_width((2.2 + bs * 1.5) * user_scale);
    c.stroke_polyline(&rose_pts);

    // -------------------------------------------------------------------------
    // 2. PUMPING CENTRAL DISC & NEON RING
    // -------------------------------------------------------------------------
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.set_stroke(Fill::Solid(glow));
    c.set_line_width((3.5 + be * 2.5) * user_scale);
    c.set_shadow(glow, (20.0 + bs * 15.0) * user_scale);
    c.stroke_circle(cx, cy, inner_r);

    c.set_fill(Fill::Solid(Color::hex("#000000")));
    c.set_shadow(Color::TRANSPARENT, 0.0);
    c.fill_circle(cx, cy, inner_r * 0.96);

    draw_radial_center_image(c, ctx, cx, cy, inner_r * 0.90);

    let _ = s_col;
    c.set_global_alpha(1.0);
    c.restore();
}
