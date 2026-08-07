//! Hologram Stage style renderer (`hologramStage`).
//!
//! Renders a 3D cylindrical hologram podium stage complete with stacked glowing magenta rings,
//! cyan dotted floor boundary ring, 3D center stage text, and a glossy floor reflection.

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
  let center_y = height * 0.54;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.95);
  let electric_cyan = Color::rgba(0.0, 0.85, 1.0, 0.95);
  let neon_purple = Color::rgba(0.7, 0.1, 1.0, 0.95);

  let tilt_y = 0.35f32; // 3D vertical tilt factor
  let base_rx = (width.min(height) * 0.32).clamp(110.0, 360.0);
  let _base_ry = base_rx * tilt_y;

  // -------------------------------------------------------------------------
  // 1. OUTER CYAN DOTTED FLOOR BOUNDARY RING (PHOTO 1)
  // -------------------------------------------------------------------------
  let num_dots = 48usize;
  let floor_rx = base_rx * 1.15 + (be * 15.0);
  let floor_ry = floor_rx * tilt_y;

  for d in 0..num_dots {
    let angle = -rot * 0.8 + (d as f32 / num_dots as f32) * TAU;
    let dx = center_x + angle.cos() * floor_rx;
    let dy = center_y + angle.sin() * floor_ry;

    c.set_fill(Fill::Solid(electric_cyan));
    c.set_shadow(electric_cyan, 8.0);
    c.fill_rounded_rect(dx - 3.0, dy - 1.5, 6.0, 3.0, 1.0);
  }

  // -------------------------------------------------------------------------
  // 2. STACKED 3D MAGENTA CYLINDER PODIUM RINGS (PHOTO 1)
  // -------------------------------------------------------------------------
  let num_cylinder_layers = 5usize;
  let layer_spacing = (height * 0.025).clamp(8.0, 18.0);

  for layer in 0..num_cylinder_layers {
    let l_ratio = layer as f32 / (num_cylinder_layers - 1) as f32;
    let layer_y = center_y - layer as f32 * layer_spacing;

    let bin_idx = (layer * freq.len() / num_cylinder_layers).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin_idx).unwrap_or(&0) as f32 / 255.0;

    let rx = (base_rx * (0.85 - l_ratio * 0.18) + (fv * 20.0 * sensitivity)).clamp(20.0, width);
    let ry = rx * tilt_y;

    let mut ring_pts = Vec::with_capacity(37);
    for k in 0..=36 {
      let a = (k as f32 / 36.0) * TAU + rot * (if layer % 2 == 0 { 1.0 } else { -1.0 });
      ring_pts.push((center_x + a.cos() * rx, layer_y + a.sin() * ry));
    }

    let ring_col = if layer == num_cylinder_layers - 1 {
      hot_pink
    } else {
      neon_purple
    };

    c.set_stroke(Fill::Solid(ring_col.with_alpha(0.85)));
    c.set_line_width(2.0 + l_ratio * 1.5);
    c.set_shadow(ring_col.with_alpha(0.8), 12.0 + bs * 6.0);
    c.stroke_polyline(&ring_pts);
  }

  // -------------------------------------------------------------------------
  // 3. 3D CENTER TEXT & GLOSSY FLOOR REFLECTION ("TSIXOM") (PHOTO 1)
  // -------------------------------------------------------------------------
  let stage_top_y = center_y - (num_cylinder_layers - 1) as f32 * layer_spacing;
  let font_sz = (base_rx * 0.22).clamp(18.0, 48.0);

  // Main 3D Text Standing on Stage ("TSIXOM" or Title)
  let stage_text = "TSIXOM";

  c.draw_text(
    stage_text,
    center_x,
    stage_top_y - font_sz * 0.6,
    font_sz,
    "sans-serif",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(1.0, 0.2, 0.85, 0.98)),
    1.0,
    &Default::default(),
  );

  // Inverted Glossy Floor Reflection Underneath (Photo 1)
  c.draw_text(
    stage_text,
    center_x,
    stage_top_y + font_sz * 0.5,
    font_sz,
    "sans-serif",
    700.0,
    false,
    TextAlign::Center,
    Fill::Solid(hot_pink.with_alpha(0.38)),
    1.0,
    &Default::default(),
  );

  c.restore();
}
