//! Dual Portal Bridge style renderer (`dualPortalBridge`).
//!
//! Recreates the exact Dual Rainbow Spiral Vortex Portals from the reference image:
//! Left lime/green concentric portal + Right cyan/magenta/red concentric portal,
//! connected by an undulating 7-color rainbow laser beam with a glowing white central core line.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const BEAM_PTS: usize = 120;
const CORE_PTS: usize = 36;

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

  // Portal centers pushed outward toward canvas edges
  let portal_dist = (width * 0.36).clamp(160.0, 600.0);
  let left_cx = center_x - portal_dist;
  let right_cx = center_x + portal_dist;

  let num_rings = 42usize;
  let max_r = (width.max(height) * 0.65).clamp(240.0, 900.0);

  // -------------------------------------------------------------------------
  // 1. LEFT PORTAL: LIME / GREEN CONCENTRIC VORTEX (REFERENCE IMAGE)
  // -------------------------------------------------------------------------
  for r_idx in 1..=num_rings {
    let r_ratio = r_idx as f32 / num_rings as f32;
    let r = r_ratio * max_r + (be * 10.0);

    // Lime Green palette (Hue 80° to 125°)
    let hue = (80.0 + r_ratio * 45.0 + (rot * 3.0).sin() * 5.0) % 360.0;
    let col = super::super::hsl_to_color(hue, 0.95, 0.55, 0.85);

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(1.8);
    c.stroke_circle(left_cx, center_y, r);
  }

  // Left Portal: Wavy Red Core Ring
  let mut left_core_pts = Vec::with_capacity(CORE_PTS + 1);
  for k in 0..=CORE_PTS {
    let a = (k as f32 / CORE_PTS as f32) * std::f32::consts::TAU;
    let bin = (k * freq.len() / CORE_PTS).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let cr = 22.0 + fv * 16.0 * sensitivity;
    left_core_pts.push((left_cx + a.cos() * cr, center_y + a.sin() * cr));
  }
  c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.1, 0.1, 0.95)));
  c.set_line_width(2.5);
  c.set_shadow(Color::rgba(1.0, 0.1, 0.1, 0.9), 10.0);
  c.stroke_polyline(&left_core_pts);

  // -------------------------------------------------------------------------
  // 2. RIGHT PORTAL: CYAN / MAGENTA / RED CONCENTRIC VORTEX (REFERENCE IMAGE)
  // -------------------------------------------------------------------------
  for r_idx in 1..=num_rings {
    let r_ratio = r_idx as f32 / num_rings as f32;
    let r = r_ratio * max_r + (be * 10.0);

    // Cyan (center) -> Magenta -> Red (outer) palette
    let hue = (190.0 + r_ratio * 160.0) % 360.0;
    let col = super::super::hsl_to_color(hue, 0.95, 0.55, 0.85);

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(1.8);
    c.stroke_circle(right_cx, center_y, r);
  }

  // Right Portal: Wavy Cyan Core Ring
  let mut right_core_pts = Vec::with_capacity(CORE_PTS + 1);
  for k in 0..=CORE_PTS {
    let a = (k as f32 / CORE_PTS as f32) * std::f32::consts::TAU;
    let bin = (k * freq.len() / CORE_PTS).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let cr = 22.0 + fv * 16.0 * sensitivity;
    right_core_pts.push((right_cx + a.cos() * cr, center_y + a.sin() * cr));
  }
  c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.85, 1.0, 0.95)));
  c.set_line_width(2.5);
  c.set_shadow(Color::rgba(0.0, 0.85, 1.0, 0.9), 10.0);
  c.stroke_polyline(&right_core_pts);

  // -------------------------------------------------------------------------
  // 3. CONNECTING 7-COLOR RAINBOW ENERGY BEAM WITH WHITE CORE (EXACT REFERENCE)
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

  let beam_span = (height * 0.10).clamp(16.0, 60.0);
  let step = (freq.len() / BEAM_PTS).max(1);

  for (l_idx, &col) in rainbow_colors.iter().enumerate() {
    let offset_y = (l_idx as f32 / (rainbow_colors.len() - 1) as f32 - 0.5) * beam_span;

    let mut line_pts = Vec::with_capacity(BEAM_PTS + 1);
    for p in 0..=BEAM_PTS {
      let t = p as f32 / BEAM_PTS as f32;
      let bx = left_cx + t * (right_cx - left_cx);

      let bin = (p * step).min(freq.len().saturating_sub(1));
      let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

      // Smooth S-curve wave modulation
      let wave_mod = (t * std::f32::consts::PI * 2.0 + rot * 1.8).sin() * (10.0 + fv * sensitivity * 24.0);
      let by = center_y + offset_y + wave_mod;

      line_pts.push((bx, by));
    }

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(3.0 + bs * 1.5);
    c.set_shadow(col, 10.0 + bs * 5.0);
    c.stroke_polyline(&line_pts);
  }

  // -------------------------------------------------------------------------
  // 4. BRIGHT WHITE CENTRAL LASER LINE (RUNNING THROUGH BEAM & PORTALS)
  // -------------------------------------------------------------------------
  let mut white_core_pts = Vec::with_capacity(BEAM_PTS + 1);
  for p in 0..=BEAM_PTS {
    let t = p as f32 / BEAM_PTS as f32;
    let bx = left_cx + t * (right_cx - left_cx);
    let bin = (p * step).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let wave_mod = (t * std::f32::consts::PI * 2.0 + rot * 1.8).sin() * (10.0 + fv * sensitivity * 24.0);
    white_core_pts.push((bx, center_y + wave_mod));
  }
  c.set_stroke(Fill::Solid(Color::WHITE));
  c.set_line_width(2.5);
  c.set_shadow(Color::WHITE, 14.0);
  c.stroke_polyline(&white_core_pts);

  // Center white glowing dots at portal origins
  let dot_r = (8.0 + be * 6.0).clamp(4.0, 18.0);
  for &cx in &[left_cx, right_cx] {
    c.set_fill(Fill::Solid(Color::WHITE));
    c.set_shadow(Color::WHITE, 18.0);
    c.fill_ellipse(cx, center_y, dot_r, dot_r);
  }

  c.restore();
}
