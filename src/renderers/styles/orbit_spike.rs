//! Orbit Spike style renderer (`orbitSpike`).
//!
//! Recreates the EXACT vector emblem from the reference image:
//! A pure white circular ring on pitch black background with two smooth, organic
//! crescent/teardrop-shaped spikes at opposite poles that taper to sharp points.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const RING_PTS: usize = 120;
const SPIKE_CURVE_PTS: usize = 30;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle * 0.85;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let base_r = (width.min(height) * 0.25).clamp(85.0, 290.0);
  let r = base_r + (be * 16.0);
  let stroke_w = (base_r * 0.04).clamp(3.5, 10.0);

  // -------------------------------------------------------------------------
  // 1. FULL WHITE CIRCULAR RING
  // -------------------------------------------------------------------------
  let mut ring_pts = Vec::with_capacity(RING_PTS + 1);
  for i in 0..=RING_PTS {
    let angle = (i as f32 / RING_PTS as f32) * std::f32::consts::TAU;
    let bin = (i * freq.len() / RING_PTS).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let wave = fv * sensitivity * 8.0;
    let radius = r + wave;
    ring_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
  }

  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(stroke_w);
  c.stroke_polyline(&ring_pts);

  // -------------------------------------------------------------------------
  // 2. TWO SMOOTH ORGANIC CRESCENT/TEARDROP SPIKES (EXACT REFERENCE SHAPE)
  // -------------------------------------------------------------------------
  // Each spike is a smooth filled shape that:
  //   - Starts on the ring at angle (pole - spread)
  //   - Curves outward to a sharp tip far from the ring
  //   - Returns to the ring at angle (pole + spread)
  // The outer curve bulges out (teardrop) and tapers to a point.

  let spike_reach = (base_r * 0.50 + be * 25.0 + bs * 10.0).clamp(40.0, 180.0);
  let angular_spread = 0.38f32; // How wide the spike base spans on the ring (radians)

  for &pole in &[rot, rot + std::f32::consts::PI] {
    let mut spike_pts: Vec<(f32, f32)> = Vec::with_capacity(SPIKE_CURVE_PTS * 2 + 3);

    // --- Inner edge: follows the ring from (pole - spread) to pole ---
    for i in 0..=SPIKE_CURVE_PTS {
      let t = i as f32 / SPIKE_CURVE_PTS as f32;
      let angle = pole - angular_spread + t * angular_spread;

      // Smoothly lift off the ring as we approach the pole
      let lift = t * t; // quadratic ease-in
      let radius = r + lift * spike_reach;

      spike_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
    }

    // --- Sharp tip at the pole (furthest point) ---
    // Already included as last point above (t=1.0, angle=pole, radius=r+spike_reach)

    // --- Outer edge: from pole to (pole + spread), curving back to ring ---
    for i in 1..=SPIKE_CURVE_PTS {
      let t = i as f32 / SPIKE_CURVE_PTS as f32;
      let angle = pole + t * angular_spread;

      // Smoothly return to ring: reverse quadratic
      let lift = (1.0 - t) * (1.0 - t); // quadratic ease-out
      let radius = r + lift * spike_reach;

      spike_pts.push((center_x + angle.cos() * radius, center_y + angle.sin() * radius));
    }

    // Close the shape
    spike_pts.push(spike_pts[0]);

    // Fill the smooth teardrop spike solid white
    c.set_fill(Fill::Solid(Color::WHITE));
    let base_y = spike_pts.iter().map(|p| p.1).fold(f32::NAN, f32::max);
    c.fill_polyline_to_base(&spike_pts, base_y);

    // Stroke outline to ensure clean edges
    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.0);
    c.stroke_polyline(&spike_pts);
  }

  c.restore();
}
