//! Hologram Stage style renderer (`hologramStage`).
//!
//! 3D Cylindrical Hologram Podium Stage: dark background, stacked glowing
//! neon magenta rings forming a 3D cylinder podium, inner holographic light beam,
//! and outer cyan dashed floor boundary ring.

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
  let base_y = height * 0.62; // Bottom of the cylinder (floor level)

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.98);
  let cyan_col = Color::rgba(0.0, 0.85, 1.0, 0.95);

  let tilt_y = 0.32f32; // 3D perspective tilt
  let base_rx = (width.min(height) * 0.28).clamp(100.0, 340.0);

  // -------------------------------------------------------------------------
  // 1. OUTER CYAN DASHED FLOOR BOUNDARY RING
  // -------------------------------------------------------------------------
  let floor_rx = base_rx * 1.35 + (be * 12.0);
  let floor_ry = floor_rx * tilt_y;
  let num_dashes = 36;

  for d in 0..num_dashes {
    if d % 2 == 0 {
      let a1 = -rot * 0.8 + (d as f32 / num_dashes as f32) * TAU;
      let a2 = -rot * 0.8 + ((d as f32 + 0.55) / num_dashes as f32) * TAU;

      let mut pts = Vec::with_capacity(10);
      for k in 0..10 {
        let angle = a1 + (k as f32 / 9.0) * (a2 - a1);
        pts.push((center_x + angle.cos() * floor_rx, base_y + angle.sin() * floor_ry));
      }

      c.set_stroke(Fill::Solid(cyan_col));
      c.set_line_width(3.5);
      c.set_shadow(cyan_col, 10.0);
      c.stroke_polyline(&pts);
    }
  }

  // -------------------------------------------------------------------------
  // 2. INNER HOLOGRAPHIC LIGHT BEAM / GLOW CORE
  // -------------------------------------------------------------------------
  let cylinder_height = (height * 0.28).clamp(60.0, 200.0);
  let top_y = base_y - cylinder_height;

  // Solid floor base disc fill
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.65, 0.15)));
  c.set_shadow(hot_pink, 20.0 + bs * 10.0);
  c.fill_ellipse(center_x, base_y, base_rx, base_rx * tilt_y);

  // Vertical hologram beam column
  let beam_pts = [
    (center_x - base_rx * 0.7, base_y),
    (center_x - base_rx * 0.5, top_y),
    (center_x + base_rx * 0.5, top_y),
    (center_x + base_rx * 0.7, base_y),
  ];
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.65, 0.08)));
  c.fill_polyline_to_base(&beam_pts, base_y);

  // -------------------------------------------------------------------------
  // 3. STACKED MAGENTA RINGS FORMING A 3D CYLINDER PODIUM
  // -------------------------------------------------------------------------
  let num_layers = 12;
  let layer_spacing = cylinder_height / (num_layers - 1) as f32;

  for l in 0..num_layers {
    let l_ratio = l as f32 / (num_layers - 1) as f32;
    let ly = base_y - l as f32 * layer_spacing;

    let bin = (l * freq.len() / num_layers).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

    let rx = base_rx + fv * 14.0 * sensitivity;
    let ry = rx * tilt_y;

    let mut ring_pts = Vec::with_capacity(50);
    for k in 0..=40 {
      let a = (k as f32 / 40.0) * TAU;
      ring_pts.push((center_x + a.cos() * rx, ly + a.sin() * ry));
    }

    let alpha = 0.55 + (l_ratio - 0.5).abs() * 0.9;
    let ring_col = hot_pink.with_alpha(alpha.clamp(0.45, 0.98));

    c.set_stroke(Fill::Solid(ring_col));
    c.set_line_width(2.0 + l_ratio * 1.0);
    c.set_shadow(hot_pink, 10.0 + bs * 6.0);
    c.stroke_polyline(&ring_pts);
  }

  c.restore();
}
