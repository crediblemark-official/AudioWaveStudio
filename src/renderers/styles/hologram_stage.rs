//! Hologram Stage style renderer (`hologramStage`).
//!
//! Recreates the exact 3D Cylindrical Hologram Podium Stage from the reference image:
//! Dark navy background, stacked glowing neon magenta rings forming a cylindrical podium,
//! cyan dotted floor boundary ring, bright 3D center text ("TSIXOM"), and exact upside-down floor mirror reflection.

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
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
  let center_y = height * 0.55;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Dark Navy Space Background (Reference Image)
  c.set_fill(Fill::Solid(Color::rgba(0.04, 0.05, 0.12, 1.0)));
  c.fill_rect(0.0, 0.0, width, height);

  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.98);
  let cyan_col = Color::rgba(0.0, 0.85, 1.0, 0.95);

  let tilt_y = 0.36f32; // 3D perspective vertical tilt
  let base_rx = (width.min(height) * 0.32).clamp(110.0, 360.0);

  // -------------------------------------------------------------------------
  // 1. OUTER CYAN DOTTED FLOOR BOUNDARY RING (REFERENCE IMAGE)
  // -------------------------------------------------------------------------
  let floor_rx = base_rx * 1.25 + (be * 15.0);
  let floor_ry = floor_rx * tilt_y;
  let num_dots = 42;

  for d in 0..num_dots {
    if d % 2 == 0 {
      let a1 = -rot * 0.8 + (d as f32 / num_dots as f32) * TAU;
      let a2 = -rot * 0.8 + ((d as f32 + 0.6) / num_dots as f32) * TAU;

      let mut pts = Vec::with_capacity(8);
      for k in 0..8 {
        let angle = a1 + (k as f32 / 7.0) * (a2 - a1);
        pts.push((center_x + angle.cos() * floor_rx, center_y + angle.sin() * floor_ry));
      }

      c.set_stroke(Fill::Solid(cyan_col));
      c.set_line_width(3.0);
      c.set_shadow(cyan_col, 10.0);
      c.stroke_polyline(&pts);
    }
  }

  // -------------------------------------------------------------------------
  // 2. STACKED GLOWING NEON MAGENTA PODIUM RINGS (EXACT REFERENCE)
  // -------------------------------------------------------------------------
  let num_layers = 5;
  let layer_h = (height * 0.024).clamp(8.0, 20.0);

  for l in 0..num_layers {
    let l_ratio = l as f32 / (num_layers - 1) as f32;
    let ly = center_y - l as f32 * layer_h;

    let bin = (l * freq.len() / num_layers).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

    let rx = (base_rx * (0.88 - l_ratio * 0.15) + fv * 18.0 * sensitivity).clamp(30.0, width);
    let ry = rx * tilt_y;

    let mut ring_pts = Vec::with_capacity(40);
    for k in 0..=36 {
      let a = (k as f32 / 36.0) * TAU;
      ring_pts.push((center_x + a.cos() * rx, ly + a.sin() * ry));
    }

    c.set_stroke(Fill::Solid(hot_pink));
    c.set_line_width(2.5 + l_ratio * 1.5);
    c.set_shadow(hot_pink, 14.0 + bs * 8.0);
    c.stroke_polyline(&ring_pts);
  }

  // -------------------------------------------------------------------------
  // 3. 3D CENTER TEXT & UPSIDE-DOWN FLOOR REFLECTION ("TSIXOM") (EXACT REFERENCE)
  // -------------------------------------------------------------------------
  let top_stage_y = center_y - (num_layers - 1) as f32 * layer_h;
  let font_sz = (base_rx * 0.24).clamp(20.0, 52.0);

  let text_val = "TSIXOM";

  // Main Bright 3D Magenta Text Standing on Stage
  c.draw_text(
    text_val,
    center_x,
    top_stage_y - font_sz * 0.65,
    font_sz,
    "sans-serif",
    800.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(1.0, 0.25, 0.9, 0.98)),
    1.0,
    &Default::default(),
  );

  // Inverted Mirror Floor Reflection ("TSIXOM" Vertically Mirrored Directly Below Stage)
  c.draw_text(
    text_val,
    center_x,
    top_stage_y + font_sz * 0.45,
    font_sz,
    "sans-serif",
    800.0,
    false,
    TextAlign::Center,
    Fill::Solid(hot_pink.with_alpha(0.42)),
    1.0,
    &Default::default(),
  );

  c.restore();
}
