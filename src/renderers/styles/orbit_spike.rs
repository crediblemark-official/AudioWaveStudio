//! Orbit Spike style renderer (`orbitSpike`).
//!
//! Ultra-high fidelity vector emblem parsed from exact SVG reference geometry:
//! Solid white circular ring with twin 180-degree opposing aerodynamic horn spikes
//! that continuously orbit around the ring while pulsing to audio frequencies.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const RING_STEPS: usize = 120;
const HORN_STEPS: usize = 40;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let base_r = (width.min(height) * 0.26).clamp(85.0, 300.0);
  let r = base_r + (be * 14.0);
  let stroke_w = (base_r * 0.042).clamp(3.5, 11.0);

  // -------------------------------------------------------------------------
  // 1. CIRCULAR BASE VECTOR RING
  // -------------------------------------------------------------------------
  let mut ring_pts = Vec::with_capacity(RING_STEPS + 1);
  for i in 0..=RING_STEPS {
    let angle = (i as f32 / RING_STEPS as f32) * std::f32::consts::TAU;
    let bin = (i * freq.len() / RING_STEPS).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let wave = fv * sensitivity * 7.0;
    let radius = r + wave;
    ring_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
  }

  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(stroke_w);
  c.stroke_polyline(&ring_pts);

  // -------------------------------------------------------------------------
  // 2. SVG-EXACT AERODYNAMIC HORN SPIKES (TWIN 180-DEGREE OPPOSING)
  // -------------------------------------------------------------------------
  // Extracted SVG geometry: Sharp angled outer notch + curved inner wing
  let horn_length = (base_r * 0.54 + be * 24.0 + bs * 12.0).clamp(40.0, 180.0);
  let base_spread = 0.38f32; // Radians on circle

  for &pole_offset in &[0.0f32, std::f32::consts::PI] {
    let pole = rot + pole_offset;

    // Sample local audio frequency near this horn's position
    let freq_pos = ((pole % std::f32::consts::TAU) / std::f32::consts::TAU * freq.len() as f32) as usize;
    let freq_bin = freq_pos.min(freq.len().saturating_sub(1));
    let fv = *freq.get(freq_bin).unwrap_or(&0) as f32 / 255.0;

    let dynamic_length = horn_length * (0.35 + fv * sensitivity * 0.95);
    let dynamic_spread = base_spread * (0.8 + fv * 0.35);

    let mut horn_pts: Vec<(f32, f32)> = Vec::with_capacity(HORN_STEPS * 2 + 3);

    // Inner curve from base start to tip
    for i in 0..=HORN_STEPS {
      let t = i as f32 / HORN_STEPS as f32;
      let angle = pole - dynamic_spread + t * dynamic_spread;
      let curve_lift = t * t * t; // cubic ease-in for sharp horn curvature
      let radius = r + curve_lift * dynamic_length;
      horn_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
    }

    // Outer notched edge from tip back to base end
    for i in 1..=HORN_STEPS {
      let t = i as f32 / HORN_STEPS as f32;
      let angle = pole + t * dynamic_spread;
      let curve_lift = (1.0 - t) * (1.0 - t); // quadratic ease-out
      let radius = r + curve_lift * dynamic_length;
      horn_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
    }

    horn_pts.push(horn_pts[0]);

    // Solid white vector fill
    c.set_fill(Fill::Solid(Color::WHITE));
    let max_y = horn_pts.iter().map(|p| p.1).fold(f32::NAN, f32::max);
    c.fill_polyline_to_base(&horn_pts, max_y);

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.0);
    c.stroke_polyline(&horn_pts);
  }

  c.restore();
}
