//! Dual Portal Bridge style renderer (`dualPortalBridge`).
//!
//! Recreates the exact Dual Rainbow Spiral Vortex Portals from the reference image:
//! Pure black background, left lime/green concentric spiral portal, right cyan/magenta/red
//! concentric spiral portal, connected by a 7-color rainbow laser energy beam running
//! between the glowing white center dots.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const BEAM_PTS: usize = 90;

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

  // Pure Black Background (Exact Reference Image)
  c.set_fill(Fill::Solid(Color::BLACK));
  c.fill_rect(0.0, 0.0, width, height);

  let portal_dist = (width * 0.32).clamp(120.0, 480.0);
  let left_cx = center_x - portal_dist;
  let right_cx = center_x + portal_dist;

  let num_rings = 24usize;
  let max_r = (height * 0.44).clamp(110.0, 420.0);

  // -------------------------------------------------------------------------
  // 1. LEFT PORTAL: LIME / GREEN CONCENTRIC SPIRAL VORTEX (REFERENCE IMAGE)
  // -------------------------------------------------------------------------
  for r_idx in 1..=num_rings {
    let r_ratio = r_idx as f32 / num_rings as f32;
    let r = r_ratio * max_r + (be * 12.0);

    // Lime Green / Yellow HSL palette
    let hue = (90.0 + r_ratio * 50.0 + rot * 5.0) % 360.0;
    let col = super::super::hsl_to_color(hue, 0.95, 0.58, 0.85);

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(2.0);
    c.stroke_circle(left_cx, center_y, r);
  }

  // Left Wavy Red Core Outline
  let red_core_col = Color::rgba(1.0, 0.1, 0.1, 0.9);
  c.set_stroke(Fill::Solid(red_core_col));
  c.set_line_width(2.5);
  c.stroke_circle(left_cx, center_y, 24.0 + be * 8.0);

  // -------------------------------------------------------------------------
  // 2. RIGHT PORTAL: CYAN / MAGENTA / RED CONCENTRIC SPIRAL VORTEX (REFERENCE)
  // -------------------------------------------------------------------------
  for r_idx in 1..=num_rings {
    let r_ratio = r_idx as f32 / num_rings as f32;
    let r = r_ratio * max_r + (be * 12.0);

    // Cyan / Magenta / Red HSL palette
    let hue = (330.0 + r_ratio * 70.0 - rot * 5.0) % 360.0;
    let col = super::super::hsl_to_color(hue, 0.95, 0.58, 0.85);

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(2.0);
    c.stroke_circle(right_cx, center_y, r);
  }

  // Right Wavy Blue Core Outline
  let blue_core_col = Color::rgba(0.0, 0.8, 1.0, 0.9);
  c.set_stroke(Fill::Solid(blue_core_col));
  c.set_line_width(2.5);
  c.stroke_circle(right_cx, center_y, 24.0 + be * 8.0);

  // -------------------------------------------------------------------------
  // 3. CONNECTING 7-COLOR RAINBOW ENERGY BEAM (EXACT REFERENCE IMAGE)
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

  let beam_span = (height * 0.12).clamp(16.0, 65.0);
  let step = (freq.len() / BEAM_PTS).max(1);

  for (l_idx, &col) in rainbow_colors.iter().enumerate() {
    let offset_y = (l_idx as f32 / (rainbow_colors.len() - 1) as f32 - 0.5) * beam_span;

    let mut line_pts = Vec::with_capacity(BEAM_PTS + 1);
    for p in 0..=BEAM_PTS {
      let t = p as f32 / BEAM_PTS as f32;
      let bx = left_cx + t * (right_cx - left_cx);

      let bin = (p * step).min(freq.len().saturating_sub(1));
      let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

      let wave_mod = (t * std::f32::consts::PI * 2.0 + rot * 2.0).sin() * (8.0 + fv * sensitivity * 22.0);
      let by = center_y + offset_y + wave_mod;

      line_pts.push((bx, by));
    }

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(3.2 + bs * 2.0);
    c.set_shadow(col, 12.0 + bs * 6.0);
    c.stroke_polyline(&line_pts);
  }

  // -------------------------------------------------------------------------
  // 4. GLOWING WHITE CENTER DOTS (EXACT REFERENCE)
  // -------------------------------------------------------------------------
  let dot_r = (10.0 + be * 8.0).clamp(5.0, 24.0);
  for &cx in &[left_cx, right_cx] {
    c.set_fill(Fill::Solid(Color::WHITE));
    c.set_shadow(Color::WHITE, 16.0);
    c.fill_ellipse(cx, center_y, dot_r, dot_r);
  }

  c.restore();
}
