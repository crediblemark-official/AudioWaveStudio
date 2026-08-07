//! Shared woofer rendering helper for speaker styles.
//! Mirrors `drawWoofer` in `src/services/renderers/speakerTrio.ts` and
//! `drawSplatterWoofer` in `src/services/renderers/speakerSplatter.ts`.

use std::f32::consts::TAU;

use crate::gpu2d::{Color, Fill, GpuCanvas};

pub struct WooferStyle<'a> {
  pub rim_stops: &'a [(f32, Color)],
  pub bolt_r: f32,
  /// Bolt fill color (trio uses #DDDDDD, splatter uses #E5E5EA).
  pub bolt_color: Color,
  pub ring_alpha: f32,
  pub ring_step: f32,
  /// First concentric ridge radius offset from the cone inner radius
  /// (trio: +6, splatter: +4) and the margin before the surround inner edge
  /// (trio: 4, splatter: 2).
  pub ring_start: f32,
  pub ring_end_margin: f32,
  /// Ridge ring stroke width (trio: 1.2, splatter: 1.0).
  pub ring_width: f32,
  /// Cone diaphragm radius as a ratio of the woofer radius
  /// (trio: 0.32, splatter: 0.30). Drives the dust-cap radius and ridges too.
  pub cone_inner_ratio: f32,
  /// Rubber surround gradient stops (trio: #1A1A1E/#3A3A40/#101014,
  /// splatter: #1C1C20/#3C3C44/#0F0F12).
  pub rubber_stops: &'a [(f32, Color)],
  /// Cone diaphragm gradient stops (trio: #444855/#22242C/#111216,
  /// splatter: #444856/#22242D/#0E0F14).
  pub cone_stops: &'a [(f32, Color)],
  /// Dust cap gradient 0.4 stop (trio: #30333D, splatter: #30333E).
  pub dust_mid: Color,
  pub shadow_blur: f32,
  /// Drop-shadow color (trio: rgba(0,0,0,0.6); splatter: rgba(0,0,0,0.95)).
  pub shadow_color: Color,
  /// Dust cap bass pulse factor (trio: `dustR = coneInnerR * (1 + be*0.06)`,
  /// splatter: `* (1 + be*0.05)`).
  pub dust_scale: f32,
  /// Dust cap shadow blur (trio: 10, splatter: 8).
  pub dust_shadow: f32,
  /// Glossy crescent glare alpha (trio: 0.35, splatter: 0.45).
  pub crescent_alpha: f32,
}

pub fn draw_woofer(c: &mut GpuCanvas, x: f32, y: f32, r: f32, is_center: bool, be: f32, style: &WooferStyle) {
  let outer_r = r;
  let inner_r = r * 0.86;
  let bolt_r = (outer_r + inner_r) / 2.0;

  // TS: `c.shadowBlur = isCenter ? 25 : 18` with a per-style color/offset.
  let shadow = if is_center { 25.0 } else { style.shadow_blur };
  c.set_shadow(style.shadow_color, shadow);

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
  c.set_fill(Fill::Solid(style.bolt_color));
  for k in 0..4 {
    let angle = k as f32 * TAU / 4.0;
    let bx = x + angle.cos() * bolt_r;
    let by = y + angle.sin() * bolt_r;
    c.fill_circle(bx, by, style.bolt_r);
  }

  let surround_inner = r * 0.72;
  let rubber = Fill::radial_gradient(x, y, surround_inner, x, y, inner_r, style.rubber_stops);
  c.set_fill(rubber);
  c.fill_ring(x, y, inner_r, surround_inner);

  let cone_inner = r * style.cone_inner_ratio;
  let cone = Fill::radial_gradient(x - r * 0.2, y - r * 0.2, cone_inner * 0.5, x, y, surround_inner, style.cone_stops);
  c.set_fill(cone);
  c.fill_circle(x, y, surround_inner);

  c.set_stroke(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, style.ring_alpha)));
  c.set_line_width(style.ring_width);
  let mut ring = cone_inner + style.ring_start;
  while ring < surround_inner - style.ring_end_margin {
    c.stroke_circle(x, y, ring);
    ring += style.ring_step;
  }

  // TS trio/splatter pulse the dust cap with bass:
  //   trio:      coneInnerR * (1 + be * 0.06)
  //   splatter:  coneInnerR * (1 + be * 0.05)
  let dust_r = cone_inner * (1.0 + be * style.dust_scale);
  let dust = Fill::radial_gradient(x - dust_r * 0.3, y - dust_r * 0.3, 0.0, x, y, dust_r, &[
    (0.0, Color::hex("#666A78")),
    (0.4, style.dust_mid),
    (1.0, Color::hex("#0C0D10")),
  ]);
  c.set_shadow(Color::hex("#000000"), style.dust_shadow);
  c.set_fill(dust);
  c.fill_circle(x, y, dust_r);

  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, style.crescent_alpha)));
  c.fill_ring_arc(
    x - dust_r * 0.15,
    y - dust_r * 0.15,
    dust_r * 0.65,
    dust_r * 0.45,
    TAU * 0.5,
    TAU * 0.925,
  );
}
