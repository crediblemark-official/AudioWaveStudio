//! Orbit Spike style renderer (`orbitSpike`).
//!
//! Recreates the exact black & white minimalist vector visualizer from the reference image:
//! A pure high-contrast white circular orbit ring with two sharp triangular arrowheads/spikes
//! attached at 180-degree opposite poles on a pitch black background.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const RING_PTS: usize = 120;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let rot = ctx.rotation_angle * 0.9;

  let center_x = width * 0.5;
  let center_y = height * 0.5;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Pure High-Contrast Dark Canvas
  c.set_fill(Fill::Solid(Color::BLACK));
  c.fill_rect(0.0, 0.0, width, height);

  let base_r = (width.min(height) * 0.23).clamp(80.0, 280.0);
  let r = base_r + (be * 18.0) + (bs * 8.0);
  let stroke_w = (base_r * 0.045).clamp(4.0, 12.0);

  // -------------------------------------------------------------------------
  // 1. PURE WHITE CIRCULAR ORBIT RING (EXACT REFERENCE GRAPHIC)
  // -------------------------------------------------------------------------
  let mut ring_pts = Vec::with_capacity(RING_PTS + 1);
  for i in 0..=RING_PTS {
    let angle = (i as f32 / RING_PTS as f32) * TAU;
    let px = center_x + angle.cos() * r;
    let py = center_y + angle.sin() * r;
    ring_pts.push((px, py));
  }

  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(stroke_w);
  c.stroke_polyline(&ring_pts);

  // -------------------------------------------------------------------------
  // 2. TWO SHARP TRIANGULAR ARROWHEADS / HORNS (180° OPPOSITE POLES)
  // -------------------------------------------------------------------------
  // In the reference image: Two sharp triangular arrowheads attached at opposite sides,
  // tapering smoothly outward from the ring.
  let spike_length = (base_r * 0.52 + be * 25.0).clamp(35.0, 180.0);
  let spike_base_w = (base_r * 0.22).clamp(16.0, 60.0);

  for &pole in &[0.0f32, std::f32::consts::PI] {
    let angle = rot + pole;
    let tang = angle + std::f32::consts::FRAC_PI_2;

    // Base point on ring
    let contact_x = center_x + angle.cos() * r;
    let contact_y = center_y + angle.sin() * r;

    // Tip of sharp arrow head
    let tip_x = contact_x + (angle.cos() * 0.65 + tang.cos() * 0.75) * spike_length;
    let tip_y = contact_y + (angle.sin() * 0.65 + tang.sin() * 0.75) * spike_length;

    // Two base points on ring boundary
    let b1_x = contact_x - tang.cos() * (spike_base_w * 0.5);
    let b1_y = contact_y - tang.sin() * (spike_base_w * 0.5);

    let b2_x = contact_x + angle.cos() * (spike_base_w * 0.4);
    let b2_y = contact_y + angle.sin() * (spike_base_w * 0.4);

    // Draw solid filled white triangle
    c.set_fill(Fill::Solid(Color::WHITE));
    c.fill_polyline_to_base(&[(b1_x, b1_y), (tip_x, tip_y), (b2_x, b2_y)], b1_y);

    c.set_stroke(Fill::Solid(Color::WHITE));
    c.set_line_width(2.0);
    c.stroke_polyline(&[(b1_x, b1_y), (tip_x, tip_y), (b2_x, b2_y), (b1_x, b1_y)]);
  }

  c.restore();
}
