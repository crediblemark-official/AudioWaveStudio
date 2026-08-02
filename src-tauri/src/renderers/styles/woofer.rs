//! Shared woofer rendering helper for speaker styles.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};

pub struct WooferStyle<'a> {
  pub rim_stops: &'a [(f32, Color)],
  pub bolt_r: f32,
  pub ring_alpha: f32,
  pub ring_step: f32,
  pub shadow_blur: f32,
}

pub fn draw_woofer(c: &mut GpuCanvas, x: f32, y: f32, r: f32, is_center: bool, style: &WooferStyle) {
  let outer_r = r;
  let inner_r = r * 0.86;
  let bolt_r = (outer_r + inner_r) / 2.0;

  let shadow = if is_center { style.shadow_blur } else { style.shadow_blur * 0.72 };
  c.set_shadow(Color::rgba(0.0, 0.0, 0.0, 0.6), shadow);

  let metallic = Fill::linear_gradient(
    x - r,
    y - r,
    x + r,
    y + r,
    style.rim_stops,
  );
  c.set_fill(metallic);
  c.fill_ring(x, y, outer_r, inner_r);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::hex("#DDDDDD")));
  for k in 0..4 {
    let angle = k as f32 * TAU / 4.0;
    let bx = x + angle.cos() * bolt_r;
    let by = y + angle.sin() * bolt_r;
    c.fill_circle(bx, by, style.bolt_r);
  }

  let surround_inner = r * 0.72;
  let rubber = Fill::radial_gradient(x, y, surround_inner, x, y, inner_r, &[
    (0.0, Color::hex("#1A1A1E")),
    (0.5, Color::hex("#3A3A40")),
    (1.0, Color::hex("#101014")),
  ]);
  c.set_fill(rubber);
  c.fill_ring(x, y, inner_r, surround_inner);

  let cone_inner = r * 0.32;
  let cone = Fill::radial_gradient(x - r * 0.2, y - r * 0.2, cone_inner * 0.5, x, y, surround_inner, &[
    (0.0, Color::hex("#444855")),
    (0.6, Color::hex("#22242C")),
    (1.0, Color::hex("#111216")),
  ]);
  c.set_fill(cone);
  c.fill_circle(x, y, surround_inner);

  c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, style.ring_alpha)));
  c.set_line_width(1.2);
  let mut ring = cone_inner + 6.0;
  while ring < surround_inner - 4.0 {
    c.stroke_circle(x, y, ring);
    ring += style.ring_step;
  }

  let dust_r = cone_inner * 1.0;
  let dust = Fill::radial_gradient(x - dust_r * 0.3, y - dust_r * 0.3, 0.0, x, y, dust_r, &[
    (0.0, Color::hex("#666A78")),
    (0.4, Color::hex("#30333D")),
    (1.0, Color::hex("#0C0D10")),
  ]);
  c.set_shadow(Color::hex("#000000"), 10.0);
  c.set_fill(dust);
  c.fill_circle(x, y, dust_r);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.35)));
  c.fill_ring_arc(
    x - dust_r * 0.15,
    y - dust_r * 0.15,
    dust_r * 0.65,
    dust_r * 0.45,
    TAU * 0.5,
    TAU * 0.925,
  );
}
