//! Dual Portal Bridge style renderer (`dualPortalBridge`).
//!
//! Complete overhaul: Dual Rainbow Spiral Vortex Portals
//! - Left Portal: Clean lime-green concentric circles
//! - Right Portal: Clean cyan-magenta-red concentric circles
//! - Rainbow Bridge: Tightly bundled 7 parallel rainbow lines with smooth S-curve wave modulation
//!   and a bright white laser core running directly through the centers of both portals.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const BEAM_PTS: usize = 100;

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

  // Portal centers situated on left and right sides
  let portal_dist = (width * 0.35).clamp(150.0, 520.0);
  let left_cx = center_x - portal_dist;
  let right_cx = center_x + portal_dist;

  let num_rings = 32usize;
  let max_r = (width.max(height) * 0.55).clamp(200.0, 750.0);

  // -------------------------------------------------------------------------
  // 1. LEFT PORTAL: LIME / GREEN CONCENTRIC VORTEX
  // -------------------------------------------------------------------------
  for r_idx in 1..=num_rings {
    let r_ratio = r_idx as f32 / num_rings as f32;
    let r = r_ratio * max_r + (be * 8.0);

    let hue = (80.0 + r_ratio * 40.0) % 360.0;
    let col = super::super::hsl_to_color(hue, 0.95, 0.55, 0.85);

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(1.8);
    c.stroke_circle(left_cx, center_y, r);
  }

  // Left Portal Wavy Red Inner Core Ring
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.1, 0.15, 0.95)));
  c.set_line_width(2.5);
  c.stroke_circle(left_cx, center_y, 22.0 + be * 8.0);

  // -------------------------------------------------------------------------
  // 2. RIGHT PORTAL: CYAN / MAGENTA / RED CONCENTRIC VORTEX
  // -------------------------------------------------------------------------
  for r_idx in 1..=num_rings {
    let r_ratio = r_idx as f32 / num_rings as f32;
    let r = r_ratio * max_r + (be * 8.0);

    let hue = (190.0 + r_ratio * 150.0) % 360.0;
    let col = super::super::hsl_to_color(hue, 0.95, 0.55, 0.85);

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(1.8);
    c.stroke_circle(right_cx, center_y, r);
  }

  // Right Portal Wavy Cyan Inner Core Ring
  c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.88, 1.0, 0.95)));
  c.set_line_width(2.5);
  c.stroke_circle(right_cx, center_y, 22.0 + be * 8.0);

  // -------------------------------------------------------------------------
  // 3. TIGHTLY BUNDLED 7-COLOR RAINBOW BRIDGE BEAM
  // -------------------------------------------------------------------------
  let rainbow_colors = [
    Color::rgba(1.0, 0.1, 0.15, 0.98), // Red
    Color::rgba(1.0, 0.55, 0.0, 0.98), // Orange
    Color::rgba(1.0, 0.9, 0.0, 0.98),  // Yellow
    Color::rgba(0.1, 0.95, 0.2, 0.98), // Lime Green
    Color::rgba(0.0, 0.85, 1.0, 0.98), // Cyan
    Color::rgba(0.2, 0.35, 1.0, 0.98), // Blue
    Color::rgba(0.95, 0.1, 0.9, 0.98), // Magenta
  ];

  let beam_thickness = (height * 0.07).clamp(14.0, 48.0);
  let step = (freq.len() / BEAM_PTS).max(1);

  for (l_idx, &col) in rainbow_colors.iter().enumerate() {
    let offset_y = (l_idx as f32 / (rainbow_colors.len() - 1) as f32 - 0.5) * beam_thickness;

    let mut line_pts = Vec::with_capacity(BEAM_PTS + 1);
    for p in 0..=BEAM_PTS {
      let t = p as f32 / BEAM_PTS as f32;
      let bx = left_cx + t * (right_cx - left_cx);

      let bin = (p * step).min(freq.len().saturating_sub(1));
      let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

      let wave_mod = (t * std::f32::consts::PI * 2.0 + rot * 1.8).sin() * (8.0 + fv * sensitivity * 18.0);
      let by = center_y + offset_y + wave_mod;

      line_pts.push((bx, by));
    }

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(2.8 + bs * 1.2);
    c.set_shadow(col, 8.0 + bs * 4.0);
    c.stroke_polyline(&line_pts);
  }

  // -------------------------------------------------------------------------
  // 4. BRIGHT WHITE LASER CORE LINE RUNNING THROUGH BEAM & PORTALS
  // -------------------------------------------------------------------------
  let mut white_core_pts = Vec::with_capacity(BEAM_PTS + 1);
  for p in 0..=BEAM_PTS {
    let t = p as f32 / BEAM_PTS as f32;
    let bx = left_cx + t * (right_cx - left_cx);
    let bin = (p * step).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let wave_mod = (t * std::f32::consts::PI * 2.0 + rot * 1.8).sin() * (8.0 + fv * sensitivity * 18.0);
    white_core_pts.push((bx, center_y + wave_mod));
  }
  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(2.2);
  c.set_shadow(Color::WHITE, 12.0);
  c.stroke_polyline(&white_core_pts);

  // Center white glowing dots at portal origins
  let dot_r = (8.0 + be * 5.0).clamp(4.0, 16.0);
  for &cx in &[left_cx, right_cx] {
    c.set_fill(Fill::Solid(Color::WHITE));
    c.set_shadow(Color::WHITE, 16.0);
    c.fill_ellipse(cx, center_y, dot_r, dot_r);
  }

  c.restore();
}
