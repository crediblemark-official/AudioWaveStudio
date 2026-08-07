//! Orbit Spike style renderer (`orbitSpike`).
//!
//! Renders a sleek modern audio orbit ring featuring two sharp opposing arrow/horn spikes
//! at 180-degree opposite poles that spin continuously, pulse with bass hits,
//! and modulate along audio frequency spectrum waves.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const RING_POINTS: usize = 120;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;
  let theme = &ctx.config.theme;
  let p = crate::renderers::theme_primary(theme);
  let s = crate::renderers::theme_secondary(theme);

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle * 0.8;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  let base_r = (width.min(height) * 0.22).clamp(70.0, 260.0);
  let orbit_r = base_r + (be * 35.0) + (bs * 15.0);

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. FREQUENCY-MODULATED CENTRAL ORBIT RING
  // -------------------------------------------------------------------------
  let step = (freq.len() / (RING_POINTS / 2)).max(1);
  let mut ring_pts = Vec::with_capacity(RING_POINTS + 1);

  for i in 0..=RING_POINTS {
    let angle = (i as f32 / RING_POINTS as f32) * TAU + rot;

    let bin_i = if i <= RING_POINTS / 2 {
      (i * step).min(freq.len().saturating_sub(1))
    } else {
      ((RING_POINTS - i) * step).min(freq.len().saturating_sub(1))
    };

    let raw_v = *freq.get(bin_i).unwrap_or(&0) as f32 / 255.0;
    let wave_mod = raw_v * sensitivity * 24.0;

    let r = orbit_r + wave_mod;
    let px = center_x + angle.cos() * r;
    let py = center_y + angle.sin() * r;
    ring_pts.push((px, py));
  }

  // Outer Glow Shadow
  c.set_stroke(Fill::Solid(p));
  c.set_line_width(5.0 + bs * 3.0);
  c.set_shadow(p.with_alpha(0.85), 18.0 + bs * 8.0);
  c.stroke_polyline(&ring_pts);

  // Inner Core Bright White Line
  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(3.0);
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.stroke_polyline(&ring_pts);

  // -------------------------------------------------------------------------
  // 2. TWO OPPOSING SHARP ARROW / HORN SPIKES (180 DEGREES APART)
  // -------------------------------------------------------------------------
  let spike_len = (base_r * 0.45 + be * 40.0 + bs * 20.0).clamp(25.0, 160.0);
  let spike_width = (base_r * 0.16).clamp(12.0, 36.0);

  for &pole in &[0.0f32, std::f32::consts::PI] {
    let pole_angle = rot + pole;

    // Contact point on orbit ring
    let base_cx = center_x + pole_angle.cos() * orbit_r;
    let base_cy = center_y + pole_angle.sin() * orbit_r;

    // Tangent angle along circle movement
    let tangent_angle = pole_angle + std::f32::consts::FRAC_PI_2;

    // Sharp pointed tip location
    let tip_x = base_cx + (pole_angle.cos() * 0.7 + tangent_angle.cos() * 0.7) * spike_len;
    let tip_y = base_cy + (pole_angle.sin() * 0.7 + tangent_angle.sin() * 0.7) * spike_len;

    // Base corners of the arrow spike
    let norm_angle = pole_angle;
    let b1_x = base_cx - norm_angle.cos() * (spike_width * 0.5);
    let b1_y = base_cy - norm_angle.sin() * (spike_width * 0.5);
    let b2_x = base_cx + norm_angle.cos() * (spike_width * 0.5);
    let b2_y = base_cy + norm_angle.sin() * (spike_width * 0.5);

    // Render sharp pointed arrow spike (matching user photo)
    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.5);
    c.set_shadow(s.with_alpha(0.9), 16.0);

    c.stroke_polyline(&[(b1_x, b1_y), (tip_x, tip_y), (b2_x, b2_y), (b1_x, b1_y)]);
  }

  // -------------------------------------------------------------------------
  // 3. INNER CORE PULSING DOTS & ORBITING PARTICLES
  // -------------------------------------------------------------------------
  let core_r = (orbit_r * 0.25 + be * 10.0).clamp(10.0, 60.0);
  c.set_stroke(Fill::Solid(s.with_alpha(0.6)));
  c.set_line_width(1.5);
  c.stroke_circle(center_x, center_y, core_r);

  c.restore();
}
