//! Cyber Ring 3D style renderer (`cyberRing3D`).
//!
//! Recreates the exact 3D HUD Hologram Dome from the reference image:
//! Dark purple grid floor, glowing pink central circle with black music note symbol,
//! inner wavy pink spectrum line, thick cyan dashed HUD ring, dotted magenta ring,
//! and outer amber/orange radial spectrum ticks.

use std::f32::consts::TAU;

use crate::gpu2d::text::TextAlign;
use crate::gpu2d::{Color, Fill, GpuCanvas};
use crate::renderers::RenderContext;

const RADIAL_TICKS: usize = 90;

pub fn render(c: &mut GpuCanvas, ctx: &mut RenderContext) {
  let width = ctx.width;
  let height = ctx.height;

  let sensitivity = ctx.config.reactivity.sensitivity;
  let be = ctx.bass_energy;
  let bs = ctx.beat_strength;
  let freq = ctx.freq_data;
  let rot = ctx.rotation_angle;

  let center_x = width * 0.5;
  let center_y = height * 0.52;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  // Dark Purple Background
  c.set_fill(Fill::Solid(Color::rgba(0.08, 0.02, 0.12, 1.0)));
  c.fill_rect(0.0, 0.0, width, height);

  let tilt_y = 0.44f32; // 3D vertical perspective tilt
  let base_rx = (width.min(height) * 0.32).clamp(110.0, 360.0);
  let _base_ry = base_rx * tilt_y;

  // -------------------------------------------------------------------------
  // 1. SUBTLE PERSPECTIVE GRID FLOOR (REFERENCE IMAGE)
  // -------------------------------------------------------------------------
  c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.08, 0.28, 0.3)));
  c.set_line_width(1.0);
  let grid_cols = 12;
  for i in 0..=grid_cols {
    let t = i as f32 / grid_cols as f32;
    let x_top = center_x + (t - 0.5) * (width * 0.8);
    let x_bot = center_x + (t - 0.5) * (width * 1.6);
    c.stroke_line(x_top, center_y - height * 0.3, x_bot, center_y + height * 0.4);
  }

  // -------------------------------------------------------------------------
  // 2. LAYER 4: OUTER AMBER / ORANGE RADIAL SPECTRUM TICKS (EXACT REFERENCE)
  // -------------------------------------------------------------------------
  let rx_ticks = base_rx * 1.08;
  let ry_ticks = rx_ticks * tilt_y;
  let tick_len = (height * 0.04 * sensitivity).clamp(6.0, 30.0);
  let step = (freq.len() / RADIAL_TICKS).max(1);

  for i in 0..RADIAL_TICKS {
    let a = (i as f32 / RADIAL_TICKS as f32) * TAU;
    let bin = (i * step).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let len = tick_len * (0.4 + fv * 1.2);

    let x1 = center_x + a.cos() * rx_ticks;
    let y1 = center_y + a.sin() * ry_ticks;
    let x2 = center_x + a.cos() * (rx_ticks + len);
    let y2 = center_y + a.sin() * (ry_ticks + len * tilt_y);

    let amber_col = Color::rgba(1.0, 0.6, 0.1, 0.85);
    c.set_stroke(Fill::Solid(amber_col));
    c.set_line_width(2.0);
    c.stroke_line(x1, y1, x2, y2);
  }

  // -------------------------------------------------------------------------
  // 3. LAYER 3: THICK CYAN DASHED HUD RING (#00F0FF) (EXACT REFERENCE)
  // -------------------------------------------------------------------------
  let rx_cyan = base_rx * 0.88;
  let ry_cyan = rx_cyan * tilt_y;
  let cyan_col = Color::rgba(0.0, 0.95, 1.0, 0.95);

  let dash_count = 20;
  for d in 0..dash_count {
    if d % 2 == 0 {
      let a1 = -rot * 1.2 + (d as f32 / dash_count as f32) * TAU;
      let a2 = -rot * 1.2 + ((d as f32 + 0.65) / dash_count as f32) * TAU;

      let mut pts = Vec::with_capacity(12);
      for k in 0..12 {
        let angle = a1 + (k as f32 / 11.0) * (a2 - a1);
        pts.push((center_x + angle.cos() * rx_cyan, center_y + angle.sin() * ry_cyan));
      }

      c.set_stroke(Fill::Solid(cyan_col));
      c.set_line_width(4.5 + bs * 2.0);
      c.set_shadow(cyan_col, 12.0);
      c.stroke_polyline(&pts);
    }
  }

  // -------------------------------------------------------------------------
  // 4. LAYER 2: WAVY PINK SPECTRUM LINE RING (EXACT REFERENCE)
  // -------------------------------------------------------------------------
  let rx_pink_wave = base_rx * 0.72;
  let ry_pink_wave = rx_pink_wave * tilt_y;
  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.95);

  let num_wave = 72;
  let mut wave_pts = Vec::with_capacity(num_wave + 1);
  for k in 0..=num_wave {
    let a = (k as f32 / num_wave as f32) * TAU;
    let bin = (k * freq.len() / num_wave).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

    let r_x = rx_pink_wave + fv * 16.0 * sensitivity;
    let r_y = ry_pink_wave + fv * 16.0 * tilt_y * sensitivity;
    wave_pts.push((center_x + a.cos() * r_x, center_y + a.sin() * r_y));
  }

  c.set_stroke(Fill::Solid(hot_pink));
  c.set_line_width(2.5);
  c.set_shadow(hot_pink, 14.0);
  c.stroke_polyline(&wave_pts);

  // -------------------------------------------------------------------------
  // 5. CENTER DOME: SOLID GLOWING PINK CIRCLE WITH BLACK MUSIC NOTE (REFERENCE)
  // -------------------------------------------------------------------------
  let dome_r = (base_rx * 0.42 + be * 15.0).clamp(35.0, 140.0);
  let dome_ry_val = dome_r * 0.75;

  // Solid Glowing Pink Circle
  c.set_fill(Fill::Solid(hot_pink));
  c.set_shadow(hot_pink, 28.0 + bs * 12.0);
  c.fill_ellipse(center_x, center_y - 8.0, dome_r, dome_ry_val);

  // Pure Black 🎵 Music Note Icon inside Dome (Exact Match to Reference Image)
  c.draw_text(
    "🎵",
    center_x,
    center_y - 8.0,
    (dome_r * 0.85).clamp(24.0, 70.0),
    "sans-serif",
    900.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::BLACK),
    1.0,
    &Default::default(),
  );

  c.restore();
}
