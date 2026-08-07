//! Cyber Ring 3D style renderer (`cyberRing3D`).
//!
//! 3D HUD Hologram Dome: dark purple grid floor, glowing pink 3D hemisphere dome,
//! music note text, stacked concentric rings, cyan dashed HUD ring,
//! wavy pink spectrum, and outer amber radial spectrum ticks.

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

  let tilt_y = 0.44f32;
  let base_rx = (width.min(height) * 0.32).clamp(110.0, 360.0);

  // -------------------------------------------------------------------------
  // 1. PERSPECTIVE GRID FLOOR
  // -------------------------------------------------------------------------
  c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.08, 0.28, 0.3)));
  c.set_line_width(1.0);
  let grid_cols = 14;
  for i in 0..=grid_cols {
    let t = i as f32 / grid_cols as f32;
    let x_top = center_x + (t - 0.5) * (width * 0.7);
    let x_bot = center_x + (t - 0.5) * (width * 1.5);
    c.stroke_line(x_top, center_y - height * 0.3, x_bot, center_y + height * 0.4);
  }
  // Horizontal grid lines
  for j in 0..8 {
    let ty = center_y + (j as f32 / 7.0) * height * 0.35;
    let spread = 0.5 + (j as f32 / 7.0) * 0.5;
    c.stroke_line(center_x - width * spread * 0.5, ty, center_x + width * spread * 0.5, ty);
  }

  // -------------------------------------------------------------------------
  // 2. OUTER AMBER/ORANGE RADIAL SPECTRUM TICKS
  // -------------------------------------------------------------------------
  let rx_ticks = base_rx * 1.08;
  let ry_ticks = rx_ticks * tilt_y;
  let tick_len = (height * 0.04 * sensitivity).clamp(6.0, 30.0);
  let step = (freq.len() / RADIAL_TICKS).max(1);

  for i in 0..RADIAL_TICKS {
    let a = (i as f32 / RADIAL_TICKS as f32) * TAU;
    let bin = (i * step).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let len = tick_len * (0.3 + fv * 1.3);

    let x1 = center_x + a.cos() * rx_ticks;
    let y1 = center_y + a.sin() * ry_ticks;
    let x2 = center_x + a.cos() * (rx_ticks + len);
    let y2 = center_y + a.sin() * (ry_ticks + len * tilt_y);

    c.set_stroke(Fill::Solid(Color::rgba(1.0, 0.6, 0.1, 0.85)));
    c.set_line_width(2.0);
    c.stroke_line(x1, y1, x2, y2);
  }

  // -------------------------------------------------------------------------
  // 3. THICK CYAN DASHED HUD RING
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
  // 4. DOTTED MAGENTA RING (BETWEEN CYAN AND DOME)
  // -------------------------------------------------------------------------
  let rx_dot = base_rx * 0.76;
  let ry_dot = rx_dot * tilt_y;
  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.95);

  let num_dots = 40;
  for d in 0..num_dots {
    let a = rot * 1.1 + (d as f32 / num_dots as f32) * TAU;
    let dx = center_x + a.cos() * rx_dot;
    let dy = center_y + a.sin() * ry_dot;
    c.set_fill(Fill::Solid(hot_pink));
    c.set_shadow(hot_pink, 6.0);
    c.fill_ellipse(dx, dy, 3.5, 2.5);
  }

  // -------------------------------------------------------------------------
  // 5. WAVY PINK SPECTRUM LINE RING
  // -------------------------------------------------------------------------
  let rx_wave = base_rx * 0.64;
  let ry_wave = rx_wave * tilt_y;

  let num_wave = 72;
  let mut wave_pts = Vec::with_capacity(num_wave + 1);
  for k in 0..=num_wave {
    let a = (k as f32 / num_wave as f32) * TAU;
    let bin = (k * freq.len() / num_wave).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;

    let r_x = rx_wave + fv * 18.0 * sensitivity;
    let r_y = ry_wave + fv * 18.0 * tilt_y * sensitivity;
    wave_pts.push((center_x + a.cos() * r_x, center_y + a.sin() * r_y));
  }

  c.set_stroke(Fill::Solid(hot_pink));
  c.set_line_width(2.5);
  c.set_shadow(hot_pink, 14.0);
  c.stroke_polyline(&wave_pts);

  // -------------------------------------------------------------------------
  // 6. 3D HEMISPHERE DOME (LAYERED GRADIENT FOR 3D EFFECT)
  // -------------------------------------------------------------------------
  let dome_r = (base_rx * 0.45 + be * 15.0).clamp(40.0, 150.0);
  let dome_ry = dome_r * 0.72;
  let dome_cy = center_y - 6.0;

  // Layer 1: Outer glow halo
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.65, 0.15)));
  c.set_shadow(hot_pink, 35.0 + bs * 15.0);
  c.fill_ellipse(center_x, dome_cy, dome_r * 1.15, dome_ry * 1.15);

  // Layer 2: Main solid pink dome
  c.set_fill(Fill::Solid(hot_pink));
  c.set_shadow(hot_pink, 20.0);
  c.fill_ellipse(center_x, dome_cy, dome_r, dome_ry);

  // Layer 3: Lighter inner highlight (simulates 3D sphere lighting)
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.35, 0.85, 0.55)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_ellipse(center_x - dome_r * 0.12, dome_cy - dome_ry * 0.15, dome_r * 0.65, dome_ry * 0.55);

  // Layer 4: Top specular white highlight
  c.set_fill(Fill::Solid(Color::rgba(1.0, 1.0, 1.0, 0.25)));
  c.fill_ellipse(center_x - dome_r * 0.15, dome_cy - dome_ry * 0.28, dome_r * 0.35, dome_ry * 0.25);

  // -------------------------------------------------------------------------
  // 7. MUSIC NOTE TEXT (plain text, NOT emoji)
  // -------------------------------------------------------------------------
  let note_sz = (dome_r * 0.65).clamp(22.0, 60.0);

  c.draw_text(
    "♫",
    center_x,
    dome_cy,
    note_sz,
    "serif",
    900.0,
    false,
    TextAlign::Center,
    Fill::Solid(Color::rgba(0.15, 0.0, 0.1, 0.85)),
    1.0,
    &Default::default(),
  );

  c.restore();
}
