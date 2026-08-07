//! Cyber Ring 3D style renderer (`cyberRing3D`).
//!
//! Full Structural Vector Architecture Rebuild:
//! - 3D Perspective Grid Floor
//! - Outer Floor Rim: Amber radial spectrum ticks
//! - Base HUD Tier: 8 wide cyan thick dashed arcs
//! - Mid HUD Tier: 32 magenta dotted circular beads
//! - Upper HUD Tier: Pink wavy audio spectrum ring
//! - Top Podium Badge: Glossy pink 3D disc with black double music note (♫) symbol

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
  let center_y = height * 0.58;

  c.save();
  c.set_shadow(Color::TRANSPARENT, 0.0);

  let tilt_y = 0.38f32; // Perspective 3D tilt
  let base_rx = (width.min(height) * 0.34).clamp(110.0, 360.0);

  let hot_pink = Color::rgba(1.0, 0.0, 0.65, 0.95);
  let cyan_col = Color::rgba(0.0, 0.92, 1.0, 0.95);
  let amber_col = Color::rgba(1.0, 0.65, 0.1, 0.85);

  // -------------------------------------------------------------------------
  // 1. PERSPECTIVE FLOOR GRID LINES
  // -------------------------------------------------------------------------
  c.set_stroke(Fill::Solid(Color::rgba(0.28, 0.08, 0.32, 0.35)));
  c.set_line_width(1.0);

  let grid_cols = 12;
  for i in 0..=grid_cols {
    let t = i as f32 / grid_cols as f32;
    let x_top = center_x + (t - 0.5) * (width * 0.65);
    let x_bot = center_x + (t - 0.5) * (width * 1.4);
    c.stroke_line(x_top, center_y - height * 0.3, x_bot, center_y + height * 0.35);
  }
  for j in 0..6 {
    let ty = center_y + (j as f32 / 5.0) * height * 0.3;
    let spread = 0.5 + (j as f32 / 5.0) * 0.5;
    c.stroke_line(center_x - width * spread * 0.5, ty, center_x + width * spread * 0.5, ty);
  }

  // -------------------------------------------------------------------------
  // 2. OUTER AMBER RADIAL SPECTRUM TICKS (FLOOR RIM)
  // -------------------------------------------------------------------------
  let rx_ticks = base_rx * 1.06;
  let ry_ticks = rx_ticks * tilt_y;
  let tick_count = 72;
  let tick_len = (height * 0.04 * sensitivity).clamp(6.0, 28.0);
  let step = (freq.len() / tick_count).max(1);

  for i in 0..tick_count {
    let a = (i as f32 / tick_count as f32) * TAU;
    let bin = (i * step).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let len = tick_len * (0.35 + fv * 1.2);

    let x1 = center_x + a.cos() * rx_ticks;
    let y1 = center_y + a.sin() * ry_ticks;
    let x2 = center_x + a.cos() * (rx_ticks + len);
    let y2 = center_y + a.sin() * (ry_ticks + len * tilt_y);

    c.set_stroke(Fill::Solid(amber_col));
    c.set_line_width(2.0);
    c.stroke_line(x1, y1, x2, y2);
  }

  // -------------------------------------------------------------------------
  // 3. BASE CYAN DASHED HUD RING (8 CLEAN DASHED ARCS)
  // -------------------------------------------------------------------------
  let rx_cyan = base_rx * 0.88;
  let ry_cyan = rx_cyan * tilt_y;
  let dash_count = 8;

  for d in 0..dash_count {
    let a1 = rot * 0.8 + (d as f32 / dash_count as f32) * TAU;
    let a2 = rot * 0.8 + ((d as f32 + 0.45) / dash_count as f32) * TAU;

    let mut arc_pts = Vec::with_capacity(10);
    for k in 0..10 {
      let angle = a1 + (k as f32 / 9.0) * (a2 - a1);
      arc_pts.push((center_x + angle.cos() * rx_cyan, center_y + angle.sin() * ry_cyan));
    }

    c.set_stroke(Fill::Solid(cyan_col));
    c.set_line_width(4.5 + bs * 2.0);
    c.set_shadow(cyan_col, 10.0);
    c.stroke_polyline(&arc_pts);
  }

  // -------------------------------------------------------------------------
  // 4. MID ELEVATED MAGENTA DOTTED HUD RING (32 DOTS)
  // -------------------------------------------------------------------------
  let y_mid = center_y - (height * 0.045);
  let rx_dot = base_rx * 0.72;
  let ry_dot = rx_dot * tilt_y;
  let dot_count = 32;

  for d in 0..dot_count {
    let a = -rot * 0.9 + (d as f32 / dot_count as f32) * TAU;
    let dx = center_x + a.cos() * rx_dot;
    let dy = y_mid + a.sin() * ry_dot;
    c.set_fill(Fill::Solid(hot_pink));
    c.set_shadow(hot_pink, 6.0);
    c.fill_ellipse(dx, dy, 3.5, 2.2);
  }

  // -------------------------------------------------------------------------
  // 5. UPPER MAGENTA WAVY SPECTRUM LINE RING
  // -------------------------------------------------------------------------
  let y_upper = center_y - (height * 0.09);
  let rx_wave = base_rx * 0.60;
  let ry_wave = rx_wave * tilt_y;

  let wave_count = 64;
  let mut wave_pts = Vec::with_capacity(wave_count + 1);
  for k in 0..=wave_count {
    let a = (k as f32 / wave_count as f32) * TAU;
    let bin = (k * freq.len() / wave_count).min(freq.len().saturating_sub(1));
    let fv = *freq.get(bin).unwrap_or(&0) as f32 / 255.0;
    let r_x = rx_wave + fv * 16.0 * sensitivity;
    let r_y = ry_wave + fv * 16.0 * tilt_y * sensitivity;
    wave_pts.push((center_x + a.cos() * r_x, y_upper + a.sin() * r_y));
  }
  c.set_stroke(Fill::Solid(hot_pink));
  c.set_line_width(2.5);
  c.set_shadow(hot_pink, 12.0);
  c.stroke_polyline(&wave_pts);

  // -------------------------------------------------------------------------
  // 6. ELEVATED TOP PINK MUSIC BADGE (PROPORTIONAL 3D BADGE)
  // -------------------------------------------------------------------------
  let y_badge = center_y - (height * 0.17);
  let badge_r = (base_rx * 0.28 + be * 10.0).clamp(28.0, 95.0);
  let badge_ry = badge_r * 0.65;

  // Outer glow aura
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.0, 0.65, 0.25)));
  c.set_shadow(hot_pink, 25.0 + bs * 10.0);
  c.fill_ellipse(center_x, y_badge, badge_r * 1.2, badge_ry * 1.2);

  // Main pink disc
  c.set_fill(Fill::Solid(hot_pink));
  c.set_shadow(hot_pink, 16.0);
  c.fill_ellipse(center_x, y_badge, badge_r, badge_ry);

  // Inner highlight reflection spot
  c.set_fill(Fill::Solid(Color::rgba(1.0, 0.4, 0.9, 0.55)));
  c.set_shadow(Color::TRANSPARENT, 0.0);
  c.fill_ellipse(center_x - badge_r * 0.1, y_badge - badge_ry * 0.15, badge_r * 0.6, badge_ry * 0.5);

  // Black double music note symbol ♫ inside badge
  let note_sz = (badge_r * 0.70).clamp(18.0, 50.0);
  c.draw_text(
    "♫",
    center_x,
    y_badge,
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
