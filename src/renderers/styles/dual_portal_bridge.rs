//! Dual Portal Bridge style renderer (`dualPortalBridge`).
//!
//! Renders dual left and right orbital portals connected by an undulating 7-color
//! rainbow laser energy beam that reacts dynamically to audio spectrum data.

use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const BEAM_POINTS: usize = 80;

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

  let portal_dist = (width * 0.32).clamp(120.0, 480.0);
  let left_cx = center_x - portal_dist;
  let right_cx = center_x + portal_dist;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // -------------------------------------------------------------------------
  // 1. LEFT & RIGHT CONCENTRIC PORTAL RINGS (PHOTO 1)
  // -------------------------------------------------------------------------
  let num_rings = 18usize;
  let max_r = (height * 0.42).clamp(100.0, 400.0);

  // Left Portal (Lime / Green Rings)
  for r_idx in 1..=num_rings {
    let r_ratio = r_idx as f32 / num_rings as f32;
    let r = r_ratio * max_r + (be * 15.0);
    let hue = (120.0 + r_ratio * 40.0 + rot * 10.0) % 360.0;
    let col = super::super::hsl_to_color(hue, 0.9, 0.65, 0.75);

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(2.0);
    c.stroke_circle(left_cx, center_y, r);
  }

  // Right Portal (Cyan / Magenta / Red Rings)
  for r_idx in 1..=num_rings {
    let r_ratio = r_idx as f32 / num_rings as f32;
    let r = r_ratio * max_r + (be * 15.0);
    let hue = (320.0 + r_ratio * 60.0 - rot * 10.0) % 360.0;
    let col = super::super::hsl_to_color(hue, 0.9, 0.65, 0.75);

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(2.0);
    c.stroke_circle(right_cx, center_y, r);
  }

  // -------------------------------------------------------------------------
  // 2. CONNECTING 7-COLOR RAINBOW LASER ENERGY BEAM (PHOTO 1)
  // -------------------------------------------------------------------------
  let rainbow_colors = [
    Color::rgba(1.0, 0.1, 0.25, 0.95), // Red
    Color::rgba(1.0, 0.5, 0.1, 0.95),  // Orange
    Color::rgba(1.0, 0.85, 0.1, 0.95), // Yellow
    Color::rgba(0.1, 0.95, 0.3, 0.95), // Green
    Color::rgba(0.0, 0.85, 1.0, 0.95), // Cyan
    Color::rgba(0.2, 0.3, 1.0, 0.95),  // Blue
    Color::rgba(0.9, 0.1, 0.9, 0.95),  // Magenta
  ];

  let beam_width = (height * 0.12).clamp(16.0, 70.0);
  let step = (freq.len() / BEAM_POINTS).max(1);

  for (line_idx, &col) in rainbow_colors.iter().enumerate() {
    let line_offset = (line_idx as f32 / (rainbow_colors.len() - 1) as f32 - 0.5) * beam_width;

    let mut beam_pts = Vec::with_capacity(BEAM_POINTS + 1);
    for p_idx in 0..=BEAM_POINTS {
      let t_val = p_idx as f32 / BEAM_POINTS as f32;
      let bx = left_cx + t_val * (right_cx - left_cx);

      let bin = (p_idx * step).min(freq.len().saturating_sub(1));
      let f_val = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

      let wave_y = (t_val * std::f32::consts::PI * 2.0 + rot * 2.0).sin() * (12.0 + f_val * sensitivity * 25.0);
      let by = center_y + line_offset + wave_y;

      beam_pts.push((bx, by));
    }

    c.set_stroke(Fill::Solid(col));
    c.set_line_width(3.0 + bs * 2.0);
    c.set_shadow(col, 10.0 + bs * 6.0);
    c.stroke_polyline(&beam_pts);
  }

  // -------------------------------------------------------------------------
  // 3. PULSING WHITE CORE CENTERS
  // -------------------------------------------------------------------------
  let core_r = (12.0 + be * 10.0).clamp(6.0, 30.0);
  for &cx in &[left_cx, right_cx] {
    c.set_fill(Fill::Solid(Color::WHITE));
    c.set_shadow(Color::rgba(1.0, 1.0, 1.0, 0.9), 16.0);
    c.fill_ellipse(cx, center_y, core_r, core_r);
  }

  c.restore();
}
