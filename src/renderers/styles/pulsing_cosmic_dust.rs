//! Pulsing Cosmic Dust style renderer (`pulsingCosmicDust`).
//!
//! Replaces the generic "bars + floating motes" with a completely
//! different concept: Stellar Constellation Web.
//!
//! Visual concept:
//! - Frequencies map to star nodes on a ring.
//! - High-amplitude nodes glow brighter and larger.
//! - Adjacent nodes and cross-circle nodes are connected by translucent
//!   constellation lines whose brightness tracks the min amplitude of both endpoints.
//! - No radial bars — purely a point + line constellation star map.
//! - Pumping central logo disc on bass beats.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::helpers::{draw_radial_center_image, mix};
use crate::renderers::{
    theme_accent, theme_glow, theme_primary, theme_secondary, RenderContext,
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
    let bar_count    = ctx.config.reactivity.bar_count.clamp(8, 48);

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

    let step = ((freq.len() as f32) / bar_count as f32).floor().max(1.0) as usize;
    let rot = frame_time * 0.08;

    // -------------------------------------------------------------------------
    // 1. BUILD STAR NODE POSITIONS & AMPLITUDES
    // -------------------------------------------------------------------------
    let mut star_pts: Vec<(f32, f32)> = Vec::with_capacity(bar_count);
    let mut star_vals: Vec<f32> = Vec::with_capacity(bar_count);

    for i in 0..bar_count {
        let t = i as f32 / bar_count as f32;
        let angle = t * TAU + rot;

        let base_wave = (frame_time * 1.8 + i as f32).sin() * 0.12 + 0.18;
        let audio_v = crate::renderers::styles::radial_common::full_random_scattered_bin(
            freq, step, i, bar_count, ctx.beat_count,
        ) * sensitivity;
        let val = (base_wave + audio_v * 0.85 + be * 0.30).clamp(0.10, 1.8);

        // Star node floats slightly in/out based on amplitude
        let star_r = inner_r + (20.0 + val * 80.0) * user_scale;
        star_pts.push((cx + angle.cos() * star_r, cy + angle.sin() * star_r));
        star_vals.push(val);
    }

    // -------------------------------------------------------------------------
    // 2. CONSTELLATION LINES — adjacent + cross-connections
    // -------------------------------------------------------------------------
    let n = bar_count;

    // Adjacent edges
    for i in 0..n {
        let j = (i + 1) % n;
        let line_alpha = (star_vals[i].min(star_vals[j]) * 0.6).clamp(0.05, 0.8);
        let line_col = mix(p_col, glow, i as f32 / n as f32).with_alpha(line_alpha);

        c.set_stroke(Fill::Solid(line_col));
        c.set_line_width(1.2 * user_scale);
        c.stroke_line(star_pts[i].0, star_pts[i].1, star_pts[j].0, star_pts[j].1);
    }

    // Cross connections (every n/4 skip for a star polygon pattern)
    let skip = (n / 4).max(2);
    for i in 0..n {
        let j = (i + skip) % n;
        let line_alpha = (star_vals[i].min(star_vals[j]) * 0.35).clamp(0.03, 0.50);
        let line_col = accent.with_alpha(line_alpha);

        c.set_stroke(Fill::Solid(line_col));
        c.set_line_width(0.8 * user_scale);
        c.stroke_line(star_pts[i].0, star_pts[i].1, star_pts[j].0, star_pts[j].1);
    }

    // -------------------------------------------------------------------------
    // 3. GLOWING STAR NODES
    // -------------------------------------------------------------------------
    for i in 0..n {
        let val = star_vals[i];
        let star_col = mix(p_col, Color::WHITE, (val * 0.6).min(0.9));

        c.set_fill(Fill::Solid(star_col));
        c.set_shadow(star_col, (8.0 + val * 12.0) * user_scale);
        c.fill_circle(star_pts[i].0, star_pts[i].1, (2.5 + val * 3.5) * user_scale);
    }

    // -------------------------------------------------------------------------
    // 4. PUMPING CENTRAL DISC & NEON RING
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
