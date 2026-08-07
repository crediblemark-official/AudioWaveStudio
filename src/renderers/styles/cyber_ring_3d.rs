//! Cyber Ring 3D style renderer (`cyberRing3D`).
//!
//! Recreates the exact 3D HUD Hologram Stage from the reference image:
//! - Dark purple grid floor with perspective grid lines
//! - Outer amber/gold radial spectrum ticks radiating on the floor
//! - Tiered 3D HUD structure with multiple Z-stacked cyan dashed rings and magenta dotted rings
//! - Outer cyan & magenta wavy audio spectrum lines
//! - Glass hologram dome translucent arc outline
//! - Elevated top pink glowing disc badge with bold black double music note (♫) symbol

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
  let center_y = height * 0.54;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let tilt_y = 0.42f32; // Perspective tilt
  let base_rx = (width.min(height) * 0.33).clamp(110.0, 360.0);

  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.95);
  let cyan_col = Color::rgba(0.0, 0.92, 1.0, 0.95);
  let amber_col = Color::rgba(1.0, 0.6, 0.1, 0.85);

  // -------------------------------------------------------------------------
  // 1. PERSPECTIVE GRID FLOOR (REFERENCE IMAGE)
  // -------------------------------------------------------------------------
  c.set_stroke(Fill::Solid(Color::rgba(0.25, 0.08, 0.28, 0.35)));
  c.set_line_width(1.0);
  let grid_cols = 14;
  for i in 0..=grid_cols {
    let t = i as f32 / grid_cols as f32;
    let x_top = center_x + (t - 0.5) * (width * 0.7);
    let x_bot = center_x + (t - 0.5) * (width * 1.5);
    c.stroke_line(x_top, center_y - height * 0.35, x_bot, center_y + height * 0.4);
  }
  for j in 0..8 {
    let ty = center_y + (j as f32 / 7.0) * height * 0.35 - height * 0.05;
    let spread = 0.5 + (j as f32 / 7.0) * 0.5;
    c.stroke_line(center_x - width * spread * 0.5, ty, center_x + width * spread * 0.5, ty);
  }

  // -------------------------------------------------------------------------
  // 2. LAYER 1: OUTER AMBER/GOLD RADIAL SPECTRUM TICKS (FLOOR LEVEL)
  // -------------------------------------------------------------------------
  let rx_ticks = base_rx * 1.08;
  let ry_ticks = rx_ticks * tilt_y;
  let tick_len = (height * 0.045 * sensitivity).clamp(6.0, 32.0);
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

    c.set_stroke(Fill::Solid(amber_col));
    c.set_line_width(2.0);
    c.stroke_line(x1, y1, x2, y2);
  }

  // -------------------------------------------------------------------------
  // 3. LAYER 2: OUTER MAGENTA & CYAN WAVY SPECTRUM LINES
  // -------------------------------------------------------------------------
  let rx_wave = base_rx * 0.98;
  let ry_wave = rx_wave * tilt_y;
  let num_wave = 80;

  // Magenta outer spectrum line
  let mut wave_pts_pink = Vec::with_capacity(num_wave + 1);
  for k in 0..=num_wave {
    let a = (k as f32 / num_wave as f32) * TAU;
    let bin = (k * freq.len() / num_wave).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let r_x = rx_wave + fv * 20.0 * sensitivity;
    let r_y = ry_wave + fv * 20.0 * tilt_y * sensitivity;
    wave_pts_pink.push((center_x + a.cos() * r_x, center_y + a.sin() * r_y));
  }
  c.set_stroke(Fill::Solid(hot_pink));
  c.set_line_width(2.0);
  c.set_shadow(hot_pink, 10.0);
  c.stroke_polyline(&wave_pts_pink);

  // -------------------------------------------------------------------------
  // 4. LAYER 3: LOWER CYAN DASHED HUD RING (LEVEL 0 - BASE)
  // -------------------------------------------------------------------------
  let rx_cyan1 = base_rx * 0.88;
  let ry_cyan1 = rx_cyan1 * tilt_y;
  let dash_count = 18;

  for d in 0..dash_count {
    if d % 2 == 0 {
      let a1 = -rot * 1.2 + (d as f32 / dash_count as f32) * TAU;
      let a2 = -rot * 1.2 + ((d as f32 + 0.65) / dash_count as f32) * TAU;

      let mut pts = Vec::with_capacity(12);
      for k in 0..12 {
        let angle = a1 + (k as f32 / 11.0) * (a2 - a1);
        pts.push((center_x + angle.cos() * rx_cyan1, center_y + angle.sin() * ry_cyan1));
      }

      c.set_stroke(Fill::Solid(cyan_col));
      c.set_line_width(4.5 + bs * 2.0);
      c.set_shadow(cyan_col, 12.0);
      c.stroke_polyline(&pts);
    }
  }

  // -------------------------------------------------------------------------
  // 5. LAYER 4: MID ELEVATED CYAN DASHED HUD RING (LEVEL 1 - ELEVATED Y)
  // -------------------------------------------------------------------------
  let h_step = (height * 0.045).clamp(12.0, 32.0);
  let y_level1 = center_y - h_step * 1.0;
  let rx_cyan2 = base_rx * 0.74;
  let ry_cyan2 = rx_cyan2 * tilt_y;

  for d in 0..dash_count {
    if d % 2 != 0 {
      let a1 = rot * 1.0 + (d as f32 / dash_count as f32) * TAU;
      let a2 = rot * 1.0 + ((d as f32 + 0.65) / dash_count as f32) * TAU;

      let mut pts = Vec::with_capacity(12);
      for k in 0..12 {
        let angle = a1 + (k as f32 / 11.0) * (a2 - a1);
        pts.push((center_x + angle.cos() * rx_cyan2, y_level1 + angle.sin() * ry_cyan2));
      }

      c.set_stroke(Fill::Solid(cyan_col));
      c.set_line_width(4.0);
      c.set_shadow(cyan_col, 10.0);
      c.stroke_polyline(&pts);
    }
  }

  // -------------------------------------------------------------------------
  // 6. LAYER 5: HIGH ELEVATED DOTTED MAGENTA HUD RING (LEVEL 2)
  // -------------------------------------------------------------------------
  let y_level2 = center_y - h_step * 1.8;
  let rx_dot = base_rx * 0.60;
  let ry_dot = rx_dot * tilt_y;
  let num_dots = 32;

  for d in 0..num_dots {
    let a = -rot * 0.9 + (d as f32 / num_dots as f32) * TAU;
    let dx = center_x + a.cos() * rx_dot;
    let dy = y_level2 + a.sin() * ry_dot;
    c.set_fill(Fill::Solid(hot_pink));
    c.set_shadow(hot_pink, 6.0);
    c.fill_ellipse(dx, dy, 3.0, 2.0);
  }

  // -------------------------------------------------------------------------
  // 7. LAYER 6: TRANSLUCENT HOLOGRAPHIC GLASS DOME ARC OUTLINE
  // -------------------------------------------------------------------------
  let dome_height = h_step * 2.8 + (be * 10.0);
  let dome_rx = base_rx * 0.85;

  // Glass dome outer arch outline
  let mut dome_arc = Vec::with_capacity(40);
  for k in 0..=36 {
    let a = (k as f32 / 36.0) * std::f32::consts::PI; // Top semi-circle arc
    let dx = center_x + a.cos() * dome_rx;
    let dy = center_y - a.sin() * dome_height;
    dome_arc.push((dx, dy));
  }
  c.set_stroke(Fill::Solid(Color::rgba(0.0, 0.92, 1.0, 0.4)));
  c.set_line_width(1.5);
  c.set_shadow(cyan_col, 8.0);
  c.stroke_polyline(&dome_arc);

  // -------------------------------------------------------------------------
  // 8. TOP ELEVATED PINK GLOWING DISC BADGE WITH MUSIC NOTE (EXACT REFERENCE)
  // -------------------------------------------------------------------------
  let y_top_badge = center_y - dome_height * 0.65;
  let badge_r = (base_rx * 0.38 + be * 10.0).clamp(36.0, 130.0);
  let badge_ry = badge_r * 0.68;

  // Outer glowing aura
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.65, 0.35)));
  c.set_shadow(hot_pink, 30.0 + bs * 12.0);
  c.fill_ellipse(center_x, y_top_badge, badge_r * 1.15, badge_ry * 1.15);

  // Main bright glowing magenta/pink disc
  c.set_fill(Fill::Solid(hot_pink));
  c.set_shadow(hot_pink, 20.0);
  c.fill_ellipse(center_x, y_top_badge, badge_r, badge_ry);

  // Inner highlight reflection
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.4, 0.9, 0.6)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_ellipse(center_x - badge_r * 0.1, y_top_badge - badge_ry * 0.15, badge_r * 0.65, badge_ry * 0.55);

  // Bold black double musical note symbol (♫) in the center of the top pink disc
  let note_sz = (badge_r * 0.75).clamp(24.0, 64.0);
  c.draw_text(
    "♫",
    center_x,
    y_top_badge,
    note_sz,
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
