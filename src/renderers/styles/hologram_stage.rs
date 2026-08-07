//! Hologram Stage style renderer (`hologramStage`).
//!
//! Complete overhaul: 3D Hologram Cylinder Podium
//! - Outer cyan dashed floor ring on floor base
//! - 14 stacked neon magenta rings forming a 3D cylindrical stage column
//! - Inner vertical hologram light column (beam projection)
//! - Top rim neon spotlight glow with bass reactivity

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let base_y = height * 0.64; // Floor base

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.95);
  let cyan_col = Color::rgba(0.0, 0.88, 1.0, 0.95);

  let tilt_y = 0.32f32; // 3D perspective tilt
  let base_rx = (width.min(height) * 0.28).clamp(95.0, 320.0);

  // -------------------------------------------------------------------------
  // 1. OUTER CYAN DASHED FLOOR RING (ON FLOOR BASE)
  // -------------------------------------------------------------------------
  let floor_rx = base_rx * 1.35 + (be * 10.0);
  let floor_ry = floor_rx * tilt_y;
  let dash_count = 24;

  for d in 0..dash_count {
    if d % 2 == 0 {
      let a1 = -rot * 0.8 + (d as f32 / dash_count as f32) * TAU;
      let a2 = -rot * 0.8 + ((d as f32 + 0.6) / dash_count as f32) * TAU;

      let mut pts = Vec::with_capacity(10);
      for k in 0..10 {
        let angle = a1 + (k as f32 / 9.0) * (a2 - a1);
        pts.push((center_x + angle.cos() * floor_rx, base_y + angle.sin() * floor_ry));
      }

      c.set_stroke(Fill::Solid(cyan_col));
      c.set_line_width(3.2);
      c.set_shadow(cyan_col, 10.0);
      c.stroke_polyline(&pts);
    }
  }

  // -------------------------------------------------------------------------
  // 2. INNER VERTICAL HOLOGRAM LIGHT BEAM COLUMN
  // -------------------------------------------------------------------------
  let cylinder_h = (height * 0.26).clamp(55.0, 190.0);
  let top_y = base_y - cylinder_h;

  // Floor base disc glow
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.65, 0.15)));
  c.set_shadow(hot_pink, 20.0 + bs * 10.0);
  c.fill_ellipse(center_x, base_y, base_rx, base_rx * tilt_y);

  // Transparent hologram beam projection column
  let beam_pts = [
    (center_x - base_rx * 0.8, base_y),
    (center_x - base_rx * 0.7, top_y),
    (center_x + base_rx * 0.7, top_y),
    (center_x + base_rx * 0.8, base_y),
  ];
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.65, 0.07)));
  c.fill_polygon(&beam_pts);

  // -------------------------------------------------------------------------
  // 3. STACKED NEON MAGENTA CYLINDER RINGS (14 LAYERS)
  // -------------------------------------------------------------------------
  let num_layers = 14;
  let layer_spacing = cylinder_h / (num_layers - 1) as f32;

  for l in 0..num_layers {
    let l_ratio = l as f32 / (num_layers - 1) as f32;
    let ly = base_y - l as f32 * layer_spacing;

    let bin = (l * freq.len() / num_layers).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

    let rx = base_rx + fv * 12.0 * sensitivity;
    let ry = rx * tilt_y;

    let mut ring_pts = Vec::with_capacity(42);
    for k in 0..=40 {
      let a = (k as f32 / 40.0) * TAU;
      ring_pts.push((center_x + a.cos() * rx, ly + a.sin() * ry));
    }

    let alpha = 0.50 + (l_ratio - 0.5).abs() * 0.85;
    let ring_col = hot_pink.with_alpha(alpha.clamp(0.40, 0.98));

    c.set_stroke(Fill::Solid(ring_col));
    c.set_line_width(1.8 + l_ratio * 1.2);
    c.set_shadow(hot_pink, 8.0 + bs * 6.0);
    c.stroke_polyline(&ring_pts);
  }

  // Top rim spotlight glow ring
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.4, 0.9, 0.98)));
  c.set_line_width(3.0);
  c.set_shadow(hot_pink, 18.0);
  let mut top_rim_pts = Vec::with_capacity(42);
  for k in 0..=40 {
    let a = (k as f32 / 40.0) * TAU;
    top_rim_pts.push((center_x + a.cos() * base_rx, top_y + a.sin() * (base_rx * tilt_y)));
  }
  c.stroke_polyline(&top_rim_pts);

  c.restore();
}
